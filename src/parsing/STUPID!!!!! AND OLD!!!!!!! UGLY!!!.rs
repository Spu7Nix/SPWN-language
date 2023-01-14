use logos::{Lexer, Logos};

use crate::sources::{CodeArea, CodeSpan, SpwnSource};

use super::{
    ast::{ExprNode, Expression, ImportType, Statement, Statements, StmtNode},
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
            src: self.src,
        }
    }
    pub fn skip_tok(&mut self, skip: Token) -> bool {
        if self.peek() == skip {
            self.next();
            true
        } else {
            false
        }
    }

    pub fn expect_tok_named(&mut self, expect: Token, name: &str) -> ParseResult<()> {
        let next = self.next();
        if next != expect {
            return Err(SyntaxError::ExpectedToken {
                found: next,
                expected: name.to_string(),
                area: self.make_area(self.span()),
            });
        }
        Ok(())
    }
    pub fn expect_tok(&mut self, expect: Token) -> Result<(), SyntaxError> {
        self.expect_tok_named(expect, expect.to_string())
    }

    // fn parse_id(&mut self) -> ParseResult<ExprNode> {
    //     self.next();
    //     let span = self.span();
    //     let content: &str = self.lexer.slice();
    //     let class = match &content[(content.len() - 1)..(content.len())] {
    //         "g" => IdClass::Group,
    //         "c" => IdClass::Color,
    //         "b" => IdClass::Block,
    //         "i" => IdClass::Item,
    //         _ => unreachable!(),
    //     };
    //     let value = content[0..(content.len() - 1)].parse::<u16>().ok();

    //     Expression::Id { class, value }.span(span)
    // }

    // pub fn parse_float(&mut self) -> ParseResult<ExprNode> {
    //     self.next();
    //     let span = self.span();
    //     let content: &str = self.lexer.slice();
    //     let float = content.replace('_', "").parse::<f64>().unwrap();

    //     Expression::Float(float).span(span)
    // }

    // pub fn parse_dictlike(&mut self) -> ParseParseResult<ExprNode> {
    //     let mut items = vec![];

    //     while self.peek() != Token::RBracket {
    //         let peek = self.peek();

    //         let key = match peek {
    //             Token::String => {
    //                 self.next();
    //                 self.parse_string()?
    //             }
    //             Token::Ident => {
    //                 self.next();
    //                 self.slice().to_string()
    //             }
    //             _ => {
    //                 return Err(SyntaxError::ExpectedToken {
    //                     expected: "key".into(),
    //                     found: peek.to_string(),
    //                     area: self.make_area(self.span()),
    //                 })
    //             }
    //         };

    //         let key_span = self.span();

    //         let elem = if self.peek() == Token::Colon {
    //             self.next();
    //             Some(self.parse_expr()?)
    //         } else {
    //             None
    //         };

    //         items.push((key, elem).span(key_span));
    //         self.peek_expect_tok(Token::RBracket | Token::Comma)?;
    //         self.skip_tok(Token::Comma);
    //     }
    //     self.next();

    //     Expression::Dict(items).span(self.span())
    // }

    // pub fn parse_objlike(&mut self) -> ParseResult<ExprNode> {
    //     let obj_mode = if self.peek() == Token::Obj {
    //         ObjectMode::Object
    //     } else {
    //         ObjectMode::Trigger
    //     };

    //     self.next();
    //     self.expect_tok(Token::LBracket)?;

    //     let mut items = vec![];

    //     while self.peek() != Token::RBracket {
    //         let key = self.parse_expr()?;
    //         let key_span = data.span(key);

    //         self.expect_tok(Token::Colon)?;
    //         let value = self.parse_expr()?;

    //         items.push((key, value).span(key_span));
    //         self.peek_expect_tok(Token::RBracket | Token::Comma)?;
    //         self.skip_tok(Token::Comma);
    //     }
    //     self.next();

    //     Expression::Obj(obj_mode, items)
    //         .span(self.span())
    //
    // }

    pub fn parse_int(&self, s: &str) -> i64 {
        if s.len() > 2 {
            match &s[0..2] {
                "0x" => return i64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap(),
                "0b" => return i64::from_str_radix(s.trim_start_matches("0b"), 2).unwrap(),
                _ => (),
            }
        }
        s.parse::<i64>().unwrap()
    }
    pub fn parse_string(&self, s: &str) -> String {
        // string flags shit here eventually amognsu
        s.into()
    }

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
                Expression::Float(self.slice().parse::<f64>().unwrap()).into_node(start)
            }
            Token::String => {
                self.next();
                Expression::String(self.parse_string(self.slice())).into_node(start)
            }
            // Token::Id => self.parse_id(data),
            Token::Dollar => {
                self.next();
                Expression::Builtins.into_node(start)
            }
            Token::True => {
                self.next();
                Expression::Bool(true).into_node(start)
            }
            Token::False => {
                self.next();
                Expression::Bool(false).into_node(start)
            }
            Token::Ident => self.parse_ident_or_macro()?,
            Token::TypeIndicator => {
                self.next();
                let name = self.slice()[1..].to_string();
                Expression::Type(name).into_node(start)
            }

            Token::Import => {
                self.next();
                let expr = match self.next() {
                    Token::String => {
                        Expression::Import(ImportType::Module(self.parse_string(self.slice())))
                    }
                    Token::Ident => {
                        Expression::Import(ImportType::Library(self.slice().to_string()))
                    }
                    _ => todo!(),
                };
                expr.into_node(start)
            }

            Token::LParen => self.parse_paren_or_macro()?,

            // let value = self.parse_expr()?;
            // self.expect_tok(Token::RParen, ")")?;
            // Ok(value)
            Token::LSqBracket => {
                self.next();

                let mut elems = vec![];
                while self.peek() != Token::RSqBracket {
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

            // Token::Split => {
            //     self.next();
            //     let a = self.parse_expr()?;
            //     self.expect_tok(Token::Colon)?;
            //     let b = self.parse_expr()?;
            //     Expression::Split(a.span(b), start)
            // }
            Token::TrigFnBracket => {
                self.next();
                let code = self.parse_statements()?;
                self.expect_tok(Token::RBracket)?;
                Expression::TriggerFunc(code).into_node(start)
            }

            Token::Obj | Token::Trigger => todo!(),
            unary_op
                if {
                    unary = unary_prec(unary_op);
                    unary.is_some()
                } =>
            {
                let unary_prec = unary.unwrap();
                let next_prec = operators::next_infix(unary_prec);
                let val = match next_prec {
                    Some(next_prec) => self.parse_op(next_prec)?,
                    None => self.parse_value()?,
                };
                Expression::Unary(unary_op, val).into_node(start.extend(self.span()))
            }

            other => {
                return Err(SyntaxError::ExpectedToken {
                    expected: "expression".into(),
                    found: other,
                    area: self.make_area(start),
                })
            }
        })
    }

    fn parse_paren_or_macro(&mut self) -> ParseResult<ExprNode> {
        todo!()
        // self.next();
        // let start = self.span();
        // if self.peek() == Token::RParen
        //     && !matches!(
        //         self.peek_many(2),
        //         Token::FatArrow | Token::Arrow | Token::LBracket
        //     )
        // {
        //     self.next();
        //     Expression::Empty.span(start.extend(self.span()))
        // } else {
        //     let mut depth = 1;
        //     let mut check = self.clone();

        //     loop {
        //         match check.peek() {
        //             Token::LParen => {
        //                 check.next();
        //                 depth += 1;
        //             }
        //             Token::RParen => {
        //                 check.next();
        //                 depth -= 1;
        //                 if depth == 0 {
        //                     break;
        //                 }
        //             }
        //             Token::Eof => {
        //                 return Err(SyntaxError::UnmatchedChar {
        //                     for_char: "(".into(),
        //                     not_found: ")".into(),
        //                     area: self.make_area(start),
        //                 })
        //             }
        //             _ => {
        //                 check.next();
        //             }
        //         }
        //     }

        //     enum IsMacro {
        //         No,
        //         Yes { lambda: bool },
        //     }

        //     let is_macro = match check.peek() {
        //         // `() => {} ` = `IsMacro::Yes { has_arrow: true }`,
        //         Token::FatArrow => IsMacro::Yes { lambda: true },
        //         //`() {} ` = `IsMacro::Yes { has_arrow: false }`,
        //         Token::LBracket => IsMacro::Yes { lambda: false },
        //         // `(...) ->`
        //         Token::Arrow => {
        //             // skips the arrow (`->`)
        //             check.next();
        //             // parses value following arrow
        //             check.parse_expr()?;

        //             // if the next token is `=>` or `{` the previous value was a return type,
        //             // otherwise it's a macro pattern
        //             match check.peek() {
        //                 // `() -> @string => {} ` = `IsMacro::Yes { has_arrow: true }`,
        //                 Token::FatArrow => IsMacro::Yes { lambda: true },
        //                 //`() -> @string {} ` = `IsMacro::Yes { has_arrow: false }`,
        //                 Token::LBracket => IsMacro::Yes { lambda: false },
        //                 // `() -> @string` = `IsMacro::No` (pattern)
        //                 _ => IsMacro::No,
        //             }
        //         }
        //         _ => {
        //             let value = self.parse_expr()?;
        //             self.expect_tok(Token::RParen)?;
        //             return Ok(value);
        //         }
        //     };

        //     if let IsMacro::Yes { lambda } = is_macro {
        //         let mut args = vec![];

        //         while self.peek() != Token::RParen {
        //             self.expect_or_tok(Token::Ident, "argument name")?;
        //             let arg_name = self.slice().to_string();
        //             let arg_span = self.span();
        //             let arg_type = if self.peek() == Token::Colon {
        //                 self.next();
        //                 Some(self.parse_expr()?)
        //             } else {
        //                 None
        //             };
        //             let arg_default = if self.peek() == Token::Assign {
        //                 self.next();
        //                 Some(self.parse_expr()?)
        //             } else {
        //                 None
        //             };
        //             args.push((arg_name, arg_type, arg_default).span(arg_span));

        //             self.peek_expect_tok(Token::RParen | Token::Comma)?;
        //             self.skip_tok(Token::Comma);
        //         }
        //         self.next();
        //         let ret_type = if self.peek() == Token::Arrow {
        //             self.next();
        //             Some(self.parse_expr()?)
        //         } else {
        //             None
        //         };
        //         let key = if lambda {
        //             self.expect_tok(Token::FatArrow)?;
        //             let code = self.parse_expr()?;

        //             Expression::Macro {
        //                 args,
        //                 ret_type,
        //                 code: MacroCode::Lambda(code),
        //             }
        //             .span(start.extend(self.span()))
        //             ?
        //         } else {
        //             self.expect_tok(Token::LBracket)?;
        //             let code = self.parse_statements()?;
        //             self.expect_tok(Token::RBracket)?;

        //             Expression::Macro {
        //                 args,
        //                 ret_type,
        //                 code: MacroCode::Normal(code),
        //             }
        //             .span(start.extend(self.span()))
        //             ?
        //         };

        //         Ok(key)
        //     } else {
        //         let mut args = vec![];
        //         while self.peek() != Token::RParen {
        //             let arg = self.parse_expr()?;
        //             args.push(arg);
        //             self.peek_expect_tok(Token::RParen | Token::Comma)?;
        //             self.skip_tok(Token::Comma);
        //         }
        //         self.next();
        //         self.expect_tok(Token::Arrow)?;
        //         let ret_type = self.parse_expr()?;

        //         Expression::MacroPattern { args, ret_type }.span(start.extend(self.span()))
        //     }
        // }
    }

    fn parse_ident_or_macro(&mut self) -> ParseResult<ExprNode> {
        todo!()
        // self.next();
        // let start = self.span();
        // let name = self.slice().to_string();
        // let arg_span = self.span();
        // if self.peek() == Token::FatArrow {
        //     self.next();
        //     let code = self.parse_expr()?;

        //     let key = Expression::Macro {
        //         args: vec![(name, None, None).span(arg_span)],
        //         ret_type: None,
        //         code: MacroCode::Lambda(code),
        //     }
        //     .span(start)
        //     ?;

        //     Ok(key)
        // } else {
        //     Expression::Var(name).span(start)
        // }
    }

    pub fn parse_value(&mut self) -> ParseResult<ExprNode> {
        let start = self.peek_span();
        let mut value = self.parse_unit()?;

        while matches!(
            self.peek(),
            Token::LSqBracket
                | Token::If
                | Token::LParen
                | Token::QMark
                | Token::DoubleColon
                | Token::ExclMark
                | Token::Dot
        ) {
            match self.peek() {
                Token::LSqBracket => {
                    self.next();
                    let index = self.parse_expr()?;
                    self.expect_tok(Token::RSqBracket)?;

                    value = Expression::Index { base: value, index }
                        .into_node(start.extend(self.span()));
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
                    .into_node(start.extend(self.span()));
                }

                Token::LParen => {
                    self.next();
                    let mut params = vec![];
                    let mut named_params = vec![];

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
                                    let arg = self.parse_expr()?;

                                    named_params.push((name, arg).span(start.extend(self.span())));
                                }
                                _ => {
                                    let start = self.peek_span();
                                    let arg = self.parse_expr()?;

                                    params.push(arg);
                                }
                            }
                        } else {
                            let start = self.peek_span();
                            self.expect_or_tok(Token::Ident, "parameter name")?;
                            let name = self.slice().to_string();
                            self.expect_tok(Token::Assign)?;
                            let arg = self.parse_expr()?;

                            named_params.push((name, arg).span(start.extend(self.span())));
                        }
                        self.peek_expect_tok(Token::RParen | Token::Comma)?;
                        self.skip_tok(Token::Comma);
                    }
                    self.next();

                    let key = Expression::Call {
                        base: value,
                        params,
                        named_params,
                    }
                    .span(start.extend(self.span()))?;

                    value = key;
                }
                Token::QMark => {
                    self.next();
                    value = Expression::Maybe(Some(value)).span(start.extend(self.span()))?;
                }
                Token::DoubleColon => {
                    self.next();
                    match self.next() {
                        Token::Ident => {
                            let name = self.slice().to_string();
                            value = Expression::Associated { base: value, name }
                                .span(start.extend(self.span()))?
                        }
                        Token::LBracket => {
                            let dictlike = self.parse_dictlike()?;

                            value = Expression::Instance(value, dictlike)
                                .span(start.extend(self.span()))?;
                        }
                        _ => todo!("members + calls"),
                    }
                }
                Token::Dot => {
                    self.next();
                    match self.next() {
                        Token::Ident => {
                            let name = self.slice().to_string();

                            let key = Expression::Member { base: value, name }
                                .span(start.extend(self.span()))?;

                            value = key;
                        }
                        Token::Type => {
                            let key = Expression::TypeOf { base: value }
                                .span(start.extend(self.span()))?;

                            value = key;
                        }
                        _ => {
                            return Err(SyntaxError::ExpectedToken {
                                expected: Token::Ident.to_string(),
                                found: self.slice().to_string(),
                                area: self.make_area(self.span()),
                            });
                        }
                    }
                }
                Token::ExclMark => {
                    self.next();

                    value = Expression::TriggerFuncCall(value).span(start.extend(self.span()))?;
                }
                _ => unreachable!(),
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
            let new_span = left.area.span.extend(right.area.span);
            left = Expression::Op(left, op, right).into_node(new_span)
        }
        Ok(left)
    }

    pub fn parse_block(&mut self) -> ParseResult<Vec<StmtNode>> {
        self.expect_tok(Token::LBracket)?;
        let code = self.parse_statements()?;
        self.expect_tok(Token::RBracket)?;

        Ok(code)
    }

    pub fn parse_statement(&mut self) -> ParseResult<StmtNode> {
        let peek = self.peek();
        let start = self.peek_span();

        let is_arrow = if peek == Token::Arrow {
            self.next();
            true
        } else {
            false
        };

        let stmt = match self.peek() {
            Token::Let => {
                self.next();
                self.expect_or_tok(Token::Ident, "variable name")?;
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

                while self.peek() == Token::Else {
                    self.next();
                    if self.peek() == Token::If {
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
            Token::Try => {
                self.next();
                let try_branch = self.parse_block()?;
                self.expect_tok(Token::Catch)?;
                self.expect_or_tok(Token::Ident, "variable name")?;
                let catch_var = self.slice().to_string();
                let catch = self.parse_block()?;

                Statement::TryCatch {
                    try_branch,
                    catch,
                    catch_var,
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
                self.expect_or_tok(Token::Ident, "variable name")?;
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
                if matches!(self.peek(), Token::Eol | Token::RBracket) {
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
            Token::Type => {
                self.next();
                self.expect_tok(Token::TypeIndicator)?;
                let typ_name = self.slice()[1..].to_string();

                Statement::TypeDef(typ_name)
            }
            Token::Impl => {
                self.next();

                let typ = self.parse_expr()?;

                self.expect_tok(Token::LBracket)?;
                let dictlike = self.parse_dictlike()?;

                Statement::Impl(typ, dictlike)
            }
            Token::Ident => {
                if self.peek_many(2) == Token::Assign {
                    self.next();
                    let var = self.slice().to_string();
                    self.next();
                    let value = self.parse_expr()?;

                    Statement::Assign(var, value)
                } else {
                    Statement::Expr(self.parse_expr()?)
                }
            }
            _ => Statement::Expr(self.parse_expr()?),
        };
        if self.slice() != "}" {
            self.expect_tok(Token::Eol)?;
        }
        self.skip_tok(Token::Eol);

        let key = stmt.into_node(start.extend(self.span()), is_arrow);

        Ok(key)
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
