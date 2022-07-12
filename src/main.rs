mod compiler;
// mod docgen;
mod error;
mod interpreter;
mod leveldata;
mod parser;
mod sources;

use std::io::{self, Write};
use std::{fs, path::PathBuf};

#[macro_use]
extern crate mopa;

use ariadne::{Color, Fmt};
use compiler::compiler::{Compiler, Scope};

use compiler::error::CompilerError;
use error::RaiseError;
use interpreter::error::RuntimeError;
use interpreter::interpreter::{execute_code, Globals};
use parser::ast::ASTData;
use parser::error::SyntaxError;
use parser::parser::Parser;
use sources::SpwnSource;

// use docgen::docgen::parse_doc_comments;

fn run_spwn(code: String, source: SpwnSource, doctest: bool) {
    // if doctest {
    //     parse_doc_comments(code.clone());
    //     return;
    // }

    macro_rules! handle {
        ($a:expr) => {
            match $a {
                Ok(a) => a,
                Err(e) => {
                    e.raise(&code, source);
                    return;
                }
            }
        };
    }

    let (ast_data, stmts) = handle!(parse_stage(code.clone(), &source));

    // compile to bytocode ir
    let compiler = handle!(bytecode_generation(ast_data, source.clone(), stmts));

    println!("\n\n\n");
    // interpret/compile to triggers
    let globals = handle!(interpret_stage(compiler));
    // optimize here
    post_stage(globals);
}

fn post_stage(globals: Globals) {
    // postprocess/add objects to level
    let mut all_objects = globals.objects;

    all_objects.extend(globals.triggers);
    match leveldata::postprocess::append_objects(all_objects, "") {
        Ok((new_ls, used_ids)) => {
            println!("\n{}", "Level:".fg(Color::Magenta));
            for (i, len) in used_ids.iter().enumerate() {
                if *len > 0 {
                    println!(
                        "{} {}",
                        len,
                        ["groups", "colors", "block IDs", "item IDs"][i].fg(Color::White)
                    );
                }
            }
            println!("Level string:\n{}", new_ls);
        }
        Err(e) => eprintln!("{}", e.fg(Color::Red)),
    };
}

fn interpret_stage(compiler: Compiler) -> Result<Globals, RuntimeError> {
    let mut globals = Globals::new();
    globals.init();
    execute_code(&mut globals, &compiler.code)?;
    Ok(globals)
}

fn bytecode_generation(
    ast_data: ASTData,
    source: Option<PathBuf>,
    stmts: Vec<parser::ast::StmtKey>,
) -> Result<Compiler, CompilerError> {
    let mut compiler = Compiler::new(ast_data, source.clone());
    compiler.start_compile(stmts)?;
    compiler.code.debug();
    Ok(compiler)
}

fn parse_stage(
    code: String,
    source: &Option<PathBuf>,
) -> Result<(ASTData, Vec<parser::ast::StmtKey>), SyntaxError> {
    let mut parser = Parser::new(&code, source.clone());
    let mut ast_data = ASTData::default();
    let stmts = parser.parse(&mut ast_data)?;
    ast_data.debug(&stmts);
    Ok((ast_data, stmts))
}

fn main() {
    print!("\x1B[2J\x1B[1;1H");

    io::stdout().flush().unwrap();

    let file = std::env::args().nth(1).expect("no filename given");
    let doctest: bool = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "false".to_string())
        .parse()
        .expect("expected bool for doctest");
    let buf = PathBuf::from(file);

    let code = fs::read_to_string(&buf).unwrap();
    run_spwn(code, Some(buf), doctest);
    // println!("{}", std::mem::size_of::<Instruction>());
}
