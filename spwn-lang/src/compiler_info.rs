use crate::parser::FileRange;

use std::{
    ops::Range,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerInfo {
    pub depth: u8,
    pub path: Vec<String>,
    pub current_module: String, // empty string means script
    pub position: CodeArea,
    pub includes: Vec<PathBuf>,
}

impl CompilerInfo {
    pub fn new() -> Self {
        CompilerInfo {
            depth: 0,
            path: vec!["main scope".to_string()],

            current_module: String::new(),
            position: CodeArea {
                file: PathBuf::new(),
                pos: (0, 0),
            },
            includes: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeArea {
    pub file: PathBuf,
    pub pos: FileRange,
}

use ariadne::Span;

impl Span for CodeArea {
    type SourceId = Path;

    fn source(&self) -> &Self::SourceId {
        self.file.as_path()
    }
    fn start(&self) -> usize {
        self.pos.0
    }
    fn end(&self) -> usize {
        self.pos.1
    }
}
