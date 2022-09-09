use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use super::error::SyntaxError;
use crate::regex_color_replace;
use crate::sources::{CodeSpan, SpwnSource};
use crate::{leveldata::object_data::ObjectMode, parsing::lexer::Token};

new_key_type! {
    pub struct ExprKey;
    pub struct StmtKey;
}

pub type ExpressionMap = SlotMap<ExprKey, Spanned<Expression>>;

pub struct ASTData {
    pub source: SpwnSource,

    pub exprs: ExpressionMap,
    pub stmts: SlotMap<StmtKey, Spanned<Statement>>,

    pub stmt_arrows: SecondaryMap<StmtKey, bool>,
}

impl ASTData {
    pub fn new(source: SpwnSource) -> Self {
        Self {
            source,
            exprs: SlotMap::default(),
            stmts: SlotMap::default(),
            // for_loop_iter_spans: SecondaryMap::default(),
            // func_arg_spans: SecondaryMap::default(),

            // dictlike_spans: SecondaryMap::default(),
            // objlike_spans: SecondaryMap::default(),
            // impl_spans: SecondaryMap::default(),
            stmt_arrows: SecondaryMap::default(),
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug(&self, stmts: &Statements) {
        let mut debug_str = String::new();
        use std::fmt::Write;

        debug_str += "-------- exprs --------\n";
        for (k, e) in &self.exprs {
            writeln!(&mut debug_str, "{:?}:\t\t{:?}", k, e).unwrap();
        }
        debug_str += "-------- stmts --------\n";
        for (k, e) in &self.stmts {
            writeln!(&mut debug_str, "{:?}:\t\t{:?}", k, e).unwrap();
        }
        debug_str += "-----------------------\n";

        for i in stmts {
            writeln!(&mut debug_str, "{:?}", i).unwrap();
        }

        regex_color_replace!(
            debug_str,
            r"(ExprKey\([^)]*\))", "$1", Yellow
            r"(StmtKey\([^)]*\))", "$1", Blue
        );

        println!("{}", debug_str);
    }
}

pub trait ASTInsert<T: Debug, K> {
    fn insert(&mut self, v: T, area: CodeSpan) -> K;
    fn get(&mut self, v: K) -> Spanned<T>;

    fn span(&mut self, v: K) -> CodeSpan {
        self.get(v).span
    }
}

impl ASTInsert<Expression, ExprKey> for ASTData {
    fn insert(&mut self, v: Expression, area: CodeSpan) -> ExprKey {
        self.exprs.insert(v.span(area))
    }

    fn get(&mut self, v: ExprKey) -> Spanned<Expression> {
        self.exprs[v].clone()
    }
}
impl ASTInsert<Statement, StmtKey> for ASTData {
    fn insert(&mut self, v: Statement, area: CodeSpan) -> StmtKey {
        self.stmts.insert(v.span(area))
    }

    fn get(&mut self, v: StmtKey) -> Spanned<Statement> {
        self.stmts[v].clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Copy)]
pub enum IdClass {
    Group = 0,
    Color = 1,
    Block = 2,
    Item = 3,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroCode {
    Normal(Statements),
    Lambda(ExprKey),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Id {
        class: IdClass,
        value: Option<u16>, // None = ? (arbirtary)
    },

    Op(ExprKey, Token, ExprKey),
    Unary(Token, ExprKey),

    Var(String),
    Type(String),

    Array(Vec<ExprKey>),
    //................key.....value
    Dict(Vec<Spanned<(String, Option<ExprKey>)>>),
    //..mode.....................obj key..value
    Obj(ObjectMode, Vec<Spanned<(ExprKey, ExprKey)>>),

    Empty,

    Macro {
        ///................name....type.............default value
        args: Vec<Spanned<(String, Option<ExprKey>, Option<ExprKey>)>>,
        ret_type: Option<ExprKey>,
        code: MacroCode,
    },
    MacroPattern {
        args: Vec<ExprKey>,
        ret_type: ExprKey,
    },

    Ternary {
        cond: ExprKey,
        if_true: ExprKey,
        if_false: ExprKey,
    },

    Index {
        base: ExprKey,
        index: ExprKey,
    },

    Member {
        base: ExprKey,
        name: String,
    },

    Associated {
        base: ExprKey,
        name: String,
    },

    TypeOf {
        base: ExprKey,
    },

    Call {
        base: ExprKey,
        params: Vec<ExprKey>,
        //.........................name....value
        named_params: Vec<Spanned<(String, ExprKey)>>,
    },
    TriggerFuncCall(ExprKey),

    Maybe(Option<ExprKey>),

    TriggerFunc(Statements),

    //.......name.....fields
    Instance(ExprKey, ExprKey),

    Split(ExprKey, ExprKey),
    Builtins,
    Import(ImportType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportType {
    Module(String),
    Library(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expr(ExprKey),
    Let(String, ExprKey),
    Assign(String, ExprKey),
    If {
        branches: Vec<(ExprKey, Statements)>,
        else_branch: Option<Statements>,
    },
    TryCatch {
        try_branch: Statements,
        catch: Statements,
        catch_var: String,
    },
    While {
        cond: ExprKey,
        code: Statements,
    },
    For {
        var: String,
        iterator: ExprKey,
        code: Statements,
    },
    Return(Option<ExprKey>),
    Break,
    Continue,

    TypeDef(String),
    Impl(ExprKey, ExprKey),
    Print(ExprKey),
    Add(ExprKey),
    //Arrow(Statement),
}

pub type Statements = Vec<StmtKey>;

#[derive(Clone)]
pub struct Spanned<T: Debug> {
    pub t: T,
    pub span: CodeSpan,
}

impl<T: Debug + PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t
    }
}

impl<T: Debug> Debug for Spanned<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.t.fmt(f)
    }
}

impl<T: Debug> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.t
    }
}

pub trait ToSpanned<T: Debug> {
    fn span(self, span: CodeSpan) -> Spanned<T>;
}

// cant blanket implement as it overrides the `span` method of `Token` so the parser breaks
// probably would be better with a macro but eh
impl ToSpanned<Expression> for Expression {
    fn span(self, span: CodeSpan) -> Spanned<Expression> {
        Spanned { t: self, span }
    }
}

impl ToSpanned<Statement> for Statement {
    fn span(self, span: CodeSpan) -> Spanned<Statement> {
        Spanned { t: self, span }
    }
}

impl ToSpanned<(String, Option<ExprKey>)> for (String, Option<ExprKey>) {
    fn span(self, span: CodeSpan) -> Spanned<(String, Option<ExprKey>)> {
        Spanned { t: self, span }
    }
}

impl ToSpanned<(String, ExprKey)> for (String, ExprKey) {
    fn span(self, span: CodeSpan) -> Spanned<(String, ExprKey)> {
        Spanned { t: self, span }
    }
}

impl ToSpanned<(ExprKey, ExprKey)> for (ExprKey, ExprKey) {
    fn span(self, span: CodeSpan) -> Spanned<(ExprKey, ExprKey)> {
        Spanned { t: self, span }
    }
}

impl ToSpanned<(String, Option<ExprKey>, Option<ExprKey>)>
    for (String, Option<ExprKey>, Option<ExprKey>)
{
    fn span(self, span: CodeSpan) -> Spanned<(String, Option<ExprKey>, Option<ExprKey>)> {
        Spanned { t: self, span }
    }
}

impl Spanned<Expression> {
    pub fn insert(self, data: &mut ASTData) -> Result<ExprKey, SyntaxError> {
        Ok(data.exprs.insert(self))
    }
}

impl Spanned<Statement> {
    pub fn insert(self, data: &mut ASTData) -> Result<StmtKey, SyntaxError> {
        Ok(data.stmts.insert(self))
    }
}
