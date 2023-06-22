#![allow(clippy::result_large_err)]
#![deny(unused_must_use)]

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use clap::Parser as _;
use cli::Settings;
use colored::Colorize;
use lasso::Rodeo;
use sources::BytecodeMap;

use crate::cli::{Arguments, Command};
use crate::compiling::builder::ProtoBytecode;
use crate::compiling::bytecode::ConstID;
use crate::compiling::compiler::Compiler;
use crate::compiling::opcodes::OptOpcode;
use crate::parsing::parser::Parser;
use crate::sources::SpwnSource;
use crate::util::{BasicError, RandomState};

mod cli;
mod compiling;
mod error;
mod gd;
mod interpreting;
mod lexing;
mod parsing;
mod sources;
mod util;

fn run_spwn(settings: &Settings) -> Result<(), Box<dyn Error>> {
    let src = Rc::new(SpwnSource::File("test.spwn".into()));
    let code = src
        .read()
        .ok_or_else(|| BasicError("Failed to read SPWN file".into()))?;

    let interner = Rc::new(RefCell::new(Rodeo::with_hasher(RandomState::new())));
    let mut parser: Parser<'_> = Parser::new(&code, Rc::clone(&src), Rc::clone(&interner));

    let ast = parser.parse().map_err(|e| e.to_report())?;

    let mut bytecode_map = BytecodeMap::default();

    let mut cum = Compiler::new(Rc::clone(&src), settings, &mut bytecode_map, interner);

    cum.compile(&ast, (0..code.len()).into())
        .map_err(|e| e.to_report())?;

    for (src, code) in &*bytecode_map {
        println!("{}", format!("{:?} =============", src).bright_green());
        code.debug_str(&Rc::new(src.clone()))
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    assert_eq!(4, std::mem::size_of::<OptOpcode>());

    let args = Arguments::parse();

    let id: ConstID = 6usize.into();
    println!("{}", id);

    match args.command {
        Command::Build { file, settings } => {
            match run_spwn(&settings) {
                Ok(o) => o,
                Err(e) => {
                    eprint!("❌  {e}");

                    std::process::exit(1);
                },
            };
        },
        _ => todo!(),
    }

    Ok(())
}
