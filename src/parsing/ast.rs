use std::cell::RefCell;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ahash::AHashMap;
use base64::Engine;
use delve::{EnumDisplay, EnumToStr};
use derive_more::Deref;
use lasso::Spur;
use serde::{Deserialize, Serialize};

use super::attributes::{Attributes, FileAttribute};
use super::operators::operators::{AssignOp, BinOp, Operator, UnaryOp};
use crate::gd::ids::IDClass;
use crate::interpreting::value::Value;
use crate::sources::{CodeSpan, Spannable, Spanned, SpwnSource};
use crate::util::{Either, ImmutStr, Interner};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum StringType {
    Normal(Spur),
    FString(Vec<Either<Spur, ExprNode>>),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct StringContent {
    pub s: StringType,
    pub bytes: bool,
    pub base64: bool,
    pub unindent: bool,
}

impl StringContent {
    pub fn normal(s: Spur) -> Self {
        StringContent {
            s: StringType::Normal(s),
            bytes: false,
            base64: false,
            unindent: false,
        }
    }

    pub fn get_compile_time(&self, interner: &Rc<RefCell<Interner>>) -> Option<String> {
        if self.bytes {
            return None;
        }
        let mut s = match self.s {
            StringType::Normal(k) => interner.borrow().resolve(&k).to_string(),
            _ => return None,
        };
        if self.unindent {
            s = unindent::unindent(&s)
        }
        if self.base64 {
            s = base64::engine::general_purpose::URL_SAFE.encode(s)
        }
        Some(s)
    }
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
    pub pat: Box<Pattern<Spur, PatternNode, ExprNode, Spur>>,
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
pub enum MatchBranch {
    Expr(ExprNode),
    Block(Statements),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, EnumToStr)]
pub enum Expression {
    Int(i64),
    Float(f64),
    String(StringContent),
    Bool(bool),

    Id(IDClass, Option<u16>),
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
    Match {
        value: ExprNode,
        branches: Vec<(PatternNode, MatchBranch)>,
    },
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
    Assign(PatternNode, ExprNode),
    AssignOp(PatternNode, AssignOp, ExprNode),

    If {
        branches: Vec<(ExprNode, Statements)>,
        else_branch: Option<Statements>,
    },
    While {
        cond: ExprNode,
        code: Statements,
    },
    For {
        iter: PatternNode,
        iterator: ExprNode,
        code: Statements,
    },
    TryCatch {
        try_code: Statements,
        catch_pat: Option<PatternNode>,
        catch_code: Statements,
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

// T = type, P = pattern, E = expression, S = string
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pattern<T, P, E, S: Hash + Eq> {
    Any, // _

    Type(T),      // @<type>
    Either(P, P), // <pattern> | <pattern>
    Both(P, P),   // <pattern> & <pattern>, <pattern>: <pattern>

    Eq(E),  // == <expr>
    Neq(E), // != <expr>
    Lt(E),  // < <expr>
    Lte(E), // <= <expr>
    Gt(E),  // > <expr>
    Gte(E), // >= <expr>

    In(E), // in <pattern>

    ArrayPattern(P, P), // <pattern>[...]
    DictPattern(P),     // <pattern>{:}

    ArrayDestructure(Vec<P>),                // [ <pattern> ]
    DictDestructure(AHashMap<S, Option<P>>), // { key: <pattern> }
    MaybeDestructure(Option<P>),             // <pattern>? or ?
    InstanceDestructure(T, AHashMap<S, Option<P>>),

    Path {
        var: S,
        path: Vec<AssignPath<E, S>>,
        is_ref: bool,
    },
    Mut {
        name: S,
        is_ref: bool,
    },

    MacroPattern {
        args: Vec<P>,
        ret_type: P,
    },
}

// T = type, E = expression, S = string
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignPath<E, S: Hash + Eq> {
    Index(E),
    Member(S),
    Associated(S),
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

pub trait VisTrait
where
    Self: Sized,
{
    type Value;

    fn is_priv(&self) -> bool;
    fn is_pub(&self) -> bool;
    fn value(&self) -> &Self::Value;

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

    fn source(&self) -> Option<&Rc<SpwnSource>> {
        match self {
            VisSource::Public(..) => None,
            VisSource::Private(.., s) => Some(s),
        }
    }
}

impl<T> VisSource<T> {
    pub fn map<F, O>(self, f: F) -> VisSource<O>
    where
        F: FnOnce(T) -> O,
    {
        match self {
            VisSource::Public(v) => VisSource::Public(f(v)),
            VisSource::Private(v, s) => VisSource::Private(f(v), s),
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
}

impl<T> Vis<T> {
    pub fn map<F, O>(self, f: F) -> Vis<O>
    where
        F: FnOnce(T) -> O,
    {
        match self {
            Vis::Public(v) => Vis::Public(f(v)),
            Vis::Private(v) => Vis::Private(f(v)),
        }
    }
}
