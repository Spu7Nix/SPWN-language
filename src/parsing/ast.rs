use std::fmt::{write, Display};
use std::path::PathBuf;

use delve::{EnumDisplay, EnumToStr};
use lasso::Spur;
use serde::{Deserialize, Serialize};

use super::attributes::{ExprAttribute, ScriptAttribute, StmtAttribute};
use super::utils::operators::{AssignOp, BinOp, UnaryOp};
use crate::gd::ids::IDClass;
use crate::gd::object_keys::ObjectKey;
use crate::sources::CodeSpan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportType {
    Module(String),
    Library(String),
}

impl ImportType {
    pub fn to_path_name(&self) -> (String, PathBuf) {
        match self {
            ImportType::Module(s) => {
                let rel_path = PathBuf::from(s);
                (
                    rel_path.file_stem().unwrap().to_str().unwrap().to_string(),
                    rel_path,
                )
            }
            ImportType::Library(name) => {
                let rel_path = PathBuf::from(format!("libraries/{name}/lib.spwn"));
                (
                    rel_path
                        .parent()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                    rel_path,
                )
            }
        }
    }
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
    pub span: CodeSpan,
}

#[derive(Debug, Clone)]
pub struct StmtNode {
    pub stmt: Box<Statement>,
    pub attributes: Vec<Spanned<StmtAttribute>>,
    pub span: CodeSpan,
}

pub type DictItems = Vec<(Spanned<Spur>, Option<ExprNode>)>;

#[derive(Debug, Clone, EnumToStr)]
pub enum Expression {
    Int(i64),
    Float(f64),
    String(Spur),
    Bool(bool),

    Id(IDClass, Option<u16>),

    Op(ExprNode, BinOp, ExprNode),
    Unary(UnaryOp, ExprNode),

    Var(Spur),
    Type(Spur),

    Array(Vec<ExprNode>),
    Dict(DictItems),

    Maybe(Option<ExprNode>),

    Index {
        base: ExprNode,
        index: ExprNode,
    },
    Member {
        base: ExprNode,
        name: Spanned<Spur>,
    },
    TypeMember {
        base: ExprNode,
        name: Spanned<Spur>,
    },
    Associated {
        base: ExprNode,
        name: Spanned<Spur>,
    },

    Call {
        base: ExprNode,
        params: Vec<ExprNode>,
        named_params: Vec<(Spanned<Spur>, ExprNode)>,
    },

    Macro {
        args: Vec<(Spanned<Spur>, Option<ExprNode>, Option<ExprNode>)>,
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

    Typeof(ExprNode),

    Builtins,
    Empty,

    Import(ImportType),

    Instance {
        base: ExprNode,
        items: DictItems,
    },
    Obj(ObjectType, Vec<(Spanned<ObjKeyType>, ExprNode)>),
}

#[derive(Debug, Clone, Copy, EnumToStr, PartialEq, Eq)]
pub enum ObjectType {
    Object,
    Trigger,
}

#[derive(Debug, Clone, Copy, EnumToStr, PartialEq, Eq, Hash, EnumDisplay)]
pub enum ObjKeyType {
    #[delve(display = |o: &ObjectKey| format!("{}", <&ObjectKey as Into<&'static str>>::into(o)))]
    Name(ObjectKey),
    #[delve(display = |n: &u8| format!("{n}"))]
    Num(u8),
}

#[derive(Debug, Clone, EnumToStr)]
pub enum Statement {
    Expr(ExprNode),
    Let(ExprNode, ExprNode),
    AssignOp(ExprNode, AssignOp, ExprNode),

    If {
        branches: Vec<(ExprNode, Statements)>,
        else_branch: Option<Statements>,
    },
    While {
        cond: ExprNode,
        code: Statements,
    },
    For {
        iter: ExprNode,
        iterator: ExprNode,
        code: Statements,
    },
    TryCatch {
        try_code: Statements,
        error_var: Option<Spur>,
        catch_code: Statements,
    },

    Arrow(Box<StmtNode>),

    Return(Option<ExprNode>),
    Break,
    Continue,

    TypeDef(Spur),

    ExtractImport(ImportType),

    Impl {
        typ: Spur,
        items: DictItems,
    },

    Print(ExprNode),
}

pub type Statements = Vec<StmtNode>;

impl Expression {
    pub fn into_node(self, attributes: Vec<ExprAttribute>, span: CodeSpan) -> ExprNode {
        ExprNode {
            expr: Box::new(self),
            attributes,
            span,
        }
    }
}
impl Statement {
    pub fn into_node(self, attributes: Vec<Spanned<StmtAttribute>>, span: CodeSpan) -> StmtNode {
        StmtNode {
            stmt: Box::new(self),
            attributes,
            span,
        }
    }
}

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
#[derive(Debug)]
pub struct Ast {
    pub statements: Vec<StmtNode>,
    pub file_attributes: Vec<ScriptAttribute>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl<T: Copy> Copy for Spanned<T> {}

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
