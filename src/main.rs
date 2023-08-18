#![deny(unused_must_use, clippy::nonstandard_macro_braces)]
#![allow(
    clippy::result_large_err,
    clippy::too_many_arguments,
    clippy::type_complexity
)]
#![warn(clippy::branches_sharing_code)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]

use std::cell::RefCell;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

use clap::Parser as _;
use cli::{BuildSettings, DocSettings};
use colored::Colorize;
use doc::DocCompiler;
use error::SpwnError;
use gd::gd_object::{GdObject, TriggerObject};
use interpreting::vm::Vm;
use lasso::Rodeo;
use sources::BytecodeMap;

use crate::cli::{Arguments, Command};
use crate::compiling::builder::ProtoBytecode;
use crate::compiling::bytecode::Register;
use crate::compiling::compiler::Compiler;
use crate::compiling::opcodes::{ConstID, OptOpcode};
use crate::gd::gd_object::{self, SPWN_SIGNATURE_GROUP};
use crate::gd::ids::IDClass;
use crate::gd::levelstring;
use crate::gd::optimizer::{optimize, ReservedIds};
use crate::interpreting::context::{CallInfo, Context};
use crate::interpreting::vm::{FuncCoord, Program};
#[cfg(target_os = "windows")]
use crate::liveeditor::win::LiveEditorClient;
#[cfg(target_os = "windows")]
use crate::liveeditor::Message;
use crate::parsing::parser::Parser;
use crate::sources::{SpwnSource, TypeDefMap};
use crate::util::interner::Interner;
use crate::util::spinner::Spinner;
use crate::util::{hyperlink, HexColorize};

mod cli;
mod compiling;
mod doc;
mod error;
mod gd;
mod interpreting;
mod lexing;
#[cfg(target_os = "windows")]
mod liveeditor;
mod parsing;
mod sources;
mod util;

const CORE_PATH: &str = "./libraries/core/";

const READING_COLOR: u32 = 0x7F94FF;
const PARSING_COLOR: u32 = 0x59C7FF;
const COMPILING_COLOR: u32 = 0xFFC759;
const RUNNING_COLOR: u32 = 0xFF59C7;
const OPTIMISING_COLOR: u32 = 0xA74AFF;
const CONNECTING_COLOR: u32 = 0xBAFF4A;

struct SpwnOutput {
    pub objects: Vec<GdObject>,
    pub triggers: Vec<TriggerObject>,
    pub id_counters: [u16; 4],
}

fn main() -> Result<(), SpwnError> {
    assert_eq!(4, std::mem::size_of::<OptOpcode>());

    let args = Arguments::parse();

    if args.no_color {
        std::env::set_var("NO_COLOR", "true");
    }
    if args.use_ascii_errors {
        std::env::set_var("USE_ASCII", "true");
    }

    let mut spinner = Spinner::new(args.no_spinner);

    match args.command {
        Command::Build { file, settings } => {
            let is_live_editor = if cfg!(target_os = "windows") {
                settings.live_editor
            } else {
                false
            };

            let gd_path = if !(settings.no_level || is_live_editor) {
                Some(if let Some(ref sf) = settings.save_file {
                    sf.clone()
                } else if cfg!(target_os = "windows") {
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
                    return Err(SpwnError::UsuppportedOS);
                })
            } else {
                None
            };

            let (level_string, level_name) = if !(settings.no_level || is_live_editor) {
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
                        },
                    }

                    let (mut level_string, level_name) = match levelstring::get_level_string(
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
                        },
                    };

                    spinner.complete(None);

                    gd_object::remove_spwn_objects(&mut level_string);

                    (level_string, level_name)
                } else {
                    (String::new(), String::new())
                }
            } else {
                (String::new(), String::new())
            };

            let SpwnOutput {
                mut objects,
                mut triggers,
                id_counters,
            } = match run_spwn(file, &settings, &mut spinner) {
                Ok(o) => o,
                Err(e) => {
                    spinner.fail(Some(format!("❌  {e}")));

                    std::process::exit(1);
                },
            };

            let reserved = ReservedIds::from_objects(&objects, &triggers);

            if !triggers.is_empty() && !settings.no_optimize {
                spinner.start(format!(
                    "{:20}",
                    "Optimizing triggers...".color_hex(OPTIMISING_COLOR).bold()
                ));

                triggers =
                    optimize::optimize(triggers, id_counters[IDClass::Group as usize], reserved);

                spinner.complete(None);
            }

            objects.extend(gd_object::apply_triggers(triggers));

            println!(
                "\n{} total objects",
                (objects.len()).to_string().bright_white().bold()
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

            println!();

            if settings.console_output {
                println!("{}", hyperlink("todo <playground link>", Some(&new_ls)));
            }

            if is_live_editor {
                #[cfg(target_os = "windows")]
                {
                    spinner.start(format!(
                        "{:20}",
                        "Connecting to Live Editor..."
                            .color_hex(CONNECTING_COLOR)
                            .bold()
                    ));

                    let mut client = LiveEditorClient::try_create_client()?;

                    spinner.complete(None);

                    spinner.start(format!(
                        "{:20}",
                        "Sending level string...".bright_green().bold()
                    ));

                    client.try_send_message(Message::RemoveObjectsByGroup(1001))?;
                    client.try_send_message(Message::AddObjects(&new_ls))?;

                    spinner.complete(None);
                }
            } else if let Some(gd_path) = gd_path {
                spinner.start(format!(
                    r#"{} "{}" {:20}"#,
                    "Writing back to".bright_cyan().bold(),
                    level_name.bright_white().bold(),
                    "..."
                ));

                levelstring::encrypt_level_string(
                    new_ls,
                    level_string,
                    gd_path,
                    &settings.level_name,
                )?;

                spinner.complete(Some(format!(
                    "\n👍  {}  🙂",
                    "Written to save. You can now open Geometry Dash again!"
                        .bright_green()
                        .bold(),
                )));
            }
        },
        Command::Doc { settings } => {
            //
            generate_doc(&settings, &mut spinner)?;
        },
    };

    Ok(())
}

fn run_spwn(
    file: PathBuf,
    settings: &BuildSettings,
    spinner: &mut Spinner,
) -> Result<SpwnOutput, SpwnError> {
    let src = Rc::new(SpwnSource::File(file));
    let code = src.read().ok_or(SpwnError::FailedToReadSpwnFile)?;

    let interner = Interner::new();

    let mut parser: Parser<'_> = Parser::new(&code, Rc::clone(&src), interner.clone());

    spinner.start(format!(
        "{:20}",
        "Parsing...".color_hex(PARSING_COLOR).bold()
    ));

    let ast = parser.parse().map_err(|e| e.to_report())?;

    spinner.complete(None);

    let mut bytecode_map = BytecodeMap::default();
    let mut type_def_map = TypeDefMap::default();

    spinner.start(format!(
        "{:20}",
        "Compiling...".color_hex(COMPILING_COLOR).bold()
    ));

    let mut compiler = Compiler::new(
        Rc::clone(&src),
        settings,
        &mut bytecode_map,
        &mut type_def_map,
        interner,
        parser.deprecated_features,
    );

    compiler
        .compile(&ast, (0..code.len()).into())
        .map_err(|e| e.to_report())?;

    spinner.complete(None);

    if settings.debug_bytecode {
        for (src, code) in &*bytecode_map {
            code.debug_str(&Rc::new(src.clone()), None)
        }
    }

    let mut vm = Vm::new(type_def_map, bytecode_map, settings.trailing.clone());

    let program = Program {
        bytecode: vm.bytecode_map.get(&src).unwrap().clone(),
        src,
    };
    let start = FuncCoord {
        program: Rc::new(program),
        func: 0,
    };

    println!("{:20}", "Building...".color_hex(RUNNING_COLOR).bold());
    println!("\n{}", "════ Output ══════════════════════".dimmed().bold());

    let out = vm.run_function(
        Context::new(),
        CallInfo {
            func: start,
            call_area: None,
            is_builtin: None,
        },
        Box::new(|_| {}),
    );

    for (_, v) in out {
        if let Err(e) = v {
            let g = e.to_report(&vm);
            g.fuck();
            Err(e.to_report(&vm))?;
        }
    }

    println!("\n{}", "══════════════════════════════════".dimmed().bold());

    Ok(SpwnOutput {
        objects: vm.objects,
        triggers: vm.triggers,
        id_counters: vm.id_counters,
    })
}

fn generate_doc(settings: &DocSettings, spinner: &mut Spinner) -> Result<(), SpwnError> {
    let root: PathBuf = if let Some(file) = &settings.file {
        file.into()
    } else if let Some(lib) = &settings.lib {
        let p: PathBuf = lib.into();

        if p.is_file() {
            Err(SpwnError::DocExpectedLibrary)?;
        }

        p.join("lib.spwn")
    } else {
        unreachable!("BUG: no file/lib specified")
    };

    spinner.start(format!(
        "{:20}",
        "Generating Documentation..."
            .color_hex(PARSING_COLOR)
            .bold()
    ));

    let mut compiler = DocCompiler::new(settings, root);
    compiler.compile()?;

    spinner.complete(None);

    Ok(())
}
