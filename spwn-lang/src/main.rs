mod ast;
//mod compiler;
//mod compiler_types;
//mod levelstring;
//mod native;
mod parser;

use parser::*;

use std::env;
use std::path::PathBuf;

//#[macro_use]
extern crate lazy_static;

fn main() {
    let args: Vec<String> = env::args().collect();
    let script_path = PathBuf::from(&args[1]); //&args[1]
    println!("Starting...");
    //let (statements, notes) = parse_spwn(&script_path);

    let parsed = parse_spwn(&script_path);
    println!("parsed: {:?}", parsed);
    // for statement in statements.iter() {
    //     println!("{:?}\n\n", statement);
    // }
    /*let gd_path = if cfg!(target_os = "windows") {
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
    };

    let (mut compiled, old_ls) =
        compiler::compile_spwn(statements, script_path, gd_path.clone(), notes);
    let level_string = levelstring::serialize_triggers(compiled.func_ids);

    compiled.closed_groups.sort();
    compiled.closed_groups.dedup();

    println!("Using {} groups", compiled.closed_groups.len());
    levelstring::encrypt_level_string(level_string, old_ls, gd_path);
    println!("Written to save. You can now open Geometry Dash again!");*/
}
