use ahash::AHashMap;
use lasso::Spur;

use super::{ParseResult, Parser};
use crate::lexing::tokens::Token;
use crate::list_helper;
use crate::parsing::ast::{
    AssignPath, DestructurePattern, ModifyPattern, ModifyPatternNode, PatternNode,
};
use crate::parsing::error::SyntaxError;
use crate::sources::Spannable;

// mod modify_pattern {
//     use super::*;

//     impl Parser<'_> {

//     }
// }

impl<'a> Parser<'a> {
    // pub fn parse_destructure_pattern<T, P>(
    //     &mut self,
    //     pattern_fn: fn(&mut Parser<'a>) -> ParseResult<P>,
    // ) -> ParseResult<DestructurePattern<T, P, Spur>> {
    //     Ok(match self.next() {
    //         Token::LSqBracket => {
    //             self.next();
    //             let mut v = vec![];
    //             list_helper!(self, RSqBracket {
    //                 let p = pattern_fn(self)?;
    //                 v.push(p);
    //             });
    //             DestructurePattern::Array(v)
    //         },
    //         Token::LBracket => {
    //             self.next();
    //             let mut map = AHashMap::new();

    //             list_helper!(self, RBracket {
    //                 let key = match self.next() {
    //                     Token::Int => self.intern_string(self.parse_int(self.slice()).to_string()),
    //                     Token::String => self.parse_plain_string(self.slice(), self.span())?,
    //                     Token::Ident => self.intern_string(self.slice()),
    //                     other => {
    //                         return Err(SyntaxError::UnexpectedToken {
    //                             expected: "key".into(),
    //                             found: other,
    //                             area: self.make_area(self.span()),
    //                         })
    //                     }
    //                 };

    //                 let elem = if self.next_is(Token::Colon) {
    //                     self.next();
    //                     Some(pattern_fn(self)?)
    //                 } else {
    //                     None
    //                 };

    //                 map.insert(key, elem);
    //             });

    //             DestructurePattern::Dict(map)
    //         },
    //         _ => todo!(),
    //     })
    // }

    // pub fn parse_modify_pattern_unit(&mut self) -> ParseResult<ModifyPatternNode> {
    //     let start = self.peek_span();
    //     let expr = match self.peek() {
    //         Token::Ident => {
    //             self.next();
    //             let v = self.slice_interned();
    //             ModifyPattern::Var(v).spanned(start.extend(self.span()))
    //         },
    //         Token::TypeIndicator => {
    //             self.next();
    //             let name = self.slice()[1..].to_string();
    //             let s = self.intern_string(name);

    //             self.expect_tok(Token::DoubleColon)?;
    //             self.expect_tok(Token::LBracket)?;
    //             let mut map = AHashMap::new();

    //             list_helper!(self, RBracket {
    //                 let key = match self.next() {
    //                     Token::Int => self.intern_string(self.parse_int(self.slice()).to_string()),
    //                     Token::String => self.parse_plain_string(self.slice(), self.span())?,
    //                     Token::Ident => self.intern_string(self.slice()),
    //                     other => {
    //                         return Err(SyntaxError::UnexpectedToken {
    //                             expected: "key".into(),
    //                             found: other,
    //                             area: self.make_area(self.span()),
    //                         })
    //                     }
    //                 };

    //                 let elem = if self.next_is(Token::Colon) {
    //                     self.next();
    //                     Some(self.parse_modify_pattern()?)
    //                 } else {
    //                     None
    //                 };

    //                 map.insert(key, elem);
    //             });

    //             ModifyPattern::Destructure(DestructurePattern::Instance(s, map))
    //                 .spanned(start.extend(self.span()))
    //         },
    //         Token::LSqBracket => {
    //             self.next();
    //             let mut v = vec![];
    //             list_helper!(self, RSqBracket {
    //                 let p = self.parse_modify_pattern()?;
    //                 v.push(p);
    //             });
    //             ModifyPattern::Destructure(DestructurePattern::Array(v))
    //                 .spanned(start.extend(self.span()))
    //         },
    //         Token::LBracket => {
    //             self.next();
    //             let mut map = AHashMap::new();

    //             list_helper!(self, RBracket {
    //                 let key = match self.next() {
    //                     Token::Int => self.intern_string(self.parse_int(self.slice()).to_string()),
    //                     Token::String => self.parse_plain_string(self.slice(), self.span())?,
    //                     Token::Ident => self.intern_string(self.slice()),
    //                     other => {
    //                         return Err(SyntaxError::UnexpectedToken {
    //                             expected: "key".into(),
    //                             found: other,
    //                             area: self.make_area(self.span()),
    //                         })
    //                     }
    //                 };

    //                 let elem = if self.next_is(Token::Colon) {
    //                     self.next();
    //                     Some(self.parse_modify_pattern()?)
    //                 } else {
    //                     None
    //                 };

    //                 map.insert(key, elem);
    //             });

    //             ModifyPattern::Destructure(DestructurePattern::Dict(map))
    //                 .spanned(start.extend(self.span()))
    //         },
    //         Token::QMark => ModifyPattern::Destructure(DestructurePattern::Maybe(None))
    //             .spanned(start.extend(self.span())),
    //         other => {
    //             return Err(SyntaxError::UnexpectedToken {
    //                 expected: "pattern".into(),
    //                 found: other,
    //                 area: self.make_area(start),
    //             });
    //         },
    //     };

    //     Ok(ModifyPatternNode {
    //         span: expr.span,
    //         pat: Box::new(expr.value),
    //     })
    // }

    // pub fn parse_modify_pattern(&mut self) -> ParseResult<ModifyPatternNode> {
    //     let mut value = self.parse_modify_pattern_unit()?;

    //     loop {
    //         let prev_span = value.span;

    //         value = match self.peek_or_newline() {
    //             Token::LSqBracket => {
    //                 self.next();
    //                 let index = self.parse_expr(true)?;
    //                 self.expect_tok(Token::RSqBracket)?;

    //                 ModifyPattern::Index(value, index)
    //             }
    //             Token::QMark => {
    //                 self.next();

    //                 ModifyPattern::Destructure(DestructurePattern::Maybe(Some(value)))
    //             }
    //             _ => match self.peek() {
    //                 Token::Dot => {
    //                     self.next();
    //                     match self.next() {
    //                         Token::Ident => {
    //                             let name = self.slice_interned();
    //                             Expression::Member {
    //                                 base: value,
    //                                 name: name.spanned(self.span()),
    //                             }
    //                         }
    //                         Token::TypeIndicator => {
    //                             let name = self.slice()[1..].to_string();
    //                             Expression::TypeMember {
    //                                 base: value,
    //                                 name: self.intern_string(name).spanned(self.span()),
    //                             }
    //                         }
    //                         Token::Type => Expression::Typeof(value),
    //                         other => {
    //                             return Err(SyntaxError::UnexpectedToken {
    //                                 expected: "member name".into(),
    //                                 found: other,
    //                                 area: self.make_area(self.span()),
    //                             })
    //                         }
    //                     }
    //                 }
    //                 Token::DoubleColon => {
    //                     self.next();
    //                     match self.next() {
    //                         Token::Ident => {
    //                             let name = self.slice_interned();
    //                             Expression::Associated {
    //                                 base: value,
    //                                 name: name.spanned(self.span()),
    //                             }
    //                         }
    //                         Token::LBracket => {
    //                             let items = self.parse_dictlike(true)?;
    //                             Expression::Instance { base: value, items }
    //                         }
    //                         other => {
    //                             return Err(SyntaxError::UnexpectedToken {
    //                                 expected: "associated member name or instance fields".into(),
    //                                 found: other,
    //                                 area: self.make_area(self.span()),
    //                             })
    //                         }
    //                     }
    //                 }
    //                 // Token::C
    //                 _ => break,
    //             }
    //         }.into_node(vec![], prev_span.extend(self.span()));
    //     }
    //     Ok(value)
    // }

    pub fn parse_modify_pattern(&mut self) -> ParseResult<ModifyPatternNode> {
        let start = self.peek_span();
        let pat = match self.next() {
            Token::Ident => {
                let name = self.slice_interned();

                let mut path = AssignPath::Var(name);

                loop {
                    path = match self.peek_strict() {
                        Token::LSqBracket => {
                            self.next();
                            let index = self.parse_expr(true)?;
                            self.expect_tok(Token::RSqBracket)?;

                            AssignPath::Index(Box::new(path), index)
                        },
                        _ => match self.peek() {
                            Token::Dot => {
                                self.next();
                                self.expect_tok(Token::Ident)?;
                                let member = self.slice_interned();
                                AssignPath::Member(Box::new(path), member)
                            },
                            Token::DoubleColon => {
                                self.next();
                                self.expect_tok(Token::Ident)?;
                                let member = self.slice_interned();
                                AssignPath::Associated(Box::new(path), member)
                            },
                            _ => break,
                        },
                    }
                }

                ModifyPattern::Path(path)
            },
            Token::LSqBracket => {
                let mut v = vec![];
                list_helper!(self, RSqBracket {
                    let p = self.parse_modify_pattern()?;
                    v.push(p);
                });
                ModifyPattern::Destructure(DestructurePattern::Array(v))
            },
            tok @ (Token::TypeIndicator | Token::LBracket) => {
                let typ = if tok == Token::TypeIndicator {
                    let s = self.slice_interned();
                    self.expect_tok(Token::DoubleColon)?;
                    Some(s)
                } else {
                    None
                };

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
                        Some(self.parse_modify_pattern()?)
                    } else {
                        None
                    };

                    map.insert(key, elem);
                });

                if let Some(s) = typ {
                    ModifyPattern::Destructure(DestructurePattern::Instance(s, map))
                } else {
                    ModifyPattern::Destructure(DestructurePattern::Dict(map))
                }
            },
            Token::QMark => ModifyPattern::Destructure(DestructurePattern::Maybe(None)),
            other => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: "pattern".into(),
                    found: other,
                    area: self.make_area(start),
                })
            },
        };
        let mut node = ModifyPatternNode {
            pat: Box::new(pat),
            span: start.extend(self.span()),
        };
        loop {
            let start_span = node.span;
            let pat = match self.peek_strict() {
                Token::QMark => {
                    self.next();
                    ModifyPattern::Destructure(DestructurePattern::Maybe(Some(node)))
                },
                _ => break,
            };
            node = ModifyPatternNode {
                pat: Box::new(pat),
                span: start_span.extend(self.span()),
            };
        }
        Ok(node)
    }

    pub fn parse_pattern(&mut self) -> ParseResult<PatternNode> {
        todo!()
        // let start = self.peek_span();

        // let mut pat = match self.next() {
        //     Token::TypeIndicator => {
        //         let name = self.slice()[1..].to_string();

        //         Pattern::Type(self.intern_string(name))
        //     },
        //     Token::Any => Pattern::Any,
        //     Token::Eq => {
        //         let val = self.parse_value(true)?;
        //         Pattern::Eq(val)
        //     },
        //     Token::Neq => {
        //         let val = self.parse_value(true)?;
        //         Pattern::Neq(val)
        //     },
        //     Token::Gt => {
        //         let val = self.parse_value(true)?;
        //         Pattern::Gt(val)
        //     },
        //     Token::Gte => {
        //         let val = self.parse_value(true)?;
        //         Pattern::Gte(val)
        //     },
        //     Token::Lt => {
        //         let val = self.parse_value(true)?;
        //         Pattern::Lt(val)
        //     },
        //     Token::Lte => {
        //         let val = self.parse_value(true)?;
        //         Pattern::Lte(val)
        //     },
        //     Token::LParen => {
        //         let pat = self.parse_pattern()?;
        //         self.expect_tok(Token::RParen)?;
        //         *pat.pat
        //     },
        //     other => {
        //         return Err(SyntaxError::UnexpectedToken {
        //             expected: "pattern".into(),
        //             found: other,
        //             area: self.make_area(start),
        //         });
        //     },
        // };

        // match self.peek() {
        //     Token::BinOr => {
        //         let left = MatchPatternNode {
        //             pat: Box::new(pat),
        //             span: start.extend(self.span()),
        //         };

        //         self.next();
        //         let right = self.parse_pattern()?;
        //         pat = Pattern::Either(left, right);
        //     },
        //     Token::BinAnd => {
        //         let left = MatchPatternNode {
        //             pat: Box::new(pat),
        //             span: start.extend(self.span()),
        //         };

        //         self.next();
        //         let right = self.parse_pattern()?;
        //         pat = Pattern::Both(left, right);
        //     },
        //     _ => (),
        // }

        // Ok(MatchPatternNode {
        //     pat: Box::new(pat),
        //     span: start.extend(self.span()),
        // })
    }
}
