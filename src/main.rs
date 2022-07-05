mod compiler;
mod contexts;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod sources;
mod value;

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use ariadne::Cache;

use compiler::Compiler;

use lexer::lex;
use parser::{parse, ASTData, ParseData};
use sources::SpwnSource;

use crate::compiler::Instruction;

fn run(code: String, source: SpwnSource) {
    let tokens = lex(code);

    let mut ast_data = ASTData::default();
    let parse_data = ParseData { source: source.clone(), tokens };

    let ast = parse(&parse_data, &mut ast_data);

    match ast {
        Ok(stmts) => {
            ast_data.debug(&stmts);

            let mut compiler = Compiler::new(ast_data);
            compiler.code.instructions.push(vec![]);

            compiler.compile_stmts(stmts, 0);

            compiler.code.debug();
        }
        Err(e) => {
            e.raise(source);
        }
    }
}

fn main() {
    print!("\x1B[2J\x1B[1;1H");

    io::stdout().flush().unwrap();
    let mut buf = PathBuf::new();
    buf.push("test.spwn");
    let code = fs::read_to_string(buf.clone()).unwrap();
    run(code, SpwnSource::File(buf));

    println!("{}", std::mem::size_of::<Instruction>());
}
