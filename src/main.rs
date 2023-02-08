#![deny(unused_must_use)]
#![allow(clippy::result_large_err)] // shut the fuck up clippy Lmao
#![allow(clippy::type_complexity)] // shut the fuck up clippy Lmao
#![allow(clippy::unit_arg)] // shut the fuck up clippy Lmao

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
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser as _;
use cli::Settings;
use colored::Colorize;
use gd::gd_object::GdObject;
use lasso::Rodeo;
use slotmap::SecondaryMap;
use spinoff::spinners::SpinnerFrames;
use spinoff::{Spinner as SSpinner, *};

use crate::cli::{Arguments, Command};
use crate::compiling::compiler::{Compiler, TypeDefMap};
use crate::gd::{gd_object, levelstring};
use crate::parsing::ast::Spannable;
use crate::parsing::parser::Parser;
use crate::sources::{BytecodeMap, SpwnSource};
use crate::util::{HexColorize, RandomState};
use crate::vm::interpreter::{FuncCoord, Vm};
use crate::vm::opcodes::{Opcode, Register};

struct Spinner {
    frames: SpinnerFrames,
    spinner: Option<(SSpinner, String)>,
}
impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: spinner!(["◜ ", "◠ ", "◝ ", "◞ ", "◡ ", "◟ "], 50),
            spinner: None,
        }
    }

    pub fn start(&mut self, msg: String) {
        self.spinner = Some((SSpinner::new(self.frames.clone(), msg.clone(), None), msg));
    }

    pub fn fail(&mut self, msg: Option<String>) {
        let (spinner, curr_msg) = self.spinner.take().unwrap();
        spinner.stop_with_message(&format!(
            "{} {:>20}",
            curr_msg,
            "(Failed)".bright_red().bold()
        ));
        if let Some(m) = msg {
            eprintln!("{m}");
        }
    }

    pub fn complete(&mut self, msg: Option<String>) {
        let (spinner, curr_msg) = self.spinner.take().unwrap();

        spinner.stop_with_message(&format!(
            "{} ✅",
            &(if let Some(m) = msg { m } else { curr_msg }),
        ));
    }
}

const READING_COLOR: u32 = 0x7F94FF;
const PARSING_COLOR: u32 = 0x59C7FF;
const COMPILING_COLOR: u32 = 0xFFC759;
const RUNNING_COLOR: u32 = 0xFF59C7;

fn main() -> Result<(), Box<dyn Error>> {
    // println!("{:?}", format!("{}⠋{}", "[".dimmed(), "]".dimmed()));
    assert_eq!(4, std::mem::size_of::<Opcode<Register>>());

    let args = Arguments::parse();
    let mut spinner = Spinner::new();

    if args.no_color {
        std::env::set_var("NO_COLOR", "true");
    }

    println!();

    match args.command {
        Command::Build { file, settings } => {
            let gd_path = if !settings.no_level {
                Some(
                    // if save_file != None {
                    //     PathBuf::from(save_file.expect("what"))
                    // } else
                    if cfg!(target_os = "windows") {
                        PathBuf::from(std::env::var("localappdata").expect("No local app data"))
                            .join("GeometryDash/CCLocalLevels.dat")
                    } else if cfg!(target_os = "macos") {
                        PathBuf::from(std::env::var("HOME").expect("No home directory"))
                            .join("Library/Application Support/GeometryDash/CCLocalLevels.dat")
                    } else if cfg!(target_os = "linux") {
                        PathBuf::from(std::env::var("HOME").expect("No home directory"))
                        .join(".steam/steam/steamapps/compatdata/322170/pfx/drive_c/users/steamuser/Local Settings/Application Data/GeometryDash/CCLocalLevels.dat")
                    } else if cfg!(target_os = "android") {
                        PathBuf::from("/data/data/com.robtopx.geometryjump/CCLocalLevels.dat")
                    } else {
                        panic!("Unsupported operating system");
                    },
                )
            } else {
                None
            };

            let level_string = if !settings.no_level {
                if let Some(gd_path) = &gd_path {
                    spinner.start(format!(
                        "{:20}",
                        "Reading savefile...".color_hex(READING_COLOR).bold()
                    ));

                    let mut file = fs::File::open(gd_path)?;
                    let mut file_content = Vec::new();

                    match file.read_to_end(&mut file_content) {
                        Ok(..) => (),
                        Err(e) => {
                            spinner.fail(Some(format!(
                                "❌  {} {}",
                                "Error reading savefile:".bright_red().bold(),
                                e
                            )));

                            std::process::exit(1);
                        }
                    }

                    let mut level_string = match levelstring::get_level_string(
                        file_content,
                        settings.level_name.as_ref(),
                    ) {
                        Ok(s) => s,
                        Err(e) => {
                            spinner.fail(Some(format!(
                                "❌  {} {}",
                                "Error reading level:".bright_red().bold(),
                                e
                            )));

                            std::process::exit(1);
                        }
                    };

                    spinner.complete(None);

                    if level_string.is_empty() {}

                    gd_object::remove_spwn_objects(&mut level_string);

                    level_string
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let SpwnOutput {
                objects,
                triggers,
                id_counters,
            } = run_spwn(file, &settings, &mut spinner)?;

            println!(
                "\n{} objects added",
                objects.len().to_string().bright_white().bold()
            );

            let (new_ls, used_ids) = gd_object::append_objects(objects, &level_string)?;

            println!("\n{}", "Level uses:".bright_green().bold());

            for (i, len) in used_ids.iter().enumerate() {
                println!(
                    "{}",
                    &format!(
                        "{} {}",
                        len.to_string().bright_white().bold(),
                        ["groups", "channels", "block IDs", "item IDs"][i]
                    ),
                );
            }

            match gd_path {
                Some(gd_path) => {
                    println!(
                        "\n📝  {:20}",
                        "Writing back to savefile...".bright_cyan().bold()
                    );

                    levelstring::encrypt_level_string(
                        new_ls,
                        level_string,
                        gd_path,
                        settings.level_name,
                    )?;

                    println!(
                        "\n👍  {}  🙂",
                        "Written to save. You can now open Geometry Dash again!"
                            .bright_green()
                            .bold(),
                    );
                }

                None => println!("\nOutput: {new_ls}",),
            };
        }
    };

    Ok(())
}

struct SpwnOutput {
    pub objects: Vec<GdObject>,
    pub triggers: Vec<GdObject>,
    pub id_counters: [usize; 4],
}

fn run_spwn(
    file: PathBuf,
    settings: &Settings,
    spinner: &mut Spinner,
) -> Result<SpwnOutput, Box<dyn Error>> {
    let interner = Rc::new(RefCell::new(Rodeo::with_hasher(RandomState::new())));

    let src = SpwnSource::File(file);
    let code = match src.read() {
        Some(c) => c,
        None => {
            spinner.fail(Some(format!(
                "❌  {}",
                "Error reading SPWN file".bright_red().bold()
            )));
            std::process::exit(1);
        }
    };

    spinner.start(format!(
        "{:20}",
        "Parsing...".color_hex(PARSING_COLOR).bold()
    ));

    let mut parser = Parser::new(&code, src, Rc::clone(&interner));

    match parser.parse() {
        Ok(ast) => {
            spinner.complete(None);

            spinner.start(format!(
                "{:20}",
                "Compiling...".color_hex(COMPILING_COLOR).bold()
            ));

            let mut map = BytecodeMap::default();
            let mut typedefs = TypeDefMap::default();

            let mut compiler = Compiler::new(
                Rc::clone(&interner),
                parser.src.clone(),
                settings,
                &mut map,
                &mut typedefs,
            );

            match compiler.compile(ast.statements) {
                Ok(_) => (),
                Err(err) => {
                    spinner.fail(None);
                    err.to_report().display();

                    std::process::exit(1);
                }
            }

            spinner.complete(None);
            println!("{:#?}", compiler.type_defs.def_map);

            let mut types = SecondaryMap::new();
            for (info, k) in &compiler.type_defs.def_map {
                types.insert(k.value, info.clone().spanned(k.span));
            }

            if settings.debug_bytecode {
                for (k, b) in &map.map {
                    b.debug_str(k)
                }
            }

            // spinner.start(format!(
            //     "{:20}",
            //     "Building...".color_hex(RUNNING_COLOR).bold()
            // ));

            println!(
                "\n{}",
                "════╡ Output ╞══════════════════════"
                    .bright_yellow()
                    .bold(),
            );

            let mut vm = Vm::new(&map, interner, types);

            let key = vm.src_map[&parser.src];
            let start = FuncCoord::new(0, key);

            vm.push_call_stack(start, 0, false, None);

            match vm.run_program() {
                Ok(_) => Ok({
                    println!(
                        "\n{}",
                        "════════════════════════════════════"
                            .bright_yellow()
                            .bold()
                    );

                    //spinner.complete(None);

                    SpwnOutput {
                        objects: vm.objects,
                        triggers: vm.triggers,
                        id_counters: vm.id_counters,
                    }
                }),
                Err(err) => {
                    println!(
                        "\n{}",
                        "════════════════════════════════════"
                            .bright_yellow()
                            .bold()
                    );

                    //spinner.fail(None);

                    err.to_report().display();
                    std::process::exit(1);
                }
            }
        }
        Err(err) => {
            spinner.fail(None);

            err.to_report().display();
            std::process::exit(1);
        }
    }
}
