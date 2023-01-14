use std::str::Chars;

use logos::{Lexer, Logos};

use crate::sources::{CodeArea, CodeSpan, SpwnSource};

use super::{
    ast::{ExprNode, Expression, IDClass, Statement, Statements, StmtNode},
    error::SyntaxError,
    lexer::Token,
    utils::operators::{self, unary_prec},
};

#[derive(Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a, Token>,
    src: SpwnSource,
}

type ParseResult<T> = Result<T, SyntaxError>;

impl<'a> Parser<'a> {
    pub fn new(code: &'a str, src: SpwnSource) -> Self {
        let lexer = Token::lexer(code);
        Parser { lexer, src }
    }
}

impl Parser<'_> {
    pub fn next(&mut self) -> Token {
        self.lexer.next().unwrap_or(Token::Eof)
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
        peek.next().unwrap_or(Token::Eof)
    }
    pub fn next_is(&self, tok: Token) -> bool {
        self.peek() == tok
    }
    pub fn next_are(&self, toks: &[Token]) -> bool {
        let mut peek = self.lexer.clone();
        for tok in toks {
            if peek.next().unwrap_or(Token::Eof) != *tok {
                return false;
            }
        }
        true
    }

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

    pub fn parse_int(&self, s: &str) -> i64 {
        println!("cock {}", s);

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

    // fn to_int_radix(&self, from: &str, radix: u32) -> Result<i64> {
    //     i64::from_str_radix(&from.replace('_', ""), radix).map_err(|_| {
    //         SyntaxError::InvalidLiteral {
    //             literal: from.into(),
    //             area: self.make_area(self.span()),
    //         }
    //     })
    // }

    fn parse_id(&self, s: &str) -> (IDClass, Option<u16>) {
        println!("shitfuck {}", s);
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

    pub fn parse_string(&self, s: &str) -> String {
        // let span = self.span();
        // let content: &str = self.lexer.slice();
        // let mut chars = content.chars();

        // // Remove trailing "
        // chars.next_back();

        // let flag = chars
        //     .by_ref()
        //     .take_while(|c| !matches!(c, '"' | '\''))
        //     .collect::<String>();

        // let out = match &*flag {
        //     "b" => todo!("byte string"),
        //     "u" => todo!("unindented string"),
        //     "" => self.parse_escapes(&mut chars)?,
        //     other => {
        //         return Err(SyntaxError::UnexpectedFlag {
        //             flag: other.to_string(),
        //             area: self.make_area(self.span()),
        //         });
        //     }
        // };
        // Ok(out)

        // string flags shit here eventually amognsu
        s.into()
    }

    // fn parse_escapes(&self, chars: &mut Chars) -> ParseResult<String> {
    //     let mut out = String::new();

    //     loop {
    //         match chars.next() {
    //             Some('\\') => out.push(match chars.next() {
    //                 Some('n') => '\n',
    //                 Some('r') => '\r',
    //                 Some('t') => '\t',
    //                 Some('"') => '"',
    //                 Some('\'') => '\'',
    //                 Some('\\') => '\\',
    //                 Some('u') => self.parse_unicode(chars)?,
    //                 Some(c) => {
    //                     return Err(SyntaxError::InvalidEscape {
    //                         character: c,
    //                         area: self.make_area(self.span()),
    //                     })
    //                 }
    //                 None => {
    //                     return Err(SyntaxError::InvalidEscape {
    //                         character: ' ',
    //                         area: self.make_area(self.span()),
    //                     })
    //                 }
    //             }),
    //             Some(c) => {
    //                 if c != '\'' && c != '"' {
    //                     out.push(c)
    //                 }
    //             }
    //             None => break,
    //         }
    //     }

    //     Ok(out)
    // }

    // fn parse_unicode(&self, chars: &mut Chars) -> ParseResult<char> {
    //     let next = chars.next().unwrap_or(' ');

    //     if !matches!(next, '{') {
    //         return Err(SyntaxError::UnxpectedCharacter {
    //             expected: Token::LBracket,
    //             found: next.to_string(),
    //             area: self.make_area(self.span()),
    //         });
    //     }

    //     // `take_while` will always consume the matched chars +1 (in order to check whether it matches)
    //     // this means `unwrap_or` would always use the default, so instead clone it to not affect
    //     // the actual chars iterator
    //     let hex = chars
    //         .clone()
    //         .take_while(|c| matches!(*c, '0'..='9' | 'a'..='f' | 'A'..='F'))
    //         .collect::<String>();

    //     let mut schars = chars.skip(hex.len());

    //     let next = schars.next().unwrap_or(' ');

    //     if !matches!(next, '}') {
    //         return Err(SyntaxError::UnxpectedCharacter {
    //             expected: Token::RBracket,
    //             found: next.to_string(),
    //             area: self.make_area(self.span()),
    //         });
    //     }

    //     Ok(char::from_u32(self.to_int_radix(&hex, 16)? as u32).unwrap_or('\u{FFFD}'))
    // }

    pub fn parse_unit(&mut self) -> ParseResult<ExprNode> {
        let peek = self.peek();
        let start = self.peek_span();

        let unary;
        Ok(match peek {
            Token::Int => {
                self.next();
                Expression::Int(self.parse_int(self.slice())).into_node(start)
            }
            Token::Float => {
                self.next();
                Expression::Float(self.slice().replace('_', "").parse::<f64>().unwrap())
                    .into_node(start)
            }
            Token::String => {
                self.next();
                Expression::String(self.parse_string(self.slice())).into_node(start)
            }
            Token::Id => {
                self.next();

                let (id_class, value) = self.parse_id(self.slice());
                Expression::Id(id_class, value).into_node(start)
            }
            Token::True => {
                self.next();
                Expression::Bool(true).into_node(start)
            }
            Token::False => {
                self.next();
                Expression::Bool(false).into_node(start)
            }
            Token::Ident => {
                self.next();
                Expression::Var(self.slice().into()).into_node(start)
            }
            Token::TypeIndicator => {
                self.next();
                let name = self.slice()[1..].to_string();
                Expression::Type(name).into_node(start)
            }

            Token::LParen => {
                self.next();
                let value = self.parse_expr()?;
                self.expect_tok(Token::RParen)?;
                value.extended(self.span())
            }
            Token::LSqBracket => {
                self.next();

                let mut elems = vec![];
                while !self.next_is(Token::RSqBracket) {
                    elems.push(self.parse_expr()?);
                    if !self.skip_tok(Token::Comma) {
                        break;
                    }
                }
                self.expect_tok(Token::RSqBracket)?;

                Expression::Array(elems).into_node(start.extend(self.span()))
            }
            Token::LBracket => {
                self.next();

                todo!()
                // self.parse_dictlike()
            }

            Token::QMark => {
                self.next();
                Expression::Maybe(None).into_node(start)
            }

            Token::TrigFnBracket => {
                self.next();
                let code = self.parse_statements()?;
                self.expect_tok(Token::RBracket)?;
                Expression::TriggerFunc(code).into_node(start)
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
                Expression::Unary(unary_op, val).into_node(start.extend(self.span()))
            }

            other => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: "expression".into(),
                    found: other,
                    area: self.make_area(start),
                })
            }
        })
    }

    pub fn parse_value(&mut self) -> ParseResult<ExprNode> {
        let mut value = self.parse_unit()?;
        #[allow(clippy::while_let_loop)]
        loop {
            match self.peek() {
                Token::LSqBracket => {
                    self.next();
                    let index = self.parse_expr()?;
                    self.expect_tok(Token::RSqBracket)?;

                    let new_span = value.span.extend(self.span());
                    value = Expression::Index { base: value, index }.into_node(new_span);
                }
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
            left = Expression::Op(left, op, right).into_node(new_span)
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
            // Token::Return => {
            //     self.next();
            //     if matches!(self.peek(), Token::Eol | Token::RBracket) {
            //         Statement::Return(None)
            //     } else {
            //         let val = self.parse_expr()?;

            //         Statement::Return(Some(val))
            //     }
            // }
            // Token::Continue => {
            //     self.next();

            //     Statement::Continue
            // }
            // Token::Break => {
            //     self.next();

            //     Statement::Break
            // }
            _ => Statement::Expr(self.parse_expr()?),
        };
        if self.slice() != "}" {
            self.expect_tok(Token::Eol)?;
        }
        // self.skip_tok(Token::Eol);

        let stmt = if is_arrow {
            Statement::Arrow(Box::new(stmt))
        } else {
            stmt
        }
        .into_node(start.extend(self.span()));

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

    pub fn parse(&mut self) -> ParseResult<Statements> {
        let stmts = self.parse_statements()?;
        self.expect_tok(Token::Eof)?;
        Ok(stmts)
    }
}
