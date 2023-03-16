use std::fs;
use std::ops::Range;
use std::path::PathBuf;

use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::compiling::bytecode::Bytecode;
use crate::parsing::ast::ModuleImport;
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

    pub fn change_path(&self, path: PathBuf) -> Self {
        match self {
            Self::File(_) => Self::File(path),
            Self::Std(_) => Self::Std(path),
            Self::Core(_) => Self::Core(path),
        }
    }

    pub fn change_path_conditional(&self, path: PathBuf, parent_typ: ModuleImport) -> Self {
        match self {
            SpwnSource::File(_) => match parent_typ {
                ModuleImport::Regular => SpwnSource::File(path),
                ModuleImport::Core => SpwnSource::Core(path),
                ModuleImport::Std => SpwnSource::Std(path),
            },

            _ => self.change_path(path),
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

#[allow(clippy::derived_hash_with_manual_eq)]
#[allow(renamed_and_removed_lints)]
#[allow(unknown_lints)]
#[allow(clippy::derive_hash_xor_eq)]
#[cfg_attr(not(test), derive(PartialEq))]
#[derive(Debug, Clone, Eq, Copy, Default, Serialize, Deserialize, Hash)]
pub struct CodeSpan {
    pub start: usize,
    pub end: usize,
}

#[cfg(test)]
impl PartialEq for CodeSpan {
    fn eq(&self, _: &Self) -> bool {
        true
    }
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
        CodeSpan { start: 1, end: 0 }
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

// rmdir -r -fo ~\.spwn\versions\0.9.0\libraries; cp .\libraries\ ~\.spwn\versions\0.9.0 -r
