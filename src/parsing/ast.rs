use crate::sources::CodeSpan;

use super::lexer::Token;

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
pub struct ExprNode {
    pub expr: Box<Expression>,
    pub span: CodeSpan,
}
#[derive(Debug, Clone)]
pub enum Expression {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),

    Id(IDClass, Option<u16>),

    Op(ExprNode, Token, ExprNode),
    Unary(Token, ExprNode),

    Array(Vec<ExprNode>),
    Maybe(Option<ExprNode>),

    Var(String),

    Index { base: ExprNode, index: ExprNode },

    TriggerFunc(Statements),

    Type(String),
}

#[derive(Debug, Clone)]
pub struct StmtNode {
    pub stmt: Box<Statement>,
    //pub has_arrow: bool,
    pub span: CodeSpan,
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

impl Expression {
    pub fn into_node(self, span: CodeSpan) -> ExprNode {
        ExprNode {
            span,
            expr: Box::new(self),
        }
    }
}
impl Statement {
    pub fn into_node(self, span: CodeSpan) -> StmtNode {
        StmtNode {
            span,
            stmt: Box::new(self),
            //has_arrow,
        }
    }
}

pub type Statements = Vec<StmtNode>;

impl ExprNode {
    pub fn extended(self, other: CodeSpan) -> Self {
        Self {
            span: self.span.extend(other),
            ..self
        }
    }
}
impl StmtNode {
    pub fn extended(self, other: CodeSpan) -> Self {
        Self {
            span: self.span.extend(other),
            ..self
        }
    }
}
