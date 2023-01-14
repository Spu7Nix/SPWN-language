use crate::{lexing::tokens::Token, sources::CodeSpan};

#[derive(Debug, Clone)]
pub enum ImportType {
    Module(String),
    Library(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum IDClass {
    Group = 0,
    Color = 1,
    Block = 2,
    Item = 3,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum FileAttribute {
    CacheOutput,
    NoStd,
    ConsoleOutput,
    NoLevel,
    NoBytecodeCache,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Attribute {
    NoOptimise,
}

#[derive(Debug, Clone)]
pub enum MacroCode {
    Normal(Statements),
    Lambda(ExprNode),
}

pub type ExprNode = Spanned<Expression>;
pub type StmtNode = Spanned<Statement>;

pub type DictItems = Vec<(Spanned<String>, Option<ExprNode>)>;

#[derive(Debug, Clone)]
pub enum Expression {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),

    Id(IDClass, Option<u16>),

    Op(ExprNode, Token, ExprNode),
    Unary(Token, ExprNode),

    Var(String),
    Type(String),

    Array(Vec<ExprNode>),
    Dict(DictItems),

    Maybe(Option<ExprNode>),

    Index {
        base: ExprNode,
        index: ExprNode,
    },
    Member {
        base: ExprNode,
        name: String,
    },
    Associated {
        base: ExprNode,
        name: String,
    },

    Macro {
        args: Vec<(Spanned<String>, Option<ExprNode>, Option<ExprNode>)>,
        ret_type: Option<ExprNode>,
        code: MacroCode,
    },

    TriggerFunc {
        code: Statements,
    },

    Builtins,
    Empty,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expr(ExprNode),
    Let(String, ExprNode),
    If {
        branches: Vec<(ExprNode, Statements)>,
        else_branch: Option<Statements>,
    },
    While {
        cond: ExprNode,
        code: Statements,
    },

    Arrow(Box<Statement>),
}

pub type Statements = Vec<StmtNode>;

#[derive(Clone, Debug)]
pub struct Spanned<T> {
    pub value: Box<T>,
    pub span: CodeSpan,
}
impl<T> Spanned<T> {
    pub fn split(self) -> (T, CodeSpan) {
        (*self.value, self.span)
    }
    pub fn extended(self, other: CodeSpan) -> Self {
        Self {
            span: self.span.extend(other),
            ..self
        }
    }
}

pub trait Spannable {
    fn spanned(self, span: CodeSpan) -> Spanned<Self>
    where
        Self: Sized;
}

impl<T> Spannable for T {
    fn spanned(self, span: CodeSpan) -> Spanned<Self>
    where
        Self: Sized,
    {
        Spanned {
            value: Box::new(self),
            span,
        }
    }
}
