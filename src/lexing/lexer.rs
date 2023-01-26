#[macro_export]
macro_rules! lexer {
    (
        $(
            $([$name:literal])?
            $tok:ident $(: $(regex($regex:literal))? $(text($text:literal))?)?,
        )*

        @skip: $skip_regex:literal;
        @error: $error_tok:ident;
    ) => {
        use lazy_static::lazy_static;
        use paste::paste;
        use regex::Regex;
        use std::ops::Range;

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Token {
            $(
                $tok,
            )*
            $error_tok,
        }

        paste! {
            lazy_static! {
                static ref SKIP_REGEX: Regex = Regex::new($skip_regex).unwrap();

                $(
                    $($(
                        static ref [<REGEX_ $tok:upper>]: Regex = Regex::new($regex).unwrap();
                    )?)?
                )*
            }

            $(
                $($(
                    const [<TEXT_ $tok:upper>]: &str = $text;
                    const [<TEXT_LEN_ $tok:upper>]: usize = $text.len();
                )?)?
            )*
        }

        #[derive(Clone)]
        pub struct Lexer<'a> {
            code: &'a str,
            last_span: Range<usize>,
            last_slice: &'a str,

            cache: [usize; Token::$error_tok as usize],
            will_match: [bool; Token::$error_tok as usize],
            total_advance: usize,
        }

        impl Token {
            pub fn lex(code: &str) -> Lexer {

                let mut will_match = [false; Token::$error_tok as usize];

                paste! {
                    $(
                        $($(
                            stringify!($regex);
                            will_match[Token::$tok as usize] = [<REGEX_ $tok:upper>].is_match(code);
                        )?
                        $(
                            stringify!($text);
                            will_match[Token::$tok as usize] = code.find([<TEXT_ $tok:upper>]).is_some();
                        )?)?
                    )*
                }

                Lexer {
                    code,
                    last_span: 0..0,
                    last_slice: "",

                    cache: [0; Token::$error_tok as usize],
                    will_match,
                    total_advance: 0,
                }
            }
        }

        impl Lexer<'_> {
            pub fn span(&self) -> Range<usize> {
                self.last_span.clone()
            }

            pub fn slice(&self) -> &str {
                self.last_slice
            }

            fn advance(&mut self, n: usize) {
                self.last_span = (self.last_span.end)..(self.last_span.end + n);
                self.last_slice = &self.code[0..n];
                self.code = &self.code[n..];

                self.total_advance += n
            }

            pub fn next_or_eof(&mut self) -> Token {
                self.next().unwrap_or(Token::Eof)
            }
        }

        impl<'a> Iterator for Lexer<'a> {
            type Item = Token;

            fn next(&mut self) -> Option<Self::Item> {

                if self.code == "" {
                    self.last_span = (self.last_span.end)..(self.last_span.end + 1);
                    self.last_slice = "";
                    return None
                }

                let mut selected: Option<(Token, usize)> = None;

                if let Some(m) = SKIP_REGEX.find(&self.code) {
                    if m.start() == 0 {
                        self.advance(m.end());
                        return self.next();
                    }
                }


                const LOOKAHEAD: usize = 1000;

                let lookahead_len = self.code.len().min(LOOKAHEAD);
                let lookahead = &self.code[0..lookahead_len];


                paste! {
                    $(
                        $($(
                            stringify!($regex);

                            if self.cache[Token::$tok as usize] <= self.total_advance && self.will_match[Token::$tok as usize] {
                                if let Some(m) = [<REGEX_ $tok:upper>].find(lookahead) {
                                    if m.start() == 0 {
                                        if if let Some(s) = &selected {
                                            m.end() > s.1
                                        } else {
                                            true
                                        } {
                                            selected = Some((Token::$tok, m.end()))
                                        }
                                    } else {
                                        self.cache[Token::$tok as usize] = m.start() + self.total_advance
                                    }
                                }
                            }
                        )?
                        $(
                            stringify!($text);

                            if if let Some(s) = &selected {
                                [<TEXT_LEN_ $tok:upper>] > s.1
                            } else {
                                true
                            } && self.will_match[Token::$tok as usize] {
                                if self.cache[Token::$tok as usize] <= self.total_advance {

                                    match lookahead.find([<TEXT_ $tok:upper>]) {
                                        Some(0) => {
                                            selected = Some((Token::$tok, [<TEXT_LEN_ $tok:upper>]))
                                        }
                                        Some(n) => {
                                            self.cache[Token::$tok as usize] = n + self.total_advance
                                        }
                                        None => {
                                            self.cache[Token::$tok as usize] = lookahead_len + self.total_advance
                                        }
                                    }
                                }
                            }
                        )?)?

                    )*
                }

                match selected {
                    Some((tok, l)) => {
                        self.advance(l);
                        Some(tok)
                    }
                    None => {
                        self.advance(1);
                        Some(Token::Error)
                    }
                }

            }
        }

        impl Token {
            pub fn to_str(self) -> &'static str {
                match self {
                    $(
                        $(
                            Self::$tok => $name,
                        )?
                        $($(
                            #[allow(unreachable_patterns)]
                            Self::$tok => $text,
                        )?)?
                    )*
                    Self::$error_tok => "unknown",
                }
            }
        }
    };
}
