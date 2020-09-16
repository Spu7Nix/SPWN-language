// tools for automatically formatting spwn files

use crate::ast::*;

fn tabs(num: u16) -> String {
    let mut out = String::new();
    for _ in 0..num {
        out += "\t";
    }
    out
}

pub fn format(input: Vec<Statement>) -> String {
    let mut out = String::new();
    for s in input {
        out += &format!("{}", s.fmt(0));
    }
    out
}

impl DictDef {
    fn fmt(&self, ind: u16) -> String {
        match self {
            DictDef::Def((name, expr)) => format!("{}{}: {}", tabs(ind), name, expr.fmt(ind)),
            DictDef::Extract(expr) => format!("{}..{}", tabs(ind), expr.fmt(ind)),
        }
    }
}

impl Statement {
    fn fmt(&self, ind: u16) -> String {
        if self.arrow {
            format!("->{}\n", self.body.fmt(ind))
        } else {
            format!("{}\n", self.body.fmt(ind))
        }
    }
}

impl StatementBody {
    fn fmt(&self, ind: u16) -> String {
        match self {
            StatementBody::Definition(def) => format!("{}", def.fmt(ind)),
            StatementBody::Call(call) => format!("{}", call.fmt(ind)),
            StatementBody::Expr(x) => format!("{}", x.fmt(ind)),
            StatementBody::TypeDef(x) => format!("type {}", x),
            StatementBody::Return(x) => match x {
                Some(expr) => format!("return {}", expr.fmt(ind)),
                None => format!("return"),
            },
            StatementBody::Impl(x) => format!("{}", x.fmt(ind)),
            StatementBody::If(x) => format!("{}", x.fmt(ind)),
            StatementBody::For(x) => format!("{}", x.fmt(ind)),
            StatementBody::Error(x) => format!("{}", x.fmt(ind)),
            StatementBody::Extract(x) => format!("extract {}", x.fmt(ind)),
        }
    }
}

impl ValueLiteral {
    pub fn fmt(&self, ind: u16) -> String {
        use ValueLiteral::*;
        match self {
            ID(x) => format!("{}", x.fmt(ind)),
            Number(x) => format!("{}", x),
            CmpStmt(x) => format!("{{\n{}{}}}", x.fmt(ind + 1), tabs(ind)),
            Dictionary(x) => {
                if x.is_empty() {
                    return format!("{}{{}}", tabs(ind));
                }
                let mut out = format!("{}{{\n", tabs(ind));

                let mut d_iter = x.iter();
                for def in &mut d_iter {
                    out += &format!("{},\n", def.fmt(ind + 1));
                }
                out.pop();
                out.pop();

                out += &format!("{}\n}}", tabs(ind)); //why do i have to do this twice? idk

                format!("{}", out)
            }
            Array(x) => {
                if x.is_empty() {
                    return format!("[]");
                }
                let mut out = String::from("[");

                let mut d_iter = x.iter();
                for def in &mut d_iter {
                    out += &format!("{},", def.fmt(ind));
                }
                out.pop();

                out += "]";

                format!("{}", out)
            }
            Symbol(x) => format!("{}", x),
            Bool(x) => format!("{}", x),
            Expression(x) => format!("({})", x.fmt(ind)),
            Str(x) => format!("\"{}\"", x),
            Import(x) => format!("import {:?}", x),
            Obj(x) => {
                if x.is_empty() {
                    return format!("{}{{}}", tabs(ind));
                }
                let mut out = format!("{}{{\n", tabs(ind));

                let mut d_iter = x.iter();
                for (def1, def2) in &mut d_iter {
                    out += &format!("{}:{},\n", def1.fmt(ind), def2.fmt(ind));
                }
                out.pop();
                out.pop();

                out += &format!("{}\n}}", tabs(ind)); //why do i have to do this twice? idk

                format!("{}", out)
            }
            Macro(x) => format!("{:?}", x.fmt(ind)),
            Resolved(_) => format!("<val>"),
            TypeIndicator(x) => format!("@{}", x),
            Null => format!("null"),
        }
    }
}

impl IDClass {
    fn fmt(&self, ind: u16) -> String {
        format!(
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

impl Path {
    fn fmt(&self, ind: u16) -> String {
        match self {
            Path::Member(def) => format!(".{}", def),
            Path::Index(call) => format!("[{}]", call.fmt(ind)),
            Path::Call(x) => {
                if x.is_empty() {
                    return format!("()");
                }
                let mut out = String::from("(");

                let mut d_iter = x.iter();
                for def in &mut d_iter {
                    out += &format!("{},", def.fmt(ind));
                }
                out.pop();

                out += ")";

                format!("{}", out)
            }
        }
    }
}

impl Argument {
    fn fmt(&self, ind: u16) -> String {
        if let Some(symbol) = &self.symbol {
            format!("{} = {}", symbol, self.value.fmt(ind))
        } else {
            format!("{}", self.value.fmt(ind))
        }
    }
}

impl Call {
    fn fmt(&self, ind: u16) -> String {
        format!("{}!", self.function.fmt(ind))
    }
}

impl For {
    fn fmt(&self, ind: u16) -> String {
        format!(
            "for {} in {} {{\n{}\n}}",
            self.symbol,
            self.array.fmt(ind),
            CompoundStatement {
                statements: self.body.clone()
            }
            .fmt(ind)
        )
    }
}

impl Variable {
    pub fn fmt(&self, ind: u16) -> String {
        let mut out = String::new();
        if let Some(op) = &self.operator {
            out += &format!("{}", op.fmt(ind));
        }

        out += &format!("{}", self.value.fmt(ind));

        for p in &self.path {
            out += &format!("{}", p.fmt(ind));
        }

        format!("{}", out)
    }
}

impl Expression {
    fn fmt(&self, ind: u16) -> String {
        let mut out = String::new();
        for (i, op) in self.operators.iter().enumerate() {
            out += &format!("{} {} ", self.values[i].fmt(ind), (*op).fmt(ind));
        }

        out += &format!("{}", self.values.last().unwrap().fmt(ind));

        format!("{}", out)
    }
}

impl ID {
    fn fmt(&self, ind: u16) -> String {
        if self.unspecified {
            format!("?{}", self.class_name.fmt(ind))
        } else {
            format!("{}{}", self.number, self.class_name.fmt(ind))
        }
    }
}

impl Operator {
    fn fmt(&self, ind: u16) -> String {
        format!(
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

impl UnaryOperator {
    fn fmt(&self, ind: u16) -> String {
        format!(
            "{}",
            match self {
                UnaryOperator::Not => "!",
                UnaryOperator::Minus => "-",
                UnaryOperator::Range => "..",
            },
        )
    }
}

impl Definition {
    fn fmt(&self, ind: u16) -> String {
        format!("let {} = {}", self.symbol, self.value.fmt(ind))
    }
}

impl Error {
    fn fmt(&self, ind: u16) -> String {
        format!("error {}", self.message.fmt(ind))
    }
}

impl CompoundStatement {
    fn fmt(&self, ind: u16) -> String {
        let mut out = String::new();

        for s in &self.statements {
            out += &format!("{}{}", tabs(ind), s.fmt(ind));
        }

        format!("{}", out)
    }
}

impl Implementation {
    fn fmt(&self, ind: u16) -> String {
        let mut out = format!("impl {} {{", self.symbol.fmt(ind));
        if self.members.is_empty() {
            out += "}";
        } else {
            for s in &self.members {
                out += &format!("\n{}{},", tabs(ind + 1), s.fmt(ind));
            }

            out.pop();
            out.pop();
            out += "\n}";
        }

        format!("{}", out)
    }
}

impl If {
    fn fmt(&self, ind: u16) -> String {
        let mut out = format!(
            "if {} {{\n{}\n}}",
            self.condition.fmt(ind),
            CompoundStatement {
                statements: self.if_body.clone()
            }
            .fmt(ind)
        );

        if let Some(body) = &self.else_body {
            out += &format!(
                "else {{\n{}\n}}",
                CompoundStatement {
                    statements: body.clone()
                }
                .fmt(ind)
            );
        }

        format!("{}", out)
    }
}

impl Macro {
    fn fmt(&self, ind: u16) -> String {
        let mut out = String::new();

        out += &self.properties.fmt(ind);

        out += "(";
        for (name, value, tag, typ) in &self.args {
            out += &tag.fmt(ind);
            out += name;
            if let Some(expr) = typ {
                out += &format!(": {}", expr.fmt(ind));
            }

            if let Some(expr) = value {
                out += &format!(" = {}", expr.fmt(ind));
            }

            out += ",";
        }
        out += &format!("{{{}\n{}}}", &self.body.fmt(ind + 1), tabs(ind));
        out
    }
}

impl Tag {
    fn fmt(&self, ind: u16) -> String {
        if self.tags.is_empty() {
            return String::new();
        }
        let mut out = String::from("#[");
        for t in &self.tags {
            out += &t.0;
            out += "(";
            for a in &t.1 {
                out += &a.fmt(ind);
                out += ",";
            }
            out += "), ";
        }
        out += "] ";

        out
    }
}
