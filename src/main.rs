mod compilation;
mod error;
mod leveldata;
mod parsing;
mod sources;
mod vm;

use std::io::{self, Write};
use std::{fs, path::PathBuf};

use compilation::compiler::Compiler;
use compilation::error::CompilerError;
use parsing::ast::ASTData;
use parsing::error::SyntaxError;
use parsing::parser::Parser;
use sources::SpwnSource;

use crate::compilation::code::Instruction;
use crate::vm::context::FullContext;
use crate::vm::interpreter::{run_func, Globals};

fn run_spwn(code: String, source: SpwnSource, _doctest: bool) {
    // if doctest {
    //     parse_doc_comments(code.clone());
    //     return;
    // }

    macro_rules! handle {
        ($a:expr $(=> $arg:expr)?) => {
            match $a {
                Ok(a) => a,
                Err(e) => {
                    e.raise(&code, source $(, $arg)?);
                    return;
                }
            }
        };
    }

    let (ast, stmts) = handle!(parse_stage(&code, &source));
    let compiler = handle!(bytecode_generation(ast, stmts, &source));

    let mut globals = Globals::new(compiler.types);
    let mut contexts = FullContext::single(compiler.code.var_count);

    let start = std::time::Instant::now();
    handle!(run_func(&mut globals, &compiler.code, 0, &mut contexts) => &globals);

    // get end time
    let end = std::time::Instant::now();
    let duration = end.duration_since(start);
    println!("Duration: {:?}", duration);

    let mut all_objects = globals.objects;

    all_objects.extend(globals.triggers);
    match leveldata::postprocess::append_objects(all_objects, "") {
        Ok((new_ls, used_ids)) => {
            println!("\n{}", ansi_term::Color::Purple.bold().paint("Level:"));
            for (i, len) in used_ids.iter().enumerate() {
                if *len > 0 {
                    println!(
                        "{} {}",
                        len,
                        ["groups", "colors", "block IDs", "item IDs"][i]
                    );
                }
            }
            println!("Level string:\n{}", new_ls);
        }
        Err(e) => eprintln!("{}", ansi_term::Color::Red.paint(e)),
    };
}

// fn post_stage(globals: Globals) {
//     // postprocess/add objects to level
//     let mut all_objects = globals.objects;

//     all_objects.extend(globals.triggers);
//     match leveldata::postprocess::append_objects(all_objects, "") {
//         Ok((new_ls, used_ids)) => {
//             println!("\n{}", "Level:".fg(Color::Magenta));
//             for (i, len) in used_ids.iter().enumerate() {
//                 if *len > 0 {
//                     println!(
//                         "{} {}",
//                         len,
//                         ["groups", "colors", "block IDs", "item IDs"][i].fg(Color::White)
//                     );
//                 }
//             }
//             println!("Level string:\n{}", new_ls);
//         }
//         Err(e) => eprintln!("{}", e.fg(Color::Red)),
//     };
// }

// fn interpret_stage(compiler: Compiler) -> Result<Globals, RuntimeError> {
//     let mut globals = Globals::new();
//     globals.init();
//     let mut context = FullContext::single();
//     execute_code(
//         &mut globals,
//         &compiler.code,
//         0,
//         &mut context,
//         vec![],
//         vec![],
//     )?;
//     Ok(globals)
// }

fn bytecode_generation(
    ast_data: ASTData,
    stmts: Vec<parsing::ast::StmtKey>,
    source: &SpwnSource,
) -> Result<Compiler, CompilerError> {
    let mut compiler = Compiler::new(ast_data, source.clone());
    compiler.start_compile(stmts)?;
    compiler.code.debug();
    Ok(compiler)
}

fn parse_stage(
    code: &str,
    source: &SpwnSource,
) -> Result<(ASTData, Vec<parsing::ast::StmtKey>), SyntaxError> {
    let mut parser = Parser::new(code, source.clone());
    let mut ast_data = ASTData::new(source.clone());
    let stmts = parser.parse(&mut ast_data)?;
    ast_data.debug(&stmts);
    Ok((ast_data, stmts))
}

fn main() {
    print!("\x1B[2J\x1B[1;1H");
    println!("{}", std::mem::size_of::<Instruction>());

    io::stdout().flush().unwrap();

    let file = std::env::args().nth(1).expect("no filename given");
    let doctest: bool = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "false".to_string())
        .parse()
        .expect("expected bool for doctest");
    let buf = PathBuf::from(file);

    let code = fs::read_to_string(&buf).unwrap();
    run_spwn(code, SpwnSource::File(buf), doctest);
    // println!("{}", std::mem::size_of::<Instruction>());
}
