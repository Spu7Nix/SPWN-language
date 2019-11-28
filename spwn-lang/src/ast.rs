//! Abstract Syntax Tree (AST) type definitions

#[derive(Debug)]
pub enum Statement {
    Definition(Definition),
    Event(Event),
    Call(Call),
    EOI,
}

#[derive(Debug)]
pub enum ValueLiteral {
    ID(ID),
    Number(f64),
    CmpStmt(CompoundStatement),
    Symbol(String)
}


#[derive(Debug)]
pub struct Definition {
    pub symbol: String,
    pub value: Variable,
}

#[derive(Debug)]
pub struct Event {
    pub symbol: String,
    pub cmp_stmt: CompoundStatement,
}

#[derive(Debug)]
pub struct Call {
    pub function: Variable
}

#[derive(Debug)]
pub struct Variable {
    pub value: ValueLiteral,
    pub symbols: Vec<String>
}

#[derive(Debug)]
pub struct CompoundStatement {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct ID {
    pub number: u16,
    pub class_name: String,
}

#[derive(Debug)]
pub struct File {
    pub statements: Vec<Statement>,
    eoi: EOI,
}

#[derive(Debug)]
struct EOI;
