//! Abstract Syntax Tree (AST) type definitions
use std::path::PathBuf;

#[derive(Clone, PartialEq, Debug)]
pub enum Statement {
    Definition(Definition),
    Event(Event),
    Call(Call),
    Native(Native),
    Macro(Macro),
    EOI,
}
#[derive(Clone, PartialEq, Debug)]
pub enum ValueLiteral {
    ID(ID),
    Number(f64),
    CmpStmt(CompoundStatement),
    Symbol(String),
    Bool(bool),
    Expression(Expression),
    Str(String),
    Import(PathBuf),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Definition {
    pub symbol: String,
    pub value: Expression,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Event {
    pub symbol: String,
    pub args: Vec<Expression>,
    pub func: Variable,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Call {
    pub function: Variable,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Native {
    pub function: Variable,
    pub args: Vec<Expression>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Macro {
    pub name: String,
    pub args: Vec<String>,
    pub body: CompoundStatement,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Variable {
    pub value: ValueLiteral,
    pub symbols: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Expression {
    pub values: Vec<Variable>,
    pub operators: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct CompoundStatement {
    pub statements: Vec<Statement>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ID {
    pub number: u16,
    pub unspecified: bool,
    pub class_name: String,
}

#[derive(Debug)]
pub struct File {
    pub statements: Vec<Statement>,
    eoi: EOI,
}

pub fn str_content(inp: String) -> String {
    let out = inp
        .clone()
        .replace("\"", "")
        .replace("'", "")
        .replace("\r", "")
        .replace("\n", "");
    out
}

#[derive(Debug)]
struct EOI;
