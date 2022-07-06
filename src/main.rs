mod compiler;
mod error;
mod interpreter;
mod parser;
mod sources;

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use ahash::AHashMap;
use compiler::compiler::{Compiler, Scope};
use interpreter::contexts::{Context, FullContext};
use interpreter::interpreter::{execute, Globals};
use parser::lexer::lex;
use parser::parser::{parse, ASTData, ParseData};
use slotmap::SlotMap;
use sources::SpwnSource;

fn run(code: String, source: SpwnSource) {
    let tokens = lex(code);

    let mut ast_data = ASTData::default();
    let parse_data = ParseData {
        source: source.clone(),
        tokens,
    };

    let ast = parse(&parse_data, &mut ast_data);

    match ast {
        Ok(stmts) => {
            ast_data.debug(&stmts);

            let mut compiler = Compiler::new(ast_data);
            compiler.code.instructions.push((vec![], vec![]));

            let mut base_scope = compiler.scopes.insert(Scope::base());

            match compiler.compile_stmts(stmts, base_scope, 0) {
                Ok(_) => {
                    compiler.code.debug();

                    // let bytes = to_bytes(&compiler.code);
                    // println!("bytes: {}", bytes.len());

                    // let mut file = File::create("test.spwnc").unwrap();
                    // file.write_all(&bytes).unwrap();

                    // let compressed = lz4_compression::prelude::compress(&bytes);
                    // println!(
                    //     "lz4 bytes: {}, {:.2}%",
                    //     compressed.len(),
                    //     (compressed.len() as f64) / (bytes.len() as f64) * 100.0
                    // );

                    // let compressed =
                    //     yazi::compress(&bytes, yazi::Format::Raw, yazi::CompressionLevel::BestSize)
                    //         .unwrap();
                    // println!(
                    //     "zlib bytes: {}, {:.2}%",
                    //     compressed.len(),
                    //     (compressed.len() as f64) / (bytes.len() as f64) * 100.0
                    // );

                    // println!("{:?}", bytes);

                    let mut globals = Globals {
                        memory: SlotMap::default(),
                        types: AHashMap::new(),
                        contexts: FullContext::single(compiler.code.var_count),
                    };
                    globals.init();
                    // let mut globals = Globals {
                    //     memory: SlotMap::default(),
                    //     contexts: FullContext::Split(
                    //         Box::new(FullContext::single(compiler.code.var_count)),
                    //         Box::new(FullContext::single(compiler.code.var_count)),
                    //     ),
                    // };

                    // if let Err(e) = execute(&mut globals, &compiler.code, 0) {
                    //     e.raise(source);
                    // }
                }
                Err(e) => e.raise(source),
            }
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
    // println!("{}", std::mem::size_of::<Instruction>());
}
