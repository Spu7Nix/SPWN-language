//! Abstract Syntax Tree (AST) type definitions

use std::path::PathBuf;

use std::fmt;

use crate::compiler_types::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum DictDef {
    Def((String, Expression)),
    Extract(Expression),
}

impl fmt::Display for DictDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DictDef::Def((name, expr)) => write!(f, "{}: {}", name, expr),
            DictDef::Extract(expr) => write!(f, "..{}", expr),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Statement {
    pub body: StatementBody,
    pub arrow: bool, //context changing
    pub line: (usize, usize),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.arrow {
            write!(f, "->{};\n", self.body)
        } else {
            write!(f, "{};\n", self.body)
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum StatementBody {
    Definition(Definition),
    Call(Call),
    Expr(Expression),

    TypeDef(String),

    Return(Option<Expression>),
    Impl(Implementation),
    If(If),
    For(For),
    Error(Error),
    Extract(Expression),
    //EOI,
}

impl fmt::Display for StatementBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatementBody::Definition(def) => write!(f, "{}", def),
            StatementBody::Call(call) => write!(f, "{}", call),
            StatementBody::Expr(x) => write!(f, "{}", x),
            StatementBody::TypeDef(x) => write!(f, "type {}", x),
            StatementBody::Return(x) => match x {
                Some(expr) => write!(f, "return {}", expr),
                None => write!(f, "return"),
            },
            StatementBody::Impl(x) => write!(f, "{}", x),
            StatementBody::If(x) => write!(f, "{}", x),
            StatementBody::For(x) => write!(f, "{}", x),
            StatementBody::Error(x) => write!(f, "{}", x),
            StatementBody::Extract(x) => write!(f, "extract {}", x),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValueLiteral {
    ID(ID),
    Number(f64),
    CmpStmt(CompoundStatement),
    Dictionary(Vec<DictDef>),
    Symbol(String),
    Bool(bool),
    Expression(Expression),
    Str(String),
    Import(PathBuf),
    Array(Vec<Expression>),
    Obj(Vec<(Expression, Expression)>),
    Macro(Macro),
    Resolved(Value),
    TypeIndicator(String),
    Null,
}

impl fmt::Display for ValueLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ValueLiteral::*;
        match self {
            ID(x) => write!(f, "{}", x),
            Number(x) => write!(f, "{}", x),
            CmpStmt(x) => write!(f, "{}", x),
            Dictionary(x) => {
                if x.is_empty() {
                    return write!(f, "{{}}");
                }
                let mut out = String::from("{\n");

                let mut d_iter = x.iter();
                for def in &mut d_iter {
                    out += &format!("{},\n", def);
                }
                out.pop();
                out.pop();

                out += "\n}"; //why do i have to do this twice? idk

                write!(f, "{}", out)
            }
            Array(x) => {
                if x.is_empty() {
                    return write!(f, "[]");
                }
                let mut out = String::from("[");

                let mut d_iter = x.iter();
                for def in &mut d_iter {
                    out += &format!("{},", def);
                }
                out.pop();

                out += "]";

                write!(f, "{}", out)
            }
            Symbol(x) => write!(f, "{}", x),
            Bool(x) => write!(f, "{}", x),
            Expression(x) => write!(f, "({})", x),
            Str(x) => write!(f, "\"{}\"", x),
            Import(x) => write!(f, "import {:?}", x),
            Obj(x) => {
                if x.is_empty() {
                    return write!(f, "{{}}");
                }
                let mut out = String::from("{\n");

                let mut d_iter = x.iter();
                for (def1, def2) in &mut d_iter {
                    out += &format!("{}:{},\n", def1, def2);
                }
                out.pop();
                out.pop();

                out += "\n}"; //why do i have to do this twice? idk

                write!(f, "{}", out)
            }
            Macro(x) => write!(f, "{:?}", x),
            Resolved(_) => write!(f, "<val>"),
            TypeIndicator(x) => write!(f, "@{}", x),
            Null => write!(f, "null"),
        }
    }
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
    Star,
    Power,
    Plus,
    Minus,
    Modulo,

    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operator::Or => "||",
                Operator::And => "&&",
                Operator::Equal => "==",
                Operator::NotEqual => "!=",
                Operator::Range => "..",
                Operator::MoreOrEqual => ">=",
                Operator::LessOrEqual => "<=",
                Operator::More => ">",
                Operator::Less => "<",
                Operator::Slash => "/",
                Operator::Star => "*",
                Operator::Power => "^",
                Operator::Plus => "+",
                Operator::Minus => "-",
                Operator::Modulo => "%",
                Operator::Assign => "=",
                Operator::Add => "+=",
                Operator::Subtract => "-=",
                Operator::Multiply => "*=",
                Operator::Divide => "/=",
            },
        )
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum UnaryOperator {
    Not,
    Minus,
    Range,
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnaryOperator::Not => "!",
                UnaryOperator::Minus => "-",
                UnaryOperator::Range => "..",
            },
        )
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum IDClass {
    Group,
    Color,
    Item,
    Block,
}

impl fmt::Display for IDClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                IDClass::Group => "g",
                IDClass::Color => "c",
                IDClass::Item => "i",
                IDClass::Block => "b",
            },
        )
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Tag {
    pub tags: Vec<(String, Vec<Argument>)>,
}

impl Tag {
    pub fn new() -> Self {
        Tag { tags: Vec::new() }
    }
    pub fn get(&self, t: &str) -> Option<Vec<Argument>> {
        for (key, args) in &self.tags {
            if t == key {
                return Some(args.clone());
            }
        }
        return None;
    }

    pub fn get_desc(&self) -> Option<String> {
        match self.get("desc") {
            Some(args) => {
                if args.is_empty() {
                    None
                } else {
                    match &args[0].value.values[0].value {
                        ValueLiteral::Str(s) => Some(s.clone()),
                        a => Some(format!("{}", a)),
                    }
                }
            }

            None => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Path {
    Member(String),
    Index(Expression),
    Call(Vec<Argument>),
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Path::Member(def) => write!(f, ".{}", def),
            Path::Index(call) => write!(f, "[{}]", call),
            Path::Call(x) => {
                if x.is_empty() {
                    return write!(f, "()");
                }
                let mut out = String::from("(");

                let mut d_iter = x.iter();
                for def in &mut d_iter {
                    out += &format!("{},", def);
                }
                out.pop();

                out += ")";

                write!(f, "{}", out)
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Definition {
    pub symbol: String,
    pub value: Expression,
    //pub mutable: bool,
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {} = {}", self.symbol, self.value)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Argument {
    pub symbol: Option<String>,
    pub value: Expression,
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(symbol) = &self.symbol {
            write!(f, "{} = {}", symbol, self.value)
        } else {
            write!(f, "{}", self.value)
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

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}!", self.function)
    }
}

/*#[derive(Clone, PartialEq, Debug)]
pub struct Native {
    pub function: Variable,
    pub args: Vec<Argument>,
}*/
//                 name         def value     props       type ind.
pub type ArgDef = (String, Option<Expression>, Tag, Option<Expression>);
#[derive(Clone, PartialEq, Debug)]
pub struct Macro {
    pub args: Vec<ArgDef>,
    pub body: CompoundStatement,
    pub properties: Tag,
}

#[derive(Clone, PartialEq, Debug)]
pub struct For {
    pub symbol: String,
    pub array: Expression,
    pub body: Vec<Statement>,
}

impl fmt::Display for For {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "for {} in {} {{\n{}\n}}",
            self.symbol,
            self.array,
            CompoundStatement {
                statements: self.body.clone()
            }
        )
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Error {
    pub message: Expression,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error {}", self.message)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Variable {
    pub operator: Option<UnaryOperator>,
    pub value: ValueLiteral,
    pub path: Vec<Path>,
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        if let Some(op) = &self.operator {
            out += &format!("{}", op);
        }

        out += &format!("{}", self.value);

        for p in &self.path {
            out += &format!("{}", p);
        }

        write!(f, "{}", out)
    }
}

/*impl Variable {
    pub fn to_expression(&self) -> Expression {
        if let ValueLiteral::Expression(e) = &self.value {
            if self.path.is_empty() {
                return e.to_owned();
            }
        }
        Expression {
            values: vec![self.clone()],
            operators: Vec::new(),
        }
    }
}*/

#[derive(Clone, PartialEq, Debug)]
pub struct Expression {
    pub values: Vec<Variable>,
    pub operators: Vec<Operator>,
}
impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        for (i, op) in self.operators.iter().enumerate() {
            out += &format!("{}{}", self.values[i], *op);
        }

        out += &format!("{}", self.values.last().unwrap());

        write!(f, "{}", out)
    }
}

impl Expression {
    pub fn to_variable(&self) -> Variable {
        Variable {
            operator: None,
            value: ValueLiteral::Expression(self.clone()),
            path: Vec::new(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct CompoundStatement {
    pub statements: Vec<Statement>,
}

impl fmt::Display for CompoundStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();

        for s in &self.statements {
            out += &format!("{}", s);
        }

        write!(f, "{}", out)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Implementation {
    pub symbol: Variable,
    pub members: Vec<DictDef>,
}

impl fmt::Display for Implementation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = format!("impl {}{{", self.symbol);
        if self.members.is_empty() {
            out += "}";
        } else {
            for s in &self.members {
                out += &format!("{},\n", s);
            }

            out.pop();
            out.pop();
            out += "\n}";
        }

        write!(f, "{}", out)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct If {
    pub condition: Expression,
    pub if_body: Vec<Statement>,
    pub else_body: Option<Vec<Statement>>,
}

impl fmt::Display for If {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = format!(
            "if {} {{\n{}\n}}",
            self.condition,
            CompoundStatement {
                statements: self.if_body.clone()
            }
        );

        if let Some(body) = &self.else_body {
            out += &format!(
                "else {{\n{}\n}}",
                CompoundStatement {
                    statements: body.clone()
                }
            );
        }

        write!(f, "{}", out)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ID {
    pub number: u16,
    pub unspecified: bool,
    pub class_name: IDClass,
}

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.unspecified {
            write!(f, "?{}", self.class_name)
        } else {
            write!(f, "{}{}", self.number, self.class_name)
        }
    }
}

pub fn str_content(inp: String) -> String {
    inp.clone().replace("\"", "")
    /*.replace("'", "")
    .replace("\r", "")
    .replace("\n", "")*/
}
