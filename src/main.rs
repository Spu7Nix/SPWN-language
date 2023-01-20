#![deny(unused_must_use)]
#![allow(clippy::result_large_err)] // shut the fuck up clippy Lmao

mod compiling;
mod error;
mod gd;
mod lexing;
mod parsing;
mod sources;
mod util;
mod vm;

use std::rc::Rc;
use std::{io::Write, path::PathBuf};

use crate::compiling::compiler::Compiler;
use crate::{
    compiling::bytecode::Constant,
    lexing::tokens::Token,
    parsing::parser::{Interner, Parser},
    sources::SpwnSource,
    vm::opcodes::Opcode,
};
use colored::Colorize;
use compiling::bytecode::BytecodeBuilder;
use lasso::Rodeo;

use ahash::RandomState;

fn main() {
    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout().flush().unwrap();

    let interner: Interner = Rodeo::with_hasher(RandomState::new());

    let path = PathBuf::from("test.spwn");

    let src = SpwnSource::File(path);
    let code = src.read().unwrap();

    let mut parser = Parser::new(code.trim_end(), src, interner);

    match parser.parse() {
        Ok(ast) => {
            let interner = Rc::try_unwrap(parser.interner)
                .expect("multiple references still held (how??????????????????)")
                .into_inner();
            let mut compiler = Compiler::new(interner, parser.src);

            match compiler.compile(ast.statements) {
                Ok(bytecode) => {
                    println!("{}", bytecode);

                    let bytes = bincode::serialize(&bytecode).unwrap();
                    println!("{:?}", bytes);

                    std::fs::write("cock.spwnc", &bytes).unwrap();
                }
                Err(err) => err.to_report().display(),
            }
        }
        Err(err) => err.to_report().display(),
    }
}
