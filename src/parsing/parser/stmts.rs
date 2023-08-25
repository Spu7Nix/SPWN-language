use super::{ParseResult, Parser};
use crate::lexing::tokens::Token;
use crate::list_helper;
use crate::parsing::ast::{Statement, Statements, StmtNode, Vis};
use crate::parsing::error::SyntaxError;
use crate::parsing::operators::operators::Operator;
use crate::sources::{CodeSpan, Spannable};

impl Parser<'_> {
    pub fn parse_block(&mut self) -> ParseResult<Statements> {
        self.expect_tok(Token::LBracket)?;
        let code = self.parse_statements()?;
        self.expect_tok(Token::RBracket)?;

        Ok(code)
    }

    pub fn parse_statement(&mut self) -> ParseResult<StmtNode> {
        let start = self.peek_span()?;

        let mut attrs = self.parse_outer_attributes()?;

        let is_arrow = if self.next_is(Token::Arrow)? {
            self.next()?;
            true
        } else {
            false
        };

        let inner_start = self.peek_span()?;

        let stmt = match self.peek()? {
            Token::If => {
                self.next()?;
                let mut branches = vec![];
                let mut else_branch = None;

                let cond = self.parse_expr(false)?;
                let code = self.parse_block()?;
                branches.push((cond, code));

                while self.next_is(Token::Else)? {
                    self.next()?;
                    if self.next_is(Token::If)? {
                        self.next()?;
                        let has_paren = self.skip_tok(Token::LParen)?;
                        let cond = self.parse_expr(false)?;
                        if has_paren {
                            self.expect_tok(Token::RParen)?;
                        }
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
            },
            Token::While => {
                self.next()?;
                let cond = self.parse_expr(false)?;
                let code = self.parse_block()?;

                Statement::While { cond, code }
            },
            Token::For => {
                self.next()?;
                let iter = self.parse_pattern()?;
                self.expect_tok(Token::In)?;
                let iterator = self.parse_expr(false)?;

                let code = self.parse_block()?;

                Statement::For {
                    iter,
                    iterator,
                    code,
                }
            },
            Token::Try => {
                self.next()?;
                let try_code = self.parse_block()?;

                self.expect_tok(Token::Catch)?;

                let catch_pat = if !self.next_is(Token::LBracket)? {
                    let v = Some(self.parse_pattern()?);
                    v
                } else {
                    None
                };

                let catch_code = self.parse_block()?;

                Statement::TryCatch {
                    try_code,
                    catch_code,
                    catch_pat,
                }
            },
            Token::Return => {
                self.next()?;
                if matches!(
                    self.peek_strict()?,
                    Token::Eol | Token::RBracket | Token::Eof | Token::Newline
                ) {
                    Statement::Return(None)
                } else {
                    let val = self.parse_expr(true)?;

                    Statement::Return(Some(val))
                }
            },
            Token::Continue => {
                self.next()?;

                Statement::Continue
            },
            Token::Break => {
                self.next()?;

                Statement::Break
            },
            t @ (Token::Private | Token::Type) => {
                let vis = if matches!(t, Token::Private) {
                    self.next()?;
                    Vis::Private
                } else {
                    Vis::Public
                };

                self.next()?;
                self.expect_tok(Token::TypeIndicator)?;
                let name = self.slice()[1..].to_string();

                if self.skip_tok(Token::LBracket)? {
                    let members = self.parse_dictlike(true)?;

                    Statement::TypeDef {
                        name: vis(self.intern_string(name)),
                        members: Some(members),
                    }
                } else {
                    let span = self.span();
                    self.deprecated_features.empty_type_def.insert(span);

                    Statement::TypeDef {
                        members: None,
                        name: vis(self.intern_string(name)),
                    }
                }
            },
            Token::Impl => {
                self.next()?;
                self.expect_tok(Token::TypeIndicator)?;
                let name_span = self.span();
                let name = self.slice()[1..].to_string();
                self.expect_tok(Token::LBracket)?;
                let items = self.parse_dictlike(true)?;

                Statement::Impl {
                    name: self.intern_string(name).spanned(name_span),
                    items,
                }
            },
            Token::Overload => {
                self.next()?;

                let tok = self.next()?;

                let op = if tok == Token::Unary {
                    let tok = self.next()?;
                    if let Some(op) = tok.to_unary_op() {
                        Operator::Unary(op)
                    } else {
                        return Err(SyntaxError::UnexpectedToken {
                            found: tok,
                            expected: "unary operator".into(),
                            area: self.make_area(self.span()),
                        });
                    }
                } else if let Some(op) = tok.to_bin_op() {
                    Operator::Bin(op)
                } else if let Some(op) = tok.to_assign_op() {
                    Operator::Assign(op)
                } else if tok == Token::Assign {
                    Operator::EqAssign
                } else {
                    return Err(SyntaxError::UnexpectedToken {
                        found: tok,
                        expected: "binary operator".into(),
                        area: self.make_area(self.span()),
                    });
                };

                self.expect_tok(Token::LBracket)?;

                let mut macros = vec![];

                list_helper!(self, RBracket {
                    let vis = if self.next_is(Token::Private)? {
                        self.next()?;
                        Vis::Private
                    } else {
                        Vis::Public
                    };

                    macros.push(vis(self.parse_expr(true)?));
                });

                Statement::Overload { op, macros }
            },
            Token::Throw => {
                self.next()?;

                Statement::Throw(self.parse_expr(false)?)
            },
            _ => {
                let mut check = self.clone();

                match check.parse_pattern() {
                    Ok(pat) => {
                        let tok = check.peek()?;
                        let ex = if tok == Token::Assign {
                            check.next()?;
                            self.lexer = check.lexer;

                            let mut e = self.parse_expr(true)?;
                            e.attributes.extend(attrs);

                            Statement::Assign(pat, e)
                        } else if let Some(op) = tok.to_assign_op() {
                            check.next()?;
                            self.lexer = check.lexer;

                            let mut e = self.parse_expr(true)?;
                            e.attributes.extend(attrs);

                            Statement::AssignOp(pat, op, e)
                        } else {
                            let mut e = self.parse_expr(true)?;
                            e.attributes.extend(attrs);

                            Statement::Expr(e)
                        };

                        self.deprecated_features.extend(check.deprecated_features);
                        attrs = vec![];

                        ex
                    },
                    Err(pattern_err) => {
                        let e = self.parse_expr(true)?;
                        if self.next_is(Token::Assign)? {
                            return Err(pattern_err);
                        }
                        Statement::Expr(e)
                    },
                }
            },
        };

        let inner_span = inner_start.extend(self.span());

        if !matches!(self.peek()?, Token::RBracket)
            && !matches!(
                self.peek_strict()?,
                Token::Eol | Token::Newline | Token::Eof
            )
        {
            return Err(SyntaxError::UnexpectedToken {
                found: self.next()?,
                expected: "statement separator (`;` or newline)".to_string(),
                area: self.make_area(self.span()),
            });
        }
        self.skip_tok(Token::Eol)?;

        let stmt = if is_arrow {
            Statement::Arrow(Box::new(stmt.into_node(vec![], inner_span)))
        } else {
            stmt
        }
        .spanned(start.extend(self.span()));

        self.check_attributes(&attrs, Some(stmt.value.target().spanned(stmt.span)))?;

        Ok(stmt.value.into_node(attrs, stmt.span))
    }

    pub fn parse_statements(&mut self) -> ParseResult<Statements> {
        let mut statements = vec![];

        while !matches!(self.peek()?, Token::Eof | Token::RBracket) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }

        Ok(statements)
    }
}
