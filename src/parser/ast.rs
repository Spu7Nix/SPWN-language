use crate::parser::lexer::Token;
use crate::sources::CodeArea;
use lasso::Spur;
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use super::lexer::CodeSpan;

new_key_type! {
    pub struct ExprKey;
    pub struct StmtKey;
}

// just helper for ASTData::area
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
    pub exprs: SlotMap<ExprKey, (Expression, CodeSpan)>,
    pub stmts: SlotMap<StmtKey, (Statement, CodeSpan)>,

    pub stmt_arrows: SecondaryMap<StmtKey, bool>,

    pub for_loop_iter_areas: SecondaryMap<StmtKey, CodeSpan>,
    pub func_arg_areas: SecondaryMap<ExprKey, Vec<CodeSpan>>,

    pub dictlike_areas: SecondaryMap<ExprKey, Vec<CodeSpan>>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Int(u64),
    Byte(u8),
    Float(f64),
    String(String),
    Bool(bool),
    Op(ExprKey, Token, ExprKey),
    Unary(Token, ExprKey),
    Ident(Spur),

    Var(Spur),
    Type(Spur),

    Array(Vec<ExprKey>),
    Dict(Vec<(Spur, Option<ExprKey>)>),

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

#[derive(Debug, Clone)]
pub enum Statement {
    Expr(ExprKey),
    Let(String, ExprKey),
    Assign(String, ExprKey),
    If {
        branches: Vec<(ExprKey, Statements)>,
        else_branch: Option<Statements>,
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
    Impl(ExprKey, Vec<(String, ExprKey)>),
    Print(ExprKey),
}

pub type Statements = Vec<StmtKey>;
