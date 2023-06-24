use std::str::Chars;

use base64::Engine;
use lasso::Spur;
use unindent::unindent;

use super::{ParseResult, Parser};
use crate::lexing::tokens::Token;
use crate::list_helper;
use crate::parsing::ast::{
    DictItem, Import, ImportSettings, ImportType, StringContent, StringType, Vis,
};
use crate::parsing::attributes::{Attributes, IsValidOn, ParseAttribute};
use crate::parsing::error::SyntaxError;
use crate::sources::{CodeSpan, Spannable, Spanned};

impl Parser<'_> {
    pub fn parse_int(&self, s: &str) -> i64 {
        if s.len() > 2 {
            match &s[0..2] {
                "0x" => {
                    return i64::from_str_radix(&s.trim_start_matches("0x").replace('_', ""), 16)
                        .unwrap()
                },
                "0b" => {
                    return i64::from_str_radix(&s.trim_start_matches("0b").replace('_', ""), 2)
                        .unwrap()
                },
                "0o" => {
                    return i64::from_str_radix(&s.trim_start_matches("0o").replace('_', ""), 8)
                        .unwrap()
                },
                _ => (),
            }
        }
        s.replace('_', "").parse::<i64>().unwrap()
    }

    // fn parse_id(&self, s: &str) -> (IDClass, Option<u16>) {
    //     let class = match &s[(s.len() - 1)..(s.len())] {
    //         "g" => IDClass::Group,
    //         "c" => IDClass::Color,
    //         "b" => IDClass::Block,
    //         "i" => IDClass::Item,
    //         _ => unreachable!(),
    //     };
    //     let value = s[0..(s.len() - 1)].parse::<u16>().ok();

    //     (class, value)
    // }

    pub fn parse_string(&self, s: &str, span: CodeSpan) -> ParseResult<StringType> {
        let mut chars = s.chars();

        // Remove trailing "
        chars.next_back();

        let raw_flags = chars
            .by_ref()
            .take_while(|c| !matches!(c, '"' | '\''))
            .collect::<String>();

        let mut out: String = chars.collect();

        let mut flags = raw_flags.split('_').collect::<Vec<_>>();
        let last = flags.pop().unwrap();

        if matches!(last, "r" | "r#" | "r##") {
            out = out[0..(out.len() - last.len() + 1)].into();
        } else {
            out = self.parse_escapes(&mut out.chars())?;

            if !raw_flags.is_empty() {
                flags.push(last);
            }
        }

        let mut is_bytes = false;

        for flag in flags {
            match flag {
                "b" => is_bytes = true,
                "u" => out = unindent(&out),
                "b64" => {
                    out = base64::engine::general_purpose::STANDARD.encode(out);
                },
                other => {
                    return Err(SyntaxError::UnexpectedFlag {
                        flag: other.to_string(),
                        area: self.make_area(span),
                    });
                },
            }
        }

        Ok(StringType {
            s: StringContent::Normal(self.intern_string(out)),
            bytes: is_bytes,
        })
    }

    pub fn parse_plain_string(&self, s: &str, span: CodeSpan) -> ParseResult<Spur> {
        let st = self.parse_string(s, span)?;

        let s = match st {
            StringType {
                s: StringContent::Normal(s),
                bytes: false,
            } => s,
            _ => {
                return Err(SyntaxError::InvalidStringType {
                    typ: "plain",
                    area: self.make_area(span),
                })
            },
        };

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
                Some(c) => {
                    if c != '\'' && c != '"' {
                        out.push(c)
                    }
                },
                None => break,
            }
        }

        Ok(out)
    }

    pub fn parse_unicode(&self, chars: &mut Chars) -> ParseResult<char> {
        let next = chars.next().unwrap_or(' ');

        if !matches!(next, '{') {
            return Err(SyntaxError::UnxpectedCharacter {
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
                return Err(SyntaxError::UnxpectedCharacter {
                    expected: Token::RBracket,
                    found: t.to_string(),
                    area: self.make_area(self.span()),
                })
            },
            None => {
                return Err(SyntaxError::UnxpectedCharacter {
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
            let attrs = if self.skip_tok(Token::Hashtag) {

                self.parse_attributes::<Attributes>()?
            } else {
                vec![]
            };

            let start = self.peek_span();

            let vis = if allow_vis && self.next_is(Token::Private) {
                self.next();
                Vis::Private
            } else {
                Vis::Public
            };

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

            let key_span = self.span();

            let elem = if self.next_is(Token::Colon) {
                self.next();
                Some(self.parse_expr(true)?)
            } else {
                None
            };

            // this is so backwards if only u could use enum variants as types. . . .
            let mut item = DictItem { name: key.spanned(key_span), attributes: vec![], value: elem }.spanned(start.extend(self.span()));

            attrs.is_valid_on(&item, &self.src)?;

            item.attributes = attrs;

            items.push(vis(item.value));
        });

        Ok(items)
    }

    pub fn parse_attributes<T: ParseAttribute>(&mut self) -> ParseResult<Vec<Spanned<T>>> {
        let mut attrs = vec![];
        self.expect_tok(Token::LSqBracket)?;

        list_helper!(self, RSqBracket {
            let start = self.peek_span();
            attrs.push(T::parse(self)?.spanned(start.extend(self.span())))
        });

        Ok(attrs)
    }

    pub fn parse_import(&mut self) -> ParseResult<Import> {
        Ok(match self.peek() {
            Token::String => {
                self.next();

                Import {
                    path: self
                        .resolve(&self.parse_plain_string(self.slice(), self.span())?)
                        .to_string()
                        .into(),
                    settings: ImportSettings {
                        typ: ImportType::File,
                        is_absolute: false,
                        allow_builtin_impl: false,
                    },
                }
            },
            Token::Ident => {
                self.next();
                Import {
                    path: self.slice().into(),
                    settings: ImportSettings {
                        typ: ImportType::Library,
                        is_absolute: false,
                        allow_builtin_impl: false,
                    },
                }
            },
            other => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: "string literal or identifier".into(),
                    found: other,
                    area: self.make_area(self.peek_span()),
                })
            },
        })
    }
}
