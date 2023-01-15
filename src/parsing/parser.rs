use std::str::{Chars, FromStr};

use base64::Engine;
use strum::VariantNames;
use unindent::Unindent;

use crate::{
    error::hyperlink,
    lexing::tokens::{Lexer, Token},
    sources::{CodeArea, CodeSpan, SpwnSource},
};

use super::{
    ast::{
        Ast, DictItems, ExprNode, Expression, IDClass, MacroCode, Spannable, Statement, Statements,
        StmtNode,
    },
    attributes::{AttributeEnum, ExprAttribute, ScriptAttribute},
    error::SyntaxError,
    utils::operators::{self, unary_prec},
};

#[derive(Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    src: SpwnSource,
}

pub type ParseResult<T> = Result<T, SyntaxError>;

impl<'a> Parser<'a> {
    pub fn new(code: &'a str, src: SpwnSource) -> Self {
        let lexer = Token::lex(code);
        Parser { lexer, src }
    }
}

#[macro_export]
macro_rules! list_helper {
    ($self:ident, $closing_tok:ident $code:block) => {
        while !$self.next_is(Token::$closing_tok) {
            $code;
            if !$self.skip_tok(Token::Comma) {
                break;
            }
        }
        $self.expect_tok(Token::$closing_tok)?;
    };
}

impl Parser<'_> {
    pub fn next(&mut self) -> Token {
        self.lexer.next_or_eof()
    }
    pub fn span(&self) -> CodeSpan {
        self.lexer.span().into()
    }
    pub fn peek_span(&self) -> CodeSpan {
        let mut peek = self.lexer.clone();
        peek.next();
        peek.span().into()
    }
    pub fn slice(&self) -> &str {
        self.lexer.slice()
    }
    pub fn peek(&self) -> Token {
        let mut peek = self.lexer.clone();
        peek.next_or_eof()
    }
    pub fn next_is(&self, tok: Token) -> bool {
        self.peek() == tok
    }
    // pub fn next_are(&self, toks: &[Token]) -> bool {
    //     let mut peek = self.lexer.clone();
    //     for tok in toks {
    //         if peek.next_or_eof() != *tok {
    //             return false;
    //         }
    //     }
    //     true
    // }

    pub fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            src: self.src.clone(),
        }
    }
    pub fn skip_tok(&mut self, skip: Token) -> bool {
        if self.next_is(skip) {
            self.next();
            true
        } else {
            false
        }
    }

    pub fn expect_tok_named(&mut self, expect: Token, name: &str) -> ParseResult<()> {
        let next = self.next();
        if next != expect {
            return Err(SyntaxError::UnexpectedToken {
                found: next,
                expected: name.to_string(),
                area: self.make_area(self.span()),
            });
        }
        Ok(())
    }
    pub fn expect_tok(&mut self, expect: Token) -> Result<(), SyntaxError> {
        self.expect_tok_named(expect, expect.to_str())
    }
    pub fn expect_toks_named(&mut self, expect: &[Token], name: &str) -> ParseResult<()> {
        let next = self.next();
        if !expect.contains(&next) {
            return Err(SyntaxError::UnexpectedToken {
                found: next,
                expected: name.to_string(),
                area: self.make_area(self.span()),
            });
        }
        Ok(())
    }

    pub fn parse_int(&self, s: &str) -> i64 {
        if s.len() > 2 {
            match &s[0..2] {
                "0x" => {
                    return i64::from_str_radix(&s.trim_start_matches("0x").replace('_', ""), 16)
                        .unwrap()
                }
                "0b" => {
                    return i64::from_str_radix(&s.trim_start_matches("0b").replace('_', ""), 2)
                        .unwrap()
                }
                "0o" => {
                    return i64::from_str_radix(&s.trim_start_matches("0o").replace('_', ""), 8)
                        .unwrap()
                }
                _ => (),
            }
        }
        s.parse::<i64>().unwrap()
    }

    fn parse_id(&self, s: &str) -> (IDClass, Option<u16>) {
        let class = match &s[(s.len() - 1)..(s.len())] {
            "g" => IDClass::Group,
            "c" => IDClass::Color,
            "b" => IDClass::Block,
            "i" => IDClass::Item,
            _ => unreachable!(),
        };
        let value = s[0..(s.len() - 1)].parse::<u16>().ok();

        (class, value)
    }

    pub fn parse_string(&self, s: &str, span: CodeSpan) -> ParseResult<String> {
        let mut chars = s.chars();

        // Remove trailing "
        chars.next_back();

        let flags = chars
            .by_ref()
            .take_while(|c| !matches!(c, '"' | '\''))
            .collect::<String>();
        let mut out: String = chars.collect();

        if flags.is_empty() {
            return self.parse_escapes(&mut out.chars());
        }

        let mut flags = flags.split('_').collect::<Vec<_>>();
        let last = flags.pop().unwrap();

        if matches!(last, "r" | "r#" | "r##") {
            out = out[0..(out.len() - last.len() + 1)].into();
        } else {
            flags.push(last);
        }

        for flag in flags {
            match flag {
                "b" => todo!("byte string"),
                "u" => out = out.unindent(),
                "b64" => {
                    out = base64::engine::general_purpose::STANDARD.encode(out);
                }
                other => {
                    return Err(SyntaxError::UnexpectedFlag {
                        flag: other.to_string(),
                        area: self.make_area(span),
                    });
                }
            }
        }

        Ok(out)
    }

    fn parse_escapes(&self, chars: &mut Chars) -> ParseResult<String> {
        let mut out = String::new();

        loop {
            match chars.next() {
                Some('\\') => out.push(match chars.next() {
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('"') => '"',
                    Some('\'') => '\'',
                    Some('\\') => '\\',
                    Some('u') => self.parse_unicode(chars)?,
                    Some(c) => {
                        return Err(SyntaxError::InvalidEscape {
                            character: c,
                            area: self.make_area(self.span()),
                        })
                    }
                    None => {
                        return Err(SyntaxError::InvalidEscape {
                            character: ' ',
                            area: self.make_area(self.span()),
                        })
                    }
                }),
                Some(c) => {
                    if c != '\'' && c != '"' {
                        out.push(c)
                    }
                }
                None => break,
            }
        }

        Ok(out)
    }

    fn parse_unicode(&self, chars: &mut Chars) -> ParseResult<char> {
        let next = chars.next().unwrap_or(' ');

        if !matches!(next, '{') {
            return Err(SyntaxError::UnxpectedCharacter {
                expected: Token::LBracket,
                found: next.to_string(),
                area: self.make_area(self.span()),
            });
        }

        // `take_while` will always consume the matched chars +1 (in order to check whether it matches)
        // this means `unwrap_or` would always use the default, so instead clone it to not affect
        // the actual chars iterator
        let hex = chars
            .clone()
            .take_while(|c| matches!(*c, '0'..='9' | 'a'..='f' | 'A'..='F'))
            .collect::<String>();

        let mut schars = chars.skip(hex.len());

        let next = schars.next();

        match next {
            Some('}') => (),
            Some(t) => {
                return Err(SyntaxError::UnxpectedCharacter {
                    expected: Token::RBracket,
                    found: t.to_string(),
                    area: self.make_area(self.span()),
                })
            }
            None => {
                return Err(SyntaxError::UnxpectedCharacter {
                    expected: Token::RBracket,
                    found: "end of string".into(),
                    area: self.make_area(self.span()),
                })
            }
        }

        Ok(char::from_u32(u32::from_str_radix(&hex, 16).map_err(|_| {
            SyntaxError::InvalidUnicode {
                sequence: hex,
                area: self.make_area(self.span()),
            }
        })?)
        .unwrap_or('\u{FFFD}'))
    }

    pub fn parse_dictlike(&mut self) -> ParseResult<DictItems> {
        let mut items = vec![];

        list_helper!(self, RBracket {
            let key = match self.next() {
                Token::Int => self.parse_int(self.slice()).to_string(),
                Token::String => self.parse_string(self.slice(), self.span())?,
                Token::Ident => self.slice().to_string(),
                other => {
                    return Err(SyntaxError::UnexpectedToken {
                        expected: "key".into(),
                        found: other,
                        area: self.make_area(self.span()),
                    })
                }
            };

            let key_span = self.span();

            let elem = if self.next_is(Token::Colon) {
                self.next();
                Some(self.parse_expr()?)
            } else {
                None
            };

            items.push((key.spanned(key_span), elem));
        });

        Ok(items)
    }

    pub fn parse_attributes<T: AttributeEnum>(&mut self) -> ParseResult<Vec<T>> {
        let mut attrs = vec![];
        self.expect_tok(Token::LSqBracket)?;

        list_helper!(self, RSqBracket {

            let attr = T::attribute_parse(self)?;

            attrs.push(attr);
        });

        Ok(attrs)
    }

    pub fn parse_unit(&mut self) -> ParseResult<ExprNode> {
        let attr_start = self.peek_span();

        let attrs = if self.next_is(Token::Hashtag) {
            self.next();

            self.parse_attributes::<ExprAttribute>()?
        } else {
            vec![]
        };

        let peek = self.peek();
        let start = self.peek_span();

        let unary;

        let mut expr = match peek {
            Token::Int => {
                self.next();
                Expression::Int(self.parse_int(self.slice())).spanned(start)
            }
            Token::Float => {
                self.next();
                Expression::Float(self.slice().replace('_', "").parse::<f64>().unwrap())
                    .spanned(start)
            }
            Token::String => {
                self.next();
                Expression::String(self.parse_string(self.slice(), self.span())?).spanned(start)
            }
            Token::Id => {
                self.next();

                let (id_class, value) = self.parse_id(self.slice());
                Expression::Id(id_class, value).spanned(start)
            }
            Token::Dollar => {
                self.next();

                Expression::Builtins.spanned(start)
            }
            Token::True => {
                self.next();
                Expression::Bool(true).spanned(start)
            }
            Token::False => {
                self.next();
                Expression::Bool(false).spanned(start)
            }
            Token::Ident => {
                self.next();
                let var_name = self.slice().to_string();

                if matches!(self.peek(), Token::FatArrow | Token::Arrow) {
                    let ret_type = if self.next_is(Token::Arrow) {
                        self.next();
                        let r = Some(self.parse_expr()?);
                        self.expect_tok(Token::FatArrow)?;
                        r
                    } else {
                        self.next();
                        None
                    };

                    let code = MacroCode::Lambda(self.parse_expr()?);

                    return Ok(Expression::Macro {
                        args: vec![(var_name.spanned(start), None, None)],
                        code,
                        ret_type,
                    }
                    .spanned(start.extend(self.span())));
                }

                Expression::Var(var_name).spanned(start)
            }
            Token::TypeIndicator => {
                self.next();
                let name = self.slice()[1..].to_string();
                Expression::Type(name).spanned(start)
            }
            Token::LParen => {
                self.next();

                let mut check = self.clone();
                let mut indent = 1;

                let after_close = loop {
                    match check.next() {
                        Token::LParen => indent += 1,
                        Token::Eof => {
                            return Err(SyntaxError::UnmatchedToken {
                                not_found: Token::RParen,
                                for_char: Token::LParen,
                                area: self.make_area(start),
                            })
                        }
                        Token::RParen => {
                            indent -= 1;
                            if indent == 0 {
                                break check.next();
                            }
                        }
                        _ => (),
                    }
                };

                let is_macro = match after_close {
                    Token::FatArrow | Token::LBracket => true,
                    Token::Arrow => {
                        check.parse_expr()?;

                        matches!(check.peek(), Token::FatArrow | Token::LBracket)
                    }
                    _ => {
                        if self.next_is(Token::RParen) {
                            self.next();
                            return Ok(Expression::Empty.spanned(start.extend(self.span())));
                        }
                        let inner = self.parse_expr()?;
                        self.expect_tok(Token::RParen)?;
                        return Ok(inner.extended(self.span()));
                    }
                };

                if is_macro {
                    let mut args = vec![];

                    list_helper!(self, RParen {
                        self.expect_tok_named(Token::Ident, "argument name")?;
                        let arg_name = self.slice().to_string().spanned(self.span());

                        let typ = if self.next_is(Token::Colon) {
                            self.next();
                            Some(self.parse_expr()?)
                        } else {
                            None
                        };
                        let default = if self.next_is(Token::Assign) {
                            self.next();
                            Some(self.parse_expr()?)
                        } else {
                            None
                        };

                        args.push((arg_name, typ, default));
                    });

                    let ret_type = if self.next_is(Token::Arrow) {
                        self.next();
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };

                    let code = if self.next_is(Token::FatArrow) {
                        self.next();
                        MacroCode::Lambda(self.parse_expr()?)
                    } else {
                        println!("fuckfart");
                        MacroCode::Normal(self.parse_block()?)
                    };

                    Expression::Macro {
                        args,
                        code,
                        ret_type,
                    }
                    .spanned(start.extend(self.span()))
                } else {
                    let mut args = vec![];

                    list_helper!(self, RParen {
                        args.push(self.parse_expr()?);
                    });

                    self.expect_tok(Token::Arrow)?;

                    let ret_type = self.parse_expr()?;

                    Expression::MacroPattern { args, ret_type }.spanned(start.extend(self.span()))
                }
            }
            Token::LSqBracket => {
                self.next();

                let mut elems = vec![];

                list_helper!(self, RSqBracket {
                    elems.push(self.parse_expr()?);
                });

                Expression::Array(elems).spanned(start.extend(self.span()))
            }
            Token::LBracket => {
                self.next();

                Expression::Dict(self.parse_dictlike()?).spanned(start.extend(self.span()))
            }
            // Token::Hashtag => {
            //     self.next();

            //     self.parse_attributes::<Attribute>()?;
            // }
            Token::QMark => {
                self.next();

                Expression::Maybe(None).spanned(start)
            }

            Token::TrigFnBracket => {
                self.next();

                let code = self.parse_statements()?;
                self.expect_tok(Token::RBracket)?;

                Expression::TriggerFunc {
                    code,
                    attributes: vec![],
                }
                .spanned(start)
            }
            unary_op
                if {
                    unary = unary_prec(unary_op);
                    unary.is_some()
                } =>
            {
                self.next();
                let unary_prec = unary.unwrap();
                let next_prec = operators::next_infix(unary_prec);
                let val = match next_prec {
                    Some(next_prec) => self.parse_op(next_prec)?,
                    None => self.parse_value()?,
                };

                Expression::Unary(unary_op, val).spanned(start.extend(self.span()))
            }

            other => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: "expression".into(),
                    found: other,
                    area: self.make_area(start),
                });
            }
        };

        if !attrs.is_empty() {
            match &mut *expr.value {
                Expression::TriggerFunc { attributes, .. } => {
                    *attributes = attrs;
                }
                _ => {
                    return Err(SyntaxError::MismatchedAttribute {
                        area: self.make_area(attr_start.extend(self.span())),
                        help: "The valid expression attribute locations are trigger functions"
                            .into(),
                    });
                }
            };
        }

        Ok(expr)
    }

    pub fn parse_value(&mut self) -> ParseResult<ExprNode> {
        let mut value = self.parse_unit()?;
        #[allow(clippy::while_let_loop)]
        loop {
            let new_span = value.span.extend(self.peek_span());
            match self.peek() {
                Token::LSqBracket => {
                    self.next();
                    let index = self.parse_expr()?;
                    self.expect_tok(Token::RSqBracket)?;

                    value = Expression::Index { base: value, index }.spanned(new_span);
                }
                Token::Dot => {
                    self.next();
                    self.expect_tok_named(Token::Ident, "member name")?;
                    let name = self.slice().to_string();

                    value = Expression::Member { base: value, name }.spanned(new_span);
                }
                Token::DoubleColon => {
                    self.next();
                    self.expect_tok_named(Token::Ident, "associated member name")?;
                    let name = self.slice().to_string();

                    value = Expression::Associated { base: value, name }.spanned(new_span);
                }
                Token::QMark => {
                    self.next();

                    value = Expression::Maybe(Some(value)).spanned(new_span);
                }
                Token::ExclMark => {
                    self.next();

                    value = Expression::TriggerFuncCall(value).spanned(new_span);
                }
                Token::If => {
                    self.next();
                    let cond = self.parse_expr()?;
                    self.expect_tok(Token::Else)?;
                    let if_false = self.parse_expr()?;

                    value = Expression::Ternary {
                        cond,
                        if_true: value,
                        if_false,
                    }
                    .spanned(new_span);
                }
                // Token::C
                _ => break,
            }
        }
        Ok(value)
    }

    pub fn parse_expr(&mut self) -> ParseResult<ExprNode> {
        self.parse_op(0)
    }

    pub fn parse_op(&mut self, prec: usize) -> ParseResult<ExprNode> {
        let next_prec = operators::next_infix(prec);

        let mut left = match next_prec {
            Some(next_prec) => self.parse_op(next_prec)?,
            None => self.parse_value()?,
        };
        while operators::is_infix_prec(self.peek(), prec) {
            let op = self.next();
            let right = if operators::prec_type(prec) == operators::OpType::Left {
                match next_prec {
                    Some(next_prec) => self.parse_op(next_prec)?,
                    None => self.parse_value()?,
                }
            } else {
                self.parse_op(prec)?
            };
            let new_span = left.span.extend(right.span);
            left = Expression::Op(left, op, right).spanned(new_span)
        }
        Ok(left)
    }

    pub fn parse_block(&mut self) -> ParseResult<Statements> {
        self.expect_tok(Token::LBracket)?;
        let code = self.parse_statements()?;
        self.expect_tok(Token::RBracket)?;

        Ok(code)
    }

    pub fn parse_statement(&mut self) -> ParseResult<StmtNode> {
        let start = self.peek_span();

        let is_arrow = if self.next_is(Token::Arrow) {
            self.next();
            true
        } else {
            false
        };

        let stmt = match self.peek() {
            Token::Let => {
                self.next();
                self.expect_tok_named(Token::Ident, "variable name")?;
                let var_name = self.slice().to_string();
                self.expect_tok(Token::Assign)?;
                let value = self.parse_expr()?;

                Statement::Let(var_name, value)
            }
            Token::If => {
                self.next();
                let mut branches = vec![];
                let mut else_branch = None;

                let cond = self.parse_expr()?;
                let code = self.parse_block()?;
                branches.push((cond, code));

                while self.next_is(Token::Else) {
                    self.next();
                    if self.next_is(Token::If) {
                        self.next();
                        let cond = self.parse_expr()?;
                        let code = self.parse_block()?;
                        branches.push((cond, code));
                    } else {
                        else_branch = Some(self.parse_block()?);
                        break;
                    }
                }

                Statement::If {
                    branches,
                    else_branch,
                }
            }
            Token::While => {
                self.next();
                let cond = self.parse_expr()?;
                let code = self.parse_block()?;

                Statement::While { cond, code }
            }
            Token::For => {
                self.next();
                self.expect_tok_named(Token::Ident, "variable name")?;
                let var = self.slice().to_string();
                self.expect_tok(Token::In)?;
                let iterator = self.parse_expr()?;

                let code = self.parse_block()?;

                Statement::For {
                    var,
                    iterator,
                    code,
                }
            }
            Token::Return => {
                self.next();
                if matches!(self.peek(), Token::Eol | Token::RBracket | Token::Eof) {
                    Statement::Return(None)
                } else {
                    let val = self.parse_expr()?;

                    Statement::Return(Some(val))
                }
            }
            Token::Continue => {
                self.next();

                Statement::Continue
            }
            Token::Break => {
                self.next();

                Statement::Break
            }
            _ => Statement::Expr(self.parse_expr()?),
        };
        if self.slice() != "}" {
            self.expect_tok(Token::Eol)?;
        }

        let stmt = if is_arrow {
            Statement::Arrow(Box::new(stmt))
        } else {
            stmt
        }
        .spanned(start.extend(self.span()));

        Ok(stmt)
    }

    pub fn parse_statements(&mut self) -> ParseResult<Statements> {
        let mut statements = vec![];

        while !matches!(self.peek(), Token::Eof | Token::RBracket) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }

        Ok(statements)
    }

    pub fn parse(&mut self) -> ParseResult<Ast> {
        let file_attributes = if self.next_is(Token::Hashtag) {
            self.next();

            self.parse_attributes::<ScriptAttribute>()?
        } else {
            vec![]
        };

        let statements = self.parse_statements()?;
        self.expect_tok(Token::Eof)?;

        Ok(Ast {
            statements,
            file_attributes,
        })
    }
}
