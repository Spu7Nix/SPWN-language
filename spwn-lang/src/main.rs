mod ast;
mod compiler;
mod compiler_types;
mod levelstring;
mod native;
mod parser;

use parser::*;

use std::env;
use std::path::PathBuf;

#[macro_use]
extern crate lazy_static;

fn main() {
    let args: Vec<String> = env::args().collect();
    let script_path = PathBuf::from(&args[1]); //&args[1]
    let (statements, notes) = parse_spwn(&script_path);
    // for statement in statements.iter() {
    //     println!("{:?}\n\n", statement);
    // }
    let gd_path = PathBuf::from(std::env::var("localappdata").expect("No local app data"))
        .join("GeometryDash/CCLocalLevels.dat");
    let (mut compiled, old_ls) =
        compiler::compile_spwn(statements, script_path, gd_path.clone(), notes);
    let mut level_string = String::new();

    for trigger in compiled.obj_list {
        level_string += &levelstring::serialize_trigger(trigger);
    }

    compiled.closed_groups.sort();
    compiled.closed_groups.dedup();

    println!("Using {} groups", compiled.closed_groups.len());
    levelstring::encrypt_level_string(level_string, old_ls, gd_path);
    println!("Written to save. You can now open Geometry Dash again!");
}
