use std::fs;
use std::ops::Range;
use std::path::PathBuf;

use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::compiling::bytecode::Bytecode;
use crate::util::hyperlink;
use crate::vm::opcodes::Register;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpwnSource {
    File(PathBuf),
    Core(PathBuf),
    Std(PathBuf),
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
            SpwnSource::Core(f) => format!("<core: {}>", f.display()),
            SpwnSource::Std(f) => format!("<std: {}>", f.display()),
        }
    }

    pub fn read(&self) -> Option<String> {
        fs::read_to_string(self.path())
            .ok()
            .map(|s| s.replace("\r\n", "\n").trim_end().to_string())
    }

    pub fn path_str(&self) -> String {
        fs::canonicalize(self.path())
            .unwrap()
            .to_str()
            .unwrap()
            .into()
    }

    pub fn hyperlink(&self) -> String {
        hyperlink(self.path_str(), Some(self.name()))
    }

    pub fn path(&self) -> &PathBuf {
        match self {
            Self::File(f) | Self::Core(f) | Self::Std(f) => f,
        }
    }

    pub fn map_path<F>(&self, cb: F) -> Self
    where
        F: FnOnce(&PathBuf) -> PathBuf,
    {
        match self {
            Self::File(f) => Self::File(cb(f)),
            Self::Std(f) => Self::Std(cb(f)),
            Self::Core(f) => Self::Core(cb(f)),
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

    // pub(crate) fn internal() -> CodeArea {
    //     CodeArea {
    //         src: SpwnSource::File(PathBuf::from("<internal>")),
    //         span: CodeSpan::internal(),
    //     }
    // }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default, Serialize, Deserialize)]
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

    pub fn invalid() -> CodeSpan {
        CodeSpan { start: 0, end: 1 }
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
    pub map: AHashMap<SpwnSource, Bytecode<Register>>,
}

// cp .\libraries\ ~\.spwn\versions\0.9.0 -r
