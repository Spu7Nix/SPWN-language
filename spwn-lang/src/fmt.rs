// tools for automatically formatting spwn files

use crate::ast::*;

pub trait SpwnFmt {
    fn fmt(&self, ind: u16) -> String;
}

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

fn element_list(elements: &Vec<impl SpwnFmt>, open: char, closing: char, ind: u16) -> String {
    if elements.is_empty() {
        return format!("{}{}", open, closing);
    }

    let mut elem_text = Vec::<String>::new();
    let mut sum = 0;

    let last = elements.len() - 1;

    for (i, el) in elements.iter().enumerate() {
        let text = el.fmt(0);

        sum += text.lines().next().unwrap().len();

        elem_text.push(text)
    }

    let vertical = if elements.len() == 1 {
        sum > 150
    } else {
        elem_text.iter().enumerate().any(|(i, x)| {
            if i != last {
                x.len() > 50 || x.contains("\n")
            } else {
                sum > 100
            }
        })
    };

    if vertical {
        let mut out = format!("{}\n", open);

        for el in &elem_text {
            for line in el.lines() {
                out += &format!("{}{}\n", tabs(ind + 1), line);
            }
            out.pop();
            out += ",\n";
        }
        /*if elements.len() == 1 {
            out.pop();
            out.pop();
            out += "\n";
        }*/

        out + &format!("{}{}", tabs(ind), closing)
    } else {
        let mut out = format!("{}", open);
        let last_elem = elem_text.pop().unwrap();
        let iter = elem_text.iter();

        for el in iter {
            out += &format!("{}, ", el);
        }

        let mut last_elem_lines = last_elem.lines();
        out += &last_elem_lines.next().unwrap();

        for line in last_elem_lines {
            out += &format!("\n{}{}", tabs(ind), line);
        }

        out.push(closing);
        out
    }
}

impl SpwnFmt for DictDef {
    fn fmt(&self, ind: u16) -> String {
        match self {
            DictDef::Def((name, expr)) => format!("{}{}: {}", tabs(ind), name, expr.fmt(ind)),
            DictDef::Extract(expr) => format!("{}..{}", tabs(ind), expr.fmt(ind)),
        }
    }
}

impl SpwnFmt for Statement {
    fn fmt(&self, ind: u16) -> String {
        let mut out = String::new();
        if let Some(comment) = &self.comment.0 {
            out += comment;
        }
        out += &if self.arrow {
            format!("-> {}\n", self.body.fmt(ind))
        } else {
            format!("{}\n", self.body.fmt(ind))
        };
        if let Some(comment) = &self.comment.1 {
            out += comment;
        }
        out
    }
}

impl SpwnFmt for StatementBody {
    fn fmt(&self, ind: u16) -> String {
        let main = match self {
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
        };
        let last = main.chars().last().unwrap();
        if last == '}' || last == '!' {
            main
        } else {
            main + ";"
        }
    }
}

//for object def
impl SpwnFmt for (Expression, Expression) {
    fn fmt(&self, ind: u16) -> String {
        format!("{}: {}", self.0.fmt(ind), self.1.fmt(ind))
    }
}

impl SpwnFmt for ValueBody {
    fn fmt(&self, ind: u16) -> String {
        use ValueBody::*;
        match self {
            ID(x) => format!("{}", x.fmt(ind)),
            Number(x) => format!("{}", x),
            CmpStmt(x) => format!("{{\n{}\n{}}}", x.fmt(ind + 1), tabs(ind)),
            Dictionary(x) => element_list(x, '{', '}', ind),
            Array(x) => element_list(x, '[', ']', ind),
            Symbol(x) => format!("{}", x),
            Bool(x) => format!("{}", x),
            Expression(x) => format!("({})", x.fmt(ind)),
            Str(x) => format!("\"{}\"", x),
            Import(x) => format!("import {:?}", x),
            Obj(x) => element_list(x, '{', '}', ind),
            Macro(x) => format!("{}", x.fmt(ind)),
            Resolved(_) => format!("<val>"),
            TypeIndicator(x) => format!("@{}", x),
            Null => format!("null"),
        }
    }
}

impl SpwnFmt for ValueLiteral {
    fn fmt(&self, ind: u16) -> String {
        let mut out = String::new();
        if let Some(comment) = &self.comment.0 {
            out += comment;
        }
        out += &self.body.fmt(ind);
        if let Some(comment) = &self.comment.1 {
            out += comment;
        }
        out
    }
}

impl SpwnFmt for IDClass {
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

impl SpwnFmt for Path {
    fn fmt(&self, ind: u16) -> String {
        match self {
            Path::Member(def) => format!(".{}", def),
            Path::Index(call) => format!("[{}]", call.fmt(ind)),
            Path::Call(x) => element_list(x, '(', ')', ind),
        }
    }
}

impl SpwnFmt for Argument {
    fn fmt(&self, ind: u16) -> String {
        if let Some(symbol) = &self.symbol {
            format!("{} = {}", symbol, self.value.fmt(ind))
        } else {
            format!("{}", self.value.fmt(ind))
        }
    }
}

impl SpwnFmt for Call {
    fn fmt(&self, ind: u16) -> String {
        format!("{}!", self.function.fmt(ind))
    }
}

impl SpwnFmt for For {
    fn fmt(&self, ind: u16) -> String {
        format!(
            "for {} in {} {{\n{}\n{}}}",
            self.symbol,
            self.array.fmt(ind),
            CompoundStatement {
                statements: self.body.clone()
            }
            .fmt(ind + 1),
            tabs(ind)
        )
    }
}

impl SpwnFmt for Variable {
    fn fmt(&self, ind: u16) -> String {
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

impl SpwnFmt for Expression {
    fn fmt(&self, ind: u16) -> String {
        let mut out = String::new();
        for (i, op) in self.operators.iter().enumerate() {
            if let Operator::Range = op {
                out += &format!("{}{}", self.values[i].fmt(ind), (*op).fmt(ind));
            } else {
                out += &format!("{} {} ", self.values[i].fmt(ind), (*op).fmt(ind));
            }
        }

        out += &format!("{}", self.values.last().unwrap().fmt(ind));

        format!("{}", out)
    }
}

impl SpwnFmt for ID {
    fn fmt(&self, ind: u16) -> String {
        if self.unspecified {
            format!("?{}", self.class_name.fmt(ind))
        } else {
            format!("{}{}", self.number, self.class_name.fmt(ind))
        }
    }
}

impl SpwnFmt for Operator {
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

impl SpwnFmt for UnaryOperator {
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

impl SpwnFmt for Definition {
    fn fmt(&self, ind: u16) -> String {
        format!("let {} = {}", self.symbol, self.value.fmt(ind))
    }
}

impl SpwnFmt for Error {
    fn fmt(&self, ind: u16) -> String {
        format!("error {}", self.message.fmt(ind))
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

impl SpwnFmt for CompoundStatement {
    fn fmt(&self, ind: u16) -> String {
        let mut out = String::new();

        for s in &self.statements {
            out += &format!("{}{}", tabs(ind), s.fmt(ind));
        }

        trim_newline(&mut out);

        out
    }
}

impl SpwnFmt for Implementation {
    fn fmt(&self, ind: u16) -> String {
        format!("impl {} ", self.symbol.fmt(ind)) + &element_list(&self.members, '{', '}', ind)
    }
}

impl SpwnFmt for If {
    fn fmt(&self, ind: u16) -> String {
        let mut out = format!(
            "if {} {{\n{}\n{}}}",
            self.condition.fmt(ind),
            CompoundStatement {
                statements: self.if_body.clone()
            }
            .fmt(ind + 1),
            tabs(ind)
        );

        if let Some(body) = &self.else_body {
            out += &format!(
                " else {{\n{}\n{}}}",
                CompoundStatement {
                    statements: body.clone()
                }
                .fmt(ind + 1),
                tabs(ind)
            );
        }

        out
    }
}

impl SpwnFmt for ArgDef {
    fn fmt(&self, ind: u16) -> String {
        let (name, value, tag, typ) = self;

        let mut out = tag.fmt(ind);
        out += name;
        if let Some(expr) = typ {
            out += &format!(": {}", expr.fmt(ind));
        }

        if let Some(expr) = value {
            out += &format!(" = {}", expr.fmt(ind));
        }
        out
    }
}

impl SpwnFmt for Macro {
    fn fmt(&self, ind: u16) -> String {
        let mut out = String::new();

        out += &self.properties.fmt(ind);

        out += &element_list(&self.args, '(', ')', ind);
        out += &format!(" {{\n{}\n{}}}", &self.body.fmt(ind + 1), tabs(ind));
        out
    }
}

impl SpwnFmt for (String, Vec<Argument>) {
    fn fmt(&self, ind: u16) -> String {
        self.0.clone() + &element_list(&self.1, '(', ')', ind)
    }
}

impl SpwnFmt for Tag {
    fn fmt(&self, ind: u16) -> String {
        if self.tags.is_empty() {
            return String::new();
        }

        let text = String::from("#") + &element_list(&self.tags, '[', ']', ind);
        if text.len() > 60 {
            text + "\n" + &tabs(ind)
        } else {
            text + " "
        }
    }
}
