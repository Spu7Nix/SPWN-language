use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author = "Spu7Nix", version = env!("CARGO_PKG_VERSION"), about)]
#[command(next_line_help = true)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,

    #[arg(short = 'r', long)]
    pub no_color: bool,

    #[arg(short = 's', long)]
    pub no_spinner: bool,

    #[arg(short = 'a', long)]
    pub use_ascii_errors: bool,
}

#[derive(Subcommand, Debug)]
#[command(rename_all = "lowercase")]
pub enum Command {
    Build {
        file: PathBuf,

        #[clap(flatten)]
        settings: BuildSettings,
    },

    Doc {
        #[clap(flatten)]
        settings: DocSettings,
    },
}

#[derive(Args, Debug, Default)]
pub struct BuildSettings {
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

    #[arg(short = 'a', long)]
    pub no_optimize_bytecode: bool,

    #[arg(short = 'f', long)]
    pub save_file: Option<PathBuf>,

    #[cfg(target_os = "windows")]
    #[arg(short = 'e', long)]
    pub live_editor: bool,
}

#[derive(Args, Debug, Default)]
pub struct DocSettings {
    #[arg(short = 'm', long)]
    pub module: Option<String>,

    #[arg(short = 'l', long)]
    pub lib: Option<String>,

    #[arg(short = 't', long)]
    pub target_dir: Option<String>,
}
