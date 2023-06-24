use std::fmt::{Debug, Display};
use std::fs;
use std::ops::Range;
use std::path::PathBuf;
use std::rc::Rc;

use ahash::AHashMap;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::compiling::bytecode::Bytecode;
use crate::new_id_wrapper;
use crate::parsing::ast::ImportSettings;
use crate::util::{hyperlink, SlabMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpwnSource {
    File(PathBuf),
    Core(PathBuf),
    Std(PathBuf),
}

impl SpwnSource {
    pub fn area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            src: Rc::new(self.clone()),
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
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CodeArea {
    pub src: Rc<SpwnSource>,
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

#[allow(renamed_and_removed_lints, unknown_lints)]
#[derive(
    Clone, PartialEq, Eq, Copy, Default, Serialize, Deserialize, Hash, derive_more::Display,
)]
#[display(fmt = "{}..{}", start, end)]
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

pub trait Spannable {
    fn spanned(self, span: CodeSpan) -> Spanned<Self>
    where
        Self: Sized;
}

impl<T> Spannable for T {
    fn spanned(self, span: CodeSpan) -> Spanned<Self>
    where
        Self: Sized,
    {
        Spanned { value: self, span }
    }
}

#[derive(Clone, Hash, Copy, Serialize, Deserialize, PartialEq, Eq, Deref, DerefMut)]
pub struct Spanned<T> {
    #[deref]
    #[deref_mut]
    pub value: T,
    pub span: CodeSpan,
}

impl<T> Spanned<T> {
    pub fn split(self) -> (T, CodeSpan) {
        (self.value, self.span)
    }

    pub fn extended(self, other: CodeSpan) -> Self {
        Self {
            span: self.span.extend(other),
            ..self
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        f(self.value).spanned(self.span)
    }
}

impl<T: Debug> Debug for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}.spanned({:?})", self.value, self.span)
    }
}
impl Debug for CodeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
impl Debug for CodeArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{:?}", self.src, self.span)
    }
}

#[derive(Default, Deref, DerefMut)]
pub struct BytecodeMap(AHashMap<SpwnSource, Bytecode>);
