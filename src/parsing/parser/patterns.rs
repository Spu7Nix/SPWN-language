use ahash::AHashMap;
use lasso::Spur;

use super::{ParseResult, Parser};
use crate::lexing::tokens::Token;
use crate::list_helper;
use crate::parsing::ast::{AssignPath, Pattern, PatternNode};
use crate::parsing::error::SyntaxError;
use crate::sources::{CodeSpan, Spannable};

impl<'a> Parser<'a> {
    fn parse_pattern_value(&mut self) -> ParseResult<PatternNode> {
        let start = self.peek_span()?;

        macro_rules! dictlike_destructure {
            () => {{
                let mut map = AHashMap::new();

                list_helper!(self, RBracket {
                    let key = match self.next()? {
                        Token::Int => self.intern_string(self.parse_int(self.slice(), 10).to_string()),
                        Token::HexInt => self.intern_string(self.parse_int(&self.slice()[2..], 16).to_string()),
                        Token::OctalInt => self.intern_string(self.parse_int(&self.slice()[2..], 8).to_string()),
                        Token::BinaryInt => self.intern_string(self.parse_int(&self.slice()[2..], 2).to_string()),
                        Token::SeximalInt => self.intern_string(self.parse_int(&self.slice()[2..], 6).to_string()),
                        Token::DozenalInt => self.intern_string(self.parse_int(&self.slice()[2..], 12).to_string()),
                        Token::String => {
                            let s = self.parse_compile_time_string()?;

                            self.intern_string(s)
                        },
                        Token::Ident => self.intern_string(self.slice()),
                        other => {
                            return Err(SyntaxError::UnexpectedToken {
                                expected: "key".into(),
                                found: other,
                                area: self.make_area(self.span()),
                            })
                        }
                    };

                    let key_span = self.span();

                    let elem = if self.next_is(Token::Colon)? {
                        self.next()?;
                        self.parse_pattern()?
                    } else {
                        PatternNode {
                            pat: Box::new(Pattern::Path {
                                var: key,
                                path: vec![],
                                is_ref: false,
                            }),
                            span: key_span,
                        }
                    };

                    map.insert(key.spanned(key_span), elem);
                });

                map
            }};
        }

        let pat = 'out_pat: {
            match self.next()? {
                Token::Mut => {
                    self.expect_tok(Token::Ident)?;
                    Pattern::Mut {
                        name: self.slice_interned(),
                        is_ref: false,
                    }
                },
                t @ (Token::Ident | Token::BinAnd) => {
                    // println!("amba {t:?}");
                    let (is_ref, name) = if t == Token::Ident {
                        (false, self.slice_interned())
                    } else {
                        if self.skip_tok(Token::Mut)? {
                            self.expect_tok(Token::Ident)?;
                            break 'out_pat Pattern::Mut {
                                name: self.slice_interned(),
                                is_ref: true,
                            };
                        }
                        self.expect_tok(Token::Ident)?;
                        (true, self.slice_interned())
                    };

                    let mut path = vec![];

                    loop {
                        match self.peek_strict()? {
                            Token::LSqBracket => {
                                self.next()?;
                                let index = self.parse_expr(true)?;
                                self.expect_tok(Token::RSqBracket)?;

                                path.push(AssignPath::Index(index));
                            },
                            _ => match self.peek()? {
                                Token::Dot => {
                                    self.next()?;
                                    self.expect_tok(Token::Ident)?;
                                    let member = self.slice_interned();
                                    path.push(AssignPath::Member(member));
                                },
                                Token::DoubleColon => {
                                    self.next()?;
                                    self.expect_tok(Token::Ident)?;
                                    let member = self.slice_interned();
                                    path.push(AssignPath::Associated(member));
                                },
                                _ => break,
                            },
                        }
                    }

                    Pattern::Path {
                        var: name,
                        path,
                        is_ref,
                    }
                },
                Token::LSqBracket => {
                    let mut v = vec![];
                    list_helper!(self, RSqBracket {
                        v.push(self.parse_pattern()?);
                    });
                    Pattern::ArrayDestructure(v)
                },
                Token::LBracket => {
                    let map = dictlike_destructure!();
                    Pattern::DictDestructure(map)
                },
                Token::TypeIndicator => {
                    let typ = self.intern_string(&self.slice()[1..]);
                    let span = self.span();
                    if self.skip_tok(Token::DoubleColon)? {
                        let map = dictlike_destructure!();
                        Pattern::InstanceDestructure(typ, map)
                    } else if self.skip_tok(Token::Arrow)? {
                        let ret = self.parse_pattern()?;
                        Pattern::MacroPattern {
                            args: vec![PatternNode {
                                pat: Box::new(Pattern::Type(typ)),
                                span,
                            }],
                            ret,
                        }
                    } else {
                        Pattern::Type(typ)
                    }
                },
                Token::Eq => {
                    let e = self.parse_value(true)?;
                    Pattern::Eq(e)
                },
                Token::Neq => {
                    let e = self.parse_value(true)?;
                    Pattern::Neq(e)
                },
                Token::Gt => {
                    let e = self.parse_value(true)?;
                    Pattern::Gt(e)
                },
                Token::Gte => {
                    let e = self.parse_value(true)?;
                    Pattern::Gte(e)
                },
                Token::Lt => {
                    let e = self.parse_value(true)?;
                    Pattern::Lt(e)
                },
                Token::Lte => {
                    let e = self.parse_value(true)?;
                    Pattern::Lte(e)
                },
                Token::In => {
                    let e = self.parse_value(true)?;
                    Pattern::In(e)
                },
                Token::LParen => {
                    if self.skip_tok(Token::RParen)? {
                        if self.skip_tok(Token::Arrow)? {
                            let ret = self.parse_pattern()?;
                            Pattern::MacroPattern { args: vec![], ret }
                        } else {
                            Pattern::Empty
                        }
                    } else {
                        let p = self.parse_pattern()?;
                        if self.skip_tok(Token::Comma)? {
                            let mut args = vec![p];
                            list_helper!(self, RParen {
                                args.push(self.parse_pattern()?);
                            });
                            self.expect_tok(Token::Arrow)?;
                            let ret = self.parse_pattern()?;
                            Pattern::MacroPattern { args, ret }
                        } else {
                            self.expect_tok(Token::RParen)?;
                            if self.skip_tok(Token::Arrow)? {
                                let ret = self.parse_pattern()?;
                                Pattern::MacroPattern { args: vec![p], ret }
                            } else {
                                *p.pat
                            }
                        }
                    }
                },
                Token::QMark => Pattern::MaybeDestructure(None),
                Token::Any => Pattern::Any,
                other => {
                    return Err(SyntaxError::UnexpectedToken {
                        expected: "pattern".into(),
                        found: other,
                        area: self.make_area(start),
                    })
                },
            }
        };

        let mut node = PatternNode {
            pat: Box::new(pat),
            span: start.extend(self.span()),
        };

        loop {
            let start_span = node.span;
            let pat = match self.peek_strict()? {
                Token::QMark => {
                    self.next()?;
                    Pattern::MaybeDestructure(Some(node))
                },
                Token::LSqBracket => {
                    self.next()?;
                    if self.skip_tok(Token::RSqBracket)? {
                        Pattern::ArrayPattern(
                            node,
                            PatternNode {
                                pat: Box::new(Pattern::Any),
                                span: start_span,
                            },
                        )
                    } else {
                        let pat = self.parse_pattern()?;

                        self.expect_tok(Token::RSqBracket)?;
                        Pattern::ArrayPattern(node, pat)
                    }
                },
                Token::LBracket
                    if self.next_are(&[Token::LBracket, Token::Colon, Token::RBracket])? =>
                {
                    self.next()?;
                    self.next()?;
                    self.next()?;
                    Pattern::DictPattern(node)
                },
                _ => {
                    break;
                },
            };
            node = PatternNode {
                pat: Box::new(pat),
                span: start_span.extend(self.span()),
            };
        }

        Ok(node)
    }

    fn parse_pattern_op(&mut self, prec: usize) -> ParseResult<PatternNode> {
        let next_prec = if prec == 2 { None } else { Some(prec + 1) };

        let mut left = match next_prec {
            Some(next_prec) => self.parse_pattern_op(next_prec)?,
            None => self.parse_pattern_value()?,
        };

        while matches!(
            (self.peek()?, prec),
            (Token::Colon, 0) | (Token::BinOr, 1) | (Token::BinAnd, 2)
        ) {
            let peek = self.next()?;
            let right = match next_prec {
                Some(next_prec) => self.parse_pattern_op(next_prec)?,
                None => self.parse_pattern_value()?,
            };
            let new_span = left.span.extend(right.span);

            left = PatternNode {
                span: new_span,
                pat: Box::new({
                    let p = match peek {
                        Token::Colon => Pattern::Both,
                        Token::BinOr => Pattern::Either,
                        Token::BinAnd => Pattern::Both,
                        _ => unreachable!(),
                    };
                    p(left, right)
                }),
            };
        }

        Ok(left)
    }

    pub fn parse_pattern(&mut self) -> ParseResult<PatternNode> {
        let mut left = self.parse_pattern_op(0)?;

        // println!("alcuckgya {:#?}", left);

        loop {
            let start_span = left.span;
            let pat = match self.peek_strict()? {
                Token::If => {
                    self.next()?;
                    let cond = self.parse_expr(false)?;
                    Pattern::IfGuard { pat: left, cond }
                },
                _ => {
                    break;
                },
            };
            left = PatternNode {
                pat: Box::new(pat),
                span: start_span.extend(self.span()),
            };
        }

        Ok(left)
    }
}
