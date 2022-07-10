use logos::{Lexer, Logos, Span};

use super::ast::{Constant, DocData, Line, LineKey, Lines, MacroArg, Value, Values};
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

    pub fn peek_many(&mut self, n: usize) -> Token {
        let mut peek = self.lexer.clone();
        let mut last = peek.next();

        for _ in 0..(n - 1) {
            last = peek.next();
        }

        last.unwrap_or(Token::Eof)
    }

    fn err_file_pos(&mut self) -> String {
        format!(
            "{} @ l{}:c{}",
            self.source.source.display(),
            self.span().start,
            self.span().end
        )
    }

    pub fn expected_err(&mut self, expected: &str, found: &str) {
        panic!(
            "expected token: `{}`, found: `{}` ({})!",
            expected,
            found.to_string(),
            self.err_file_pos(),
        )
    }

    pub fn peek_expect_tok(&mut self, tok: Token) {
        let next = self.peek();

        if next != tok {
            self.next();

            self.expected_err(&tok.to_string(), &next.to_string());
        }
    }

    pub fn expect_or_tok(&mut self, tok: Token, or: &str) {
        let next = self.next();

        if next != tok {
            self.expected_err(&tok.to_string(), or);
        }
    }

    pub fn expect_tok(&mut self, tok: Token) {
        let next = self.next();

        if next != tok {
            self.expected_err(&tok.to_string(), &next.to_string());
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
                    data.known_idents
                        .insert(var_name.clone(), self.source.clone());

                    Values::Value(Value::Ident(var_name))
                }
            }
            Token::TypeIndicator => {
                self.next();
                let type_name = self.slice().to_string();

                data.known_idents
                    .insert(type_name.clone(), self.source.clone());

                Values::Value(Value::TypeIndicator(type_name))
            }
            Token::LParen => {
                self.next();

                if self.peek() == Token::RParen && self.peek_many(2) != Token::FatArrow {
                    self.next();

                    Values::Constant(Constant::Empty)
                } else {
                    let mut depth = 1;
                    let mut check = self.clone();

                    loop {
                        match check.peek() {
                            Token::LParen => {
                                check.next();
                                depth += 1;
                            }
                            Token::RParen => {
                                check.next();
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            Token::Eof => self.expected_err("closing paren `)`", "EOF"),
                            _ => {
                                check.next();
                            }
                        }
                    }

                    let is_pattern = match check.peek() {
                        Token::FatArrow => false,
                        Token::Arrow => {
                            check.next();
                            check.parse_value(data);

                            check.peek() != Token::FatArrow
                        }
                        _ => {
                            let value = self.parse_value(data);
                            self.expect_tok(Token::RParen);

                            return value;
                        }
                    };

                    if !is_pattern {
                        let mut args = vec![];

                        while self.peek() != Token::RParen {
                            self.expect_or_tok(Token::Ident, "argument name");

                            let arg_name = Some(self.slice().to_string());
                            let arg_type = if self.peek() == Token::Colon {
                                self.next();

                                Some(self.parse_value(data))
                            } else {
                                None
                            };
                            let arg_default = if self.peek() == Token::Assign {
                                self.next();

                                Some(self.parse_value(data))
                            } else {
                                None
                            };
                            args.push(MacroArg {
                                name: arg_name,
                                typ: arg_type,
                                default: arg_default,
                            });

                            if !matches!(self.peek(), Token::RParen | Token::Comma) {
                                let found = self.next();

                                self.expected_err(", or ]", &found.to_string());
                            }

                            self.skip_tok(Token::Comma);
                        }

                        self.next();

                        let ret_type = if self.peek() == Token::Arrow {
                            self.next();
                            Some(Box::new(self.parse_value(data)))
                        } else {
                            None
                        };

                        self.expect_tok(Token::FatArrow);

                        Values::Value(Value::Macro {
                            args,
                            ret: ret_type,
                        })
                    } else {
                        let mut args = vec![];

                        while self.peek() != Token::RParen {
                            let arg_typ = Some(self.parse_value(data));

                            args.push(MacroArg {
                                name: None,
                                typ: arg_typ,
                                default: None,
                            });

                            if !matches!(self.peek(), Token::RParen | Token::Comma) {
                                let found = self.next();

                                self.expected_err(", or ]", &found.to_string())
                            }

                            self.skip_tok(Token::Comma);
                        }

                        self.next();
                        self.expect_tok(Token::Arrow);

                        let ret_type = Some(Box::new(self.parse_value(data)));

                        Values::Value(Value::Macro {
                            args,
                            ret: ret_type,
                        })
                    }
                }
            }
            Token::LSqBracket => {
                self.next();

                let mut elems = vec![];
                while self.peek() != Token::RSqBracket {
                    elems.push(Box::new(self.parse_value(data)));

                    if !matches!(self.peek(), Token::RSqBracket | Token::Comma) {
                        let found = self.next();

                        self.expected_err(", or ]", &found.to_string())
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

    pub fn parse_statement(&mut self, data: &'_ mut DocData) -> Option<LineKey> {
        let mut comments = vec![];

        match self.peek() {
            Token::DocComment => {
                let first_comment_span = self.span();

                while matches!(self.next(), Token::DocComment) {
                    comments.push(self.slice().to_string());
                }

                let line = match self.next() {
                    Token::TypeDef => {
                        self.expect_tok(Token::TypeIndicator);
                        let type_name = self.slice().to_string();

                        data.known_idents
                            .insert(type_name.clone(), self.source.clone());

                        Line::Type {
                            ident: Value::TypeIndicator(type_name),
                        }
                    }
                    Token::Impl => {
                        let type_name = self.slice().to_string();

                        data.known_idents
                            .insert(type_name.clone(), self.source.clone());

                        Line::Impl {
                            ident: Value::TypeIndicator(type_name),
                        }
                    }
                    Token::Ident => {
                        let var_name = self.slice().to_string();
                        self.next();
                        let value = self.parse_value(data);

                        data.known_idents
                            .insert(var_name.clone(), self.source.clone());

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

                return Some(data.data.insert((comments, line, self.source.clone())));
            }
            _ => None,
        }
    }

    pub fn parse(&mut self, data: &mut DocData) -> Lines {
        let mut statements = vec![];

        while !matches!(self.peek(), Token::Eof | Token::RBracket) {
            //println!("{:#?}", data.data);

            if let Some(key) = self.parse_statement(data) {
                statements.push(key);
            }
        }

        self.expect_tok(Token::Eof);

        statements
    }
}
