use crate::ast;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use std::fs;
use std::path::PathBuf;
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

pub fn parse_statements(statements: &mut pest::iterators::Pairs<Rule>) -> Vec<ast::Statement> {
    let mut stmts: Vec<ast::Statement> = vec![];

    let parse_args = |arg: Pair<Rule>| {
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
    };

    let parse_native_prop = |prop: Pair<Rule>| {
        let mut prop_inner = prop.into_inner();
        (
            prop_inner.next().unwrap().as_span().as_str().to_string(),
            prop_inner
                .next()
                .unwrap()
                .into_inner()
                .map(parse_args)
                .collect(),
        )
    };

    for unpacked in statements {
        let mut inner = unpacked.clone().into_inner();
        let mut async_arrow = false;
        let first_elem = match inner.next() {
            Some(val) => val,
            None => break, //EOF
        };
        let statement = match first_elem.as_rule() {
            Rule::async_arrow => {
                async_arrow = true;
                inner.next().unwrap()
            }
            _ => first_elem,
        };
        stmts.push(ast::Statement {
            body: match statement.as_rule() {
                Rule::def => {
                    let mut inner = statement.into_inner();
                    let first = inner.next().unwrap();
                    match first.as_rule() {
                        Rule::expr => ast::StatementBody::Definition(ast::Definition {
                            symbol: "*".to_string(),
                            value: parse_expr(first),
                            props: inner.map(parse_native_prop).collect(),
                        }),
                        Rule::symbol => {
                            let value = parse_expr(inner.next().unwrap());
                            ast::StatementBody::Definition(ast::Definition {
                                symbol: first.as_span().as_str().to_string(),
                                value,
                                props: inner.map(parse_native_prop).collect(),
                            })
                        }
                        _ => unreachable!(),
                    }
                }
                Rule::call => ast::StatementBody::Call(ast::Call {
                    function: parse_variable(statement.into_inner().next().unwrap()),
                }),

                Rule::if_stmt => {
                    let mut inner = statement.into_inner();
                    ast::StatementBody::If(ast::If {
                        condition: parse_expr(inner.next().unwrap()),
                        if_body: parse_statements(&mut inner.next().unwrap().into_inner()),
                        else_body: match inner.next() {
                            Some(body) => Some(parse_statements(&mut body.into_inner())),
                            None => None,
                        },
                    })
                }
                Rule::add_obj => {
                    ast::StatementBody::Add(parse_expr(statement.into_inner().next().unwrap()))
                }

                Rule::for_loop => {
                    let mut inner = statement.into_inner();
                    ast::StatementBody::For(ast::For {
                        symbol: inner.next().unwrap().as_span().as_str().to_string(),
                        array: parse_expr(inner.next().unwrap()),
                        body: parse_statements(&mut inner.next().unwrap().into_inner()),
                    })
                }

                Rule::implement => {
                    let mut inner = statement.into_inner();
                    ast::StatementBody::Impl(ast::Implementation {
                        symbol: parse_variable(inner.next().unwrap()),
                        members: parse_dict(inner.next().unwrap()),
                    })
                }

                Rule::error => ast::StatementBody::Error(ast::Error {
                    message: parse_expr(statement.into_inner().next().unwrap()),
                }),

                Rule::expr => ast::StatementBody::Expr(parse_expr(statement)),
                Rule::retrn => ast::StatementBody::Return(match statement.into_inner().next() {
                    Some(expr) => parse_expr(expr),

                    None => ast::Expression {
                        // null expression
                        values: vec![ast::Variable {
                            operator: None,
                            value: ast::ValueLiteral::Null,
                            path: Vec::new(),
                        }],
                        operators: Vec::new(),
                    },
                }),
                Rule::EOI => ast::StatementBody::EOI,
                _ => {
                    println!(
                        "{:?} is not added to parse_statements yet",
                        statement.as_rule()
                    );
                    ast::StatementBody::EOI
                }
            },

            arrow: async_arrow,
            line: unpacked.as_span().start_pos().line_col(),
        })
    }
    stmts
}

pub fn parse_path(pair: Pair<Rule>) -> ast::Path {
    let parse_args = |arg: Pair<Rule>| {
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
    };
    match pair.as_rule() {
        Rule::symbol => ast::Path::Member(pair.as_span().as_str().to_string()),

        Rule::index => ast::Path::Index(parse_expr(pair.into_inner().next().unwrap())),

        Rule::arguments => ast::Path::Call(pair.into_inner().map(|x| parse_args(x)).collect()),

        _ => unreachable!(),
    }
}

pub fn parse_variable(pair: Pair<Rule>) -> ast::Variable {
    let mut call_list = pair.into_inner();
    let first = call_list.next().unwrap();

    let value: ast::ValueLiteral;

    let operator = match first.as_rule() {
        Rule::unary_operator => {
            value = parse_value(call_list.next().unwrap());
            Some(first.as_span().as_str().to_string())
        }
        _ => {
            value = parse_value(first);
            None
        }
    };
    let path: Vec<ast::Path> = call_list.map(|x| parse_path(x)).collect();
    fn parse_value(pair: Pair<Rule>) -> ast::ValueLiteral {
        match pair.as_rule() {
            Rule::id => {
                let number: u16;
                let mut Func = pair.into_inner();
                let mut unspecified = false;
                let first_value = Func.next().unwrap();
                let class_name: String;

                if first_value.as_rule() == Rule::number {
                    number = first_value.as_span().as_str().parse().unwrap();
                    class_name = Func.next().unwrap().as_span().as_str().to_string();
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

            Rule::macro_def => {
                let mut inner = pair.into_inner();
                ast::ValueLiteral::Macro(ast::Macro {
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

            Rule::number => {
                ast::ValueLiteral::Number(pair.as_span().as_str().parse().expect("invalid number"))
            }

            Rule::bool => ast::ValueLiteral::Bool(pair.as_span().as_str() == "true"),

            Rule::dictionary => ast::ValueLiteral::Dictionary(parse_dict(pair)),

            Rule::cmp_stmt => ast::ValueLiteral::CmpStmt(ast::CompoundStatement {
                statements: parse_statements(&mut pair.into_inner()),
            }),

            Rule::obj => ast::ValueLiteral::Obj(
                pair.into_inner()
                    .map(|prop| {
                        let mut inner = prop.into_inner();
                        (
                            parse_expr(inner.next().unwrap()),
                            parse_expr(inner.next().unwrap()),
                        )
                    })
                    .collect(),
            ),

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

    ast::Variable {
        value,
        path,
        operator,
    }
}

fn parse_dict(pair: Pair<Rule>) -> Vec<ast::DictDef> {
    let mut out: Vec<ast::DictDef> = Vec::new();
    for def in pair.into_inner() {
        match def.as_rule() {
            Rule::dict_def => {
                let mut inner = def.into_inner();
                out.push(ast::DictDef::Def((
                    inner.next().unwrap().as_span().as_str().to_string(), //symbol
                    parse_expr(inner.next().unwrap()),                    //expression
                )));
            }

            Rule::dict_extract => out.push(ast::DictDef::Extract(parse_expr(
                def.into_inner().next().unwrap(),
            ))),

            _ => unreachable!(),
        };
    }
    out
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

/*#[cfg(test)]
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
}*/
