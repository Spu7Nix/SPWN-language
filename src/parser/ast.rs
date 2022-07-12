use crate::{leveldata::object_data::ObjectMode, parser::lexer::Token};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::sources::{CodeSpan, SpwnSource};

new_key_type! {
    pub struct ExprKey;
    pub struct StmtKey;
}

pub enum KeyType {
    Expr(ExprKey),
    StmtKey(StmtKey),
}

// just helper for ASTData::area
pub trait ASTKey {
    fn to_key(&self) -> KeyType;
}
impl ASTKey for ExprKey {
    fn to_key(&self) -> KeyType {
        KeyType::Expr(*self)
    }
}
impl ASTKey for StmtKey {
    fn to_key(&self) -> KeyType {
        KeyType::StmtKey(*self)
    }
}

#[derive(Default)]
pub struct ASTData {
    pub source: SpwnSource,

    pub exprs: SlotMap<ExprKey, (Expression, CodeSpan)>,
    pub stmts: SlotMap<StmtKey, (Statement, CodeSpan)>,

    pub for_loop_iter_spans: SecondaryMap<StmtKey, CodeSpan>,
    pub func_arg_spans: SecondaryMap<ExprKey, Vec<CodeSpan>>,

    pub dictlike_spans: SecondaryMap<ExprKey, Vec<CodeSpan>>,
    pub objlike_spans: SecondaryMap<ExprKey, Vec<CodeSpan>>,
    pub impl_spans: SecondaryMap<StmtKey, Vec<CodeSpan>>,

    pub stmt_arrows: SecondaryMap<StmtKey, bool>,
}

impl ASTData {
    // pub fn insert<T: ASTNode + 'static>(&mut self, node: T, area: CodeArea) -> ASTKey {
    //     self.map.insert((Box::new(node), area))
    // }
    pub fn get_span<K: ASTKey>(&self, k: K) -> CodeSpan {
        match k.to_key() {
            KeyType::Expr(k) => self.exprs[k].1,
            KeyType::StmtKey(k) => self.stmts[k].1,
        }
    }
    pub fn get_expr(&self, k: ExprKey) -> Expression {
        self.exprs[k].0.clone()
    }
    pub fn get_stmt(&self, k: StmtKey) -> Statement {
        self.stmts[k].0.clone()
    }
    pub fn insert_expr(&mut self, expr: Expression, area: CodeSpan) -> ExprKey {
        self.exprs.insert((expr, area))
    }
    pub fn insert_stmt(&mut self, stmt: Statement, area: CodeSpan) -> StmtKey {
        self.stmts.insert((stmt, area))
    }

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

        let re = regex::Regex::new(r"(ExprKey\([^)]*\))").unwrap();
        debug_str = re
            .replace_all(
                &debug_str,
                ansi_term::Color::Yellow.bold().paint("$1").to_string(),
            )
            .into();
        let re = regex::Regex::new(r"(StmtKey\([^)]*\))").unwrap();
        debug_str = re
            .replace_all(
                &debug_str,
                ansi_term::Color::Blue.bold().paint("$1").to_string(),
            )
            .into();

        println!("{}", debug_str);
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
pub enum Expression {
    Int(u32),
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
    Dict(Vec<(String, Option<ExprKey>)>),

    Obj(ObjectMode, Vec<(ExprKey, ExprKey)>),

    // Index { base: ExprKey, index: ExprKey },
    Empty,

    Block(Statements),

    Func {
        args: Vec<(String, Option<ExprKey>, Option<ExprKey>)>,
        ret_type: Option<ExprKey>,
        code: ExprKey,
    },
    FuncPattern {
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
    Call {
        base: ExprKey,
        params: Vec<ExprKey>,
        named_params: Vec<(String, ExprKey)>,
    },
    TriggerFuncCall(ExprKey),

    Maybe(Option<ExprKey>),

    TriggerFunc(Statements),

    Instance(ExprKey, Vec<(String, Option<ExprKey>)>),

    Split(ExprKey, ExprKey),
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
