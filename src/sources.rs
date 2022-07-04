use std::collections::hash_map::Entry;
use std::fs;
use std::path::PathBuf;

use ahash::AHashMap;
use ariadne::{Cache, Source};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpwnSource {
    File(PathBuf),
}

impl SpwnSource {
    pub fn to_area(&self, span: (usize, usize)) -> CodeArea {
        CodeArea {
            source: self.clone(),
            span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeArea {
    pub(crate) source: SpwnSource,
    pub(crate) span: (usize, usize),
}

impl ariadne::Span for CodeArea {
    type SourceId = SpwnSource;

    fn source(&self) -> &Self::SourceId {
        &self.source
    }

    fn start(&self) -> usize {
        self.span.0
    }

    fn end(&self) -> usize {
        self.span.1
    }
}

#[derive(Default)]
pub struct SpwnCache {
    files: AHashMap<SpwnSource, Source>,
}

impl Cache<SpwnSource> for SpwnCache {
    fn fetch(&mut self, id: &SpwnSource) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        Ok(match self.files.entry(id.clone()) {
            Entry::Occupied(e) => e.into_mut(),

            Entry::Vacant(e) => e.insert(Source::from(match id {
                SpwnSource::File(path) => fs::read_to_string(path).map_err(|e| Box::new(e) as _)?,
            })),
        })
    }

    fn display<'a>(&self, id: &'a SpwnSource) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match id {
            SpwnSource::File(f) => Some(Box::new(f.display())),
        }
    }
}
