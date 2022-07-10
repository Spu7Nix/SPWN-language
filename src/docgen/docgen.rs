use std::path::PathBuf;

use super::ast::DocData;
use super::parser::Parser;

#[derive(Clone, Default)]
pub struct Source {
    // location of var/type/etc in source code
    pub source: PathBuf,
    // link to the local docs file that explains the var/type/etc
    local: PathBuf,
}

// impl Source {

//     fn base()
// }

fn parse_doc_comments(code: String) {
    let mut parser = Parser::new(&code, Source::default());

    let mut data = DocData::default();

    let stmts = parser.parse(&mut data);
}

fn main() {}
