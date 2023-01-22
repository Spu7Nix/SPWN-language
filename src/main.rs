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

use std::cell::RefCell;
use std::rc::Rc;
use std::{io::Write, path::PathBuf};

use lasso::Rodeo;

use crate::compiling::compiler::Compiler;
use crate::sources::BytecodeMap;
use crate::util::RandomState;
use crate::vm::interpreter::Vm;
use crate::{lexing::tokens::Token, parsing::parser::Parser, sources::SpwnSource};

fn main() {
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
            let mut compiler = Compiler::new(Rc::clone(&interner), parser.src, &mut bytecode_map);

            match compiler.compile(ast.statements) {
                Ok(bytecode) => {
                    // let interner = Rc::try_unwrap(interner)
                    //     .expect("multiple interner references still held")
                    //     .into_inner();

                    let mut vm = Vm::new(bytecode, Rc::clone(&interner));

                    match vm.run_func(0) {
                        Ok(_) => {}
                        Err(err) => err.to_report().display(),
                    };

                    // for (src, bytecode) in &compiler.map.map {
                    //     println!(
                    //         "GOG: {:?}\n──────────────────────────────────────────────────────",
                    //         src
                    //     );
                    //     bytecode.debug_str(src);
                    //     // println!("{}", bytecode);

                    //     // let bytes = bincode::serialize(&bytecode).unwrap();
                    //     // println!("{:?}", bytes);

                    //     // std::fs::write("cock.spwnc", bytes).unwrap();
                    // }
                }
                Err(err) => err.to_report().display(),
            }
        }
        Err(err) => err.to_report().display(),
    }
}
