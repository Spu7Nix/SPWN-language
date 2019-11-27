#[derive(Debug)]
pub enum Statement {
    Definition(Definition),
    Event(Event),
    Call(Call),
    EOI
    
}

#[derive(Debug)]
pub enum Value {
    ID(ID),
    Number(f64),
    CmpStmt(CompoundStatement),
    Symbol(String)
    
}

#[derive(Debug)]
pub struct Definition {
    //#[pest_ast(outer(with(span_into_str), with(str::parse), with(Definition::unwrap)))]
    pub symbol: String,
    pub value: Value,
}

#[derive(Debug)]
pub struct Event {
    pub symbol: String,
    pub cmp_stmt: CompoundStatement,
}

#[derive(Debug)]
pub struct Call {
    pub value: Value,
    pub symbols: Vec<String>
}

#[derive(Debug)]
pub struct CompoundStatement {
    pub statements: Vec<Statement>
}

#[derive(Debug)]
pub struct ID {
    pub number: u16,
    pub class_name: String
}

#[derive(Debug)]
pub struct File {
    pub statements: Vec<Statement>,
    eoi: EOI,
}

#[derive(Debug)]
struct EOI;