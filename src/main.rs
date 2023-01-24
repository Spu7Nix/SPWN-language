#![deny(unused_must_use)]
#![allow(clippy::result_large_err)] // shut the fuck up clippy Lmao

mod cli;
mod compiling;
mod error;
mod gd;
mod lexing;
mod parsing;
mod sources;
mod util;
mod vm;

use std::cell::RefCell;
use std::rc::Rc;
use std::{io::Write, path::PathBuf};

use lasso::Rodeo;

use crate::cli::FileSettings;
use crate::compiling::compiler::Compiler;
use crate::sources::BytecodeMap;
use crate::util::RandomState;
use crate::vm::interpreter::Vm;
use crate::vm::opcodes::{Opcode, Register};
use crate::{lexing::tokens::Token, parsing::parser::Parser, sources::SpwnSource};

fn main() {
    assert_eq!(4, std::mem::size_of::<Opcode<Register>>());

    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout().flush().unwrap();

    let mut bytecode_map = BytecodeMap::default();

    let interner = Rc::new(RefCell::new(Rodeo::with_hasher(RandomState::new())));

    let path = PathBuf::from("test.spwn");

    let src = SpwnSource::File(path);
    let code = src.read().unwrap();

    let mut parser = Parser::new(code.trim_end(), src, Rc::clone(&interner));

    match parser.parse() {
        Ok(ast) => {
            let mut file_settings = FileSettings::default();
            file_settings.apply_attributes(&ast.file_attributes);

            let mut compiler = Compiler::new(
                Rc::clone(&interner),
                parser.src.clone(),
                &mut bytecode_map,
                &file_settings,
            );

            match compiler.compile(ast.statements) {
                Ok(bytecode) => {
                    if file_settings.debug_bytecode {
                        bytecode.debug_str(&parser.src);
                    }

                    let mut vm = Vm::new(bytecode, Rc::clone(&interner));

                    match vm.run_func(0) {
                        Ok(_) => {}
                        Err(err) => err.to_report().display(),
                    };
                }
                Err(err) => err.to_report().display(),
            }
        }
        Err(err) => err.to_report().display(),
    }
}
