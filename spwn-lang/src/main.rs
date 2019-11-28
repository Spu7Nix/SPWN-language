mod ast;

use std::{env, fs};
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "spwn.pest"]
pub struct SPWNParser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let unparsed = fs::read_to_string(&args[1]).expect("Something went wrong reading the file");

    let parse_tree = SPWNParser::parse(Rule::spwn, &unparsed)
        .expect("unsuccessful parse").next().unwrap(); // get and unwrap the `spwn` rule; never fails

    //println!("{:?}\n\n", &parse_tree);

    let statements = parse_statements(&mut parse_tree.into_inner());

    for s in statements.iter() {
        println!("{:?}\n\n", s);
    }
}

fn parse_statements(statements: &mut pest::iterators::Pairs<Rule>) -> Vec<ast::Statement>{
    let mut stmts: Vec<ast::Statement> = vec![];
    for statement in statements {
        
        stmts.push(match statement.as_rule() {
            Rule::def => ast::Statement::Definition(ast::Definition {
                symbol: statement.clone().into_inner().next().unwrap().as_span().as_str().to_string(),
                value:  parse_value(statement.into_inner().nth(1).unwrap().into_inner().next().unwrap())
            }),
            Rule::event => ast::Statement::Event(ast::Event {
                symbol:   statement.clone().into_inner().next().unwrap().as_span().as_str().to_string(),
                cmp_stmt: ast::CompoundStatement {
                    statements: parse_statements(&mut statement.into_inner().nth(1).unwrap().into_inner())
                }
            }),
            Rule::call => {
                let mut call_list = statement.into_inner();
                let value = parse_value(call_list.next().unwrap().into_inner().next().unwrap());
                let symbols: Vec<String> = call_list.map(|x| x.as_span().as_str().to_string()).collect();
                
                ast::Statement::Call(ast::Call { value, symbols })
            },
            Rule::EOI => ast::Statement::EOI,
            _ => {
                println!("{:?} is not added to parse_statements yet", statement.as_rule());
                ast::Statement::Definition(ast::Definition {
                    symbol: "none".to_string(),
                    value: ast::Value::Number(0.0)
                }) 
            }
        })
    }
    stmts
}

fn parse_value(pair: Pair<Rule>) -> ast::Value {
    match pair.as_rule() {
        Rule::id => ast::Value::ID(ast::ID {
            number: pair.clone().into_inner().next().unwrap().as_span().as_str().parse().unwrap(),
            class_name: pair.into_inner().nth(1).unwrap().as_span().as_str().to_string()
        }),
        Rule::number => ast::Value::Number(pair.as_span().as_str().parse().unwrap()),
        Rule::cmp_stmt => ast::Value::CmpStmt(ast::CompoundStatement {
            statements: parse_statements(&mut pair.into_inner())
        }),
        Rule::symbol => ast::Value::Symbol(pair.as_span().as_str().to_string()),
        _ => {
            println!("{:?} is not added to parse_values yet", pair.as_rule());
            ast::Value::Number(0.0)
        }
    }
}
