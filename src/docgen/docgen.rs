use std::process;

use super::parser::Parser;

fn parse_doc_comments(code: String) {
    let mut parser = Parser::new(&code);

    match parser.parse() {
        Ok(stmts) => {
            println!("{:#?}", stmts)
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            process::exit(1);
        }
    }
}

fn main() {}
