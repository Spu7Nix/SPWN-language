use std::{ops::Range, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpwnSource {
    File(PathBuf),
}

impl SpwnSource {
    pub fn area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            source: self.clone(),
            span,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CodeArea {
    pub source: SpwnSource,
    pub span: CodeSpan,
}

pub fn source_name(source: &SpwnSource) -> String {
    match source {
        SpwnSource::File(f) => f.display().to_string(),
    }
}

impl CodeArea {
    pub fn name(&self) -> String {
        source_name(&self.source)
    }

    pub fn label(&self) -> (String, Range<usize>) {
        (self.name(), self.span.into())
    }

    pub(crate) fn unknown() -> CodeArea {
        // fix this lmao idk what im doiung
        CodeArea {
            source: SpwnSource::File(PathBuf::from("<unknown>")),
            span: CodeSpan { start: 0, end: 0 },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy, Default)]
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
