use clap::{Args, Parser, Subcommand};

use crate::parsing::attributes::ScriptAttribute;

// cli will come later

#[derive(Parser, Debug)]
#[command(author = "Spu7Nix", version = env!("CARGO_PKG_VERSION"), about)]
pub struct Arguments {
    #[clap(flatten)]
    pub settings: FileSettings,

    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
#[command(rename_all = "lowercase")]
pub enum Commands {
    Build { file: String },
}

#[derive(Args, Debug, Default)]
pub struct FileSettings {
    #[arg(short, long)]
    pub cache_output: bool,

    #[arg(short, long)]
    pub no_std: bool,

    #[arg(short, long)]
    pub console_output: bool,

    #[arg(short, long)]
    pub no_level: bool,

    #[arg(short, long)]
    pub no_bytecode_cache: bool,

    #[arg(short, long)]
    pub debug_bytecode: bool,

    #[arg(short, long)]
    pub no_optimize: bool,

    #[arg(short, long)]
    pub no_optimize_bytecode: bool,
}

impl FileSettings {
    pub fn apply_attributes(&mut self, attrs: &Vec<ScriptAttribute>) {
        for attr in attrs {
            match attr {
                ScriptAttribute::CacheOutput => self.cache_output = true,
                ScriptAttribute::NoStd => self.no_std = true,
                ScriptAttribute::ConsoleOutput => self.console_output = true,
                ScriptAttribute::NoLevel => self.no_level = true,
                ScriptAttribute::NoBytecodeCache => self.no_bytecode_cache = true,
                ScriptAttribute::DebugBytecode => self.debug_bytecode = true,
                ScriptAttribute::NoOptimizeTriggers => self.no_optimize = true,
                ScriptAttribute::NoOptimizeBytecode => self.no_optimize_bytecode = true,
            }
        }
    }
}
