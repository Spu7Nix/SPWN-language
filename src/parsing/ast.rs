use std::path::{Path, PathBuf};
use std::rc::Rc;

use delve::{EnumDisplay, EnumToStr};
use derive_more::Deref;
use lasso::Spur;
use serde::{Deserialize, Serialize};

use super::attributes::{Attributes, FileAttribute};
use super::utils::operators::{AssignOp, BinOp, Operator, UnaryOp};
use crate::sources::{CodeSpan, Spannable, Spanned, SpwnSource};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum StringContent {
    Normal(Spur),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct StringType {
    pub s: StringContent,
    pub bytes: bool,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ImportSettings {
    pub typ: ImportType,
    pub is_absolute: bool,
    pub allow_builtin_impl: bool,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImportType {
    File,
    Library,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub path: PathBuf,
    pub settings: ImportSettings,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum MacroCode {
    Normal(Statements),
    Lambda(ExprNode),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct ExprNode {
    pub expr: Box<Expression>,
    pub attributes: Vec<Attributes>,
    pub span: CodeSpan,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct StmtNode {
    pub stmt: Box<Statement>,
    pub attributes: Vec<Spanned<Attributes>>,
    pub span: CodeSpan,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct PatternNode {
    pub pat: Box<Pattern<Spur, PatternNode, ExprNode>>,
    pub span: CodeSpan,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct DictItem {
    pub name: Spanned<Spur>,
    pub attributes: Vec<Spanned<Attributes>>,
    pub value: Option<ExprNode>,
}

impl From<DictItem> for &'static str {
    fn from(_: DictItem) -> Self {
        "Dict Item"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroArg<N, D, P> {
    Single {
        name: N,
        pattern: Option<P>,
        default: Option<D>,
        is_ref: bool,
    },
    Spread {
        name: N,
        pattern: Option<P>,
    },
}

impl<N, D, P> MacroArg<N, D, P> {
    pub fn name(&self) -> &N {
        match self {
            MacroArg::Single { name, .. } | MacroArg::Spread { name, .. } => name,
        }
    }

    pub fn default(&self) -> &Option<D> {
        match self {
            MacroArg::Single { default, .. } => default,
            _ => unreachable!(),
        }
    }

    pub fn default_mut(&mut self) -> &mut Option<D> {
        match self {
            MacroArg::Single { default, .. } => default,
            _ => unreachable!(),
        }
    }

    pub fn pattern(&self) -> &Option<P> {
        match self {
            MacroArg::Single { pattern, .. } | MacroArg::Spread { pattern, .. } => pattern,
        }
    }

    pub fn pattern_mut(&mut self) -> &mut Option<P> {
        match self {
            MacroArg::Single { pattern, .. } | MacroArg::Spread { pattern, .. } => pattern,
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, EnumToStr)]
pub enum Expression {
    Int(i64),
    Float(f64),
    String(StringType),
    Bool(bool),

    // Id(IDClass, Option<u16>),
    Op(ExprNode, BinOp, ExprNode),
    Unary(UnaryOp, ExprNode),

    Var(Spur),
    Type(Spur),

    Array(Vec<ExprNode>),
    Dict(Vec<Vis<DictItem>>),

    Maybe(Option<ExprNode>),

    Is(ExprNode, PatternNode),

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
        args: Vec<MacroArg<Spanned<Spur>, ExprNode, PatternNode>>,
        ret_type: Option<ExprNode>,
        code: MacroCode,
    },

    TriggerFunc {
        attributes: Vec<Attributes>,
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
    Epsilon,

    Import(Import),

    Instance {
        base: ExprNode,
        items: Vec<Vis<DictItem>>,
    },
    // Obj(ObjectType, Vec<(Spanned<ObjKeyType>, ExprNode)>),
}

#[derive(Debug, Clone, Copy, EnumToStr, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Object,
    Trigger,
}

// #[derive(Debug, Clone, Copy, EnumToStr, PartialEq, Eq, Hash, EnumDisplay)]
// pub enum ObjKeyType {
//     #[delve(display = |o: &ObjectKey| <&ObjectKey as Into<&str>>::into(o).to_string())]
//     Name(ObjectKey),
//     #[delve(display = |n: &u8| format!("{n}"))]
//     Num(u8),
// }

#[cfg_attr(test, derive(PartialEq))]
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
        iter_var: ExprNode,
        iterator: ExprNode,
        code: Statements,
    },
    TryCatch {
        try_code: Statements,
        // error_var: Option<Spur>,
        // catch_code: Statements,
        branches: Vec<(ExprNode, Statements)>,
        catch_all: Option<Statements>,
    },

    Arrow(Box<StmtNode>),

    Return(Option<ExprNode>),
    Break,
    Continue,

    TypeDef(Vis<Spur>),

    ExtractImport(Import),

    Impl {
        base: ExprNode,
        items: Vec<Vis<DictItem>>,
    },
    Overload {
        op: Operator,
        macros: Vec<ExprNode>,
    },

    Dbg(ExprNode),

    Throw(ExprNode),
}

pub type Statements = Vec<StmtNode>;

// T = type, P = pattern, E = expression
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pattern<T, P, E> {
    Any,

    Type(T),
    Either(P, P),
    Both(P, P),

    Eq(E),
    Neq(E),
    Lt(E),
    Lte(E),
    Gt(E),
    Gte(E),

    MacroPattern { args: Vec<P>, ret_type: P },
}

impl Expression {
    pub fn into_node(self, attributes: Vec<Attributes>, span: CodeSpan) -> ExprNode {
        ExprNode {
            expr: Box::new(self),
            attributes,
            span,
        }
    }
}
impl Statement {
    pub fn into_node(self, attributes: Vec<Spanned<Attributes>>, span: CodeSpan) -> StmtNode {
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
    pub file_attributes: Vec<FileAttribute>,
}

pub trait VisTrait {
    type Value;

    fn is_priv(&self) -> bool;
    fn is_pub(&self) -> bool;
    fn value(&self) -> &Self::Value;
    fn value_mut(&mut self) -> &mut Self::Value;

    fn source(&self) -> Option<&Rc<SpwnSource>> {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VisSource<T> {
    Public(T),
    Private(T, Rc<SpwnSource>),
}

impl<T> VisTrait for VisSource<T> {
    type Value = T;

    fn is_priv(&self) -> bool {
        matches!(self, VisSource::Private { .. })
    }

    fn is_pub(&self) -> bool {
        matches!(self, VisSource::Public { .. })
    }

    fn value(&self) -> &Self::Value {
        match self {
            VisSource::Public(v) => v,
            VisSource::Private(v, _) => v,
        }
    }

    fn value_mut(&mut self) -> &mut Self::Value {
        match self {
            VisSource::Public(v) => v,
            VisSource::Private(v, _) => v,
        }
    }

    fn source(&self) -> Option<&Rc<SpwnSource>> {
        match self {
            VisSource::Public(..) => None,
            VisSource::Private(.., s) => Some(s),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Vis<T> {
    Public(T),
    Private(T),
}

impl<T> VisTrait for Vis<T> {
    type Value = T;

    fn is_priv(&self) -> bool {
        matches!(self, Vis::Private { .. })
    }

    fn is_pub(&self) -> bool {
        matches!(self, Vis::Public { .. })
    }

    fn value(&self) -> &Self::Value {
        match self {
            Vis::Public(v) => v,
            Vis::Private(v) => v,
        }
    }

    fn value_mut(&mut self) -> &mut Self::Value {
        match self {
            Vis::Public(v) => v,
            Vis::Private(v) => v,
        }
    }
}
