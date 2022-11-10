pub mod util;
pub mod add;
pub mod remove;
pub mod restore;
pub mod package;
pub mod new;

use std::path::PathBuf;
use clap::{ArgMatches, Command, arg, value_parser, ValueHint};

pub async fn run(args: &ArgMatches) {
    match args.subcommand().unwrap() {
        ("add", cmd) => {
            add::add(
                cmd.get_many::<String>("LIBRARIES").unwrap().collect()
            ).await;
        },
        ("remove", cmd) => {
            remove::remove(
                cmd.get_many::<String>("LIBRARIES").unwrap().collect()
            ).await;
        },
        ("restore", _cmd) => {
            restore::restore().await;
        },
        ("new", cmd) => {
            new::new(
                cmd.get_one::<PathBuf>("FOLDER").unwrap().to_path_buf()
            );
        },
        (_,_) => unreachable!(),
    }
}

#[inline]
pub fn pckp_subcommand() -> Command<'static> {
    Command::new("pckp")
    .visible_alias("p")
    .about("Libraries manager for spwn")
    .subcommands([
        Command::new("add")
            .visible_alias("a")
            .about("Adds a library to the pckp file")
            .arg(
                arg!(<LIBRARIES> ... "Libraries to add")
                .value_parser(value_parser!(String))
            ),
        Command::new("remove")
            .visible_alias("r")
            .about("Removes a library from the pckp file")
            .arg(
                arg!(<LIBRARIES> ... "Libraries to remove")
                .value_parser(value_parser!(String))
            ),
        Command::new("restore")
            .about("Makes sure every dependency is in order"),
        Command::new("new")
            .visible_alias("n")
            .about("Creates a new SPWN project")
            .arg(
                arg!(<FOLDER> "Target folder")
                .value_parser(value_parser!(PathBuf))
                .value_hint(ValueHint::DirPath)
            ),
    ])
    .arg_required_else_help(true)
}