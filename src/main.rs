#![deny(unused_must_use)]

use std::{io::Write, path::PathBuf};

use crate::{lexing::tokens::Token, sources::SpwnSource};

mod error;
mod lexing;
mod sources;

fn main() {
    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout().flush().unwrap();

    let path = PathBuf::from("test.spwn");

    let src = SpwnSource::File(path);
    let code = src.read().unwrap();

    let mut lexer = Token::lex(&code);
    while let Some(t) = lexer.next() {
        if t == Token::Error {
            println!("sex!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!")
        }
        println!("{:?} {} {:?}", t, lexer.slice(), lexer.span())
    }

    // let mut parser = Parser::new(&code, src);

    // match parser.parse() {
    //     Ok(stmts) => {
    //         println!("{:#?}", stmts)
    //     }
    //     Err(err) => {
    //         println!("{:?}", err)
    //     }
    // }
}
