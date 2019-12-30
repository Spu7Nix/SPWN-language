mod ast;
mod compiler;
mod levelstring;
mod native;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use std::path::PathBuf;
use std::{env, fs};

//use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "spwn.pest"]
pub struct SPWNParser;

pub fn parse_spwn(path: &PathBuf) -> Vec<ast::Statement> {
    let unparsed = fs::read_to_string(path).expect("Something went wrong reading the file");

    let parse_tree = SPWNParser::parse(Rule::spwn, &unparsed)
        .expect("unsuccessful parse")
        .next()
        .unwrap(); // get and unwrap the `spwn` rule; never fails

    parse_statements(&mut parse_tree.into_inner())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let script_path = PathBuf::from(&args[1]);
    let statements = parse_spwn(&script_path);
    // for statement in statements.iter() {
    //     println!("{:?}\n\n", statement);
    // }

    let compiled = compiler::compile_spwn(statements, script_path);
    let mut level_string = String::new();

    for trigger in compiled.obj_list {
        level_string += &levelstring::serialize_trigger(trigger);
    }
    println!("Using {} groups", compiled.closed_groups.len());

    println!("{:?}", level_string);
}

fn parse_statements(statements: &mut pest::iterators::Pairs<Rule>) -> Vec<ast::Statement> {
    let mut stmts: Vec<ast::Statement> = vec![];

    for statement in statements {
        stmts.push(match statement.as_rule() {
            Rule::def => {
                let mut inner = statement.into_inner();
                let first = inner.next().unwrap();
                match first.as_rule() {
                    Rule::expr => ast::Statement::Definition(ast::Definition {
                        symbol: "*".to_string(),
                        value: parse_expr(first),
                    }),
                    Rule::symbol => {
                        let value = parse_expr(inner.next().unwrap());
                        ast::Statement::Definition(ast::Definition {
                            symbol: first.as_span().as_str().to_string(),
                            value,
                        })
                    }
                    _ => unreachable!(),
                }
            }
            Rule::event => {
                let mut info = statement.into_inner();
                ast::Statement::Event(ast::Event {
                    symbol: info.next().unwrap().as_span().as_str().to_string(),
                    args: info
                        .next()
                        .unwrap()
                        .into_inner()
                        .map(|arg| parse_expr(arg))
                        .collect(),
                    func: parse_variable(info.next().unwrap()),
                })
            }
            Rule::call => ast::Statement::Call(ast::Call {
                function: parse_variable(statement.into_inner().next().unwrap()),
            }),
            Rule::native => {
                let mut info = statement.into_inner();
                ast::Statement::Native(ast::Native {
                    function: parse_variable(info.next().unwrap()),
                    args: info
                        .next()
                        .unwrap()
                        .into_inner()
                        .map(|arg| {
                            let mut argument = arg.into_inner();
                            let first = argument.next().unwrap();
                            match first.as_rule() {
                                Rule::symbol => ast::Argument {
                                    symbol: Some(first.as_span().as_str().to_string()),
                                    value: parse_expr(argument.next().unwrap()),
                                },

                                Rule::expr => ast::Argument {
                                    symbol: None,
                                    value: parse_expr(first),
                                },

                                _ => unreachable!(),
                            }
                        })
                        .collect(),
                })
            }
            Rule::macro_def => {
                let mut inner = statement.into_inner();
                ast::Statement::Macro(ast::Macro {
                    name: inner.next().unwrap().as_span().as_str().to_string(),
                    args: inner
                        .next()
                        .unwrap()
                        .into_inner()
                        .map(|arg| {
                            let mut full = arg.into_inner();
                            let name = full.next().unwrap().as_span().as_str().to_string();
                            let default = match full.next() {
                                Some(value) => Some(parse_expr(value)),
                                None => None,
                            };

                            (name, default)
                        })
                        .collect(),
                    body: ast::CompoundStatement {
                        statements: parse_statements(&mut inner.next().unwrap().into_inner()),
                    },
                })
            }
            Rule::retrn => ast::Statement::Return,
            Rule::EOI => ast::Statement::EOI,
            _ => {
                println!(
                    "{:?} is not added to parse_statements yet",
                    statement.as_rule()
                );
                ast::Statement::EOI
            }
        })
    }
    stmts
}

fn parse_path(pair: Pair<Rule>) -> ast::Path {
    match pair.as_rule() {
        Rule::symbol => ast::Path::Member(pair.as_span().as_str().to_string()),

        Rule::index => ast::Path::Index(parse_expr(pair.into_inner().next().unwrap())),

        Rule::arguments => ast::Path::Call(pair.into_inner().map(|x| parse_expr(x)).collect()),

        _ => unreachable!(),
    }
}

fn parse_variable(pair: Pair<Rule>) -> ast::Variable {
    let mut call_list = pair.into_inner();
    let value = parse_value(call_list.next().unwrap());
    let path: Vec<ast::Path> = call_list.map(|x| parse_path(x)).collect();
    fn parse_value(pair: Pair<Rule>) -> ast::ValueLiteral {
        match pair.as_rule() {
            Rule::id => {
                let number: u16;
                let mut scope = pair.into_inner();
                let mut unspecified = false;
                let first_value = scope.next().unwrap();
                let class_name: String;

                if first_value.as_rule() == Rule::number {
                    number = first_value.as_span().as_str().parse().unwrap();
                    class_name = scope.next().unwrap().as_span().as_str().to_string();
                } else {
                    unspecified = true;
                    number = 0;
                    class_name = first_value.as_span().as_str().to_string();
                }

                ast::ValueLiteral::ID(ast::ID {
                    number,
                    unspecified,
                    class_name,
                })
            }
            Rule::number => {
                ast::ValueLiteral::Number(pair.as_span().as_str().parse().expect("invalid number"))
            }

            Rule::bool => ast::ValueLiteral::Bool(pair.as_span().as_str() == "true"),

            Rule::dictionary => ast::ValueLiteral::Dictionary(ast::Dictionary {
                members: parse_statements(&mut pair.into_inner()),
            }),

            Rule::cmp_stmt => ast::ValueLiteral::CmpStmt(ast::CompoundStatement {
                statements: parse_statements(&mut pair.into_inner()),
            }),

            Rule::value_literal => parse_value(pair.into_inner().next().unwrap()),
            Rule::variable => parse_value(pair.into_inner().next().unwrap()),
            Rule::expr => ast::ValueLiteral::Expression(parse_expr(pair)),
            Rule::symbol => ast::ValueLiteral::Symbol(pair.as_span().as_str().to_string()),
            Rule::string => {
                ast::ValueLiteral::Str(ast::str_content(pair.as_span().as_str().to_string()))
            }
            Rule::array => {
                ast::ValueLiteral::Array(pair.into_inner().map(|x| parse_expr(x)).collect())
            }
            Rule::import => ast::ValueLiteral::Import(PathBuf::from(ast::str_content(
                pair.into_inner()
                    .next()
                    .unwrap()
                    .as_span()
                    .as_str()
                    .to_string(),
            ))),
            _ => {
                println!("{:?} is not added to parse_values yet", pair.as_rule());
                ast::ValueLiteral::Number(0.0)
            }
        }
    }

    ast::Variable { value, path }
}

fn parse_expr(pair: Pair<Rule>) -> ast::Expression {
    let mut values: Vec<ast::Variable> = Vec::new();
    let mut operators: Vec<String> = Vec::new();

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::operator => operators.push(item.as_span().as_str().to_string()),
            _ => values.push(parse_variable(item)),
        }
    }

    ast::Expression { operators, values }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decrypt() {
        //not working on mac :(
        let file_content = fs::read_to_string(
            "/Users/August/Library/Application Support/GeometryDash/CCLocalLevels.dat",
        )
        .expect("Something went wrong reading the file");
        println!(
            "{}",
            levelstring::get_level_string(file_content.to_string())
        );
    }
}
