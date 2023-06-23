mod exprs;
mod patterns;
mod stmts;
mod util;

use std::cell::RefCell;
use std::rc::Rc;

use lasso::Spur;

use super::ast::Ast;
use super::attributes::FileAttribute;
use super::error::SyntaxError;
use crate::lexing::tokens::{Lexer, Token};
use crate::sources::{CodeArea, CodeSpan, SpwnSource};
use crate::util::Interner;

#[derive(Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    pub src: Rc<SpwnSource>,
    interner: Rc<RefCell<Interner>>,
}

pub type ParseResult<T> = Result<T, SyntaxError>;

impl<'a> Parser<'a> {
    pub fn new(code: &'a str, src: Rc<SpwnSource>, interner: Rc<RefCell<Interner>>) -> Self {
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

    pub fn peek_strict(&self) -> Token {
        let mut peek = self.lexer.clone();
        peek.next_or_eof()
    }

    pub fn next_is(&self, tok: Token) -> bool {
        self.peek() == tok
    }

    pub fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            src: Rc::clone(&self.src),
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

    fn intern_string<T: AsRef<str>>(&self, string: T) -> Spur {
        self.interner.borrow_mut().get_or_intern(string)
    }

    pub fn resolve(&self, s: &Spur) -> Box<str> {
        self.interner.borrow_mut().resolve(s).into()
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
