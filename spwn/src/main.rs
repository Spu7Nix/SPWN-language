//#![feature(arbitrary_enum_discriminant)]
use ::compiler::builtins;
use ::compiler::compiler;

use ::docgen::documentation;

use ::compiler::leveldata;

use optimizer::optimize;

use ariadne::Cache;

use optimize::optimize;

use ::parser::parser::*;
use builtins::BuiltinPermissions;

use shared::SpwnSource;
use spwn::SpwnCache;

use std::env;
use std::path::PathBuf;

use editorlive::editorlive::editor_paste;
use std::fs;

use ::pckp::config_file;

const ERROR_EXIT_CODE: i32 = 1;

use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use errors::{create_report, ErrorReport};

const HELP: &str = include_str!("../help.txt");

fn print_with_color(text: &str, color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(color)))
        .unwrap();
    writeln!(&mut stdout, "{}", text).unwrap();
    stdout.set_color(&ColorSpec::new()).unwrap();
}

fn eprint_with_color(text: &str, color: termcolor::Color) {
    let mut stdout = StandardStream::stderr(ColorChoice::Always);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(color)))
        .unwrap();
    writeln!(&mut stdout, "{}", text).unwrap();
    stdout.set_color(&ColorSpec::new()).unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut args_iter = args.iter();
    args_iter.next();

    match &args_iter.next() {
        Some(a) => {
            match a as &str {
                "help" => {
                    println!("{}", HELP);
                    Ok(())
                }
                "version" | "-v" | "--version" => {
                    println!("v{}", env!("CARGO_PKG_VERSION"));
                    Ok(())
                }
                "build" | "b" => {
                    let script_path = match args_iter.next() {
                        Some(a) => PathBuf::from(a),
                        None => return Err(std::boxed::Box::from("Expected script file argument")),
                    };

                    let mut gd_enabled = true;
                    let mut opti_enabled = true;
                    let mut compile_only = false;
                    let mut level_name = None;
                    let mut live_editor = false;

                    let mut save_file = None;
                    let mut included_paths = vec![
                        std::env::current_dir().expect("Cannot access current directory"),
                        std::env::current_exe()
                            .expect("Cannot access directory of executable")
                            .parent()
                            .expect("Executable must be in a directory")
                            .to_path_buf(),
                    ];

                    let mut permissions = BuiltinPermissions::new();
                    //change to current_exe before release (from current_dir)
                    use std::str::FromStr;

                    while let Some(arg) = args_iter.next() {
                        match arg.as_ref() {
                            "--console-output" | "-c" => gd_enabled = false,
                            "--no-level" | "-l" => {
                                gd_enabled = false;
                                compile_only = true;
                            }
                            "--no-optimize" | "-o" => opti_enabled = false,
                            "--level-name" | "-n" => level_name = args_iter.next().cloned(),
                            "--live-editor" | "-e" => live_editor = true,
                            "--save-file" | "-s" => save_file = args_iter.next().cloned(),
                            "--included-path" | "-i" => included_paths.push({
                                let path = PathBuf::from(
                                    args_iter.next().cloned().expect("No path provided"),
                                );
                                if path.exists() {
                                    path
                                } else {
                                    return Err(Box::from("Invalid path".to_string()));
                                }
                            }),
                            "--allow" | "-a" => {
                                let b = args_iter
                                    .next()
                                    .cloned()
                                    .expect("Expected built-in function name");
                                permissions.set(
                                    builtins::Builtin::from_str(&b)
                                        .unwrap_or_else(|_| panic!("Invalid builtin name: {}", b)),
                                    true,
                                );
                            }
                            "--deny" | "-d" => {
                                let b = args_iter
                                    .next()
                                    .cloned()
                                    .expect("Expected built-in function name");
                                permissions.set(
                                    builtins::Builtin::from_str(&b)
                                        .unwrap_or_else(|_| panic!("Invalid builtin name: {}", b)),
                                    false,
                                );
                            }
                            _ => (),
                        };
                    }

                    let mut cache = SpwnCache::default();
                    match cache.fetch(&SpwnSource::File(script_path.clone())) {
                        Ok(_) => (),
                        Err(_) => {
                            return Err(Box::from("File does not exist".to_string()));
                        }
                    }

                    print_with_color("Parsing ...", Color::Green);
                    let unparsed = fs::read_to_string(script_path.clone())?;

                    let (statements, notes) = match parse_spwn(
                        unparsed,
                        SpwnSource::File(script_path.clone()),
                        ::compiler::builtins::BUILTIN_NAMES,
                    ) {
                        Err(err) => {
                            create_report(ErrorReport::from(err)).eprint(cache).unwrap();
                            std::process::exit(ERROR_EXIT_CODE);
                        }
                        Ok(p) => p,
                    };
                    //dbg!(&statements);

                    let tags = notes.tag.tags.iter();
                    for tag in tags {
                        match tag.0.as_str() {
                            "console_output" => gd_enabled = false,
                            "no_level" => {
                                gd_enabled = false;
                                compile_only = true;
                            }
                            _ => (),
                        }
                    }

                    let gd_path = if gd_enabled {
                        Some(if save_file != None {
                            PathBuf::from(save_file.expect("what"))
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
                            panic!("Unsupported operating system");
                        })
                    } else {
                        None
                    };

                    let level_string = if !compile_only {
                        if let Some(gd_path) = &gd_path {
                            print_with_color("Reading savefile...", Color::Cyan);
                            let mut file = fs::File::open(gd_path)?;
                            let mut file_content = Vec::new();
                            use std::io::Read;
                            file.read_to_end(&mut file_content)
                                .expect("Problem reading savefile");
                            let mut level_string = match levelstring::get_level_string(
                                file_content,
                                level_name.clone(),
                            ) {
                                Ok(s) => s,
                                Err(e) => {
                                    eprint_with_color(
                                        &format!("Error reading level:\n{}", e),
                                        Color::Red,
                                    );

                                    std::process::exit(ERROR_EXIT_CODE);
                                }
                            };
                            if level_string.is_empty() {}
                            leveldata::remove_spwn_objects(&mut level_string);
                            level_string
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };

                    let pckp_path = script_path.parent().unwrap().to_path_buf();
                    let cfg_file = config_file::get_config(Some(pckp_path.clone()));

                    let pckp_package = match config_file::config_to_package(cfg_file) {
                        Ok(p) => p,
                        Err(e) => {
                            eprint_with_color(
                                &format!("Error reading pckp file:\n{}", e.to_string()),
                                Color::Red,
                            );

                            std::process::exit(ERROR_EXIT_CODE);
                        }
                    };
                    if let Some(pack) = pckp_package {
                        match pack.install_dependencies(pckp_path) {
                            Ok(_) => (),
                            Err(e) => {
                                eprint_with_color(
                                    &format!("Error installing dependencies:\n{}", e.to_string()),
                                    Color::Red,
                                );

                                std::process::exit(ERROR_EXIT_CODE);
                            }
                        }
                    }

                    let mut std_out = std::io::stdout();
                    let mut compiled = match compiler::compile_spwn(
                        statements,
                        SpwnSource::File(script_path),
                        included_paths,
                        notes,
                        permissions,
                        level_string.clone(),
                        &mut std_out,
                    ) {
                        Err(err) => {
                            create_report(ErrorReport::from(err)).eprint(cache).unwrap();
                            std::process::exit(ERROR_EXIT_CODE);
                        }
                        Ok(p) => p,
                    };

                    //dbg!(&compiled.prev_imports);

                    if !compile_only {
                        let mut reserved = optimize::ReservedIds {
                            object_groups: Default::default(),
                            trigger_groups: Default::default(),
                            object_colors: Default::default(),

                            object_blocks: Default::default(),

                            object_items: Default::default(),
                        };
                        for obj in &compiled.objects {
                            for param in obj.params.values() {
                                match &param {
                                    leveldata::ObjParam::Group(g) => {
                                        reserved.object_groups.insert(g.id);
                                    }
                                    leveldata::ObjParam::GroupList(g) => {
                                        reserved.object_groups.extend(g.iter().map(|g| g.id));
                                    }

                                    leveldata::ObjParam::Color(g) => {
                                        reserved.object_colors.insert(g.id);
                                    }

                                    leveldata::ObjParam::Block(g) => {
                                        reserved.object_blocks.insert(g.id);
                                    }

                                    leveldata::ObjParam::Item(g) => {
                                        reserved.object_items.insert(g.id);
                                    }
                                    _ => (),
                                }
                            }
                        }

                        for fn_id in &compiled.func_ids {
                            for (trigger, _) in &fn_id.obj_list {
                                for (prop, param) in trigger.params.iter() {
                                    if *prop == 57 {
                                        match &param {
                                            leveldata::ObjParam::Group(g) => {
                                                reserved.trigger_groups.insert(g.id);
                                            }
                                            leveldata::ObjParam::GroupList(g) => {
                                                reserved
                                                    .trigger_groups
                                                    .extend(g.iter().map(|g| g.id));
                                            }

                                            _ => (),
                                        }
                                    }
                                }
                            }
                        }

                        let has_stuff = compiled.func_ids.iter().any(|x| !x.obj_list.is_empty());
                        if opti_enabled && has_stuff {
                            print_with_color("Optimizing triggers...", Color::Cyan);
                            compiled.func_ids =
                                optimize(compiled.func_ids, compiled.closed_groups, reserved);
                        }

                        let mut objects = leveldata::apply_fn_ids(&compiled.func_ids);

                        objects.extend(compiled.objects);

                        print_with_color(&format!("{} objects added", objects.len()), Color::White);

                        let (new_ls, used_ids) = leveldata::append_objects(objects, &level_string)?;

                        print_with_color("\nLevel:", Color::Magenta);
                        for (i, len) in used_ids.iter().enumerate() {
                            if *len > 0 {
                                print_with_color(
                                    &format!(
                                        "{} {}",
                                        len,
                                        ["groups", "colors", "block IDs", "item IDs"][i]
                                    ),
                                    Color::White,
                                );
                            }
                        }
                        //println!("level_string: {}", level_string);
                        if live_editor {
                            match editor_paste(&new_ls) {
                                Err(e) => {
                                    eprint_with_color(
                                        &format!("Error pasting into editor:\n{}", e),
                                        Color::Red,
                                    );

                                    std::process::exit(ERROR_EXIT_CODE);
                                }
                                Ok(_) => {
                                    print_with_color("Pasted into the editor!", Color::Green);
                                }
                            }
                        } else {
                            match gd_path {
                                Some(gd_path) => {
                                    print_with_color("\nWriting back to savefile...", Color::Cyan);
                                    levelstring::encrypt_level_string(
                                        new_ls,
                                        level_string,
                                        gd_path,
                                        level_name,
                                    )?;

                                    print_with_color(
                                        "Written to save. You can now open Geometry Dash again!",
                                        Color::Green,
                                    );
                                }

                                None => println!("Output: {}", new_ls),
                            };
                        }
                    };

                    let mut stdout = StandardStream::stdout(ColorChoice::Always);
                    stdout.set_color(&ColorSpec::new()).unwrap();

                    Ok(())
                }

                "doc" => {
                    //use std::fs::File;

                    let lib_path = match args_iter.next() {
                        Some(a) => a,
                        None => {
                            return Err(std::boxed::Box::from("Expected library name argument"))
                        }
                    };

                    if "$" == lib_path {
                        // doc builtins
                        let doc = builtins::builtin_docs();
                        fs::write("builtins.md", doc)?;
                        print_with_color("Written to ./builtins.md", Color::Green);
                    } else {
                        let cache = SpwnCache::default();

                        match documentation::document_lib(lib_path) {
                            Ok(_) => (),
                            Err(e) => {
                                create_report(ErrorReport::from(e)).eprint(cache).unwrap();
                                std::process::exit(ERROR_EXIT_CODE);
                            }
                        };
                    }

                    //println!("doc {:?}", documentation);

                    Ok(())
                }
                // "format" => {
                //     use std::fs::File;
                //     //use std::io::Write;
                //     let script_path = match args_iter.next() {
                //         Some(a) => PathBuf::from(a),
                //         None => return Err(std::boxed::Box::from("Expected script file argument")),
                //     };

                //     println!("Formatting is not good yet, i will finish it before the final version is released.");

                //     let unparsed = fs::read_to_string(script_path.clone())?;

                //     let (parsed, _) = match parse_spwn(unparsed, script_path) {
                //         Err(err) => {
                //             std::process::exit(ERROR_EXIT_CODE);
                //         }
                //         Ok(p) => p,
                //     };

                //     let formatted = fmt::_format(parsed);

                //     let mut output_file = File::create("test/formatted.spwn")?;
                //     output_file.write_all(formatted.as_bytes())?;

                //     Ok(())
                // }
                a => {
                    eprint_with_color(&format!("Unknown subcommand: {}", a), Color::Red);
                    println!("{}", HELP);
                    std::process::exit(ERROR_EXIT_CODE);
                }
            }
        }
        None => {
            println!("{}", HELP);
            Ok(())
        }
    }
}
