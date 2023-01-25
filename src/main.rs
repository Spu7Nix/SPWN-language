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
use crate::sources::{BytecodeMap, SpwnSource};
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

    let mut map = BytecodeMap::default();

    match parser.parse() {
        Ok(ast) => {
            let mut file_settings = FileSettings::default();
            file_settings.apply_attributes(&ast.file_attributes);

            let mut compiler = Compiler::new(
                Rc::clone(&interner),
                parser.src.clone(),
                &file_settings,
                &mut map,
            );

            match compiler.compile(ast.statements) {
                Ok(_) => (),
                Err(err) => {
                    err.to_report().display();
                    return;
                }
            }

            if file_settings.debug_bytecode {
                for (k, b) in &map.map {
                    b.debug_str(k)
                }
                // bytecode.debug_str(&parser.src);
            }

            let bytecode = &map.map[&parser.src];

            let mut vm = Vm::new(interner);

            let key = vm.programs.insert(bytecode);
            let start = FuncCoord::new(0, key);

            // vm.push_call_stack(start, 0, false);

            // match vm.run_program() {
            //     Ok(_) => {}
            //     Err(err) => err.to_report().display(),
            // };
        }
        Err(err) => err.to_report().display(),
    }
}
