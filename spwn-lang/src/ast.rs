//! Abstract Syntax Tree (AST) type definitions

use std::path::PathBuf;

use crate::compiler_types::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum DictDef {
    Def((String, Expression)),
    Extract(Expression),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Statement {
    pub body: StatementBody,
    pub arrow: bool, //context changing
    pub line: (usize, usize),
}

#[derive(Clone, PartialEq, Debug)]
pub enum StatementBody {
    Definition(Definition),
    Call(Call),
    Expr(Expression),
    Add(Expression),
    Return(Expression),
    Impl(Implementation),
    If(If),
    For(For),
    Error(Error),
    EOI,
}
#[derive(Clone, PartialEq, Debug)]
pub enum ValueLiteral {
    ID(ID),
    Number(f64),
    CmpStmt(CompoundStatement),
    Dictionary(Vec<DictDef>),
    Symbol(String),
    Bool(bool),
    Expression(Expression),
    Str(String),
    Import(PathBuf),
    Array(Vec<Expression>),
    Obj(Vec<(Expression, Expression)>),
    Macro(Macro),
    Resolved(Value),
    Null,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Path {
    Member(String),
    Index(Expression),
    Call(Vec<Argument>),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Definition {
    pub symbol: String,
    pub value: Expression,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Argument {
    pub symbol: Option<String>,
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
    pub args: Vec<Argument>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Macro {
    pub args: Vec<(String, Option<Expression>)>,
    pub body: CompoundStatement,
}

#[derive(Clone, PartialEq, Debug)]
pub struct For {
    pub symbol: String,
    pub array: Expression,
    pub body: Vec<Statement>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Error {
    pub message: Expression,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Variable {
    pub operator: Option<String>,
    pub value: ValueLiteral,
    pub path: Vec<Path>,
}

/*impl Variable {
    pub fn to_expression(&self) -> Expression {
        if let ValueLiteral::Expression(e) = &self.value {
            if self.path.is_empty() {
                return e.to_owned();
            }
        }
        Expression {
            values: vec![self.clone()],
            operators: Vec::new(),
        }
    }
}*/

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
pub struct Implementation {
    pub symbol: Variable,
    pub members: Vec<DictDef>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct If {
    pub condition: Expression,
    pub if_body: Vec<Statement>,
    pub else_body: Option<Vec<Statement>>,
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
    inp.clone()
        .replace("\"", "")
        .replace("'", "")
        .replace("\r", "")
        .replace("\n", "")
}

#[derive(Debug)]
struct EOI;
