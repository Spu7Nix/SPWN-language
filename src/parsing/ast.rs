use strum::{EnumProperty, EnumString, EnumVariantNames};

use crate::{lexing::tokens::Token, sources::CodeSpan};

use super::attributes::{ExprAttribute, ScriptAttribute};

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

// #[derive(Debug, Clone, PartialEq, Eq, Copy)]
// struct AttributeArgs<const LEN: usize, K, V>
// where
//     K: Default + PartialEq,
//     V: Default + PartialEq,
// {
//     args: [(K, V); LEN],
// }

// impl<const LEN: usize, K, V> Default for AttributeArgs<LEN, K, V>
// where
//     K: Default + PartialEq,
//     V: Default + PartialEq,
// {
//     fn default() -> Self {
//         Self {
//             args: [(K::default(), V::default()); LEN],
//         }
//     }
// }

// impl<const LEN: usize, K, V> std::ops::Index<K> for AttributeArgs<LEN, K, V>
// where
//     K: Default + PartialEq,
//     V: Default + PartialEq,
// {
//     type Output = Option<V>;

//     fn index(&self, index: K) -> &Self::Output {
//         if let Some(a) = self.args.iter().find(|a| a.0 == index) {
//             return &Some(a.1);
//         }
//         &None
//     }
// }

// impl<const LEN: usize, K, V> std::ops::IndexMut<K> for AttributeArgs<LEN, K, V>
// where
//     K: Default + PartialEq,
//     V: Default + PartialEq,
// {
//     fn index_mut(&mut self, index: K) -> &mut Self::Output {
//         if let Some(a) = self.args.iter().find(|a| a.0 == index) {
//             return &mut Some(a.1);
//         }
//         &mut None
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq, Copy, EnumString, EnumVariantNames, EnumProperty)]
// #[strum(serialize_all = "snake_case")]
// pub enum ScriptAttribute {
//     CacheOutput,
//     NoStd,
//     ConsoleOutput,
//     NoLevel,
//     NoBytecodeCache,
// }

// #[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumVariantNames, EnumProperty)]
// #[strum(serialize_all = "snake_case")]
// pub enum ExprAttribute {
//     NoOptimize,

//     #[strum(props(args = "2", arg0 = "since", arg1 = "note"))]
//     Deprecated {
//         //args: AttributeArgs<2, String, String>,
//     },
// }

#[derive(Debug, Clone)]
pub enum MacroCode {
    Normal(Statements),
    Lambda(ExprNode),
}

pub type ExprNode = Spanned<Expression>;
pub type StmtNode = Spanned<Statement>;

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

pub struct Ast {
    pub statements: Vec<StmtNode>,
    pub file_attributes: Vec<ScriptAttribute>,
}

#[derive(Clone, Debug)]
pub struct Spanned<T> {
    pub value: Box<T>,
    pub span: CodeSpan,
}
impl<T> Spanned<T> {
    pub fn split(self) -> (T, CodeSpan) {
        (*self.value, self.span)
    }
    pub fn extended(self, other: CodeSpan) -> Self {
        Self {
            span: self.span.extend(other),
            ..self
        }
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
        Spanned {
            value: Box::new(self),
            span,
        }
    }
}
