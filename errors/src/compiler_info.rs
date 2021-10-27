use internment::LocalIntern;
use shared::FileRange;
use std::path::PathBuf;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerInfo {
    pub depth: u8,
    pub call_stack: Vec<CodeArea>,
    pub current_module: String, // empty string means script
    pub position: CodeArea,
}

impl CompilerInfo {
    pub fn new() -> Self {
        CompilerInfo {
            depth: 0,
            call_stack: Vec::new(),

            current_module: String::new(),
            position: CodeArea::new(),
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
        self.call_stack.push(self.position);
        self.position = new;
    }
}

impl Default for CompilerInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeArea {
    pub file: LocalIntern<shared::SpwnSource>,
    pub pos: FileRange,
}

impl CodeArea {
    pub fn new() -> Self {
        CodeArea {
            file: LocalIntern::new(shared::SpwnSource::File(PathBuf::new())),
            pos: (0, 0),
        }
    }
}

impl Default for CodeArea {
    fn default() -> Self {
        Self::new()
    }
}
use ariadne::Span;

impl Span for CodeArea {
    type SourceId = shared::SpwnSource;

    fn source(&self) -> &Self::SourceId {
        &self.file
    }
    fn start(&self) -> usize {
        self.pos.0
    }
    fn end(&self) -> usize {
        self.pos.1
    }
}
