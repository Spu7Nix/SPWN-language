use std::path::PathBuf;

use super::parser::Parser;
use super::ast::DocData;

pub struct Source {
    // location of var/type/etc in source code
    source: PathBuf,
    // link to the local docs file that explains the var/type/etc
    local: PathBuf,
}

fn parse_doc_comments(code: String) {
    let mut parser = Parser::new(&code);

    let mut data = DocData::default();

    let stmts = parser.parse(&mut data);
}

fn main() {}
