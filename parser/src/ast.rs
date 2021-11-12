//! Abstract Syntax Tree (AST) type definitions

use internment::LocalIntern;

use crate::fmt::SpwnFmt;
use shared::FileRange;
use shared::ImportType;
use shared::StoredValue;
#[derive(Clone, PartialEq, Debug)]
pub enum DictDef {
    Def((LocalIntern<String>, Expression)),
    Extract(Expression),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ArrayPrefix {
    Collect,
    Spread,
    // future-proofing
}

#[derive(Clone, PartialEq, Debug)]
pub struct ArrayDef {
    pub value: Expression,
    pub operator: Option<ArrayPrefix>,
}

//pub type Comment = (Option<String>, Option<String>);

#[derive(Clone, PartialEq, Debug)]
pub struct Statement {
    pub body: StatementBody,
    pub arrow: bool, /*context changing */
    pub pos: FileRange,
    //pub comment: Comment,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StatementBody {
    //Definition(Definition),
    Call(Call),
    Expr(Expression),
    Definition(Definition),

    TypeDef(String),

    Return(Option<Expression>),
    Impl(Implementation),
    If(If),
    For(For),
    While(While),
    Error(Error),
    Extract(Expression),

    Break,
    Continue,
    //EOI,
}

// TODO: implement this in parser and compiler

#[derive(Clone, PartialEq, Debug)]
pub struct Definition {
    pub symbol: Variable,
    pub value: Option<Expression>,
    pub mutable: bool,
}
#[derive(Clone, PartialEq, Debug)]
pub struct ValueLiteral {
    pub body: ValueBody,
    //pub comment: Comment,
}

impl ValueLiteral {
    pub fn new(body: ValueBody) -> Self {
        ValueLiteral {
            body,
            //comment: (None, None),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValueBody {
    Id(Id),
    Number(f64),
    CmpStmt(CompoundStatement),
    Dictionary(Vec<DictDef>),
    Symbol(LocalIntern<String>),
    Bool(bool),
    Expression(Expression),
    Str(StrInner),
    Import(ImportType, bool),
    Switch(Expression, Vec<Case>),
    Array(Vec<ArrayDef>),
    ListComp(Comprehension),
    Obj(ObjectLiteral),
    Macro(Macro),
    Resolved(StoredValue),
    TypeIndicator(String),
    SelfVal,
    Ternary(Ternary),
    Null,
}

impl ValueBody {
    pub fn to_variable(&self, pos: FileRange) -> Variable {
        Variable {
            value: ValueLiteral { body: self.clone() },
            operator: None,
            pos,
            //comment: (None, None),
            path: Vec::new(),
            tag: Attribute::new(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Copy, Hash)]
pub enum ObjectMode {
    Object,
    Trigger,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ObjectLiteral {
    pub props: Vec<(Expression, Expression)>,
    pub mode: ObjectMode,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StrInner {
    pub inner: String,
    pub flags: Option<StringFlags>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StringFlags {
    Raw,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Operator {
    Or,
    And,
    Equal,
    NotEqual,
    Range,
    Is,
    MoreOrEqual,
    LessOrEqual,
    More,
    Less,
    Slash,
    IntDividedBy,
    Star,
    Power,
    Plus,
    Minus,
    Modulo,

    Either,
    Both,

    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
    IntDivide,
    As,
    In,

    Exponate,
    Modulate,
    Swap,
}

#[derive(Clone, PartialEq, Debug)]
pub enum UnaryOperator {
    Not,
    Minus,
    Increment,
    Decrement,

    EqPattern,
    NotEqPattern,
    MorePattern,
    LessPattern,
    MoreOrEqPattern,
    LessOrEqPattern,
    InPattern,
}

#[derive(Clone, PartialEq, Debug)]
pub enum IdClass {
    Group,
    Color,
    Item,
    Block,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Attribute {
    pub tags: Vec<(String, Vec<Argument>)>,
}

impl Attribute {
    pub fn new() -> Self {
        Attribute { tags: Vec::new() }
    }
    pub fn get(&self, t: &str) -> Option<Vec<Argument>> {
        for (key, args) in &self.tags {
            if t == key {
                return Some(args.clone());
            }
        }
        None
    }

    pub fn get_desc(&self) -> Option<String> {
        match self.get("desc") {
            Some(args) => {
                if args.is_empty() {
                    None
                } else {
                    match &args[0].value.values[0].value.body {
                        ValueBody::Str(s) => Some(s.inner.clone()),
                        a => Some(a.fmt(0)),
                    }
                }
            }

            None => None,
        }
    }

    pub fn get_example(&self) -> Option<String> {
        if let Some(args) = self.get("example") {
            if args.is_empty() {
                None
            } else {
                match &args[0].value.values[0].value.body {
                    ValueBody::Str(s) => Some(s.inner.trim().to_string()),
                    val => Some(val.fmt(0)),
                }
            }
        } else {
            None
        }
    }
}

impl Default for Attribute {
    fn default() -> Self {
        Self::new()
    }
}

pub trait CountSymbols {
    fn symbols(&self) -> fnv::FnvHashSet<LocalIntern<String>>;

    fn properties(&self) -> fnv::FnvHashSet<LocalIntern<String>> {
        Default::default()
    }

    fn all(&self) -> fnv::FnvHashSet<LocalIntern<String>> {
        let mut out = self.symbols();
        out.extend(self.properties());
        out
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Path {
    Member(LocalIntern<String>),
    Associated(LocalIntern<String>),
    Index(Expression),
    NSlice(Vec<Slice>),
    Call(Vec<Argument>),
    Constructor(Vec<DictDef>),
    Increment,
    Decrement,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Argument {
    pub symbol: Option<LocalIntern<String>>,
    pub value: Expression,
    pub pos: FileRange,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Slice {
    pub left: Option<Expression>,
    pub right: Option<Expression>,
    pub step: Option<Expression>,
}

impl Argument {
    pub fn from(val: StoredValue, pos: FileRange) -> Self {
        Argument {
            symbol: None,
            value: Expression {
                values: vec![Variable {
                    value: ValueLiteral::new(ValueBody::Resolved(val)),
                    path: Vec::new(),
                    operator: None,
                    pos,
                    //comment: (None, None),
                    tag: Attribute::new(),
                }],
                operators: Vec::new(),
            },
            pos,
        }
    }
}

/*#[derive(Clone, PartialEq, Debug)]
pub struct Event {
    pub symbol: String,
    pub args: Vec<Expression>,
    pub func: Variable,
}*/

#[derive(Clone, PartialEq, Debug)]
pub struct Call {
    pub function: Variable,
}

/*#[derive(Clone, PartialEq, Debug)]
pub struct Native {
    pub function: Variable,
    pub args: Vec<Argument>,
}*/
//     name     def value     props     type ind.     location in file     is reference
pub type ArgDef = (
    LocalIntern<String>,
    Option<Expression>,
    Attribute,
    Option<Expression>,
    FileRange,
    bool,
);
#[derive(Clone, PartialEq, Debug)]
pub struct Macro {
    pub args: Vec<ArgDef>,
    pub body: CompoundStatement,
    pub properties: Attribute,
    pub arg_pos: FileRange,
    pub ret_type: Option<Expression>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct For {
    pub symbol: Expression,
    pub array: Expression,
    pub body: Vec<Statement>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct While {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum CaseType {
    //Value(Expression),
    Pattern(Expression),
    Default,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Case {
    pub typ: CaseType,
    pub body: Expression,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Switch {
    pub value: Expression,
    pub cases: Vec<Case>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Error {
    pub message: Expression,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Variable {
    pub operator: Option<UnaryOperator>,
    pub value: ValueLiteral,
    pub path: Vec<Path>,
    pub pos: FileRange,
    pub tag: Attribute,
}

impl Variable {
    pub fn to_expression(&self) -> Expression {
        if let ValueBody::Expression(e) = &self.value.body {
            if self.path.is_empty() {
                return e.to_owned();
            }
        }
        Expression {
            values: vec![self.clone()],
            operators: Vec::new(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Expression {
    pub values: Vec<Variable>,
    pub operators: Vec<Operator>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Ternary {
    pub condition: Expression,
    pub if_expr: Expression,
    pub else_expr: Expression,
    pub is_pattern: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Comprehension {
    pub symbol: LocalIntern<String>,
    pub iterator: Expression,
    pub condition: Option<Expression>,
    pub body: Expression,
}

impl Expression {
    pub fn to_variable(&self) -> Variable {
        Variable {
            operator: None,
            value: ValueLiteral::new(ValueBody::Expression(self.clone())),
            pos: self.get_pos(),
            path: Vec::new(),
            //comment: (None, None),
            tag: Attribute::new(),
        }
    }

    pub fn get_pos(&self) -> FileRange {
        let start = self.values.first().unwrap().pos.0;
        let end = self.values.last().unwrap().pos.1;
        (start, end)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct CompoundStatement {
    pub statements: Vec<Statement>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Implementation {
    pub symbol: Variable,
    pub members: Vec<DictDef>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct If {
    pub condition: Expression,
    pub if_body: Vec<Statement>,
    pub else_body: Option<Vec<Statement>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Id {
    pub number: u16,
    pub unspecified: bool,
    pub class_name: IdClass,
}
