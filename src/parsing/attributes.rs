use std::sync::Arc;

use once_cell::sync::Lazy;

use super::ast::{AttrStyle, Expression, Statement};
use crate::util::ImmutStr;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(unused)]
pub enum AttributeTarget {
    Int,
    String,
    Float,
    Bool,
    Id,
    Var,
    Type,
    Array,
    Dict,
    Maybe,
    Macro,
    TriggerFunc,
    TriggerFuncCall,
    Import,
    Match,
    Obj,

    Assign,
    If,
    While,
    For,
    TryCatch,
    Arrow,
    Return,
    Break,
    Continue,
    TypeDef,
    ExtractImport,
    Impl,
    Overload,
    Throw,

    DictItem,

    Unknown,
}

impl Expression {
    pub fn target(&self) -> AttributeTarget {
        use AttributeTarget as AT;
        match self {
            Expression::Int(..) => AT::Int,
            Expression::Float(..) => AT::Float,
            Expression::String(..) => AT::String,
            Expression::Bool(..) => AT::Bool,
            Expression::Id(..) => AT::Id,
            Expression::Var(..) => AT::Var,
            Expression::Type(..) => AT::Type,
            Expression::Array(..) => AT::Array,
            Expression::Dict(..) => AT::Dict,
            Expression::Maybe(..) => AT::Maybe,
            Expression::Macro { .. } => AT::Macro,
            Expression::TriggerFunc { .. } => AT::TriggerFunc,
            Expression::TriggerFuncCall(..) => AT::TriggerFuncCall,
            Expression::Import(..) => AT::Import,
            Expression::Match { .. } => AT::Match,
            _ => AT::Unknown,
        }
    }
}

impl Statement {
    pub fn target(&self) -> AttributeTarget {
        use AttributeTarget as AT;
        match self {
            Statement::Expr(e) => e.expr.target(),
            Statement::Assign(..) => AT::Assign,
            Statement::If { .. } => AT::If,
            Statement::While { .. } => AT::While,
            Statement::For { .. } => AT::For,
            Statement::TryCatch { .. } => AT::TryCatch,
            Statement::Arrow(..) => AT::Arrow,
            Statement::Return(..) => AT::Return,
            Statement::Break => AT::Break,
            Statement::Continue => AT::Continue,
            Statement::TypeDef(..) => AT::TypeDef,
            Statement::ExtractImport(..) => AT::ExtractImport,
            Statement::Impl { .. } => AT::Impl,
            Statement::Overload { .. } => AT::Overload,
            Statement::Throw(..) => AT::Throw,
            _ => AT::Unknown,
        }
    }
}

#[derive(Clone, Copy, Default)]
#[allow(unused)]
pub enum AttributeDuplicates {
    #[default]
    DuplicatesOk,
    WarnFollowing,
    ErrorFollowing,
}

#[allow(unused)]
pub enum ListArg {
    Optional(&'static str),
    Required(&'static str),
}

pub struct AttributeTemplate {
    pub word: bool,
    pub list: Option<&'static [ListArg]>,
    pub name_value: bool,
}

impl AttributeTemplate {
    const WORD: AttributeTemplate = AttributeTemplate {
        word: true,
        list: None,
        name_value: false,
    };
}

pub struct Attribute {
    pub namespace: Option<&'static str>,
    pub name: &'static str,
    pub template: AttributeTemplate,
    pub duplicates: AttributeDuplicates,
    pub style: &'static [AttrStyle],
    pub targets: &'static [AttributeTarget],
}

pub(crate) mod attr_names {
    pub const DOC: &str = "doc";
    pub const DEBUG_BYTECODE: &str = "debug_bytecode";
    pub const BUILTIN: &str = "builtin";
    pub const ALIAS: &str = "alias";
    pub const NO_STD: &str = "no_std";
}

pub static ATTRIBUTES: Lazy<Arc<Vec<Attribute>>> = Lazy::new(|| {
    Arc::new(vec![
        // ================================================================== file attributes
        Attribute {
            namespace: None,
            name: attr_names::NO_STD,
            template: AttributeTemplate::WORD,
            duplicates: AttributeDuplicates::ErrorFollowing,
            style: &[AttrStyle::Inner],
            targets: &[],
        },
        // ================================================================== attributes
        Attribute {
            namespace: None,
            name: attr_names::DEBUG_BYTECODE,
            template: AttributeTemplate::WORD,
            duplicates: AttributeDuplicates::ErrorFollowing,
            style: &[AttrStyle::Outer],
            targets: &[AttributeTarget::Macro],
        },
        Attribute {
            namespace: None,
            name: attr_names::BUILTIN,
            template: AttributeTemplate::WORD,
            duplicates: AttributeDuplicates::ErrorFollowing,
            style: &[AttrStyle::Outer],
            targets: &[AttributeTarget::Macro],
        },
        // Attribute {
        //     namespace: None,
        //     name: "deprecated",
        //     template: AttributeTemplate {
        //         word: true,
        //         list: Some(&[ListArg::Required("reason"), ListArg::Optional("since")]),
        //         name_value: false,
        //     },
        //     duplicates: AttributeDuplicates::ErrorFollowing,
        // },
        Attribute {
            namespace: None,
            name: attr_names::DOC,
            template: AttributeTemplate {
                word: true,
                list: None,
                name_value: true,
            },
            duplicates: AttributeDuplicates::DuplicatesOk,
            style: &[AttrStyle::Inner, AttrStyle::Outer],
            targets: &[
                AttributeTarget::Assign,
                AttributeTarget::TypeDef,
                AttributeTarget::DictItem,
            ],
        },
        Attribute {
            namespace: None,
            name: attr_names::ALIAS,
            template: AttributeTemplate {
                word: false,
                list: None,
                name_value: true,
            },
            duplicates: AttributeDuplicates::ErrorFollowing,
            style: &[AttrStyle::Outer],
            targets: &[AttributeTarget::DictItem],
        },
    ])
});

pub fn namespace_exists(namespace: &ImmutStr) -> bool {
    ATTRIBUTES.iter().any(|a| a.namespace == Some(namespace))
}

pub fn get_attr_by_name_in_namespace<'a>(
    namespace: &Option<ImmutStr>,
    name: &ImmutStr,
) -> Option<&'a Attribute> {
    ATTRIBUTES
        .iter()
        .find(|&attr| attr.name == &**name && attr.namespace == namespace.as_deref())
}
