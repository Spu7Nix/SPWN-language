#![deny(unused_must_use)]
#![allow(clippy::result_large_err)] // shut the fuck up clippy Lmao

mod error;
mod lexing;
mod parsing;
mod sources;
mod pckp;

use std::{io::Write, path::PathBuf};

use crate::{lexing::tokens::Token, parsing::parser::Parser, sources::SpwnSource};
use lasso::Rodeo;

use ahash::RandomState;

fn main() {
    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout().flush().unwrap();

    let interner: Rodeo<lasso::Spur, RandomState> = Rodeo::with_hasher(RandomState::new());

    let path = PathBuf::from("test.spwn");

    let src = SpwnSource::File(path);
    let code = src.read().unwrap();

    let mut parser = Parser::new(code.trim_end(), src, interner);

    match parser.parse() {
        Ok(ast) => {
            println!("{:#?}", ast)
        }
        Err(err) => err.to_report().display(),
    }
}
