use std::str::Chars;

use logos::{Lexer, Logos};

use super::ast::ASTData;
use super::ast::{ExprKey, Expression, Statement, Statements, StmtKey};
use super::error::SyntaxError;
use super::lexer::{Token, Tokens};
use super::parser_util::{operators, OpType, ParsedDictlike, StringType};

use crate::sources::{CodeArea, CodeSpan, SpwnSource};

#[derive(Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a, Token>,
    source: SpwnSource,
}

impl Parser<'_> {
    pub fn next(&mut self) -> Token {
        self.lexer.next().unwrap_or(Token::Eof)
    }
    pub fn peek(&mut self) -> Token {
        let mut peek = self.lexer.clone();
        peek.next().unwrap_or(Token::Eof)
    }
    pub fn peek_span(&mut self) -> CodeSpan {
        let mut peek = self.lexer.clone();
        peek.next();
        peek.span().into()
    }
    pub fn peek_many(&mut self, n: usize) -> Token {
        let mut peek = self.lexer.clone();
        let mut last = peek.next();

        for _ in 0..(n - 1) {
            last = peek.next();
        }

        last.unwrap_or(Token::Eof)
    }

    pub fn span(&self) -> CodeSpan {
        self.lexer.span().into()
    }
    pub fn slice(&self) -> &str {
        self.lexer.slice()
    }

    pub fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            source: self.source.clone(),
        }
    }

    pub fn new<S: AsRef<str>>(code: S, source: SpwnSource) -> Self {
        let code = unsafe { Parser::make_static(code.as_ref()) };
        let lexer = Token::lexer(code);
        Parser { lexer, source }
    }
    unsafe fn make_static<'a>(d: &'a str) -> &'static str {
        std::mem::transmute::<&'a str, &'static str>(d)
    }

    pub fn peek_expect_or_tok<T, U>(&mut self, tok: T, or: U) -> Result<(), SyntaxError>
    where
        T: Into<Tokens> + ToString,
        U: ToString,
    {
        let next = self.peek();
        let toks: Tokens = tok.into();

        if !toks.0.contains(&next) {
            self.next();
            return Err(SyntaxError::ExpectedToken {
                expected: or.to_string(),
                found: next.to_string(),
                area: self.make_area(self.span()),
            });
        }

        Ok(())
    }

    pub fn peek_expect_tok<T>(&mut self, tok: T) -> Result<(), SyntaxError>
    where
        T: Into<Tokens> + ToString + Clone,
    {
        self.peek_expect_or_tok(tok.clone(), tok)
    }

    pub fn expect_or_tok<T, U>(&mut self, tok: T, or: U) -> Result<(), SyntaxError>
    where
        T: Into<Tokens> + ToString,
        U: ToString,
    {
        let next = self.next();
        let toks: Tokens = tok.into();

        if !toks.0.contains(&next) {
            return Err(SyntaxError::ExpectedToken {
                expected: or.to_string(),
                found: next.to_string(),
                area: self.make_area(self.span()),
            });
        }

        Ok(())
    }

    pub fn expect_tok<T>(&mut self, tok: T) -> Result<(), SyntaxError>
    where
        T: Into<Tokens> + ToString + Clone,
    {
        self.expect_or_tok(tok.clone(), tok)
    }

    pub fn skip_tok(&mut self, tok: Token) {
        if self.peek() == tok {
            self.next();
        }
    }

    pub fn parse_int(&self, literal: &str) -> Result<u32, SyntaxError> {
        if literal.len() > 2 {
            Ok(match &literal[0..2] {
                "0b" => self.to_int_radix(&literal[2..], 2)?,
                _ => self.to_int_radix(literal, 10)?,
            })
        } else {
            Ok(literal.parse::<u32>().unwrap())
        }
    }

    fn to_int_radix(&self, from: &str, radix: u32) -> Result<u32, SyntaxError> {
        u32::from_str_radix(from, radix).map_err(|_| SyntaxError::InvalidLiteral {
            literal: from.into(),
            area: self.make_area(self.span()),
        })
    }

    pub fn parse_string(&self, literal: &str) -> Result<StringType, SyntaxError> {
        let mut chars = literal.chars();

        let flag = chars
            .by_ref()
            .take_while(|c| !matches!(c, '"' | '\''))
            .collect::<String>();

        let out = match &flag[..] {
            "b" => todo!("byte string"),
            "f" => todo!("f-string"),
            "r" => todo!("raw string"),
            "u" => todo!("unindented string"),
            "" => self.parse_escapes(&mut chars)?,
            other => {
                return Err(SyntaxError::UnexpectedFlag {
                    flag: other.to_string(),
                    area: self.make_area(self.span()),
                });
            }
        };

        Ok(StringType::Normal(out))
    }

    fn parse_escapes(&self, chars: &mut Chars) -> Result<String, SyntaxError> {
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

    fn parse_unicode(&self, chars: &mut Chars) -> Result<char, SyntaxError> {
        let next = chars.next().unwrap_or(' ');

        if !matches!(next, '{') {
            return Err(SyntaxError::ExpectedToken {
                expected: "{".into(),
                found: next.to_string(),
                area: self.make_area(self.span()),
            });
        }

        let hex = chars
            .take_while(|c| matches!(*c, '0'..='9' | 'a'..='f' | 'A'..='F'))
            .collect::<String>();

        let next = chars.next().unwrap_or(' ');

        if !matches!(next, '}') {
            return Err(SyntaxError::ExpectedToken {
                expected: "}".into(),
                found: next.to_string(),
                area: self.make_area(self.span()),
            });
        }

        Ok(char::from_u32(self.to_int_radix(&hex, 16)? as u32).unwrap_or('�'))
    }

    pub fn parse_dictlike(&mut self, data: &mut ASTData) -> Result<ParsedDictlike, SyntaxError> {
        let mut items = vec![];
        let mut item_spans = vec![];

        while self.peek() != Token::RBracket {
            self.expect_or_tok(Token::Ident, "key")?;
            let key = self.slice().to_string();
            let key_span = self.span();
            item_spans.push(key_span);

            let elem = if self.peek() == Token::Colon {
                self.next();
                Some(self.parse_expr(data)?)
            } else {
                None
            };

            items.push((key, elem));
            self.peek_expect_tok(Token::RBracket | Token::Comma)?;
            self.skip_tok(Token::Comma);
        }
        self.next();
        Ok(ParsedDictlike { items, item_spans })
    }

    pub fn parse_unit(&mut self, data: &mut ASTData) -> Result<ExprKey, SyntaxError> {
        let peek = self.peek();
        let start = self.peek_span();

        match peek {
            Token::Int => {
                self.next();
                let int = self.parse_int(self.slice())?;
                Ok(data.insert_expr(Expression::Int(int), start))
            }
            Token::Float => {
                self.next();
                let float = self.slice().parse::<f64>().unwrap();
                Ok(data.insert_expr(Expression::Float(float), start))
            }
            Token::String => {
                self.next();
                let s = self.parse_string(self.slice())?;
                let expr = match s {
                    StringType::Normal(s) => Expression::String(s),
                    _ => todo!(),
                };
                Ok(data.insert_expr(expr, start))
            }
            Token::True => {
                self.next();
                Ok(data.insert_expr(Expression::Bool(true), start))
            }
            Token::False => {
                self.next();
                Ok(data.insert_expr(Expression::Bool(false), start))
            }
            Token::Ident => {
                self.next();
                let name = self.slice().to_string();
                let arg_span = self.span();
                if self.peek() == Token::FatArrow {
                    self.next();
                    let code = self.parse_expr(data)?;

                    let key = data.insert_expr(
                        Expression::Func {
                            args: vec![(name, None, None)],
                            ret_type: None,
                            code,
                        },
                        start,
                    );

                    data.func_arg_spans.insert(key, vec![arg_span]);

                    Ok(key)
                } else {
                    Ok(data.insert_expr(Expression::Var(name), start))
                }
            }
            Token::TypeIndicator => {
                self.next();
                let name = self.slice()[1..].to_string();
                Ok(data.insert_expr(Expression::Type(name), start))
            }

            Token::LParen => {
                self.next();
                if self.peek() == Token::RParen && self.peek_many(2) != Token::FatArrow {
                    self.next();
                    Ok(data.insert_expr(Expression::Empty, start.extend(self.span())))
                } else {
                    let mut depth = 1;
                    let mut check = self.clone();

                    loop {
                        match check.peek() {
                            Token::LParen => {
                                check.next();
                                depth += 1;
                            }
                            Token::RParen => {
                                check.next();
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            Token::Eof => {
                                return Err(SyntaxError::UnmatchedChar {
                                    for_char: "(".into(),
                                    not_found: ")".into(),
                                    area: self.make_area(start),
                                })
                            }
                            _ => {
                                check.next();
                            }
                        }
                    }

                    let is_pattern = match check.peek() {
                        Token::FatArrow => false,
                        Token::Arrow => {
                            check.next();
                            check.parse_expr(data)?;

                            check.peek() != Token::FatArrow
                        }
                        _ => {
                            let value = self.parse_expr(data)?;
                            self.expect_tok(Token::RParen)?;
                            return Ok(value);
                        }
                    };

                    if !is_pattern {
                        let mut args = vec![];
                        let mut arg_areas = vec![];
                        while self.peek() != Token::RParen {
                            self.expect_or_tok(Token::Ident, "argument name")?;
                            let arg_name = self.slice().to_string();
                            let arg_span = self.span();
                            let arg_type = if self.peek() == Token::Colon {
                                self.next();
                                Some(self.parse_expr(data)?)
                            } else {
                                None
                            };
                            let arg_default = if self.peek() == Token::Assign {
                                self.next();
                                Some(self.parse_expr(data)?)
                            } else {
                                None
                            };
                            args.push((arg_name, arg_type, arg_default));
                            arg_areas.push(arg_span);
                            self.peek_expect_tok(Token::RParen | Token::Comma)?;
                            self.skip_tok(Token::Comma);
                        }
                        self.next();
                        let ret_type = if self.peek() == Token::Arrow {
                            self.next();
                            Some(self.parse_expr(data)?)
                        } else {
                            None
                        };
                        self.expect_tok(Token::FatArrow)?;
                        let code = self.parse_expr(data)?;

                        let key = data.insert_expr(
                            Expression::Func {
                                args,
                                ret_type,
                                code,
                            },
                            start.extend(self.span()),
                        );

                        data.func_arg_spans.insert(key, arg_areas);

                        Ok(key)
                    } else {
                        let mut args = vec![];
                        while self.peek() != Token::RParen {
                            let arg = self.parse_expr(data)?;
                            args.push(arg);
                            self.peek_expect_tok(Token::RParen | Token::Comma)?;
                            self.skip_tok(Token::Comma);
                        }
                        self.next();
                        self.expect_tok(Token::Arrow)?;
                        let ret_type = self.parse_expr(data)?;

                        Ok(data.insert_expr(
                            Expression::FuncPattern { args, ret_type },
                            start.extend(self.span()),
                        ))
                    }
                }

                // let value = self.parse_expr(data)?;
                // self.expect_tok(Token::RParen, ")")?;
                // Ok(value)
            }

            Token::LSqBracket => {
                self.next();

                let mut elems = vec![];
                while self.peek() != Token::RSqBracket {
                    let elem = self.parse_expr(data)?;
                    elems.push(elem);
                    self.peek_expect_tok(Token::RSqBracket | Token::Comma)?;
                    self.skip_tok(Token::Comma);
                }
                self.next();

                Ok(data.insert_expr(Expression::Array(elems), start.extend(self.span())))
            }
            Token::LBracket => {
                self.next();
                if self.peek() == Token::RBracket {
                    self.next();
                    return Ok(
                        data.insert_expr(Expression::Dict(vec![]), start.extend(self.span()))
                    );
                }

                if !(self.peek() == Token::Ident && self.peek_many(2) == Token::Colon) {
                    let code = self.parse_statements(data)?;
                    self.expect_tok(Token::RBracket)?;
                    Ok(data.insert_expr(Expression::Block(code), start.extend(self.span())))
                } else {
                    let dictlike = self.parse_dictlike(data)?;
                    Ok(data.exprs.insert_with_key(|key| {
                        data.dictlike_spans.insert(key, dictlike.item_spans);
                        (Expression::Dict(dictlike.items), start.extend(self.span()))
                    }))
                }
            }

            Token::QMark => {
                self.next();
                Ok(data.insert_expr(Expression::Maybe(None), start))
            }

            Token::Split => {
                self.next();
                let a = self.parse_expr(data)?;
                self.expect_tok(Token::Colon)?;
                let b = self.parse_expr(data)?;
                Ok(data.insert_expr(Expression::Split(a, b), start))
            }

            Token::TrigFnBracket => {
                self.next();
                let code = self.parse_statements(data)?;
                self.expect_tok(Token::RBracket)?;
                Ok(data.insert_expr(Expression::TriggerFunc(code), start))
            }

            unary_op if operators::is_unary(unary_op) => {
                self.next();
                let prec = operators::unary_prec(unary_op);
                let mut next_prec = if prec + 1 < operators::prec_amount() {
                    prec + 1
                } else {
                    operators::PREC_MAX
                };
                while next_prec != operators::PREC_MAX {
                    if operators::prec_type(next_prec) == OpType::Unary {
                        next_prec += 1
                    } else {
                        break;
                    }
                    if next_prec == operators::prec_amount() {
                        next_prec = operators::PREC_MAX
                    }
                }
                let value = if next_prec != operators::PREC_MAX {
                    self.parse_op(data, next_prec)?
                } else {
                    self.parse_value(data)?
                };

                Ok(data.insert_expr(
                    Expression::Unary(unary_op, value),
                    start.extend(self.span()),
                ))
            }

            other => Err(SyntaxError::ExpectedToken {
                expected: "expression".into(),
                found: other.to_string(),
                area: self.make_area(start),
            }),
        }
    }

    pub fn parse_value(&mut self, data: &mut ASTData) -> Result<ExprKey, SyntaxError> {
        let start = self.peek_span();
        let mut value = self.parse_unit(data)?;

        while matches!(
            self.peek(),
            Token::LSqBracket
                | Token::If
                | Token::LParen
                | Token::QMark
                | Token::DoubleColon
                | Token::ExclMark
        ) {
            match self.peek() {
                Token::LSqBracket => {
                    self.next();
                    let index = self.parse_expr(data)?;
                    self.expect_tok(Token::RSqBracket)?;
                    value = data.insert_expr(
                        Expression::Index { base: value, index },
                        start.extend(self.span()),
                    )
                }
                Token::If => {
                    self.next();
                    let cond = self.parse_expr(data)?;
                    self.expect_tok(Token::Else)?;
                    let if_false = self.parse_expr(data)?;
                    value = data.insert_expr(
                        Expression::Ternary {
                            cond,
                            if_true: value,
                            if_false,
                        },
                        start.extend(self.span()),
                    )
                }
                Token::LParen => {
                    self.next();
                    let mut params = vec![];
                    let mut named_params = vec![];
                    let mut param_areas = vec![];

                    let mut started_named = false;

                    while self.peek() != Token::RParen {
                        if !started_named {
                            match (self.peek(), self.peek_many(2)) {
                                (Token::Ident, Token::Assign) => {
                                    started_named = true;
                                    self.next();
                                    let start = self.span();
                                    let name = self.slice().to_string();
                                    self.next();
                                    let arg = self.parse_expr(data)?;
                                    param_areas.push(start.extend(self.span()));
                                    named_params.push((name, arg));
                                }
                                _ => {
                                    let start = self.peek_span();
                                    let arg = self.parse_expr(data)?;
                                    param_areas.push(start.extend(self.span()));
                                    params.push(arg);
                                }
                            }
                        } else {
                            let start = self.peek_span();
                            self.expect_or_tok(Token::Ident, "parameter name")?;
                            let name = self.slice().to_string();
                            self.expect_tok(Token::Assign)?;
                            let arg = self.parse_expr(data)?;
                            param_areas.push(start.extend(self.span()));
                            named_params.push((name, arg));
                        }
                        // self.next();
                        self.peek_expect_tok(Token::RParen | Token::Comma)?;
                        self.skip_tok(Token::Comma);
                    }
                    self.next();

                    let key = data.insert_expr(
                        Expression::Call {
                            base: value,
                            params,
                            named_params,
                        },
                        start.extend(self.span()),
                    );

                    data.func_arg_spans.insert(key, param_areas);
                    value = key;
                }
                Token::QMark => {
                    self.next();
                    value =
                        data.insert_expr(Expression::Maybe(Some(value)), start.extend(self.span()))
                }
                Token::DoubleColon => {
                    self.next();
                    self.expect_tok(Token::LBracket)?;
                    let dictlike = self.parse_dictlike(data)?;
                    value = data.exprs.insert_with_key(|key| {
                        data.dictlike_spans.insert(key, dictlike.item_spans);
                        (
                            Expression::Instance(value, dictlike.items),
                            start.extend(self.span()),
                        )
                    })
                }
                Token::ExclMark => {
                    self.next();
                    value = data.insert_expr(
                        Expression::TriggerFuncCall(value),
                        start.extend(self.span()),
                    )
                }
                _ => unreachable!(),
            }
        }
        Ok(value)
    }

    pub fn parse_expr(&mut self, data: &mut ASTData) -> Result<ExprKey, SyntaxError> {
        self.parse_op(data, 0)
    }

    pub fn parse_op(&mut self, data: &mut ASTData, prec: usize) -> Result<ExprKey, SyntaxError> {
        let mut next_prec = if prec + 1 < operators::prec_amount() {
            prec + 1
        } else {
            operators::PREC_MAX
        };
        while next_prec != operators::PREC_MAX {
            if operators::prec_type(next_prec) == OpType::Unary {
                next_prec += 1
            } else {
                break;
            }
            if next_prec == operators::prec_amount() {
                next_prec = operators::PREC_MAX
            }
        }
        let mut left = if next_prec != operators::PREC_MAX {
            self.parse_op(data, next_prec)?
        } else {
            self.parse_value(data)?
        };
        while operators::infix_prec(self.peek()) == prec {
            let op = self.next();
            let right = if operators::prec_type(prec) == OpType::LeftAssoc {
                if next_prec != operators::PREC_MAX {
                    self.parse_op(data, next_prec)?
                } else {
                    self.parse_value(data)?
                }
            } else {
                self.parse_op(data, prec)?
            };
            let span = data.get_span(left).extend(data.get_span(right));
            left = data.insert_expr(Expression::Op(left, op, right), span);
        }
        Ok(left)
    }

    pub fn parse_block(&mut self, data: &mut ASTData) -> Result<Vec<StmtKey>, SyntaxError> {
        self.expect_tok(Token::LBracket)?;
        let code = self.parse_statements(data)?;
        self.expect_tok(Token::RBracket)?;

        Ok(code)
    }

    pub fn parse_statement(&mut self, data: &mut ASTData) -> Result<StmtKey, SyntaxError> {
        let peek = self.peek();
        let start = self.peek_span();
        let is_arrow = if peek == Token::Arrow {
            self.next();
            true
        } else {
            false
        };

        let stmt_key = data.insert_stmt(Statement::Break, start);
        data.stmt_arrows.insert(stmt_key, is_arrow);

        let stmt = match self.peek() {
            Token::Let => {
                self.next();
                self.expect_or_tok(Token::Ident, "variable name")?;
                let var_name = self.slice().to_string();
                self.expect_tok(Token::Assign)?;
                let value = self.parse_expr(data)?;
                Statement::Let(var_name, value)
            }
            Token::Print => {
                self.next();
                let value = self.parse_expr(data)?;
                Statement::Print(value)
            }
            Token::If => {
                self.next();
                let mut branches = vec![];
                let mut else_branch = None;

                let cond = self.parse_expr(data)?;
                let code = self.parse_block(data)?;
                branches.push((cond, code));

                while self.peek() == Token::Else {
                    self.next();
                    if self.peek() == Token::If {
                        self.next();
                        let cond = self.parse_expr(data)?;
                        let code = self.parse_block(data)?;
                        branches.push((cond, code));
                    } else {
                        else_branch = Some(self.parse_block(data)?);
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
                let cond = self.parse_expr(data)?;
                let code = self.parse_block(data)?;
                Statement::While { cond, code }
            }
            Token::For => {
                self.next();
                self.expect_or_tok(Token::Ident, "variable name")?;
                let var = self.slice().to_string();
                let var_span = self.span();
                self.expect_tok(Token::In)?;
                let iterator = self.parse_expr(data)?;
                let code = self.parse_block(data)?;

                data.for_loop_iter_spans.insert(stmt_key, var_span);

                Statement::For {
                    var,
                    iterator,
                    code,
                }
            }
            Token::Return => {
                self.next();
                if matches!(self.peek(), Token::Eol | Token::RBracket) {
                    Statement::Return(None)
                } else {
                    let val = self.parse_expr(data)?;
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
            Token::TypeDef => {
                self.next();
                self.expect_tok(Token::TypeIndicator)?;
                let typ_name = self.slice()[1..].to_string();
                Statement::TypeDef(typ_name)
            }
            Token::Impl => {
                self.next();

                let typ = self.parse_expr(data)?;

                self.expect_tok(Token::LBracket)?;
                let dictlike = self.parse_dictlike(data)?;

                data.impl_spans.insert(stmt_key, dictlike.item_spans);

                Statement::Impl(typ, dictlike.items)
            }
            Token::Ident => {
                if self.peek_many(2) == Token::Assign {
                    self.next();
                    let var = self.slice().to_string();
                    self.next();
                    let value = self.parse_expr(data)?;
                    Statement::Assign(var, value)
                } else {
                    Statement::Expr(self.parse_expr(data)?)
                }
            }
            _ => Statement::Expr(self.parse_expr(data)?),
        };
        if self.slice() != "}" {
            self.expect_tok(Token::Eol)?;
        }
        self.skip_tok(Token::Eol);

        data.stmts[stmt_key] = (stmt, start.extend(self.span()));

        Ok(stmt_key)
    }

    pub fn parse_statements(&mut self, data: &mut ASTData) -> Result<Statements, SyntaxError> {
        let mut statements = vec![];
        while !matches!(self.peek(), Token::Eof | Token::RBracket) {
            let stmt = self.parse_statement(data)?;
            statements.push(stmt);
        }
        Ok(statements)
    }

    pub fn parse(&mut self, data: &mut ASTData) -> Result<Statements, SyntaxError> {
        let stmts = self.parse_statements(data)?;
        self.expect_tok(Token::Eof)?;
        Ok(stmts)
    }
}
