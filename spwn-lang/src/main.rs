mod ast;
mod compiler;
mod compiler_types;
mod levelstring;
mod native;
mod parser;

use parser::*;

use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    let script_path = PathBuf::from(&args[1]);
    let statements = parse_spwn(&script_path);
    // for statement in statements.iter() {
    //     println!("{:?}\n\n", statement);
    // }

    let compiled = compiler::compile_spwn(statements, script_path);
    let mut level_string = String::new();

    for trigger in compiled.obj_list {
        level_string += &levelstring::serialize_trigger(trigger);
    }
    println!("Using {} groups", compiled.closed_groups.len());

    println!("{:?}", level_string);
}
