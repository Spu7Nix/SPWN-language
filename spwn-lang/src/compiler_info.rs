use crate::parser::FileRange;

use std::path::{Path, PathBuf};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerInfo {
    pub depth: u8,
    pub call_stack: Vec<CodeArea>,
    pub current_module: String, // empty string means script
    pub position: CodeArea,
    pub includes: Vec<PathBuf>,
}

impl CompilerInfo {
    pub fn new() -> Self {
        CompilerInfo {
            depth: 0,
            call_stack: Vec::new(),

            current_module: String::new(),
            position: CodeArea::new(),
            includes: vec![],
        }
    }

    pub fn from_area(a: CodeArea) -> Self {
        CompilerInfo {
            position: a,
            ..Self::new()
        }
    }

    pub fn with_area(self, a: CodeArea) -> Self {
        CompilerInfo {
            position: a,
            ..self
        }
    }

    pub fn add_to_call_stack(&mut self, new: CodeArea) {
        self.call_stack.push(self.position.clone());
        self.position = new;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeArea {
    pub file: PathBuf,
    pub pos: FileRange,
}

impl CodeArea {
    pub fn new() -> Self {
        CodeArea {
            file: PathBuf::new(),
            pos: (0, 0),
        }
    }
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
