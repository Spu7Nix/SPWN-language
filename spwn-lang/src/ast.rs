//! Abstract Syntax Tree (AST) type definitions

use crate::compiler_types::ImportType;
use crate::fmt::SpwnFmt;
use crate::parser::FileRange;
use crate::value_storage::StoredValue;
#[derive(Clone, PartialEq, Debug)]
pub enum DictDef {
    Def((String, Expression)),
    Extract(Expression),
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

    TypeDef(String),

    Return(Option<Expression>),
    Impl(Implementation),
    If(If),
    For(For),
    Error(Error),
    Extract(Expression),

    Break,
    Continue,
    //EOI,
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
    Symbol(String),
    Bool(bool),
    Expression(Expression),
    Str(String),
    Import(ImportType, bool),
    Switch(Expression, Vec<Case>),
    Array(Vec<Expression>),
    Obj(ObjectLiteral),
    Macro(Macro),
    Resolved(StoredValue),
    TypeIndicator(String),
    SelfVal,
    Ternary(Ternary),
    Null,
}

impl ValueBody {
    pub fn to_variable(&self) -> Variable {
        Variable {
            value: ValueLiteral { body: self.clone() },
            operator: None,
            pos: (0, 0),
            //comment: (None, None),
            path: Vec::new(),
            tag: Attribute::new(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum ObjectMode {
    Object,
    Trigger,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ObjectLiteral {
    pub props: Vec<(Expression, Expression)>,
    pub mode: ObjectMode,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Operator {
    Or,
    And,
    Equal,
    NotEqual,
    Range,
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

    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
    IntDivide,
    As,
    Has,

    Exponate,
    Modulate,
    Swap,
}

#[derive(Clone, PartialEq, Debug)]
pub enum UnaryOperator {
    Not,
    Minus,
    Range,
    Let,
    Increment,
    Decrement,
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
                        ValueBody::Str(s) => Some(s.clone()),
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
                    ValueBody::Str(s) => Some(s.trim().to_string()),
                    val => Some(val.fmt(0)),
                }
            }
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Path {
    Member(String),
    Associated(String),
    Index(Expression),
    Call(Vec<Argument>),
    Constructor(Vec<DictDef>),
    Increment,
    Decrement,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Definition {
    pub symbol: String,
    pub value: Expression,
    //pub mutable: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Argument {
    pub symbol: Option<String>,
    pub value: Expression,
}

impl Argument {
    pub fn from(val: StoredValue) -> Self {
        Argument {
            symbol: None,
            value: Expression {
                values: vec![Variable {
                    value: ValueLiteral::new(ValueBody::Resolved(val)),
                    path: Vec::new(),
                    operator: None,
                    pos: (0, 0),
                    //comment: (None, None),
                    tag: Attribute::new(),
                }],
                operators: Vec::new(),
            },
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
//                 name         def value     props       type ind.
pub type ArgDef = (
    String,
    Option<Expression>,
    Attribute,
    Option<Expression>,
    FileRange,
);
#[derive(Clone, PartialEq, Debug)]
pub struct Macro {
    pub args: Vec<ArgDef>,
    pub body: CompoundStatement,
    pub properties: Attribute,
}

#[derive(Clone, PartialEq, Debug)]
pub struct For {
    pub symbol: String,
    pub array: Expression,
    pub body: Vec<Statement>,
}
#[derive(Clone, PartialEq, Debug)]
pub enum CaseType {
    Value(Expression),
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
    //pub comment: Comment,
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
}

impl Expression {
    pub fn to_variable(&self) -> Variable {
        Variable {
            operator: None,
            value: ValueLiteral::new(ValueBody::Expression(self.clone())),
            pos: (0, 0),
            path: Vec::new(),
            //comment: (None, None),
            tag: Attribute::new(),
        }
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
