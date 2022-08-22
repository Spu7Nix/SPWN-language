use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::sources::{CodeSpan, SpwnSource};
use crate::{leveldata::object_data::ObjectMode, parsing::lexer::Token};
use crate::regex_color_replace;

new_key_type! {
    pub struct ExprKey;
    pub struct StmtKey;
}

pub type ExpressionMap = SlotMap<ExprKey, (Expression, CodeSpan)>;

pub struct ASTData {
    pub source: SpwnSource,

    pub exprs: ExpressionMap,
    pub stmts: SlotMap<StmtKey, (Statement, CodeSpan)>,

    pub for_loop_iter_spans: SecondaryMap<StmtKey, CodeSpan>,
    pub func_arg_spans: SecondaryMap<ExprKey, Vec<CodeSpan>>,

    pub dictlike_spans: SecondaryMap<ExprKey, Vec<CodeSpan>>,
    pub objlike_spans: SecondaryMap<ExprKey, Vec<CodeSpan>>,
    pub impl_spans: SecondaryMap<StmtKey, Vec<CodeSpan>>,

    pub stmt_arrows: SecondaryMap<StmtKey, bool>,
}

impl ASTData {
    pub fn new(source: SpwnSource) -> Self {
        Self {
            source,
            exprs: SlotMap::default(),
            stmts: SlotMap::default(),

            for_loop_iter_spans: SecondaryMap::default(),
            func_arg_spans: SecondaryMap::default(),

            dictlike_spans: SecondaryMap::default(),
            objlike_spans: SecondaryMap::default(),
            impl_spans: SecondaryMap::default(),

            stmt_arrows: SecondaryMap::default(),
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug(&self, stmts: &Statements) {
        let mut debug_str = String::new();
        use std::fmt::Write;

        debug_str += "-------- exprs --------\n";
        for (k, (e, _)) in &self.exprs {
            writeln!(&mut debug_str, "{:?}:\t\t{:?}", k, e).unwrap();
        }
        debug_str += "-------- stmts --------\n";
        for (k, (e, _)) in &self.stmts {
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

pub trait ASTInsert<T, K> {
    fn insert(&mut self, v: T, area: CodeSpan) -> K;
    fn get_full(&mut self, v: K) -> (T, CodeSpan);

    fn get(&mut self, v: K) -> T {
        self.get_full(v).0
    }
    fn span(&mut self, v: K) -> CodeSpan {
        self.get_full(v).1
    }
}

impl ASTInsert<Expression, ExprKey> for ASTData {
    fn insert(&mut self, v: Expression, area: CodeSpan) -> ExprKey {
        self.exprs.insert((v, area))
    }

    fn get_full(&mut self, v: ExprKey) -> (Expression, CodeSpan) {
        self.exprs[v].clone()
    }
}
impl ASTInsert<Statement, StmtKey> for ASTData {
    fn insert(&mut self, v: Statement, area: CodeSpan) -> StmtKey {
        self.stmts.insert((v, area))
    }

    fn get_full(&mut self, v: StmtKey) -> (Statement, CodeSpan) {
        self.stmts[v].clone()
    }
}

use serde::{Deserialize, Serialize};
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
    //........key.....value
    Dict(Vec<(String, Option<ExprKey>)>),
    //..mode.............obj key..value
    Obj(ObjectMode, Vec<(ExprKey, ExprKey)>),

    Empty,

    Macro {
        ///........name....type.............default value
        args: Vec<(String, Option<ExprKey>, Option<ExprKey>)>,
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

    TypeOf {
        base: ExprKey,
    },

    Call {
        base: ExprKey,
        params: Vec<ExprKey>,
        //.................name....value
        named_params: Vec<(String, ExprKey)>,
    },
    TriggerFuncCall(ExprKey),

    Maybe(Option<ExprKey>),

    TriggerFunc(Statements),

    //.......type name.....key.....value
    Instance(ExprKey, Vec<(String, Option<ExprKey>)>),

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
    Impl(ExprKey, Vec<(String, Option<ExprKey>)>),
    Print(ExprKey),
    Add(ExprKey),
}

pub type Statements = Vec<StmtKey>;
