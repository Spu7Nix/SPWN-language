use std::rc::Rc;
use std::str::Chars;

use ahash::AHashMap;
use base64::Engine;
use itertools::Either;
use lasso::Spur;
use unindent::unindent;

use super::{ParseResult, Parser};
use crate::gd::ids::IDClass;
use crate::lexing::tokens::Token;
use crate::list_helper;
use crate::parsing::ast::{
    Attribute, DictItem, ExprNode, Expression, Import, ImportType, StringContent, StringFlags,
    StringType, Vis,
};
use crate::parsing::attributes::AttributeTarget;
use crate::parsing::error::SyntaxError;
use crate::sources::{CodeSpan, Spannable, Spanned};
use crate::util::remove_quotes;

impl Parser<'_> {
    pub fn parse_int(&self, s: &str, base: u32) -> i64 {
        i64::from_str_radix(&s.replace('_', ""), base).unwrap()
    }

    pub fn parse_golden_float(&self, s: &str) -> f64 {
        let mut n = 0_f64;
        for (i, d) in s.bytes().enumerate() {
            let pow = s.len() - 1 - i;
            const PHI: f64 = 1.618_033_988_749_895;
            n += PHI.powf(pow as f64) * if d == b'0' { 0.0 } else { 1.0 };
        }
        n
    }

    pub fn parse_id(&self, s: &str, class: IDClass) -> Expression {
        let value = s[0..(s.len() - 1)].parse::<u16>().ok();

        Expression::Id(class, value)
    }

    pub fn parse_string(&mut self, start_tok: Token) -> ParseResult<StringContent> {
        let start_slice = self.slice();

        Ok(match start_tok {
            Token::String => {
                let s = self.parse_plain_string(start_slice)?;
                StringContent {
                    s: StringType::Normal(self.intern_string(s)),
                    flags: StringFlags::default(),
                }
            },
            Token::RawString => {
                let s = &start_slice[1..];
                let b = s.as_bytes();

                let mut i = 0;
                loop {
                    if matches!(b[i], b'"' | b'\'') {
                        break;
                    }
                    i += 1;
                }

                StringContent {
                    s: StringType::Normal(self.intern_string(&s[(i + 1)..(s.len() - 1 - i)])),
                    flags: StringFlags::default(),
                }
            },
            Token::StringFlags => {
                let mut flags = StringFlags::default();

                let mut is_f_string = false;

                for i in start_slice.bytes() {
                    let flag = match i {
                        b'b' => &mut flags.bytes,
                        b'B' => &mut flags.base64,
                        b'u' => &mut flags.unindent,
                        b'f' => &mut is_f_string,
                        f => {
                            return Err(SyntaxError::UnexpectedFlag {
                                flag: (f as char).to_string(),
                                area: self.make_area(self.span()),
                            })
                        },
                    };
                    *flag = true;
                }

                if is_f_string {
                    self.expect_tok(Token::String)?;
                    let s = self.slice();
                    let start = self.span().start + 1;
                    let v = self.parse_f_string(remove_quotes(s), start)?;
                    StringContent {
                        s: StringType::FString(v),
                        flags,
                    }
                } else {
                    let t = self.next()?;
                    let mut content = self.parse_string(t)?;
                    content.flags = flags;
                    content
                }
            },
            _ => unreachable!(),
        })
    }

    pub fn parse_f_string(
        &self,
        s: &str,
        start: usize,
    ) -> ParseResult<Vec<Either<Spur, ExprNode>>> {
        let mut i = 0;
        let b = s.as_bytes();

        let mut out = vec![Either::Left("".to_string())];

        macro_rules! add_char {
            ($c:expr) => {
                match out.last_mut().unwrap() {
                    Either::Left(s) => s.push($c as char),
                    Either::Right(_) => out.push(Either::Left(($c as char).to_string())),
                }
            };
        }

        loop {
            match (b.get(i), b.get(i + 1)) {
                (Some(b'{'), Some(b'{')) => {
                    add_char!(b'{');
                    i += 1;
                },
                (Some(b'}'), Some(b'}')) => {
                    add_char!(b'}');
                    i += 1;
                },
                (Some(b'}'), _) => {
                    return Err(SyntaxError::UnbalancedFormatStringBlock {
                        expected: "{",
                        area: self.make_area(self.span()),
                    })
                },
                (Some(b'{'), _) => {
                    let mut s = "".to_string();
                    let expr_start = i;
                    i += 1;
                    let mut depth = 1;
                    loop {
                        match b.get(i) {
                            Some(b'{') => {
                                depth += 1;
                                s.push('{');
                            },
                            Some(b'}') => {
                                depth -= 1;
                                if depth == 0 {
                                    i += 1;
                                    break;
                                }
                                s.push('}');
                            },
                            Some(t) => s.push(*t as char),
                            None => {
                                return Err(SyntaxError::UnbalancedFormatStringBlock {
                                    expected: "}",
                                    area: self.make_area(self.span()),
                                })
                            },
                        }
                        i += 1;
                    }

                    let code = " ".repeat(start + expr_start + 1) + &s;

                    let mut parser: Parser<'_> =
                        Parser::new(&code, Rc::clone(&self.src), self.interner.clone());

                    let expr = parser.parse_expr(true)?;

                    out.push(Either::Right(expr));

                    i -= 1;
                },
                (Some(t), _) => add_char!(*t),
                _ => break,
            }
            i += 1;
        }

        Ok(out
            .into_iter()
            .map(|e| match e {
                Either::Left(s) => {
                    //println!("GAGA ({s})");
                    Either::Left(self.intern_string(s))
                },
                Either::Right(e) => Either::Right(e),
            })
            .collect())
    }

    pub fn parse_plain_string(&self, s: &str) -> ParseResult<String> {
        self.parse_escapes(&mut remove_quotes(s).chars())
    }

    pub fn parse_compile_time_string(&mut self) -> ParseResult<String> {
        let s = self
            .parse_string(Token::String)?
            .get_compile_time(&self.interner)
            .ok_or(SyntaxError::InvalidDictStringKey {
                area: self.make_area(self.span()),
            })?;

        Ok(s)
    }

    pub fn parse_escapes(&self, chars: &mut Chars) -> ParseResult<String> {
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
                    },
                    None => {
                        return Err(SyntaxError::InvalidEscape {
                            character: ' ',
                            area: self.make_area(self.span()),
                        })
                    },
                }),
                Some(c) => out.push(c),
                None => break,
            }
        }

        Ok(out)
    }

    pub fn parse_unicode(&self, chars: &mut Chars) -> ParseResult<char> {
        let next = chars.next().unwrap_or(' ');

        if !matches!(next, '{') {
            return Err(SyntaxError::UnexpectedCharacter {
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
                return Err(SyntaxError::UnexpectedCharacter {
                    expected: Token::RBracket,
                    found: t.to_string(),
                    area: self.make_area(self.span()),
                })
            },
            None => {
                return Err(SyntaxError::UnexpectedCharacter {
                    expected: Token::RBracket,
                    found: "end of string".into(),
                    area: self.make_area(self.span()),
                })
            },
        }

        Ok(char::from_u32(u32::from_str_radix(&hex, 16).map_err(|_| {
            SyntaxError::InvalidUnicode {
                sequence: hex,
                area: self.make_area(self.span()),
            }
        })?)
        .unwrap_or('\u{FFFD}'))
    }

    pub fn parse_dictlike(&mut self, allow_vis: bool) -> ParseResult<Vec<Vis<DictItem>>> {
        let mut items = vec![];

        list_helper!(self, RBracket {
            let attributes = self.parse_outer_attributes()?;

            let start = self.peek_span()?;

            let vis = if allow_vis && self.next_is(Token::Private)? {
                self.next()?;
                Vis::Private
            } else {
                Vis::Public
            };

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
                Some(self.parse_expr(true)?)
            } else {
                None
            };

            let item_span = start.extend(self.span());

            self.check_attributes(&attributes, Some(AttributeTarget::DictItem.spanned(item_span)))?;

            items.push(vis(DictItem { name: key.spanned(key_span), attributes, value: elem }));
        });

        Ok(items)
    }

    pub fn parse_import(&mut self) -> ParseResult<Import> {
        Ok(match self.peek()? {
            Token::String => {
                self.next()?;

                Import {
                    path: self.parse_plain_string(self.slice())?.into(),
                    typ: ImportType::File,
                }
            },
            Token::Ident => {
                self.next()?;
                Import {
                    path: self.slice().into(),
                    typ: ImportType::Library,
                }
            },
            other => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: "string literal or identifier".into(),
                    found: other,
                    area: self.make_area(self.peek_span()?),
                })
            },
        })
    }
}
