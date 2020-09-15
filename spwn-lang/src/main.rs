#![feature(arbitrary_enum_discriminant)]

mod ast;
mod builtin;
mod compiler;
mod compiler_types;
mod documentation;
mod levelstring;
mod parser;

//mod optimize;

//use optimize::optimize;

use parser::*;

use std::env;
use std::path::PathBuf;

//#[macro_use]
extern crate lazy_static;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut args_iter = args.iter();
    args_iter.next();

    match &args_iter.next() {
        Some(a) => {
            match a as &str {
                "build" => {
                    let script_path = match args_iter.next() {
                        Some(a) => PathBuf::from(a),
                        None => return Err(std::boxed::Box::from("Expected script file argument")),
                    };

                    let mut gd_enabled = true;

                    for arg in args_iter {
                        if arg == "--no-gd" {
                            gd_enabled = false;
                        }
                    }

                    println!("Parsing...");
                    let unparsed = fs::read_to_string(script_path.clone())?;
                    let (statements, notes) = match parse_spwn(unparsed) {
                        Err(err) => {
                            eprintln!("{}", err);
                            std::process::exit(256);
                        }
                        Ok(p) => p,
                    };

                    //println!("parsed: {:?}", statements);
                    for statement in statements.iter() {
                        println!("{}\n", statement);
                    }

                    let gd_path = if gd_enabled {
                        Some(if cfg!(target_os = "windows") {
                            PathBuf::from(std::env::var("localappdata").expect("No local app data"))
                                .join("GeometryDash/CCLocalLevels.dat")
                        } else if cfg!(target_os = "macos") {
                            PathBuf::from(std::env::var("HOME").expect("No home directory"))
                                .join("Library/Application Support/GeometryDash/CCLocalLevels.dat")
                        } else if cfg!(target_os = "linux") {
                            PathBuf::from(std::env::var("HOME").expect("No home directory"))
                                .join(".steam/steam/steamapps/compatdata/322170/pfx/drive_c/users/steamuser/Local Settings/Application Data/GeometryDash/CCLocalLevels.dat")
                        } else {
                            panic!("Unsupported operating system");
                        })
                    } else {
                        None
                    };

                    let (mut compiled, old_ls) = match compiler::compile_spwn(
                        statements,
                        script_path,
                        gd_path.clone(),
                        notes,
                    ) {
                        Err(err) => {
                            eprintln!("{}", err);
                            std::process::exit(256);
                        }
                        Ok(p) => p,
                    };

                    println!("values: {:?}", compiled.stored_values.map.iter().count());

                    //println!("func ids: {:?}", compiled.func_ids);
                    let mut objects = levelstring::apply_fn_ids(compiled.func_ids);

                    println!("{} objects added", objects.len());

                    //objects = optimize(objects);

                    println!("optimized to {} objects", objects.len());

                    let level_string = levelstring::serialize_triggers(objects);

                    //let level_string = levelstring::serialize_triggers_old(compiled.func_ids);

                    //println!("{}", level_string);

                    compiled.closed_groups.sort();
                    compiled.closed_groups.dedup();

                    println!("Using {} groups", compiled.closed_groups.len());

                    //println!("level_string: {}", level_string);
                    match gd_path {
                        Some(gd_path) => {
                            levelstring::encrypt_level_string(level_string, old_ls, gd_path);
                            println!("Written to save. You can now open Geometry Dash again!");
                        }

                        None => println!("Output: {}", level_string),
                    };

                    Ok(())
                }

                "doc" => {
                    use std::fs::File;
                    use std::io::Write;
                    let lib_path = match args_iter.next() {
                        Some(a) => PathBuf::from(a),
                        None => return Err(std::boxed::Box::from("Expected script file argument")),
                    };

                    let documentation = documentation::document_lib(&lib_path)?;

                    let mut output_path = lib_path.clone();
                    output_path.pop();
                    output_path.push(PathBuf::from(format!(
                        "{}-docs.md",
                        lib_path.file_stem().unwrap().to_str().unwrap()
                    )));

                    let mut output_file = File::create(output_path)?;
                    output_file.write_all(documentation.as_bytes())?;
                    Ok(())
                }

                a => {
                    eprintln!("Unknown command: {}", a);
                    std::process::exit(256);
                }
            }
        }
        None => Ok(()),
    }
}
