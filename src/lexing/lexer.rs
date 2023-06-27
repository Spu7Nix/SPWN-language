use std::ops::Range;

use delve::EnumDisplay;

use super::tokens::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDisplay)]
pub enum LexerError {
    #[delve(display = "Invalid type indicator")]
    InvalidTypeIndicator,
    #[delve(display = "Invalid hex literal")]
    InvalidHexLiteral,
    #[delve(display = "Invalid octal literal")]
    InvalidOctalLiteral,
    #[delve(display = "Invalid binary literal")]
    InvalidBinaryLiteral,
    #[delve(display = "Invalid seximal literal")]
    InvalidSeximalLiteral,
    #[delve(display = "Invalid dozenal literal")]
    InvalidDozenalLiteral,
    #[delve(display = "Invalid base-φ literal")]
    InvalidGoldenLiteral,
    #[delve(display = "Unknown character")]
    UnknownCharacter,
    #[delve(display = "Unterminated block comment")]
    UnterminatedBlockComment,
    #[delve(display = "Unterminated string")]
    UnterminatedString,
    #[delve(display = "Invalid character for raw string")]
    InvalidCharacterForRawString,
}

#[derive(Clone)]
pub struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    token_start: usize,
    token_end: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            token_start: 0,
            token_end: 0,
        }
    }

    #[inline]
    pub fn trivia(&mut self) {
        self.token_start = self.token_end;
    }

    #[inline]
    pub fn read(&mut self) -> Option<u8> {
        self.bytes.get(self.token_end).copied()
    }

    #[inline]
    pub fn read_at(&mut self, pos: isize) -> Option<u8> {
        self.bytes
            .get((self.token_end as isize + pos) as usize)
            .copied()
    }

    #[inline]
    pub fn bump(&mut self, v: isize) {
        self.token_end = (self.token_end as isize + v) as usize;
        // self.token_end += v;
    }

    #[inline]
    pub fn span(&self) -> Range<usize> {
        self.token_start..self.token_end
    }

    #[inline]
    pub fn slice(&self) -> &str {
        &self.src[self.token_start..self.token_end]
    }

    pub fn next_or_eof(&mut self) -> Result<Token, LexerError> {
        self.next().unwrap_or(Ok(Token::Eof))
    }
}

impl<'a> Lexer<'a> {
    fn next_tok(&mut self) -> Option<Result<Token, LexerError>> {
        while let Some(c) = self.read() {
            if is_whitespace(c) {
                self.bump(1);
            } else {
                break;
            }
        }

        self.trivia();

        let curr = self.read()?;
        self.bump(1);

        macro_rules! ret {
            ($v:expr) => {
                return Some(Ok($v))
            };
        }
        macro_rules! is {
            ($pos:expr, $p:pat) => {
                matches!(self.read_at($pos), Some($p))
            };
        }
        macro_rules! numbers {
            () => {
                while is!(0, b'_' | b'0'..=b'9') {
                    self.bump(1)
                }

                match self.read_at(0) {
                    Some(b'.') => {
                        if !is!(1, b'0'..=b'9') {
                            ret!(Token::Int);
                        }
                        self.bump(2);

                        while is!(0, b'_' | b'0'..=b'9') {
                            self.bump(1)
                        }
                        ret!(Token::Float);
                    },
                    Some(b'g') => {
                        self.bump(1);
                        ret!(Token::GroupID)
                    },
                    Some(b'b') => {
                        self.bump(1);
                        ret!(Token::BlockID)
                    },
                    Some(b'c') => {
                        self.bump(1);
                        ret!(Token::ChannelID)
                    },
                    Some(b'i') => {
                        self.bump(1);
                        ret!(Token::ItemID)
                    },
                    _ => ret!(Token::Int),
                }
            };
        }

        match curr {
            b'|' => {
                ret!(match self.read() {
                    Some(b'=') => {
                        self.bump(1);
                        Token::BinOrEq
                    },
                    Some(b'|') => {
                        self.bump(1);
                        Token::Or
                    },
                    _ => {
                        Token::BinOr
                    },
                });
            },
            b'=' => {
                ret!(match self.read() {
                    Some(b'=') => {
                        self.bump(1);
                        Token::Eq
                    },
                    Some(b'>') => {
                        self.bump(1);
                        Token::FatArrow
                    },
                    _ => {
                        Token::Assign
                    },
                });
            },
            b'>' => {
                ret!(match self.read() {
                    Some(b'>') => {
                        self.bump(1);

                        match self.read() {
                            Some(b'=') => {
                                self.bump(1);
                                Token::ShiftRightEq
                            },
                            _ => Token::ShiftRight,
                        }
                    },
                    Some(b'=') => {
                        self.bump(1);

                        Token::Gte
                    },
                    _ => Token::Gt,
                });
            },
            b'<' => {
                ret!(match self.read() {
                    Some(b'<') => {
                        self.bump(1);

                        match self.read() {
                            Some(b'=') => {
                                self.bump(1);
                                Token::ShiftLeftEq
                            },
                            _ => Token::ShiftLeft,
                        }
                    },
                    Some(b'=') => {
                        self.bump(1);

                        Token::Lte
                    },
                    _ => Token::Lt,
                });
            },
            b'%' => {
                ret!(match self.read() {
                    Some(b'=') => {
                        self.bump(1);
                        Token::ModEq
                    },
                    _ => {
                        Token::Mod
                    },
                });
            },
            b'&' => {
                ret!(match self.read() {
                    Some(b'=') => {
                        self.bump(1);
                        Token::BinAndEq
                    },
                    Some(b'&') => {
                        self.bump(1);
                        Token::And
                    },
                    _ => {
                        Token::BinAnd
                    },
                });
            },
            b'\n' => {
                ret!(Token::Newline)
            },
            b'+' => {
                ret!(match self.read() {
                    Some(b'=') => {
                        self.bump(1);
                        Token::PlusEq
                    },
                    _ => {
                        Token::Plus
                    },
                });
            },
            b'-' => {
                ret!(match self.read() {
                    Some(b'=') => {
                        self.bump(1);
                        Token::MinusEq
                    },
                    Some(b'>') => {
                        self.bump(1);
                        Token::Arrow
                    },
                    _ => {
                        Token::Minus
                    },
                });
            },
            b'*' => {
                ret!(match self.read() {
                    Some(b'=') => {
                        self.bump(1);
                        Token::MultEq
                    },
                    _ => {
                        Token::Mult
                    },
                });
            },
            b'/' => {
                ret!(match self.read() {
                    Some(b'=') => {
                        self.bump(1);
                        Token::DivEq
                    },
                    Some(b'*') => {
                        self.bump(1);

                        while let Some(c) = self.read() {
                            if c == b'*' && self.read_at(1) == Some(b'/') {
                                self.bump(2);
                                return self.next();
                            }
                            self.bump(1)
                        }

                        return Some(Err(LexerError::UnterminatedBlockComment));
                    },
                    Some(b'/') => {
                        self.bump(1);

                        while let Some(c) = self.read() {
                            if c == b'\n' {
                                self.bump(1);
                                return self.next();
                            }
                            self.bump(1)
                        }

                        return None;
                    },
                    _ => Token::Div,
                })
            },
            b'^' => {
                ret!(match self.read() {
                    Some(b'=') => {
                        self.bump(1);
                        Token::PowEq
                    },
                    _ => {
                        Token::Pow
                    },
                });
            },
            b'@' => {
                if !is_id_start(self.read()) {
                    return Some(Err(LexerError::InvalidTypeIndicator));
                }
                self.bump(1);

                while is_id_continue(self.read()) {
                    self.bump(1)
                }

                ret!(Token::TypeIndicator);
            },
            b'$' => {
                ret!(Token::Dollar);
            },
            b';' => {
                ret!(Token::Eol);
            },
            b'(' => {
                ret!(Token::LParen);
            },
            b')' => {
                ret!(Token::RParen);
            },
            b'[' => {
                ret!(Token::LSqBracket);
            },
            b']' => {
                ret!(Token::RSqBracket);
            },
            b'{' => {
                ret!(Token::LBracket);
            },
            b'}' => {
                ret!(Token::RBracket);
            },
            b'#' => {
                ret!(Token::Hashtag);
            },
            b',' => {
                ret!(Token::Comma);
            },
            b':' => {
                if is!(0, b':') {
                    self.bump(1);
                    ret!(Token::DoubleColon)
                }
                ret!(Token::Colon);
            },
            0xCE => {
                if is!(0, 0xB5) {
                    self.bump(1);
                    ret!(Token::Epsilon);
                }

                Some(Err(LexerError::UnknownCharacter))
            },
            b'?' => {
                ret!(match self.read() {
                    Some(b'g') => {
                        self.bump(1);
                        Token::ArbitraryGroupID
                    },
                    Some(b'c') => {
                        self.bump(1);
                        Token::ArbitraryChannelID
                    },
                    Some(b'b') => {
                        self.bump(1);
                        Token::ArbitraryBlockID
                    },
                    Some(b'i') => {
                        self.bump(1);
                        Token::ArbitraryItemID
                    },
                    _ => {
                        Token::QMark
                    },
                });
            },
            b'!' => {
                ret!(match self.read() {
                    Some(b'=') => {
                        self.bump(1);
                        Token::Neq
                    },
                    Some(b'{') => {
                        self.bump(1);
                        Token::TrigFnBracket
                    },
                    _ => {
                        Token::ExclMark
                    },
                });
            },
            b'.' => match (self.read(), self.read_at(1)) {
                (Some(b'.'), Some(b'.')) => {
                    self.bump(2);
                    ret!(Token::Spread)
                },
                (Some(b'.'), _) => {
                    self.bump(1);
                    ret!(Token::Range)
                },
                _ => {
                    ret!(Token::Dot)
                },
            },
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                while is_id_continue(self.read()) {
                    self.bump(1)
                }

                ret!(match self.slice() {
                    "true" => Token::True,
                    "false" => Token::False,
                    "obj" => Token::Obj,
                    "trigger" => Token::Trigger,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "for" => Token::For,
                    "mut" => Token::Mut,
                    "in" => Token::In,
                    "try" => Token::Try,
                    "catch" => Token::Catch,
                    "match" => Token::Match,
                    "throw" => Token::Throw,
                    "return" => Token::Return,
                    "break" => Token::Break,
                    "continue" => Token::Continue,
                    "type" => Token::Type,
                    "impl" => Token::Impl,
                    "overload" => Token::Overload,
                    "unary" => Token::Unary,
                    "dbg" => Token::Dbg,
                    "private" => Token::Private,
                    "extract" => Token::Extract,
                    "import" => Token::Import,
                    "self" => Token::Slf,
                    "is" => Token::Is,
                    "as" => Token::As,
                    "_" => Token::Any,
                    "r" => {
                        macro_rules! raw_string {
                            ($c:expr, $hash:expr) => {
                                loop {
                                    match self.read() {
                                        Some(t) if $c == t => {
                                            self.bump(1);
                                            let mut count = $hash;

                                            loop {
                                                if count == 0 {
                                                    ret!(Token::RawString)
                                                }
                                                match self.read() {
                                                    Some(b'#') => {
                                                        count -= 1;
                                                        self.bump(1);
                                                    },
                                                    None => {
                                                        return Some(Err(
                                                            LexerError::UnterminatedString,
                                                        ));
                                                    },
                                                    _ => {
                                                        self.bump(1);
                                                    },
                                                }
                                            }
                                        },
                                        None => {
                                            return Some(Err(LexerError::UnterminatedString));
                                        },
                                        _ => self.bump(1),
                                    }
                                }
                            };
                        }

                        match self.read() {
                            Some(b'#') => {
                                let mut hash = 0_u32;
                                while is!(0, b'#') {
                                    hash += 1;
                                    self.bump(1)
                                }
                                // println!("glabba {} {:?}", hash, self.read().map(|b| b as char));
                                if !is!(0, b'"' | b'\'') {
                                    return Some(Err(LexerError::InvalidCharacterForRawString));
                                }
                                let c = self.read().unwrap();
                                self.bump(1);

                                raw_string!(c, hash);
                            },
                            Some(c @ (b'"' | b'\'')) => {
                                self.bump(1);
                                raw_string!(c, 0_u32);
                            },
                            _ => Token::Ident,
                        }
                    },
                    _ => {
                        match self.read() {
                            Some(b'"' | b'\\') => {
                                if self.read_at(-1) == Some(b'r') {
                                    self.bump(-1);
                                }
                                Token::StringFlags
                            },
                            Some(b'#') => {
                                if self.read_at(-1) == Some(b'r') {
                                    self.bump(-1);
                                    Token::StringFlags
                                } else {
                                    Token::Ident
                                }
                            },
                            _ => Token::Ident,
                        }
                    },
                });
            },
            b'0' => match self.read() {
                Some(b'x') => {
                    self.bump(1);

                    if !is!(0, b'A'..=b'F' | b'a'..=b'f' | b'0'..=b'9') {
                        return Some(Err(LexerError::InvalidHexLiteral));
                    }
                    self.bump(1);

                    while is!(0, b'A'..=b'F' | b'a'..=b'f' | b'0'..=b'9' | b'_') {
                        self.bump(1)
                    }

                    ret!(Token::HexInt);
                },
                Some(b'o') => {
                    self.bump(1);

                    if !is!(0, b'0'..=b'7') {
                        return Some(Err(LexerError::InvalidOctalLiteral));
                    }
                    self.bump(1);

                    while is!(0, b'0'..=b'7' | b'_') {
                        self.bump(1)
                    }

                    ret!(Token::OctalInt);
                },
                Some(b'b') => {
                    self.bump(1);

                    if !is!(0, b'0'..=b'1') {
                        return Some(Err(LexerError::InvalidBinaryLiteral));
                    }
                    self.bump(1);

                    while is!(0, b'0'..=b'1' | b'_') {
                        self.bump(1)
                    }

                    ret!(Token::BinaryInt);
                },
                Some(b's') => {
                    self.bump(1);

                    if !is!(0, b'0'..=b'5') {
                        return Some(Err(LexerError::InvalidSeximalLiteral));
                    }
                    self.bump(1);

                    while is!(0, b'0'..=b'5' | b'_') {
                        self.bump(1)
                    }

                    ret!(Token::SeximalInt);
                },
                _ => match [self.read_at(0), self.read_at(1)] {
                    [Some(0xcf), Some(0x87)] => {
                        self.bump(2);

                        if !is!(0, b'0'..=b'9' | b'a'..=b'b') {
                            return Some(Err(LexerError::InvalidDozenalLiteral));
                        }
                        self.bump(1);

                        while is!(0, b'0'..=b'9' | b'a'..=b'b' | b'_') {
                            self.bump(1)
                        }

                        ret!(Token::DozenalInt);
                    },
                    [Some(0xcf), Some(0x86)] => {
                        self.bump(2);

                        if !is!(0, b'0'..=b'1') {
                            return Some(Err(LexerError::InvalidGoldenLiteral));
                        }
                        self.bump(1);

                        while is!(0, b'0'..=b'1' | b'_') {
                            self.bump(1)
                        }

                        ret!(Token::GoldenFloat);
                    },
                    _ => {
                        numbers!();
                    },
                },
            },
            b'1'..=b'9' => {
                numbers!();
            },
            c @ (b'"' | b'\'') => loop {
                match self.read() {
                    Some(t) if c == t => {
                        self.bump(1);
                        ret!(Token::String)
                    },
                    Some(b'\\') => {
                        self.bump(2);
                    },
                    None => {
                        return Some(Err(LexerError::UnterminatedString));
                    },
                    _ => self.bump(1),
                }
            },
            _ => Some(Err(LexerError::UnknownCharacter)),
        }
    }
}

#[inline]
const fn is_id_start(b: Option<u8>) -> bool {
    matches!(b, Some(b'A'..=b'Z' | b'a'..=b'z' | b'_'))
}
#[inline]
const fn is_id_continue(b: Option<u8>) -> bool {
    matches!(b, Some(b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_'))
}

#[inline]
const fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r')
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_tok()
    }
}
