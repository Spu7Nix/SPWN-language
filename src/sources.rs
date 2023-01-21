use ahash::AHashMap;

use crate::{compiling::bytecode::Bytecode, util::hyperlink};
use std::{fs, ops::Range, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpwnSource {
    File(PathBuf),
}

impl SpwnSource {
    pub fn area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            src: self.clone(),
            span,
        }
    }
    pub fn name(&self) -> String {
        match self {
            SpwnSource::File(f) => f.display().to_string(),
        }
    }
    pub fn read(&self) -> Option<String> {
        match self {
            SpwnSource::File(p) => fs::read_to_string(p).ok(),
        }
    }
    pub fn path(&self) -> String {
        match self {
            SpwnSource::File(f) => fs::canonicalize(f).unwrap().to_str().unwrap().into(),
        }
    }
    pub fn hyperlink(&self) -> String {
        match self {
            SpwnSource::File(_) => hyperlink(self.path(), Some(self.name())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeArea {
    pub src: SpwnSource,
    pub span: CodeSpan,
}
impl CodeArea {
    pub fn name(&self) -> String {
        self.src.name()
    }

    pub fn label(&self) -> (String, Range<usize>) {
        (self.name(), self.span.into())
    }

    pub(crate) fn internal() -> CodeArea {
        CodeArea {
            src: SpwnSource::File(PathBuf::from("<internal>")),
            span: CodeSpan::internal(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
pub struct CodeSpan {
    pub start: usize,
    pub end: usize,
}

impl CodeSpan {
    pub fn extend(&self, other: CodeSpan) -> CodeSpan {
        CodeSpan {
            start: self.start,
            end: other.end,
        }
    }

    pub fn internal() -> CodeSpan {
        CodeSpan { start: 0, end: 0 }
    }
}

impl From<Range<usize>> for CodeSpan {
    fn from(r: Range<usize>) -> Self {
        CodeSpan {
            start: r.start,
            end: r.end,
        }
    }
}
impl From<CodeSpan> for Range<usize> {
    fn from(s: CodeSpan) -> Self {
        s.start..s.end
    }
}

#[derive(Default)]
pub struct BytecodeMap {
    pub map: AHashMap<SpwnSource, Bytecode>,
}
