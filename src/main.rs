#![allow(clippy::result_large_err)]
#![deny(unused_must_use)]

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use clap::Parser as _;
use cli::{BuildSettings, DocSettings};
use colored::Colorize;
use interpreting::vm::Vm;
use lasso::Rodeo;
use sources::BytecodeMap;

use crate::cli::{Arguments, Command};
use crate::compiling::builder::ProtoBytecode;
use crate::compiling::bytecode::Register;
use crate::compiling::compiler::Compiler;
use crate::compiling::opcodes::{ConstID, OptOpcode};
use crate::interpreting::context::{CallInfo, Context, ContextSplitMode};
use crate::interpreting::vm::{FuncCoord, Program};
use crate::parsing::parser::Parser;
use crate::sources::{SpwnSource, TypeDefMap};
use crate::util::{BasicError, RandomState};

mod cli;
mod compiling;
mod doc;
mod error;
mod gd;
mod interpreting;
mod lexing;
mod parsing;
mod sources;
mod util;

fn run_spwn(
    build_settings: BuildSettings,
    doc_settings: DocSettings,
    is_doc_gen: bool,
) -> Result<(), Box<dyn Error>> {
    let src = Rc::new(SpwnSource::File("test.spwn".into()));
    let code = src
        .read()
        .ok_or_else(|| BasicError("Failed to read SPWN file".into()))?;

    let interner = Rc::new(RefCell::new(Rodeo::with_hasher(RandomState::new())));
    let mut parser: Parser<'_> = Parser::new(&code, Rc::clone(&src), Rc::clone(&interner));

    let ast = parser.parse().map_err(|e| e.to_report())?;

    // println!("{:#?}", ast);

    // todo!();

    let mut bytecode_map = BytecodeMap::default();
    let mut type_def_map = TypeDefMap::default();

    let mut cum = Compiler::new(
        Rc::clone(&src),
        &build_settings,
        &doc_settings,
        is_doc_gen,
        &mut bytecode_map,
        &mut type_def_map,
        interner,
    );

    cum.compile(&ast, (0..code.len()).into())
        .map_err(|e| e.to_report())?;

    if build_settings.debug_bytecode {
        for (src, code) in &*bytecode_map {
            code.debug_str(&Rc::new(src.clone()), None)
        }
    }

    let mut vm = Vm::new(false, type_def_map, bytecode_map);

    let program = Program {
        bytecode: vm.bytecode_map.get(&src).unwrap().clone(),
        src,
    };
    let start = FuncCoord {
        program: Rc::new(program),
        func: 0,
    };

    println!("\n{}", "════ Output ══════════════════════".dimmed().bold());

    vm.run_function(
        Context::new(),
        CallInfo {
            func: start,
            return_dest: None,
            call_area: None,
        },
        Box::new(|_| Ok(())),
        ContextSplitMode::Allow,
    )
    .map_err(|e| e.to_report(&vm))?;

    println!("\n{}", "══════════════════════════════════".dimmed().bold());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    assert_eq!(4, std::mem::size_of::<OptOpcode>());

    let args = Arguments::parse();

    match args.command {
        Command::Build { file, settings } => {
            match run_spwn(settings, DocSettings::default(), false) {
                Ok(o) => o,
                Err(e) => {
                    eprint!("❌  {e}");

                    std::process::exit(1);
                },
            };
        },
        Command::Doc { settings } => match run_spwn(BuildSettings::default(), settings, true) {
            Ok(o) => o,
            Err(e) => {
                eprint!("❌  {e}");

                std::process::exit(1);
            },
        },
    }

    Ok(())
}
