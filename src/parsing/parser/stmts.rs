use super::{ParseResult, Parser};
use crate::lexing::tokens::Token;
use crate::list_helper;
use crate::parsing::ast::{Statement, Statements, StmtNode, Vis};
use crate::parsing::attributes::{Attributes, IsValidOn};
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
        let start = self.peek_span();

        let attrs = if self.next_is(Token::Hashtag) {
            let mut check = self.clone();

            check.next();
            check.expect_tok(Token::LSqBracket)?;

            let mut indent = 1;

            let after_close = loop {
                match check.next() {
                    Token::LSqBracket => indent += 1,
                    Token::Eof => {
                        return Err(SyntaxError::UnmatchedToken {
                            not_found: Token::RSqBracket,
                            for_char: Token::LSqBracket,
                            area: self.make_area(start),
                        })
                    },
                    Token::RSqBracket => {
                        indent -= 1;
                        if indent == 0 {
                            break check.next();
                        }
                    },
                    _ => (),
                }
            };

            if matches!(
                after_close,
                Token::Let
                    | Token::If
                    | Token::While
                    | Token::For
                    | Token::Try
                    | Token::Return
                    | Token::Continue
                    | Token::Break
                    | Token::Type
                    | Token::Impl
                    | Token::Overload
                    | Token::Extract
                    | Token::Dbg
                    | Token::Arrow
            ) {
                self.next();
                self.parse_attributes::<Attributes>()?
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let is_arrow = if self.next_is(Token::Arrow) {
            self.next();
            true
        } else {
            false
        };

        let inner_start = self.peek_span();

        let stmt = match self.peek() {
            Token::Let => {
                self.next();
                let var = self.parse_expr(true)?;
                self.expect_tok(Token::Assign)?;
                let value = self.parse_expr(true)?;

                Statement::Let(var, value)
            },
            Token::If => {
                self.next();
                let mut branches = vec![];
                let mut else_branch = None;

                let cond = self.parse_expr(false)?;
                let code = self.parse_block()?;
                branches.push((cond, code));

                while self.next_is(Token::Else) {
                    self.next();
                    if self.next_is(Token::If) {
                        self.next();
                        let has_paren = self.skip_tok(Token::LParen);
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
                self.next();
                let cond = self.parse_expr(false)?;
                let code = self.parse_block()?;

                Statement::While { cond, code }
            },
            Token::For => {
                self.next();
                let iter_var = self.parse_unit(true)?;
                self.expect_tok(Token::In)?;
                let iterator = self.parse_expr(false)?;

                let code = self.parse_block()?;

                Statement::For {
                    iter_var,
                    iterator,
                    code,
                }
            },
            Token::Try => {
                self.next();
                let mut branches = vec![];

                let try_code = self.parse_block()?;

                let mut catch_all: Option<(Statements, CodeSpan)> = None;

                while self.next_is(Token::Catch) {
                    self.next();

                    if self.next_is(Token::LBracket) {
                        if let Some((_, s)) = catch_all {
                            return Err(SyntaxError::DuplicateCatchAll {
                                area: self.make_area(s),
                                second_area: self.make_area(self.span()),
                            });
                        }
                        let catch_span = self.span();

                        let catch_all_code = self.parse_block()?;

                        catch_all = Some((catch_all_code, catch_span))
                    } else {
                        #[allow(clippy::collapsible_else_if)]
                        let error_typ = if let Some((_, s)) = catch_all {
                            return Err(SyntaxError::CatchAllNotFinal {
                                area: self.make_area(s),
                                named_catch_area: self.make_area(self.span()),
                            });
                        } else {
                            self.next();
                            self.parse_expr(true)?
                        };

                        let catch_code = self.parse_block()?;
                        branches.push((error_typ, catch_code));
                    }
                }

                Statement::TryCatch {
                    try_code,
                    branches,
                    catch_all: catch_all.map(|(v, _)| v),
                }
            },
            Token::Return => {
                self.next();
                if matches!(
                    self.peek_strict(),
                    Token::Eol | Token::RBracket | Token::Eof | Token::Newline
                ) {
                    Statement::Return(None)
                } else {
                    let val = self.parse_expr(true)?;

                    Statement::Return(Some(val))
                }
            },
            Token::Continue => {
                self.next();

                Statement::Continue
            },
            Token::Break => {
                self.next();

                Statement::Break
            },
            Token::Type => {
                self.next();
                self.expect_tok(Token::TypeIndicator)?;
                let name = self.slice()[1..].to_string();

                Statement::TypeDef(Vis::Public(self.intern_string(name)))
            },
            Token::Private => {
                self.next();
                self.expect_tok(Token::Type)?;
                self.expect_tok(Token::TypeIndicator)?;
                let name = self.slice()[1..].to_string();

                Statement::TypeDef(Vis::Private(self.intern_string(name)))
            },
            Token::Impl => {
                self.next();
                let base = self.parse_expr(true)?;
                self.expect_tok(Token::LBracket)?;
                let items = self.parse_dictlike(true)?;

                Statement::Impl { base, items }
            },
            Token::Overload => {
                self.next();

                let tok = self.next();

                let op = if tok == Token::Unary {
                    let tok = self.next();
                    if let Some(op) = tok.to_unary_op() {
                        Operator::Unary(op)
                    } else {
                        return Err(SyntaxError::UnexpectedToken {
                            found: tok,
                            expected: "unary operator".to_string(),
                            area: self.make_area(self.span()),
                        });
                    }
                } else if let Some(op) = tok.to_bin_op() {
                    Operator::Bin(op)
                } else if let Some(op) = tok.to_assign_op() {
                    Operator::Assign(op)
                } else {
                    return Err(SyntaxError::UnexpectedToken {
                        found: tok,
                        expected: "binary operator".to_string(),
                        area: self.make_area(self.span()),
                    });
                };

                self.expect_tok(Token::LBracket)?;

                let mut macros = vec![];

                list_helper!(self, RBracket {
                    macros.push(self.parse_expr(true)?);
                });

                Statement::Overload { op, macros }
            },
            Token::Extract => {
                self.next();
                self.expect_tok(Token::Import)?;

                let import_type = self.parse_import()?;

                Statement::ExtractImport(import_type)
            },
            Token::Dbg => {
                self.next();

                Statement::Dbg(self.parse_expr(true)?)
            },
            Token::Throw => {
                self.next();

                Statement::Throw(self.parse_expr(false)?)
            },
            _ => {
                let left = self.parse_expr(true)?;
                let peek = self.peek();
                if let Some(op) = peek.to_assign_op() {
                    self.next();
                    let right = self.parse_expr(true)?;
                    Statement::AssignOp(left, op, right)
                } else {
                    Statement::Expr(left)
                }
            },
        };

        let inner_span = inner_start.extend(self.span());

        if !matches!(self.peek(), Token::RBracket)
            && !matches!(self.peek_strict(), Token::Eol | Token::Newline | Token::Eof)
        {
            return Err(SyntaxError::UnexpectedToken {
                found: self.next(),
                expected: "statement separator (`;` or newline)".to_string(),
                area: self.make_area(self.span()),
            });
        }
        self.skip_tok(Token::Eol);

        let stmt = if is_arrow {
            Statement::Arrow(Box::new(stmt.into_node(vec![], inner_span)))
        } else {
            stmt
        }
        .spanned(start.extend(self.span()));

        attrs.is_valid_on(&stmt, &self.src)?;

        Ok(stmt.value.into_node(attrs, stmt.span))
    }

    pub fn parse_statements(&mut self) -> ParseResult<Statements> {
        let mut statements = vec![];

        while !matches!(self.peek(), Token::Eof | Token::RBracket) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }

        Ok(statements)
    }
}
