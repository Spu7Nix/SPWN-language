use logos::{Lexer, Logos};

use super::lexer::Token;

#[derive(Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a, Token>,
}

impl Parser<'_> {
    pub fn new<S: AsRef<str>>(code: S) -> Self {
        let code = unsafe { Parser::make_static(code.as_ref()) };
        let lexer = Token::lexer(code);
        Parser { lexer }
    }
    unsafe fn make_static<'a>(d: &'a str) -> &'static str {
        std::mem::transmute::<&'a str, &'static str>(d)
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

    pub fn slice(&self) -> &str {
        self.lexer.slice()
    }

    pub fn parse(&mut self, data: &mut ASTData) -> Result<Statements, SyntaxError> {
        let stmts = self.parse_statements(data)?;
        self.expect_tok(Token::Eof)?;
        Ok(stmts)
    }
}
