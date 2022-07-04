mod lexer;
mod parser;
mod error;
mod compiler;
mod value;


use std::{path::PathBuf, fs, collections::HashMap, io::{self, Write}};

use ariadne::Source;
use error::ToReport;
use logos::Logos;
use parser::{ParseData, parse, ASTData};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpwnSource {
    File(PathBuf),
}

impl SpwnSource {
    fn to_area(&self, span: (usize, usize)) -> CodeArea {
        CodeArea {
            source: self.clone(),
            span,
        }
    }
}



#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct CodeArea {
    source: SpwnSource,
    span: (usize, usize),
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
    files: HashMap<SpwnSource, Source>
}



impl ariadne::Cache<SpwnSource> for SpwnCache {
    fn fetch(&mut self, id: &SpwnSource) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        Ok(
            match self.files.entry(id.clone()) {
                std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
                std::collections::hash_map::Entry::Vacant(e) => e.insert(
                    Source::from(match id {
                        SpwnSource::File(path) => fs::read_to_string(path).map_err(|e| Box::new(e) as _)?,
                    })
                ),
            }
        )
    }

    fn display<'a>(&self, id: &'a SpwnSource) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match id {
            SpwnSource::File(f) => Some(Box::new(f.display())),
        }
    }
}


fn run(code: String, source: SpwnSource) {
    let code = code.trim_end().to_string();
    let mut tokens_iter = lexer::Token
        ::lexer(&code);
    let mut tokens = vec![];
    while let Some(t) = tokens_iter.next() {
        tokens.push((
            t,
            (
                tokens_iter.span().start,
                tokens_iter.span().end,
            )
        ))
    }
    tokens.push((
        lexer::Token::Eof,
        (code.len(), code.len())
    ));

    let cache = SpwnCache::default();
    // cache.fetch(&source).unwrap();

    let mut ast_data = ASTData::default();
    let parse_data = ParseData {
        source,
        tokens,
    };

    let ast = parse(&parse_data, &mut ast_data);

    match ast {
        Ok(stmts) => {

            ast_data.debug(&stmts)

            // match gen(stmts) {
            //     Ok(code) => {
            //         println!("{:?}", code.consts);
            //         for (pos, i) in code.instructions.iter().enumerate() {
            //             println!("{}: {:?}", pos, i);
            //         }
            //         println!("\n\n\n");
            //         // execute(&code);
            //     },
            //     Err(e) => {
            //         let gaga = e.to_report();
            //         gaga.print_error(cache);
            //     },
            // }
        },
        Err(e) => {
            let gaga = e.to_report();
            gaga.print_error(cache);
        },
    }

}


fn main() {
    print!("\x1B[2J\x1B[1;1H");

    io::stdout().flush().unwrap();
    let mut buf = PathBuf::new();
    buf.push("test.spwn");
    let code = fs::read_to_string(buf.clone()).unwrap();
    run(code, SpwnSource::File(buf));
}
