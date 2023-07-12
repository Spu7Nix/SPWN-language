use lasso::Spur;

use super::{ParseResult, Parser};
use crate::gd::ids::IDClass;
use crate::lexing::tokens::Token;
use crate::list_helper;
use crate::parsing::ast::{
    ExprNode, Expression, MacroArg, MacroCode, MatchBranch, MatchBranchCode, Pattern, PatternNode,
    StringContent, StringType,
};
use crate::parsing::attributes::{Attributes, IsValidOn};
use crate::parsing::error::SyntaxError;
use crate::parsing::operators::operators::{self, unary_prec};
use crate::sources::{Spannable, Spanned};

impl Parser<'_> {
    pub fn parse_unit(&mut self, allow_macros: bool) -> ParseResult<ExprNode> {
        let attrs = self.parse_attributes()?;

        dbg!(attrs);

        let peek = self.peek()?;
        let start = self.peek_span()?;

        let unary;

        let expr = 'out_expr: {
            match peek {
                Token::Int => {
                    self.next()?;
                    Expression::Int(self.parse_int(self.slice(), 10)).spanned(start)
                },
                Token::HexInt => {
                    self.next()?;
                    Expression::Int(self.parse_int(&self.slice()[2..], 16)).spanned(start)
                },
                Token::OctalInt => {
                    self.next()?;
                    Expression::Int(self.parse_int(&self.slice()[2..], 8)).spanned(start)
                },
                Token::BinaryInt => {
                    self.next()?;
                    Expression::Int(self.parse_int(&self.slice()[2..], 2)).spanned(start)
                },
                Token::SeximalInt => {
                    self.next()?;
                    Expression::Int(self.parse_int(&self.slice()[2..], 6)).spanned(start)
                },
                Token::DozenalInt => {
                    self.next()?;
                    Expression::Int(self.parse_int(&self.slice()[3..], 12)).spanned(start)
                },
                Token::GoldenFloat => {
                    self.next()?;
                    Expression::Float(self.parse_golden_float(&self.slice()[3..])).spanned(start)
                },
                Token::Float => {
                    self.next()?;
                    Expression::Float(self.slice().replace('_', "").parse::<f64>().unwrap())
                        .spanned(start)
                },
                Token::String => {
                    let t = self.next()?;
                    Expression::String(self.parse_string(t)?).spanned(start)
                },
                Token::StringFlags => {
                    let t = self.next()?;
                    Expression::String(self.parse_string(t)?).spanned(start)
                },
                Token::RawString => {
                    let t = self.next()?;
                    Expression::String(self.parse_string(t)?).spanned(start)
                },
                Token::ArbitraryGroupID => {
                    self.next()?;
                    Expression::Id(IDClass::Group, None).spanned(start)
                },
                Token::ArbitraryItemID => {
                    self.next()?;
                    Expression::Id(IDClass::Item, None).spanned(start)
                },
                Token::ArbitraryChannelID => {
                    self.next()?;
                    Expression::Id(IDClass::Channel, None).spanned(start)
                },
                Token::ArbitraryBlockID => {
                    self.next()?;
                    Expression::Id(IDClass::Block, None).spanned(start)
                },
                Token::GroupID => {
                    self.next()?;
                    self.parse_id(self.slice(), IDClass::Group).spanned(start)
                },
                Token::ItemID => {
                    self.next()?;
                    self.parse_id(self.slice(), IDClass::Item).spanned(start)
                },
                Token::ChannelID => {
                    self.next()?;
                    self.parse_id(self.slice(), IDClass::Channel).spanned(start)
                },
                Token::BlockID => {
                    self.next()?;
                    self.parse_id(self.slice(), IDClass::Block).spanned(start)
                },
                Token::Dollar => {
                    self.next()?;

                    Expression::Builtins.spanned(start)
                },
                Token::True => {
                    self.next()?;
                    Expression::Bool(true).spanned(start)
                },
                Token::False => {
                    self.next()?;
                    Expression::Bool(false).spanned(start)
                },
                Token::Epsilon => {
                    self.next()?;
                    Expression::Epsilon.spanned(start)
                },
                Token::Ident => {
                    self.next()?;
                    let var_name = self.slice_interned();
                    let var_span = self.span();

                    if matches!(self.peek_strict()?, Token::FatArrow | Token::Arrow) {
                        let ret_type = if self.next_is(Token::Arrow)? {
                            self.next()?;
                            let r = Some(self.parse_pattern()?);
                            self.expect_tok(Token::FatArrow)?;
                            r
                        } else {
                            self.next()?;
                            None
                        };

                        let code = MacroCode::Lambda(self.parse_expr(allow_macros)?);

                        Expression::Macro {
                            args: vec![MacroArg::Single {
                                pattern: PatternNode {
                                    pat: Box::new(Pattern::Path {
                                        var: var_name,
                                        path: vec![],
                                        is_ref: false,
                                    }),
                                    span: var_span,
                                },
                                default: None,
                            }],
                            code,
                            ret_pat: ret_type,
                        }
                        .spanned(start.extend(self.span()))
                    } else {
                        Expression::Var(var_name).spanned(start)
                    }
                },
                Token::TypeIndicator => {
                    self.next()?;
                    let name = self.slice()[1..].to_string();
                    Expression::Type(self.intern_string(name)).spanned(start)
                },
                Token::LParen => {
                    self.next()?;

                    let mut check = self.clone();
                    let mut indent = 1;

                    let after_close = loop {
                        match check.next()? {
                            Token::LParen => indent += 1,
                            Token::Eof => {
                                return Err(SyntaxError::UnmatchedToken {
                                    not_found: Token::RParen,
                                    for_char: Token::LParen,
                                    area: self.make_area(start),
                                })
                            },
                            Token::RParen => {
                                indent -= 1;
                                if indent == 0 {
                                    break check.next()?;
                                }
                            },
                            _ => (),
                        }
                    };

                    match after_close {
                        Token::FatArrow | Token::LBracket | Token::Arrow if allow_macros => (),
                        _ => {
                            if self.next_is(Token::RParen)? {
                                self.next()?;
                                break 'out_expr Expression::Empty
                                    .spanned(start.extend(self.span()));
                            }
                            let mut inner = self.parse_expr(true)?;
                            self.expect_tok(Token::RParen)?;
                            inner.span = start.extend(self.span());
                            return Ok(inner);
                        },
                    }

                    let mut args = vec![];

                    let mut first_spread_span = None;

                    let mut index = 0;

                    list_helper!(
                        self,
                        is_first,
                        RParen {
                            let is_spread = self.skip_tok(Token::Spread)?;

                            if is_spread {
                                if let Some(prev_s) = first_spread_span {
                                    return Err(SyntaxError::MultipleSpreadArguments {
                                        area: self.make_area(self.span()),
                                        prev_area: self.make_area(prev_s)
                                    })
                                } else {
                                    first_spread_span = Some(self.span());
                                }
                            }

                            let pat = self.parse_pattern()?;

                            if pat.pat.is_self(&self.interner) {
                                if !is_first {
                                    return Err(SyntaxError::SelfArgumentNotFirst {
                                        area: self.make_area(self.span()),
                                        pos: index,
                                    })
                                }
                                if is_spread {
                                    return Err(SyntaxError::SelfArgumentCannotBeSpread{
                                        area: self.make_area(self.span())
                                    })
                                }
                            }


                            if is_spread {
                                args.push(MacroArg::Spread { pattern: pat })
                            } else {
                                let default = if !is_first && self.skip_tok(Token::Assign)? {
                                    Some(self.parse_expr(true)?)
                                } else {
                                    None
                                };
                                args.push(MacroArg::Single { pattern: pat, default })
                            }

                            index += 1;

                        }
                    );

                    let ret_type = if self.next_is(Token::Arrow)? {
                        self.next()?;
                        Some(self.parse_pattern()?)
                    } else {
                        None
                    };

                    let code = if self.next_is(Token::FatArrow)? {
                        self.next()?;
                        MacroCode::Lambda(self.parse_expr(allow_macros)?)
                    } else {
                        MacroCode::Normal(self.parse_block()?)
                    };

                    Expression::Macro {
                        args,
                        code,
                        ret_pat: ret_type,
                    }
                    .spanned(start.extend(self.span()))
                },
                Token::LSqBracket => {
                    self.next()?;

                    let mut elems = vec![];

                    list_helper!(self, RSqBracket {
                        elems.push(self.parse_expr(true)?);
                    });

                    Expression::Array(elems).spanned(start.extend(self.span()))
                },
                // typ @ (Token::Obj | Token::Trigger) => {
                //     self.next()?;

                //     self.expect_tok(Token::LBracket)?;

                //     let mut items: Vec<(Spanned<ObjKeyType>, ExprNode)> = vec![];

                //     list_helper!(self, RBracket {
                //         let key = match self.next() {
                //             Token::Int => ObjKeyType::Num(self.parse_int(self.slice()) as u8),
                //             Token::Ident => ObjKeyType::Name(OBJECT_KEYS[self.slice()]),
                //             other => {
                //                 return Err(SyntaxError::UnexpectedToken {
                //                     expected: "key".into(),
                //                     found: other,
                //                     area: self.make_area(self.span()),
                //                 })
                //             }
                //         };

                //         let key_span = self.span();
                //         self.expect_tok(Token::Colon)?;
                //         items.push((key.spanned(key_span), self.parse_expr(true)?));
                //     });

                //     Expression::Obj(
                //         match typ {
                //             Token::Obj => ObjectType::Object,
                //             Token::Trigger => ObjectType::Trigger,
                //             _ => unreachable!(),
                //         },
                //         items,
                //     )
                //     .spanned(start.extend(self.span()))
                // },
                Token::LBracket => {
                    self.next()?;

                    Expression::Dict(self.parse_dictlike(false)?).spanned(start.extend(self.span()))
                },
                Token::QMark => {
                    self.next()?;

                    Expression::Maybe(None).spanned(start.extend(self.span()))
                },
                Token::TrigFnBracket => {
                    self.next()?;

                    let code = self.parse_statements()?;
                    self.expect_tok(Token::RBracket)?;

                    Expression::TriggerFunc { code }.spanned(start.extend(self.span()))
                },
                Token::Import => {
                    self.next()?;

                    let import_type = self.parse_import()?;

                    Expression::Import(import_type).spanned(start.extend(self.span()))
                },
                Token::Match => {
                    self.next()?;

                    let v = self.parse_expr(true)?;
                    self.expect_tok(Token::LBracket)?;

                    let mut branches = vec![];

                    list_helper!(self, RBracket {

                        let pattern = self.parse_pattern()?;
                        self.expect_tok(Token::FatArrow)?;

                        let branch = if self.next_is(Token::LBracket)? {
                            self.next()?;
                            let stmts = self.parse_statements()?;
                            self.expect_tok(Token::RBracket)?;
                            MatchBranchCode::Block(stmts)
                        } else {
                            let expr = self.parse_expr(true)?;
                            MatchBranchCode::Expr(expr)
                        };

                        branches.push(MatchBranch { pattern, code: branch });
                    });
                    Expression::Match { value: v, branches }.spanned(start.extend(self.span()))
                },
                Token::Dbg => {
                    self.next()?;

                    Expression::Dbg(self.parse_expr(true)?).spanned(start.extend(self.span()))
                },
                unary_op
                    if {
                        unary = unary_prec(unary_op);
                        unary.is_some()
                    } =>
                {
                    self.next()?;
                    let unary_prec = unary.unwrap();
                    let next_prec = operators::next_infix(unary_prec);
                    let val = match next_prec {
                        Some(next_prec) => self.parse_op(next_prec, allow_macros)?,
                        None => self.parse_value(allow_macros)?,
                    };

                    Expression::Unary(unary_op.to_unary_op().unwrap(), val)
                        .spanned(start.extend(self.span()))
                },

                other => {
                    return Err(SyntaxError::UnexpectedToken {
                        expected: "expression".into(),
                        found: other,
                        area: self.make_area(start),
                    })
                },
            }
        };

        // attrs.is_valid_on(&expr, &self.src)?;

        Ok(expr.value.into_node(vec![], expr.span))
    }

    pub fn parse_value(&mut self, allow_macros: bool) -> ParseResult<ExprNode> {
        let mut value = self.parse_unit(allow_macros)?;

        loop {
            let prev_span = value.span;

            value = match self.peek_strict()? {
                Token::LSqBracket => {
                    self.next()?;
                    let index = self.parse_expr(true)?;
                    self.expect_tok(Token::RSqBracket)?;

                    Expression::Index { base: value, index }
                }
                Token::QMark => {
                    self.next()?;

                    Expression::Maybe(Some(value))
                }
                Token::ExclMark => {
                    self.next()?;

                    Expression::TriggerFuncCall(value)
                }
                Token::If => {
                    // if there is a newline, treat as separate statement
                    if self.peek_strict()? == Token::Newline {
                        break;
                    }
                    self.next()?;
                    let cond = self.parse_expr(allow_macros)?;
                    self.expect_tok(Token::Else)?;
                    let if_false = self.parse_expr(allow_macros)?;

                    Expression::Ternary {
                        cond,
                        if_true: value,
                        if_false,
                    }
                }
                Token::Is => {
                    self.next()?;
                    let pat = self.parse_pattern()?;

                    Expression::Is(value, pat)
                }
                Token::LParen => {
                    self.next()?;

                    let mut params = vec![];
                    let mut named_params: Vec<(Spanned<Spur>, ExprNode)> = vec![];

                    let mut parsing_named = None;

                    list_helper!(self, RParen {
                        if self.next_are(&[Token::Ident, Token::Assign])? {
                            self.next()?;
                            let start = self.span();
                            let name = self.slice_interned();

                            if let Some((prev, _)) = named_params.iter().find(|(s, _)| s.value == name) {
                                return Err(SyntaxError::DuplicateKeywordArg { name: self.resolve(&name).to_string(), prev_area: self.make_area(prev.span), area: self.make_area(self.span()) })
                            }

                            self.next()?;

                            let value = self.parse_expr(true)?;
                            parsing_named = Some(start.extend(self.span()));

                            named_params.push((name.spanned(start), value));
                        } else {

                            let value = self.parse_expr(true)?;

                            if let Some(s) = parsing_named {
                                return Err(SyntaxError::PositionalArgAfterKeyword { keyword_area: self.make_area(s), area: self.make_area(value.span) })
                            }

                            params.push(value);
                        }
                    });

                    Expression::Call {
                        base: value,
                        params,
                        named_params,
                    }
                }
                _ => match self.peek()? {
                    Token::Dot => {
                        self.next()?;
                        match self.next()? {
                            Token::Ident => {
                                let name = self.slice_interned();
                                Expression::Member {
                                    base: value,
                                    name: name.spanned(self.span()),
                                }
                            }
                            Token::TypeIndicator => {
                                let name = self.slice()[1..].to_string();
                                Expression::TypeMember {
                                    base: value,
                                    name: self.intern_string(name).spanned(self.span()),
                                }
                            }
                            Token::Type => Expression::Typeof(value),
                            other => {
                                return Err(SyntaxError::UnexpectedToken {
                                    expected: "member name".into(),
                                    found: other,
                                    area: self.make_area(self.span()),
                                })
                            }
                        }
                    }
                    Token::DoubleColon => {
                        self.next()?;
                        match self.next()? {
                            Token::Ident => {
                                let name = self.slice_interned();
                                Expression::Associated {
                                    base: value,
                                    name: name.spanned(self.span()),
                                }
                            }
                            Token::LBracket => {
                                let items = self.parse_dictlike(true)?;
                                Expression::Instance { base: value, items }
                            }
                            other => {
                                return Err(SyntaxError::UnexpectedToken {
                                    expected: "associated member name or instance fields".into(),
                                    found: other,
                                    area: self.make_area(self.span()),
                                })
                            }
                        }
                    }
                    // Token::C
                    _ => break,
                }
            }.into_node(vec![], prev_span.extend(self.span()));
        }
        Ok(value)
    }

    pub fn parse_expr(&mut self, allow_macros: bool) -> ParseResult<ExprNode> {
        self.parse_op(0, allow_macros)
    }

    pub fn parse_op(&mut self, prec: usize, allow_macros: bool) -> ParseResult<ExprNode> {
        let next_prec = operators::next_infix(prec);

        let mut left = match next_prec {
            Some(next_prec) => self.parse_op(next_prec, allow_macros)?,
            None => self.parse_value(allow_macros)?,
        };

        while operators::is_infix_prec(self.peek()?, prec) {
            let op = self.next()?;
            let right = if operators::prec_type(prec) == operators::OpType::Left {
                match next_prec {
                    Some(next_prec) => self.parse_op(next_prec, allow_macros)?,
                    None => self.parse_value(allow_macros)?,
                }
            } else {
                self.parse_op(prec, allow_macros)?
            };
            let new_span = left.span.extend(right.span);
            left = Expression::Op(left, op.to_bin_op().unwrap(), right).into_node(vec![], new_span)
        }

        Ok(left)
    }
}
