use ahash::AHashMap;
use lasso::Spur;

use super::{ParseResult, Parser};
use crate::lexing::tokens::Token;
use crate::list_helper;
use crate::parsing::ast::{AssignPath, Pattern, PatternNode};
use crate::parsing::error::SyntaxError;
use crate::sources::{CodeSpan, Spannable};

// mod modify_pattern {
//     use super::*;

//     impl Parser<'_> {

//     }
// }

impl<'a> Parser<'a> {
    pub fn parse_pattern_value(&mut self) -> ParseResult<PatternNode> {
        let start = self.peek_span();

        macro_rules! dictlike_destructure {
            () => {{
                let mut map = AHashMap::new();

                list_helper!(self, RBracket {
                    let key = match self.next() {
                        Token::Int => self.intern_string(self.parse_int(self.slice()).to_string()),
                        Token::String => self.parse_plain_string(self.slice(), self.span())?,
                        Token::Ident => self.intern_string(self.slice()),
                        other => {
                            return Err(SyntaxError::UnexpectedToken {
                                expected: "key".into(),
                                found: other,
                                area: self.make_area(self.span()),
                            })
                        }
                    };

                    let elem = if self.next_is(Token::Colon) {
                        self.next();
                        Some(self.parse_pattern()?)
                    } else {
                        None
                    };

                    map.insert(key, elem);
                });

                map
            }};
        }

        let pat = 'out_pat: {
            match self.next() {
                Token::Mut => {
                    self.expect_tok(Token::Ident)?;
                    Pattern::Mut {
                        name: self.slice_interned(),
                        is_ref: false,
                    }
                },
                t @ (Token::Ident | Token::BinAnd) => {
                    let (is_ref, name) = if t == Token::Ident {
                        (false, self.slice_interned())
                    } else {
                        if self.skip_tok(Token::Mut) {
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
                        match self.peek_strict() {
                            Token::LSqBracket => {
                                self.next();
                                let index = self.parse_expr(true)?;
                                self.expect_tok(Token::RSqBracket)?;

                                path.push(AssignPath::Index(index));
                            },
                            _ => match self.peek() {
                                Token::Dot => {
                                    self.next();
                                    self.expect_tok(Token::Ident)?;
                                    let member = self.slice_interned();
                                    path.push(AssignPath::Member(member));
                                },
                                Token::DoubleColon => {
                                    self.next();
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
                    if self.skip_tok(Token::DoubleColon) {
                        let map = dictlike_destructure!();
                        Pattern::InstanceDestructure(typ, map)
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
                    let p = self.parse_pattern()?;
                    self.expect_tok(Token::RParen)?;
                    *p.pat
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
            let pat = match self.peek_strict() {
                Token::QMark => {
                    self.next();
                    Pattern::MaybeDestructure(Some(node))
                },
                Token::LSqBracket => {
                    self.next();
                    if self.skip_tok(Token::RSqBracket) {
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
                Token::LBracket => {
                    self.next();
                    self.expect_tok(Token::RBracket)?;
                    Pattern::DictPattern(node)
                },
                h => {
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

    pub fn parse_pattern(&mut self) -> ParseResult<PatternNode> {
        let mut left = self.parse_pattern_value()?;

        loop {
            left = match self.peek() {
                Token::BinAnd | Token::Colon => {
                    self.next();
                    let right = self.parse_pattern_value()?;
                    PatternNode {
                        span: left.span.extend(self.span()),
                        pat: Box::new(Pattern::Both(left, right)),
                    }
                },
                Token::BinOr => {
                    self.next();
                    let right = self.parse_pattern_value()?;
                    PatternNode {
                        span: left.span.extend(self.span()),
                        pat: Box::new(Pattern::Either(left, right)),
                    }
                },
                _ => break,
            }
        }
        Ok(left)
    }
}
