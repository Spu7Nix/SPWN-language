pub mod util;
pub mod add;
pub mod remove;
pub mod restore;

pub fn run(args: &mut std::env::Args) {
    let args = args.collect::<Vec<String>>();

    match args[2].as_str() {
        "add" => { add::add(args[3..].to_vec()) },
        "remove" => { remove::remove(args[3..].to_vec()) },
        "restore" => { restore::restore() }
        _ => { panic!("add help command lmao 2"); }
    }

    return;
}