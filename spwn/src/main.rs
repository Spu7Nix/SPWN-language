use clap::arg;
use clap::ValueHint;
//#![feature(arbitrary_enum_discriminant)]
use ::compiler::builtins;
use ::compiler::compiler;
use std::io::Read;

use ::docgen::documentation;

use ::compiler::leveldata;

use optimizer::optimize;

use ariadne::Cache;

use optimize::optimize;

use ::parser::parser::*;
use builtins::BuiltinPermissions;

use shared::SpwnSource;
use spwn::SpwnCache;

use std::path::PathBuf;

use editorlive::editorlive::editor_paste;
use std::fs;

use ::pckp::config_file;

const ERROR_EXIT_CODE: i32 = 1;

use clap::App;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use errors::{create_report, ErrorReport};

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
    let matches = App::new("SPWN").subcommands(
        [
            App::new("build")
                .about("Runs/builds a given file")
                .visible_alias("bu")
                .args(&[
                    arg!(<SCRIPT> "Path to spwn source file").value_hint(ValueHint::AnyPath),
                    arg!(-c --"console-output" "Makes the script print the created level into the console instead of writing it to your save file"),
                    arg!(-l --"no-level" "Only compiles the script, no level creation at all"),
                    arg!(-o --"no-optimize" "Removes post-optimization of triggers, making the output more readable, while also using a lot more objects and groups"),
                    arg!(-n --"level-name" [NAME] "Targets a specific level"),
                    arg!(-e --"live-editor" "Instead of writing the level to the save file, the script will use a live editor library if it's installed (Currently works only for MacOS)"),
                    arg!(-s --"save-file" [FILE] "Chooses a specific save file to write to"),
                    arg!(-i --"include-path" "Adds a search path to look for librariesAdds a search path to look for libraries").takes_value(true).multiple_occurrences(true).min_values(0),
                    arg!(-a --allow "Allow the use of a builtin").takes_value(true).multiple_occurrences(true).min_values(0),
                    arg!(-d --deny "Deny the use of a builtin").takes_value(true).multiple_occurrences(true).min_values(0),
                ]),

            App::new("doc")
            .arg(
                arg!(<LIBRARY> "Library to document")
            )
                .about("Generates documentation for a SPWN library, in the form of a markdown file"),
        ]
    ).get_matches();

    if let Some(build_cmd) = matches.subcommand_matches("build") {
        let script_path = build_cmd.value_of("SCRIPT").ok_or("unreachable")?;

        let mut permissions = BuiltinPermissions::new();
        let mut include_paths = vec![
            std::env::current_dir().expect("Cannot access current directory"),
            std::env::current_exe()?
                .parent()
                .expect("Executable must be in a directory")
                .to_path_buf(),
        ];

        let mut gd_enabled =
            !build_cmd.is_present("no-level") && !build_cmd.is_present("console-output");
        let opti_enabled = !build_cmd.is_present("no-optimize");
        let level_name = build_cmd.value_of("level-name").map(str::to_string);
        let live_editor = build_cmd.is_present("live-editor");
        let save_file = build_cmd.value_of("save-file");

        build_cmd
            .values_of("include-path")
            .unwrap_or_default()
            .for_each(|val| include_paths.push(val.into()));

        build_cmd
            .values_of("allow")
            .unwrap_or_default()
            .for_each(|val| {
                permissions.set(
                    val.parse()
                        .unwrap_or_else(|_| panic!("Invalid builtin name: {}", val)),
                    true,
                )
            });

        build_cmd
            .values_of("deny")
            .unwrap_or_default()
            .for_each(|val| {
                permissions.set(
                    val.parse()
                        .unwrap_or_else(|_| panic!("Invalid builtin name: {}", val)),
                    false,
                )
            });

        let mut cache = SpwnCache::default();
        match cache.fetch(&SpwnSource::File(script_path.into())) {
            Ok(_) => (),
            Err(_) => {
                return Err(Box::from("File does not exist".to_string()));
            }
        }

        print_with_color("Parsing ...", Color::Green);
        let unparsed = fs::read_to_string(script_path)?;

        let (statements, notes) = match parse_spwn(
            unparsed,
            SpwnSource::File(script_path.into()),
            ::compiler::builtins::BUILTIN_NAMES,
        ) {
            Err(err) => {
                create_report(ErrorReport::from(err)).eprint(cache).unwrap();
                std::process::exit(ERROR_EXIT_CODE);
            }
            Ok(p) => p,
        };

        let tags = notes.tag.tags.iter();
        for tag in tags {
            match tag.0.as_str() {
                "console_output" => gd_enabled = false,
                "no_level" => {
                    gd_enabled = false;
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

        let level_string = if gd_enabled {
            if let Some(gd_path) = &gd_path {
                print_with_color("Reading savefile...", Color::Cyan);
                let mut file = fs::File::open(gd_path)?;
                let mut file_content = Vec::new();
                file.read_to_end(&mut file_content)
                    .expect("Problem reading savefile");
                let mut level_string =
                    match levelstring::get_level_string(file_content, level_name.as_ref()) {
                        Ok(s) => s,
                        Err(e) => {
                            eprint_with_color(&format!("Error reading level:\n{}", e), Color::Red);

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

        let pckp_path = PathBuf::from(script_path).parent().unwrap().to_path_buf();
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
            SpwnSource::File(script_path.into()),
            include_paths,
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

        if gd_enabled {
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
                                    reserved.trigger_groups.extend(g.iter().map(|g| g.id));
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
                compiled.func_ids = optimize(compiled.func_ids, compiled.closed_groups, reserved);
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
    } else if let Some(doc_cmd) = matches.subcommand_matches("doc") {
        let lib_path = doc_cmd.value_of("LIBRARY").unwrap();
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
        };

        //println!("doc {:?}", documentation);

        Ok(())
    } else {
        unreachable!()
    }
}
