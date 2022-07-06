use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::parser::lexer::Span;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpwnSource {
    File(PathBuf),
}

impl SpwnSource {
    pub fn name(&self) -> String {
        match self {
            Self::File(f) => f.display().to_string(),
        }
    }

    pub fn contents(&self) -> String {
        match self {
            Self::File(f) => fs::read_to_string(f).unwrap(), // existance of file should have been already checked beforehand
        }
    }

    pub fn to_area(&self, span: (usize, usize)) -> CodeArea {
        CodeArea {
            source: self.clone(),
            span,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CodeArea {
    pub(crate) source: SpwnSource,
    pub(crate) span: Span,
}

impl CodeArea {
    pub fn name(&self) -> String {
        self.source.name()
    }

    pub fn label(&self) -> (String, std::ops::Range<usize>) {
        (self.name(), self.span.0..self.span.1)
    }
}
