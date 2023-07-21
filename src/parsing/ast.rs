use std::cell::RefCell;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ahash::AHashMap;
use base64::Engine;
use delve::{EnumDisplay, EnumToStr};
use derive_more::Deref;
use itertools::Either;
use lasso::Spur;
use serde::{Deserialize, Serialize};

use super::operators::operators::{AssignOp, BinOp, Operator, UnaryOp};
use crate::gd::ids::IDClass;
use crate::gd::object_keys::ObjectKey;
use crate::interpreting::value::Value;
use crate::sources::{CodeSpan, Spannable, Spanned, SpwnSource};
use crate::util::{ImmutStr, Interner};

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
    pub flags: StringFlags,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Default)]
pub struct StringFlags {
    pub bytes: bool,
    pub base64: bool,
    pub unindent: bool,
}

impl StringContent {
    pub fn normal(s: Spur) -> Self {
        StringContent {
            s: StringType::Normal(s),
            flags: StringFlags::default(),
        }
    }

    pub fn get_compile_time(&self, interner: &Rc<RefCell<Interner>>) -> Option<String> {
        if self.flags.bytes {
            return None;
        }
        let mut s = match self.s {
            StringType::Normal(k) => interner.borrow().resolve(&k).to_string(),
            _ => return None,
        };
        if self.flags.unindent {
            s = unindent::unindent(&s)
        }
        if self.flags.base64 {
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
    pub attributes: Vec<Attribute>,
    pub span: CodeSpan,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct StmtNode {
    pub stmt: Box<Statement>,
    pub attributes: Vec<Attribute>,
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
    pub attributes: Vec<Attribute>,
    pub value: Option<ExprNode>,
}

impl From<DictItem> for &'static str {
    fn from(_: DictItem) -> Self {
        "Dict Item"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroArg<D, P> {
    Single { pattern: P, default: Option<D> },
    Spread { pattern: P },
}

impl<D, P> MacroArg<D, P> {
    pub fn pattern(&self) -> &P {
        match self {
            MacroArg::Single { pattern, .. } => pattern,
            MacroArg::Spread { pattern } => pattern,
        }
    }
}

// impl<N, D, P> MacroArg<N, D, P> {
//     pub fn name(&self) -> &N {
//         match self {
//             MacroArg::Single { name, .. } | MacroArg::Spread { name, .. } => name,
//         }
//     }

//     pub fn default(&self) -> &Option<D> {
//         match self {
//             MacroArg::Single { default, .. } => default,
//             _ => unreachable!(),
//         }
//     }

//     pub fn default_mut(&mut self) -> &mut Option<D> {
//         match self {
//             MacroArg::Single { default, .. } => default,
//             _ => unreachable!(),
//         }
//     }

//     pub fn pattern(&self) -> &Option<P> {
//         match self {
//             MacroArg::Single { pattern, .. } | MacroArg::Spread { pattern, .. } => pattern,
//         }
//     }

//     pub fn pattern_mut(&mut self) -> &mut Option<P> {
//         match self {
//             MacroArg::Single { pattern, .. } | MacroArg::Spread { pattern, .. } => pattern,
//         }
//     }
// }

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum MatchBranchCode {
    Expr(ExprNode),
    Block(Statements),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub pattern: PatternNode,
    pub code: MatchBranchCode,
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
        args: Vec<MacroArg<ExprNode, PatternNode>>,
        ret_pat: Option<PatternNode>,
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

    Dbg(ExprNode, bool),

    Instance {
        base: ExprNode,
        items: Vec<Vis<DictItem>>,
    },

    Obj(ObjectType, Vec<(Spanned<ObjKeyType>, ExprNode)>),

    Match {
        value: ExprNode,
        branches: Vec<MatchBranch>,
    },
}

#[derive(Debug, Clone, Copy, EnumToStr, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Object,
    Trigger,
}

#[derive(Debug, Clone, Copy, EnumToStr, PartialEq, Eq, Hash, EnumDisplay)]
pub enum ObjKeyType {
    #[delve(display = |o: &ObjectKey| <&ObjectKey as Into<&str>>::into(o).to_string())]
    Name(ObjectKey),
    #[delve(display = |n: &u8| format!("{n}"))]
    Num(u8),
}

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
        name: Spanned<Spur>,
        items: Vec<Vis<DictItem>>,
    },
    Overload {
        op: Operator,
        macros: Vec<ExprNode>,
    },

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

    ArrayPattern(P, P), // <pattern>[<pattern>]
    DictPattern(P),     // <pattern>{:}

    ArrayDestructure(Vec<P>),                       // [ <pattern> ]
    DictDestructure(AHashMap<S, Option<P>>),        // { key: <pattern> ETC }
    MaybeDestructure(Option<P>),                    // <pattern>? or ?
    InstanceDestructure(T, AHashMap<S, Option<P>>), // @typ::{ key: <pattern> ETC }

    Path {
        // var[0].cock::binky[79] etc
        var: S,
        path: Vec<AssignPath<E, S>>,
        is_ref: bool,
    },
    Mut {
        // mut var OR &mut var
        name: S,
        is_ref: bool,
    },

    IfGuard {
        // <pattern> if <expr>
        pat: P,
        cond: E,
    },

    MacroPattern(Option<P>),
}

impl<T, E> Pattern<T, PatternNode, E, Spur> {
    pub fn is_self(&self, interner: &Rc<RefCell<Interner>>) -> bool {
        match self {
            Pattern::Either(a, b) => a.pat.is_self(interner) || b.pat.is_self(interner),
            Pattern::Both(a, b) => a.pat.is_self(interner) || b.pat.is_self(interner),
            Pattern::ArrayPattern(a, b) => a.pat.is_self(interner) || b.pat.is_self(interner),
            Pattern::DictPattern(a) => a.pat.is_self(interner),
            Pattern::ArrayDestructure(v) => v.iter().any(|p| p.pat.is_self(interner)),
            Pattern::DictDestructure(map) | Pattern::InstanceDestructure(_, map) => map
                .iter()
                .any(|(_, p)| p.as_ref().is_some_and(|p| p.pat.is_self(interner))),
            Pattern::MaybeDestructure(v) => v.as_ref().is_some_and(|p| p.pat.is_self(interner)),
            Pattern::Path { var, .. } => interner.borrow().resolve(var) == "self",
            Pattern::Mut { name, .. } => interner.borrow().resolve(name) == "self",
            Pattern::IfGuard { pat, .. } => pat.pat.is_self(interner),
            Pattern::MacroPattern { .. } => todo!(),
            _ => false,
        }
    }

    pub fn get_name(&self) -> Option<Spur> {
        match self {
            Pattern::Mut { name, .. } => Some(*name),
            Pattern::Path { var, path, .. } if path.is_empty() => Some(*var),
            Pattern::Both(a, ..) => a.pat.get_name(),
            _ => None,
        }
    }
}

// T = type, E = expression, S = string
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignPath<E, S: Hash + Eq> {
    Index(E),
    Member(S),
    Associated(S),
}

impl Expression {
    pub fn into_node(self, attributes: Vec<Attribute>, span: CodeSpan) -> ExprNode {
        ExprNode {
            expr: Box::new(self),
            attributes,
            span,
        }
    }
}
impl Statement {
    pub fn into_node(self, attributes: Vec<Attribute>, span: CodeSpan) -> StmtNode {
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
    pub file_attributes: Vec<Attribute>,
}

pub trait VisTrait
where
    Self: Sized,
{
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

    pub fn as_ref(&self) -> VisSource<&T> {
        match *self {
            VisSource::Public(ref v) => VisSource::Public(v),
            VisSource::Private(ref v, ref s) => VisSource::Private(v, Rc::clone(s)),
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

    pub const fn as_ref(&self) -> Vis<&T> {
        match *self {
            Vis::Public(ref v) => Vis::Public(v),
            Vis::Private(ref v) => Vis::Private(v),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumDisplay)]
pub enum AttrStyle {
    /// `#[...]`
    #[delve(display = "outer")]
    Outer,
    /// `#![...]`
    #[delve(display = "inner")]
    Inner,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum AttrArgs {
    Empty,

    Delimited(Vec<DelimArg>),

    Eq(ExprNode),
}

impl AttrArgs {
    pub fn delimited_span(&self) -> CodeSpan {
        match self {
            AttrArgs::Delimited(args) => {
                let first_span = args.first().unwrap().expr.span;
                args.last()
                    .map(|last| first_span.extend(last.expr.span))
                    .unwrap_or(first_span)
            },
            _ => unreachable!("BUG: called `delimited_span` on non-delimited args"),
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct DelimArg {
    pub name: Spanned<Spur>,
    pub expr: ExprNode,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct AttrItem {
    pub namespace: Option<Spanned<Spur>>,
    pub name: Spanned<Spur>,
    pub args: AttrArgs,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct Attribute {
    pub style: AttrStyle,
    pub item: AttrItem,
    pub span: CodeSpan,
}

impl Attribute {
    pub fn is_word(&self) -> bool {
        matches!(self.item.args, AttrArgs::Empty)
    }

    pub fn value_str(&self, interner: &Rc<RefCell<Interner>>) -> Option<String> {
        self.item.value_str(interner)
    }
}

impl AttrItem {
    fn value_str(&self, interner: &Rc<RefCell<Interner>>) -> Option<String> {
        match &self.args {
            AttrArgs::Eq(args) => match &*args.expr {
                Expression::String(s) => s.get_compile_time(interner),
                _ => None,
            },
            AttrArgs::Delimited(_) | AttrArgs::Empty => None,
        }
    }
}
