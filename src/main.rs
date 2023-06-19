mod cli;
mod compiling;
mod error;
mod lexing;
mod parsing;
mod sources;
mod util;

use std::cell::RefCell;
use std::error::Error;
use std::fmt::Debug;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser as _;
use cli::Settings;
use colored::Colorize;
use compiling::builder::ProtoBytecode;
use compiling::bytecode::Register;
use lasso::Rodeo;
use spinoff::spinners::SpinnerFrames;
use spinoff::{Spinner as SSpinner, *};

use crate::cli::{Arguments, Command};
use crate::compiling::bytecode::Constant;
use crate::compiling::opcodes::OptOpcode;
use crate::parsing::parser::Parser;
use crate::sources::{CodeSpan, SpwnSource};
use crate::util::{BasicError, HexColorize, ImmutStr, RandomState};

const CORE_PATH: &str = "./libraries/core/";

struct Spinner {
    frames: SpinnerFrames,
    disabled: bool,
    spinner: Option<(SSpinner, String)>,
}
impl Spinner {
    pub fn new(disabled: bool) -> Self {
        Self {
            frames: spinner!(["◜ ", "◠ ", "◝ ", "◞ ", "◡ ", "◟ "], 50),
            spinner: None,
            disabled,
        }
    }

    pub fn start(&mut self, msg: String) {
        if self.disabled {
            println!("{msg}");
        } else {
            self.spinner = Some((SSpinner::new(self.frames.clone(), msg.clone(), None), msg));
        }
    }

    pub fn fail(&mut self, msg: Option<String>) {
        if let Some((spinner, curr_msg)) = self.spinner.take() {
            spinner.stop_with_message(&format!("{curr_msg} ❌"));
        } else {
            println!("\n{}", "══════════════════════════════════".dimmed().bold());
        }
        if let Some(m) = msg {
            eprintln!("\n{m}");
        }
    }

    pub fn complete(&mut self, msg: Option<String>) {
        if let Some((spinner, curr_msg)) = self.spinner.take() {
            if let Some(m) = msg {
                spinner.stop_with_message(&format!("{curr_msg} ✅",));
                println!("{m}");
            } else {
                spinner.clear();
                println!("{curr_msg} ✅")
            }
            return;
        }
        if let Some(m) = msg {
            println!("{m}");
        }
    }
}

const READING_COLOR: u32 = 0x7F94FF;
const PARSING_COLOR: u32 = 0x59C7FF;
const COMPILING_COLOR: u32 = 0xFFC759;
const RUNNING_COLOR: u32 = 0xFF59C7;
const OPTIMISING_COLOR: u32 = 0xA74AFF;

fn main() -> Result<(), Box<dyn Error>> {
    assert_eq!(4, std::mem::size_of::<OptOpcode>());

    let src = SpwnSource::File("test.spwn".into());
    let code = src
        .read()
        .ok_or_else(|| BasicError("Failed to read SPWN file".into()))?;

    let interner = Rc::new(RefCell::new(Rodeo::with_hasher(RandomState::new())));
    let mut parser = Parser::new(&code, src, Rc::clone(&interner));

    let ast = parser.parse().map_err(|e| e.to_report())?;

    println!("{:#?}", ast);

    Ok(())
}
