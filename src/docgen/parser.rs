use logos::{Lexer, Logos, Span};

use super::ast::{Constant, DocData, Line, LineKey, Lines, Value, Values};
use super::docgen::Source;
use super::lexer::Token;

#[derive(Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a, Token>,
    source: Source,
}

impl Parser<'_> {
    pub fn new<S: AsRef<str>>(code: S, source: Source) -> Self {
        let code = unsafe { Parser::make_static(code.as_ref()) };
        let lexer = Token::lexer(code);
        Parser { lexer, source }
    }
    unsafe fn make_static<'a>(d: &'a str) -> &'static str {
        std::mem::transmute::<&'a str, &'static str>(d)
    }

    pub fn span(&self) -> Span {
        self.lexer.span()
    }

    pub fn next(&mut self) -> Token {
        self.lexer.next().unwrap_or(Token::Eof)
    }

    pub fn peek(&mut self) -> Token {
        let mut peek = self.lexer.clone();
        peek.next().unwrap_or(Token::Eof)
    }

    pub fn expected_err(&mut self, expected: Token, found: &str) {
        panic!(
            "expected token: `{}`, found: `{}` ({} @ l{}:c{})!",
            expected.to_string(),
            found.to_string(),
            self.source.source.display(),
            self.span().start,
            self.span().end,
        )
    }

    pub fn peek_expect_tok(&mut self, tok: Token) {
        let next = self.peek();

        if next != tok {
            self.next();

            self.expected_err(tok, &next.to_string());
        }
    }

    pub fn expect_tok(&mut self, tok: Token) {
        let next = self.next();

        if next != tok {
            self.expected_err(tok, &next.to_string());
        }
    }

    pub fn skip_tok(&mut self, tok: Token) {
        if self.peek() == tok {
            self.next();
        }
    }

    pub fn slice(&self) -> &str {
        self.lexer.slice()
    }

    pub fn parse_value(&mut self, data: &mut DocData) -> Values {
        let peek = self.peek();

        match peek {
            Token::Int => {
                self.next();
                let int = self.slice().to_string();

                Values::Constant(Constant::Int(int))
            }
            Token::Float => {
                self.next();
                let float = self.slice().to_string();

                Values::Constant(Constant::Float(float))
            }
            Token::String => {
                self.next();
                let string = self.slice().to_string();

                Values::Constant(Constant::String(string))
            }
            Token::True => {
                self.next();

                Values::Constant(Constant::True)
            }
            Token::False => {
                self.next();

                Values::Constant(Constant::False)
            }
            Token::Ident => {
                self.next();
                let var_name = self.slice().to_string();

                if self.peek() == Token::FatArrow {
                    self.next();

                    // TODO: Update when fn syntax changed
                    self.parse_value(data)
                } else {
                    Values::Value(Value::Ident(var_name))
                }
            }
            Token::TypeIndicator => {
                self.next();
                let typ_name = self.slice().to_string();

                Values::Value(Value::TypeIndicator(typ_name))
            }

            // Token::LParen => {
            //     self.next();
            //     if self.peek() == Token::RParen && self.peek_many(2) != Token::FatArrow {
            //         self.next();
            //         Ok(data.insert_expr(Expression::Empty, start.extend(self.span())))
            //     } else {
            //         let mut depth = 1;
            //         let mut check = self.clone();

            //         loop {
            //             match check.peek() {
            //                 Token::LParen => {
            //                     check.next();
            //                     depth += 1;
            //                 }
            //                 Token::RParen => {
            //                     check.next();
            //                     depth -= 1;
            //                     if depth == 0 {
            //                         break;
            //                     }
            //                 }
            //                 Token::Eof => {
            //                     return Err(SyntaxError::UnmatchedChar {
            //                         for_char: "(".into(),
            //                         not_found: ")".into(),
            //                         area: self.make_area(start),
            //                     })
            //                 }
            //                 _ => {
            //                     check.next();
            //                 }
            //             }
            //         }

            //         let is_pattern = match check.peek() {
            //             Token::FatArrow => false,
            //             Token::Arrow => {
            //                 check.next();
            //                 check.parse_expr(data)?;

            //                 check.peek() != Token::FatArrow
            //             }
            //             _ => {
            //                 let value = self.parse_expr(data)?;
            //                 self.expect_tok(Token::RParen)?;
            //                 return Ok(value);
            //             }
            //         };

            //         if !is_pattern {
            //             let mut args = vec![];
            //             let mut arg_areas = vec![];
            //             while self.peek() != Token::RParen {
            //                 self.expect_or_tok(Token::Ident, "argument name")?;
            //                 let arg_name = self.slice().to_string();
            //                 let arg_span = self.span();
            //                 let arg_type = if self.peek() == Token::Colon {
            //                     self.next();
            //                     Some(self.parse_expr(data)?)
            //                 } else {
            //                     None
            //                 };
            //                 let arg_default = if self.peek() == Token::Assign {
            //                     self.next();
            //                     Some(self.parse_expr(data)?)
            //                 } else {
            //                     None
            //                 };
            //                 args.push((arg_name, arg_type, arg_default));
            //                 arg_areas.push(arg_span);
            //                 self.peek_expect_tok(Token::RParen | Token::Comma)?;
            //                 self.skip_tok(Token::Comma);
            //             }
            //             self.next();
            //             let ret_type = if self.peek() == Token::Arrow {
            //                 self.next();
            //                 Some(self.parse_expr(data)?)
            //             } else {
            //                 None
            //             };
            //             self.expect_tok(Token::FatArrow)?;
            //             let code = self.parse_expr(data)?;

            //             let key = data.insert_expr(
            //                 Expression::Func {
            //                     args,
            //                     ret_type,
            //                     code,
            //                 },
            //                 start.extend(self.span()),
            //             );

            //             data.func_arg_spans.insert(key, arg_areas);

            //             Ok(key)
            //         } else {
            //             let mut args = vec![];
            //             while self.peek() != Token::RParen {
            //                 let arg = self.parse_expr(data)?;
            //                 args.push(arg);
            //                 self.peek_expect_tok(Token::RParen | Token::Comma)?;
            //                 self.skip_tok(Token::Comma);
            //             }
            //             self.next();
            //             self.expect_tok(Token::Arrow)?;
            //             let ret_type = self.parse_expr(data)?;

            //             Ok(data.insert_expr(
            //                 Expression::FuncPattern { args, ret_type },
            //                 start.extend(self.span()),
            //             ))
            //         }
            //     }
            // }
            Token::LSqBracket => {
                self.next();

                let mut elems = vec![];
                while self.peek() != Token::RSqBracket {
                    elems.push(Box::new(self.parse_value(data)));

                    if !matches!(self.peek(), Token::RSqBracket | Token::Comma) {
                        self.expected_err(self.next(), ", or ]");
                    }

                    self.skip_tok(Token::Comma);
                }
                self.next();

                Values::Value(Value::Array(elems))
            }

            Token::LBracket => {
                self.next();

                Values::Constant(Constant::Block)
            }

            Token::TrigFnBracket => {
                self.next();

                Values::Constant(Constant::TriggerFunc)
            }

            _ => Values::Constant(Constant::Unknown),
        }
    }

    pub fn parse_statement(&mut self, data: &mut DocData) -> LineKey {
        let first_comment_span = self.span();

        let mut comments = Vec::new();
        while matches!(self.peek(), Token::DocComment) {
            comments.push(self.slice().to_string());
        }

        let line = match self.peek() {
            Token::TypeDef => {
                self.next();
                self.expect_tok(Token::TypeIndicator);
                let typ_name = self.slice().to_string();

                Line::Type {
                    ident: Value::TypeIndicator(typ_name),
                }
            }
            Token::Impl => {
                self.next();

                let type_name = self.slice().to_string();

                Line::Impl {
                    ident: Value::TypeIndicator(type_name),
                }
            }
            Token::Ident => {
                self.next();
                let var_name = self.slice().to_string();
                self.next();
                let value = self.parse_value(data);

                Line::AssociatedConst {
                    ident: Value::Ident(var_name),
                    value,
                }
            }
            _ => {
                // module comment (first line)
                if first_comment_span.start == 0 {
                    Line::Empty
                } else {
                    panic!("doc comments can only be added to:\n - top of file (module comment)\n  - global constant variables\n - type definitions\n - type members");
                }
            }
        };

        data.data.insert((comments, line, self.source.clone()))
    }

    pub fn parse(&mut self, data: &mut DocData) -> Lines {
        let mut statements = vec![];
        while !matches!(self.peek(), Token::Eof | Token::RBracket) {
            statements.push(self.parse_statement(data));
        }
        statements
    }
}
