use logos::{Lexer, Logos};

use super::lexer::Token;
use super::ast::{Statements, DocData};

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

    pub fn parse_statement(&mut self, data: &mut DocData) -> StmtKey {
        let stmt = match self.peek() {
            Token::TypeDef => {
                // self.next();
                // self.expect_tok(Token::TypeIndicator)?;
                // let typ_name = self.slice()[1..].to_string();
                // Statement::TypeDef(typ_name)
            }
            Token::Impl => {
                // self.next();

                // let typ = self.parse_expr(data)?;

                // self.expect_tok(Token::LBracket)?;
                // let dictlike = self.parse_dictlike(data)?;

                // data.impl_spans.insert(stmt_key, dictlike.item_spans);

                // Statement::Impl(typ, dictlike.items)
            }
            Token::Ident => {
                // if self.peek_many(2) == Token::Assign {
                //     self.next();
                //     let var = self.slice().to_string();
                //     self.next();
                //     let value = self.parse_expr(data)?;
                //     Statement::Assign(var, value)
                // } else {
                //     Statement::Expr(self.parse_expr(data)?)
                // }
            }
            _ => todo!(), //Statement::Expr(self.parse_expr(data)?),
        };

        // data.stmts[stmt_key] = (stmt, start.extend(self.span()));

        // Ok(stmt_key)
    }

    pub fn parse_statements(&mut self, data: &mut DocData) -> Statements {
        let mut statements = vec![];
        while !matches!(self.peek(), Token::Eof | Token::RBracket) {
            statements.push(self.parse_statement(data));
        }
        statements
    }

    pub fn parse(&mut self, data: &mut DocData) -> Statements {
        let stmts = self.parse_statements(data);
        stmts
    }
}
