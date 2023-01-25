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
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use lasso::Rodeo;
use slotmap::SlotMap;

use crate::cli::FileSettings;
use crate::compiling::compiler::Compiler;
use crate::lexing::tokens::Token;
use crate::parsing::parser::Parser;
use crate::sources::SpwnSource;
use crate::util::RandomState;
use crate::vm::context::FullContext;
use crate::vm::interpreter::{FuncCoord, Vm};
use crate::vm::opcodes::{Opcode, Register};

fn main() {
    assert_eq!(4, std::mem::size_of::<Opcode<Register>>());

    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout().flush().unwrap();

    let interner = Rc::new(RefCell::new(Rodeo::with_hasher(RandomState::new())));

    let path = PathBuf::from("test.spwn");

    let src = SpwnSource::File(path);
    let code = src.read().unwrap();

    let mut parser = Parser::new(code.trim_end(), src, Rc::clone(&interner));

    match parser.parse() {
        Ok(ast) => {
            let mut file_settings = FileSettings::default();
            file_settings.apply_attributes(&ast.file_attributes);

            let mut compiler =
                Compiler::new(Rc::clone(&interner), parser.src.clone(), &file_settings);

            match compiler.compile(ast.statements) {
                Ok(bytecode) => {
                    if file_settings.debug_bytecode {
                        bytecode.debug_str(&parser.src);
                    }

                    let mut programs = SlotMap::default();
                    let key = programs.insert(&bytecode);
                    let start = FuncCoord::new(0, key);

                    let mut vm = Vm {
                        memory: SlotMap::default(),
                        interner,
                        programs,
                        id_counters: [0; 4],
                        contexts: FullContext::new(start),
                    };

                    vm.push_func_regs(start);

                    match vm.run_func(start) {
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
