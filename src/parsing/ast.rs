use strum::{EnumProperty, EnumString, EnumVariantNames};

use crate::{lexing::tokens::Token, sources::CodeSpan};

use super::attributes::{ExprAttribute, ScriptAttribute};

///use super::attributes::{ExprAttribute, ScriptAttribute};

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

#[derive(Debug, Clone)]
pub enum MacroCode {
    Normal(Statements),
    Lambda(ExprNode),
}

#[derive(Debug, Clone)]
pub struct ExprNode {
    pub expr: Box<Expression>,
    pub attributes: Vec<ExprAttribute>,
}

#[derive(Debug, Clone)]
pub struct StmtNode {
    pub stmt: Box<Statement>,
    pub attributes: Vec<StmtNode>,
}

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
    MacroPattern {
        args: Vec<ExprNode>,
        ret_type: ExprNode,
    },

    TriggerFunc {
        attributes: Vec<ExprAttribute>,
        code: Statements,
    },

    TriggerFuncCall(ExprNode),

    Ternary {
        cond: ExprNode,
        if_true: ExprNode,
        if_false: ExprNode,
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
    For {
        var: String,
        iterator: ExprNode,
        code: Statements,
    },

    Arrow(Box<Statement>),

    Return(Option<ExprNode>),
    Break,
    Continue,
}

pub type Statements = Vec<StmtNode>;

impl Expression {
    pub fn into_node(self, attributes: Vec<ExprAttribute>) -> ExprNode {
        ExprNode {
            expr: Box::new(self),
            attributes,
        }
    }
}
impl Statement {
    pub fn into_node(self, attributes: Vec<StmtNode>) -> StmtNode {
        StmtNode {
            stmt: Box::new(self),
            attributes,
        }
    }
}

pub struct Ast {
    pub statements: Vec<StmtNode>,
    pub file_attributes: Vec<ScriptAttribute>,
}

#[derive(Clone, Debug)]
pub struct Spanned<T> {
    pub value: T,
    pub span: CodeSpan,
}
impl<T> Spanned<T> {
    pub fn split(self) -> (T, CodeSpan) {
        (self.value, self.span)
    }
    pub fn extended(self, other: CodeSpan) -> Self {
        Self {
            span: self.span.extend(other),
            ..self
        }
    }
    pub fn apply_fn<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        f(self.value).spanned(self.span)
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
        Spanned { value: self, span }
    }
}
