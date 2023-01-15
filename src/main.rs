#![deny(unused_must_use)]

use std::{io::Write, path::PathBuf};

use crate::{lexing::tokens::Token, parsing::parser::Parser, sources::SpwnSource};

mod error;
mod lexing;
mod parsing;
mod sources;

fn main() {
    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout().flush().unwrap();

    let path = PathBuf::from("test.spwn");

    let src = SpwnSource::File(path);
    let code = src.read().unwrap();

    let mut parser = Parser::new(code.trim_end(), src);

    match parser.parse() {
        Ok(ast) => {
            println!("attrs: {:#?}", ast.file_attributes);
            println!("{:#?}", ast.statements)
        }
        Err(err) => err.to_report().display(),
    }
}
