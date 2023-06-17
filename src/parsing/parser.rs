use std::cell::RefCell;
use std::rc::Rc;
use std::str::Chars;

use base64::Engine;
use lasso::Spur;
use unindent::unindent;

use super::ast::{
    Ast, DictItem, ExprNode, Expression, ImportType, MacroArg, MacroCode, ModuleImport, ObjectType,
    Pattern, PatternNode, Spannable, Spanned, Statement, Statements, StmtNode, StringContent,
    StringType,
};
use super::attributes::{Attributes, FileAttribute, IsValidOn, ParseAttribute};
use super::error::SyntaxError;
use super::utils::operators::{self, unary_prec};
use crate::lexing::tokens::{Lexer, Token};
use crate::parsing::utils::operators::Operator;
use crate::sources::{CodeArea, CodeSpan, SpwnSource};
use crate::util::Interner;

#[derive(Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    pub src: SpwnSource,
    interner: Rc<RefCell<Interner>>,
}

pub type ParseResult<T> = Result<T, SyntaxError>;

impl<'a> Parser<'a> {
    pub fn new(code: &'a str, src: SpwnSource, interner: Rc<RefCell<Interner>>) -> Self {
        let lexer = Token::lex(code);
        Parser {
            lexer,
            src,
            interner,
        }
    }
}

#[macro_export]
macro_rules! list_helper {
    ($self:ident, $closing_tok:ident $code:block) => {
        while !$self.next_is(Token::$closing_tok) {
            $code;
            if !$self.skip_tok(Token::Comma) {
                break;
            }
        }
        $self.expect_tok(Token::$closing_tok)?;
    };

    ($self:ident, $first:ident, $closing_tok:ident $code:block) => {
        let mut $first = true;
        while !$self.next_is(Token::$closing_tok) {
            $code;
            $first = false;
            if !$self.skip_tok(Token::Comma) {
                break;
            }
        }
        $self.expect_tok(Token::$closing_tok)?;
    };
}

impl Parser<'_> {
    pub fn next(&mut self) -> Token {
        let out = self.lexer.next_or_eof();
        if out == Token::Newline {
            self.next()
        } else {
            out
        }
    }

    // pub fn next_or_newline(&mut self) -> Token {
    //     self.lexer.next_or_eof()
    // }

    pub fn span(&self) -> CodeSpan {
        self.lexer.span().into()
    }

    pub fn peek_span(&self) -> CodeSpan {
        let mut peek = self.lexer.clone();
        while peek.next_or_eof() == Token::Newline {}
        peek.span().into()
    }

    // pub fn peek_span_or_newline(&self) -> CodeSpan {
    //     let mut peek = self.lexer.clone();
    //     peek.next_or_eof();
    //     peek.span().into()
    // }

    pub fn slice(&self) -> &str {
        self.lexer.slice()
    }

    pub fn slice_interned(&self) -> Spur {
        self.interner.borrow_mut().get_or_intern(self.lexer.slice())
    }

    pub fn peek(&self) -> Token {
        let mut peek = self.lexer.clone();
        let mut out = peek.next_or_eof();
        while out == Token::Newline {
            // should theoretically never be more than one, but having a loop just in case doesn't hurt
            out = peek.next_or_eof();
        }
        out
    }

    pub fn peek_or_newline(&self) -> Token {
        let mut peek = self.lexer.clone();
        peek.next_or_eof()
    }

    pub fn next_is(&self, tok: Token) -> bool {
        self.peek() == tok
    }

    pub fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            src: self.src.clone(),
        }
    }

    pub fn skip_tok(&mut self, skip: Token) -> bool {
        if self.next_is(skip) {
            self.next();
            true
        } else {
            false
        }
    }

    pub fn expect_tok_named(&mut self, expect: Token, name: &str) -> ParseResult<()> {
        let next = self.next();
        if next != expect {
            return Err(SyntaxError::UnexpectedToken {
                found: next,
                expected: name.to_string(),
                area: self.make_area(self.span()),
            });
        }
        Ok(())
    }

    pub fn expect_tok(&mut self, expect: Token) -> Result<(), SyntaxError> {
        self.expect_tok_named(expect, expect.to_str())
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

        // TODO: a single character is a valid string in attributes cause this deletes it so it will never error
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

    fn intern_string<T: AsRef<str>>(&self, string: T) -> Spur {
        self.interner.borrow_mut().get_or_intern(string)
    }

    pub fn resolve(&self, s: &Spur) -> String {
        self.interner.borrow_mut().resolve(s).into()
    }

    fn parse_escapes(&self, chars: &mut Chars) -> ParseResult<String> {
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

    fn parse_unicode(&self, chars: &mut Chars) -> ParseResult<char> {
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

    pub fn parse_dictlike(&mut self, allow_vis: bool) -> ParseResult<Vec<DictItem>> {
        let mut items = vec![];

        list_helper!(self, RBracket {
            let attrs = if self.next_is(Token::Hashtag) {
                self.next();

                self.parse_attributes::<Attributes>()?
            } else {
                vec![]
            };

            let start = self.peek_span();

            let private = if allow_vis && self.next_is(Token::Private) {
                self.next();
                true
            } else {
                false
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
            let mut item = DictItem { name: key.spanned(key_span), attributes: vec![], value: elem, private }.spanned(start.extend(self.span()));

            attrs.is_valid_on(&item, self.src.clone())?;

            item.value.attributes = attrs;

            items.push(item.value);
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

    pub fn parse_import(&mut self) -> ParseResult<ImportType> {
        Ok(match self.peek() {
            Token::String => {
                self.next();
                ImportType::Module(
                    self.resolve(&self.parse_plain_string(self.slice(), self.span())?)
                        .into(),
                    ModuleImport::Regular,
                )
            },
            Token::Ident => {
                self.next();
                ImportType::Library(self.slice().into())
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

    pub fn parse_pattern(&mut self) -> ParseResult<PatternNode> {
        let start = self.peek_span();

        let mut pat = match self.next() {
            Token::TypeIndicator => {
                let name = self.slice()[1..].to_string();

                Pattern::Type(self.intern_string(name))
            },
            Token::Any => Pattern::Any,
            Token::Eq => {
                let val = self.parse_value(true)?;
                Pattern::Eq(val)
            },
            Token::Neq => {
                let val = self.parse_value(true)?;
                Pattern::Neq(val)
            },
            Token::Gt => {
                let val = self.parse_value(true)?;
                Pattern::Gt(val)
            },
            Token::Gte => {
                let val = self.parse_value(true)?;
                Pattern::Gte(val)
            },
            Token::Lt => {
                let val = self.parse_value(true)?;
                Pattern::Lt(val)
            },
            Token::Lte => {
                let val = self.parse_value(true)?;
                Pattern::Lte(val)
            },
            Token::LParen => {
                let pat = self.parse_pattern()?;
                self.expect_tok(Token::RParen)?;
                *pat.pat
            },
            other => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: "pattern".into(),
                    found: other,
                    area: self.make_area(start),
                });
            },
        };

        match self.peek() {
            Token::BinOr => {
                let left = PatternNode {
                    pat: Box::new(pat),
                    span: start.extend(self.span()),
                };

                self.next();
                let right = self.parse_pattern()?;
                pat = Pattern::Either(left, right);
            },
            Token::BinAnd => {
                let left = PatternNode {
                    pat: Box::new(pat),
                    span: start.extend(self.span()),
                };

                self.next();
                let right = self.parse_pattern()?;
                pat = Pattern::Both(left, right);
            },
            _ => (),
        }

        Ok(PatternNode {
            pat: Box::new(pat),
            span: start.extend(self.span()),
        })
    }

    pub fn parse_unit(&mut self, allow_macros: bool) -> ParseResult<ExprNode> {
        let attrs = if self.next_is(Token::Hashtag) {
            self.next();

            self.parse_attributes::<Attributes>()?
        } else {
            vec![]
        };

        let peek = self.peek();
        let start = self.peek_span();

        let unary;

        let expr = 'out_expr: {
            break 'out_expr match peek {
                Token::Int => {
                    self.next();
                    Expression::Int(self.parse_int(self.slice())).spanned(start)
                },
                Token::Float => {
                    self.next();
                    Expression::Float(self.slice().replace('_', "").parse::<f64>().unwrap())
                        .spanned(start)
                },
                Token::String => {
                    self.next();
                    Expression::String(self.parse_string(self.slice(), self.span())?).spanned(start)
                },
                // Token::Id => {
                //     self.next();

                //     let (id_class, value) = self.parse_id(self.slice());
                //     Expression::Id(id_class, value).spanned(start)
                // },
                Token::Dollar => {
                    self.next();

                    Expression::Builtins.spanned(start)
                },
                Token::True => {
                    self.next();
                    Expression::Bool(true).spanned(start)
                },
                Token::False => {
                    self.next();
                    Expression::Bool(false).spanned(start)
                },
                Token::Epsilon => {
                    self.next();
                    Expression::Epsilon.spanned(start)
                },
                Token::Ident => {
                    self.next();
                    let var_name = self.slice_interned();

                    if matches!(self.peek_or_newline(), Token::FatArrow | Token::Arrow) {
                        let ret_type = if self.next_is(Token::Arrow) {
                            self.next();
                            let r = Some(self.parse_expr(allow_macros)?);
                            self.expect_tok(Token::FatArrow)?;
                            r
                        } else {
                            self.next();
                            None
                        };

                        let code = MacroCode::Lambda(self.parse_expr(allow_macros)?);

                        break 'out_expr Expression::Macro {
                            args: vec![MacroArg::Single {
                                name: var_name.spanned(start),
                                pattern: None,
                                default: None,
                                is_ref: false,
                            }],
                            code,
                            ret_type,
                        }
                        .spanned(start.extend(self.span()));
                    }

                    Expression::Var(var_name).spanned(start)
                },
                Token::Slf => {
                    self.next();
                    Expression::Var(self.intern_string("self")).spanned(start)
                },
                Token::TypeIndicator => {
                    self.next();
                    let name = self.slice()[1..].to_string();
                    Expression::Type(self.intern_string(name)).spanned(start)
                },
                Token::LParen => {
                    self.next();

                    let mut check = self.clone();
                    let mut indent = 1;

                    let after_close = loop {
                        match check.next() {
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
                                    break check.next();
                                }
                            },
                            _ => (),
                        }
                    };

                    match after_close {
                        Token::FatArrow | Token::LBracket | Token::Arrow if allow_macros => (),
                        _ => {
                            if self.next_is(Token::RParen) {
                                self.next();
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

                    list_helper!(self, is_first, RParen {
                        if is_first && self.next_is(Token::Slf) {
                            self.next();
                            let span = self.span();

                            let pattern = if self.next_is(Token::Colon) {
                                self.next();
                                Some(self.parse_pattern()?)
                            } else {
                                None
                            };

                            args.push(MacroArg::Single { name: self.intern_string("self").spanned(span), pattern, default: None, is_ref: false })
                        } else if is_first && self.next_are(&[Token::BinAnd, Token::Slf]) {
                            self.next();
                            let span = self.span();

                            let pattern = if self.next_is(Token::Colon) {
                                self.next();
                                Some(self.parse_pattern()?)
                            } else {
                                None
                            };

                            args.push(MacroArg::Single { name: self.intern_string("self").spanned(span), pattern, default: None, is_ref: true })
                        } else {
                            let is_spread = if self.next_is(Token::Spread) {
                                self.next();
                                true
                            } else {
                                false
                            };

                            let is_ref = if !is_spread && self.next_is(Token::BinAnd) {
                                self.next();
                                true
                            } else {
                                false
                            };

                            self.expect_tok_named(Token::Ident, "argument name")?;
                            let arg_name = self.slice_interned().spanned(self.span());

                            if is_spread {
                                if let Some(prev_s) = first_spread_span {
                                    return Err(SyntaxError::MultipleSpreadArguments { area: self.make_area(self.span()), prev_area: self.make_area(prev_s) })
                                }
                                first_spread_span = Some(self.span())
                            }

                            let pattern = if self.next_is(Token::Colon) {
                                self.next();
                                Some(self.parse_pattern()?)
                            } else {
                                None
                            };

                            if !is_spread {
                                let default = if self.next_is(Token::Assign) {
                                    self.next();
                                    Some(self.parse_expr(true)?)
                                } else {
                                    None
                                };
                                args.push(MacroArg::Single { name: arg_name, pattern, default, is_ref });
                            } else {
                                args.push(MacroArg::Spread { name: arg_name, pattern });
                            }
                        }
                    });

                    let ret_type = if self.next_is(Token::Arrow) {
                        self.next();
                        Some(self.parse_expr(allow_macros)?)
                    } else {
                        None
                    };

                    let code = if self.next_is(Token::FatArrow) {
                        self.next();
                        MacroCode::Lambda(self.parse_expr(allow_macros)?)
                    } else {
                        MacroCode::Normal(self.parse_block()?)
                    };

                    Expression::Macro {
                        args,
                        code,
                        ret_type,
                    }
                    .spanned(start.extend(self.span()))

                    // if is_macro && allow_macros {
                    //     let mut args = vec![];

                    //     let mut first_spread_span = None;

                    //     list_helper!(self, is_first, RParen {
                    //         if is_first && self.next_is(Token::Slf) {
                    //             self.next();
                    //             let span = self.span();

                    //             let pattern = if self.next_is(Token::Colon) {
                    //                 self.next();
                    //                 Some(self.parse_expr(true)?)
                    //             } else {
                    //                 None
                    //             };

                    //             args.push(MacroArg::Single { name: self.intern_string("self").spanned(span), pattern, default: None, is_ref: true })
                    //         } else {
                    //             let is_spread = if self.next_is(Token::Spread) {
                    //                 self.next();
                    //                 true
                    //             } else {
                    //                 false
                    //             };

                    //             let is_ref = if !is_spread && self.next_is(Token::BinAnd) {
                    //                 self.next();
                    //                 true
                    //             } else {
                    //                 false
                    //             };

                    //             self.expect_tok_named(Token::Ident, "argument name")?;
                    //             let arg_name = self.slice_interned().spanned(self.span());

                    //             if is_spread {
                    //                 if let Some(prev_s) = first_spread_span {
                    //                     return Err(SyntaxError::MultipleSpreadArguments { area: self.make_area(self.span()), prev_area: self.make_area(prev_s) })
                    //                 }
                    //                 first_spread_span = Some(self.span())
                    //             }

                    //             let pattern = if self.next_is(Token::Colon) {
                    //                 self.next();
                    //                 Some(self.parse_expr(true)?)
                    //             } else {
                    //                 None
                    //             };

                    //             if !is_spread {
                    //                 let default = if self.next_is(Token::Assign) {
                    //                     self.next();
                    //                     Some(self.parse_expr(true)?)
                    //                 } else {
                    //                     None
                    //                 };
                    //                 args.push(MacroArg::Single { name: arg_name, pattern, default, is_ref });
                    //             } else {
                    //                 args.push(MacroArg::Spread { name: arg_name, pattern });
                    //             }
                    //         }
                    //     });

                    //     let ret_type = if self.next_is(Token::Arrow) {
                    //         self.next();
                    //         Some(self.parse_expr(allow_macros)?)
                    //     } else {
                    //         None
                    //     };

                    //     let code = if self.next_is(Token::FatArrow) {
                    //         self.next();
                    //         MacroCode::Lambda(self.parse_expr(allow_macros)?)
                    //     } else {
                    //         MacroCode::Normal(self.parse_block()?)
                    //     };

                    //     Expression::Macro {
                    //         args,
                    //         code,
                    //         ret_type,
                    //     }
                    //     .spanned(start.extend(self.span()))
                    // } else {
                    //     let mut args = vec![];

                    //     list_helper!(self, RParen {
                    //         args.push(self.parse_expr(true)?);
                    //     });

                    //     let next = self.next_or_newline();
                    //     if next != Token::Arrow {
                    //         return Err(SyntaxError::UnexpectedToken {
                    //             found: next,
                    //             expected: Token::Arrow.to_str().to_string(),
                    //             area: self.make_area(self.span()),
                    //         });
                    //     }

                    //     let ret_type = self.parse_expr(allow_macros)?;

                    //     Expression::MacroPattern { args, ret_type }
                    //         .spanned(start.extend(self.span()))
                    // }
                },
                Token::LSqBracket => {
                    self.next();

                    let mut elems = vec![];

                    list_helper!(self, RSqBracket {
                        elems.push(self.parse_expr(true)?);
                    });

                    Expression::Array(elems).spanned(start.extend(self.span()))
                },
                // typ @ (Token::Obj | Token::Trigger) => {
                //     self.next();

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
                    self.next();

                    Expression::Dict(self.parse_dictlike(false)?).spanned(start.extend(self.span()))
                },
                Token::QMark => {
                    self.next();

                    Expression::Maybe(None).spanned(start.extend(self.span()))
                },
                Token::TrigFnBracket => {
                    self.next();

                    let code = self.parse_statements()?;
                    self.expect_tok(Token::RBracket)?;

                    Expression::TriggerFunc {
                        code,
                        attributes: vec![],
                    }
                    .spanned(start.extend(self.span()))
                },
                Token::Import => {
                    self.next();

                    let import_type = self.parse_import()?;

                    Expression::Import(import_type).spanned(start.extend(self.span()))
                },
                unary_op
                    if {
                        unary = unary_prec(unary_op);
                        unary.is_some()
                    } =>
                {
                    self.next();
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
                    });
                },
            };
        };

        attrs.is_valid_on(&expr, self.src.clone())?;

        Ok(expr
            .value
            .into_node(attrs.into_iter().map(|a| a.value).collect(), expr.span))
    }

    pub fn parse_value(&mut self, allow_macros: bool) -> ParseResult<ExprNode> {
        let mut value = self.parse_unit(allow_macros)?;

        loop {
            let prev_span = value.span;

            value = match self.peek_or_newline() {
                Token::LSqBracket => {
                    self.next();
                    let index = self.parse_expr(true)?;
                    self.expect_tok(Token::RSqBracket)?;

                    Expression::Index { base: value, index }
                }
                Token::QMark => {
                    self.next();

                    Expression::Maybe(Some(value))
                }
                Token::ExclMark => {
                    self.next();

                    Expression::TriggerFuncCall(value)
                }
                Token::If => {
                    // if there is a newline, treat as separate statement
                    if self.peek_or_newline() == Token::Newline {
                        break;
                    }
                    self.next();
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
                    self.next();
                    let typ = self.parse_pattern()?;

                    Expression::Is(value, typ)
                }
                Token::LParen => {
                    self.next();

                    let mut params = vec![];
                    let mut named_params = vec![];

                    let mut parsing_named = None;

                    list_helper!(self, RParen {
                        if self.next_are(&[Token::Ident, Token::Assign]) {
                            self.next();
                            let start = self.span();
                            let name = self.slice_interned();
                            self.next();

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
                _ => match self.peek() {
                    Token::Dot => {
                        self.next();
                        match self.next() {
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
                        self.next();
                        match self.next() {
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

        while operators::is_infix_prec(self.peek(), prec) {
            let op = self.next();
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
                let var = self.parse_unit(true)?;
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
                    self.peek_or_newline(),
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
                Statement::TypeDef {
                    name: self.intern_string(name),
                    private: false,
                }
            },
            Token::Private => {
                self.next();
                self.expect_tok(Token::Type)?;
                self.expect_tok(Token::TypeIndicator)?;
                let name = self.slice()[1..].to_string();
                Statement::TypeDef {
                    name: self.intern_string(name),
                    private: true,
                }
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
                let v = self.parse_expr(true)?;

                Statement::Dbg(v)
            },
            Token::Throw => {
                self.next();
                self.expect_tok(Token::String)?;

                Statement::Throw(self.parse_plain_string(self.slice(), self.span())?)
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
            && !matches!(
                self.peek_or_newline(),
                Token::Eol | Token::Newline | Token::Eof
            )
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

        attrs.is_valid_on(&stmt, self.src.clone())?;

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

    pub fn parse(&mut self) -> ParseResult<Ast> {
        let file_attributes = if self.next_are(&[Token::Hashtag, Token::ExclMark]) {
            self.next();
            self.next();

            self.parse_attributes::<FileAttribute>()?
        } else {
            vec![]
        };

        let statements = self.parse_statements()?;
        self.expect_tok(Token::Eof)?;

        Ok(Ast {
            statements,
            file_attributes: file_attributes.into_iter().map(|a| a.value).collect(),
        })
    }
}
