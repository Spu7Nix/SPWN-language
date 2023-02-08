use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::parsing::attributes::ScriptAttribute;

// cli will come later

#[derive(Parser, Debug)]
#[command(author = "Spu7Nix", version = env!("CARGO_PKG_VERSION"), about)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,

    #[arg(short = 'r', long)]
    pub no_color: bool,
}

#[derive(Subcommand, Debug)]
#[command(rename_all = "lowercase")]
pub enum Command {
    Build {
        file: PathBuf,

        #[clap(flatten)]
        settings: Settings,
    },
}

#[derive(Args, Debug, Default)]
pub struct Settings {
    #[arg(short = 'l', long)]
    pub level_name: Option<String>,

    #[arg(short = 'c', long)]
    pub console_output: bool,

    #[arg(short = 'n', long)]
    pub no_level: bool,

    #[arg(short = 'b', long)]
    pub no_bytecode_cache: bool,

    #[arg(short = 'd', long)]
    pub debug_bytecode: bool,

    #[arg(short = 'o', long)]
    pub no_optimize: bool,

    #[arg(short = 'y', long)]
    pub no_optimize_bytecode: bool,

    #[arg(short = 'f', long)]
    pub save_file: Option<PathBuf>,
}
