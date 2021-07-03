use crate::parser::FileRange;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerInfo {
    pub depth: u8,
    pub path: Vec<String>,
    pub current_file: PathBuf,
    pub current_module: String, // empty string means script
    pub pos: FileRange,
    pub includes: Vec<PathBuf>,
}

impl CompilerInfo {
    pub fn new() -> Self {
        CompilerInfo {
            depth: 0,
            path: vec!["main scope".to_string()],
            current_file: PathBuf::new(),
            current_module: String::new(),
            pos: ((0, 0), (0, 0)),
            includes: vec![],
        }
    }
}
