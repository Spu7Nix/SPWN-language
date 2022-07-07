use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::parser::lexer::{CodeSpan, Span};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CodeArea {
    span: CodeSpan,
    source: Option<PathBuf>,
}

impl CodeArea {
    pub fn name(&self) -> String {
        match self.source {
            Some(s) => s.display().to_string(),
            None => "idfk",
        }
    }

    pub fn label(&self) -> (String, std::ops::Range<usize>) {
        (self.name(), self.span.0..self.span.1)
    }

    // pub fn stretch(&self, other: &CodeArea) -> CodeArea {
    //     CodeArea {
    //         source: self.source.clone(),
    //         span: (self.span.0, other.span.1),
    //     }
    // }
}
