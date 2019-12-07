//! Abstract Syntax Tree (AST) type definitions

#[derive(Clone)]
#[derive(Debug)]
pub enum Statement {
    Definition(Definition),
    Event(Event),
    Call(Call),
    Native(Native),
    EOI
}
#[derive(Clone)]
#[derive(Debug)]
pub enum ValueLiteral {
    ID(ID),
    Number(f64),
    CmpStmt(CompoundStatement),
    Symbol(String),
    Bool(bool)
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Definition {
    pub symbol: String,
    pub value: Variable,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Event {
    pub symbol: String,
    pub args: Vec<Variable>,
    pub cmp_stmt: CompoundStatement,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Call {
    pub function: Variable
}
#[derive(Clone)]
#[derive(Debug)]
pub struct Native {
    pub function: Variable,
    pub args: Vec<Variable>
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Variable {
    pub value: ValueLiteral,
    pub symbols: Vec<String>
}

#[derive(Clone)]
#[derive(Debug)]
pub struct CompoundStatement {
    pub statements: Vec<Statement>,
}

#[derive(Clone)]
#[derive(Debug)]
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

#[derive(Debug)]
struct EOI;
