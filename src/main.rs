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
use std::io::{Read, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender};
use std::thread::JoinHandle;
use std::{fs, thread};

use clap::Parser as _;
use cli::Settings;
use colored::Colorize;
use gd::gd_object::GdObject;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lasso::Rodeo;
use slotmap::SlotMap;

use crate::cli::{Arguments, Command};
use crate::compiling::compiler::Compiler;
use crate::gd::{gd_object, levelstring};
use crate::lexing::tokens::Token;
use crate::parsing::parser::Parser;
use crate::sources::{BytecodeMap, SpwnSource};
use crate::util::RandomState;
use crate::vm::context::FullContext;
use crate::vm::interpreter::{FuncCoord, Vm};
use crate::vm::opcodes::{Opcode, Register};

fn main() -> Result<(), Box<dyn Error>> {
    assert_eq!(4, std::mem::size_of::<Opcode<Register>>());

    let args = Arguments::parse();

    let (sender, receiver) = channel();

    thread::spawn(move || {
        let spinner_style = ProgressStyle::with_template("{spinner} {msg}")
            .unwrap()
            .tick_chars("⢁⡈⠔⠢ ");

        let mut pb = None;

        // let handle;

        loop {
            // let gaga = receiver.try_recv();
            // println!("{:?}", gaga);

            match receiver.try_recv() {
                Ok(e) => match e {
                    SpinnerEvent::Start(msg) => {
                        let p = ProgressBar::new(100).with_style(spinner_style.clone());
                        p.set_message(msg);

                        pb = Some(p);
                    }
                    SpinnerEvent::Stop(msg) => {
                        if let Some(pb) = &pb {
                            if let Some(s) = msg {
                                pb.finish_with_message(s)
                            } else {
                                pb.finish()
                            }
                        }
                        pb = None;
                    }
                    SpinnerEvent::Finish => break,
                },
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Empty => {
                        if let Some(pb) = &pb {
                            pb.inc(1);
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                    std::sync::mpsc::TryRecvError::Disconnected => break,
                },
            }
            // match receiver.recv().unwrap() {
            //     SpinnerEvent::Finish => return,
            //     SpinnerEvent::Start(msg) => {
            //         pb.set_message(msg);
            //         // loop {
            //         //     if let Ok(SpinnerEvent::Stop) = receiver.recv() {
            //         //         break;
            //         //     }

            //         //     pb.inc(1)
            //         // }
            //     }
            //     SpinnerEvent::Stop => {}
            // }
        }
    });

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
                    sender
                        .send(SpinnerEvent::Start(format!(
                            "📖 {}",
                            "Reading savefile...".bright_cyan().bold()
                        )))
                        .unwrap();

                    let mut file = fs::File::open(gd_path).unwrap();
                    let mut file_content = Vec::new();

                    match file.read_to_end(&mut file_content) {
                        Ok(..) => (),
                        Err(e) => {
                            eprintln!(
                                "❌  {} {}",
                                "Error reading savefile:".bright_red().bold(),
                                e
                            );
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(3000));

                    sender.send(SpinnerEvent::Stop(None)).unwrap();

                    let mut level_string = match levelstring::get_level_string(
                        file_content,
                        settings.level_name.as_ref(),
                    ) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("❌  {} {}", "Error reading level:".bright_red().bold(), e);

                            std::process::exit(1);
                        }
                    };
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
            } = run_spwn(&settings);

            println!("\n{} objects added", objects.len());

            let (new_ls, used_ids) = gd_object::append_objects(objects, &level_string)?;

            println!("\n{}", "Level uses:".bright_magenta().bold());

            for (i, len) in used_ids.iter().enumerate() {
                // if *len > 0 {
                println!(
                    "{}",
                    &format!(
                        "{} {}",
                        len,
                        ["groups", "channels", "block IDs", "item IDs"][i]
                    ),
                );
                // }
            }

            match gd_path {
                Some(gd_path) => {
                    println!(
                        "\n📝  {}",
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

                None => println!("Output: {new_ls}",),
            };
        }
    };

    sender.send(SpinnerEvent::Finish).unwrap();

    Ok(())
}

pub struct SpwnOutput {
    pub objects: Vec<GdObject>,
    pub triggers: Vec<GdObject>,
    pub id_counters: [usize; 4],
}

pub fn run_spwn(settings: &Settings) -> SpwnOutput {
    let interner = Rc::new(RefCell::new(Rodeo::with_hasher(RandomState::new())));

    let path = PathBuf::from("test.spwn");

    let src = SpwnSource::File(path);
    let code = src.read().unwrap();

    let mut parser = Parser::new(&code, src, Rc::clone(&interner));

    let mut map = BytecodeMap::default();

    println!("🤖  {}", "Parsing...".bright_blue().bold());

    match parser.parse() {
        Ok(ast) => {
            let mut compiler =
                Compiler::new(Rc::clone(&interner), parser.src.clone(), settings, &mut map);

            println!("🛠️  {}", "Compiling...".bright_green().bold());

            match compiler.compile(ast.statements) {
                Ok(_) => (),
                Err(err) => {
                    err.to_report().display();
                    std::process::exit(1);
                }
            }

            if settings.debug_bytecode {
                for (k, b) in &map.map {
                    b.debug_str(k)
                }
            }

            let mut vm = Vm::new(&map, interner);

            let key = vm.src_map[&parser.src];
            let start = FuncCoord::new(0, key);

            vm.push_call_stack(start, 0, false, None);

            match vm.run_program() {
                Ok(_) => SpwnOutput {
                    objects: vm.objects,
                    triggers: vm.triggers,
                    id_counters: vm.id_counters,
                },
                Err(err) => {
                    err.to_report().display();
                    std::process::exit(1);
                }
            }
        }
        Err(err) => {
            err.to_report().display();
            std::process::exit(1);
        }
    }
}

// fn run_async<F: Send + Sync, T: Send + Sync + 'static>(mut f: F) -> JoinHandle<T>
// where
//     F: FnMut() -> T + 'static,
// {
//     thread::spawn(move || f())
// }
#[derive(Debug)]
enum SpinnerEvent {
    Start(String),
    Stop(Option<String>),
    Finish,
}

// fn run_spinner() -> Sender<SpinnerEvent> {
//     let (sender, receiver) = channel();

//     thread::spawn(move || {
//         if receiver.recv().is_ok() {
//             return;
//         }
//     });

//     sender
// }
