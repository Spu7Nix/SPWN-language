// tools for automatically formatting spwn files

use crate::ast::*;

pub trait SpwnFmt {
    fn fmt(&self, ind: Indent) -> String;
}

type Indent = u16;

fn tabs(mut num: Indent) -> String {
    let mut out = String::new();
    while num > 4 {
        out += "\t";
        num -= 4;
    }

    for _ in 0..num {
        out += " ";
    }

    out
}

pub fn _format(input: Vec<Statement>) -> String {
    let mut out = String::new();
    for s in input {
        out += &s.fmt(0)
    }
    out
}

pub fn _format2(input: &ValueBody) -> String {
    input.fmt(0)
}

fn element_list(elements: &[impl SpwnFmt], open: char, closing: char, ind: Indent) -> String {
    if elements.is_empty() {
        return format!("{}{}", open, closing);
    }

    let mut elem_text = Vec::<String>::new();
    let mut sum = 0;

    let last = elements.len() - 1;

    for (_i, el) in elements.iter().enumerate() {
        let text = el.fmt(0);

        sum += text.lines().next().unwrap().len();

        elem_text.push(text)
    }

    let vertical = if elements.len() == 1 {
        sum > 150
    } else {
        elem_text.iter().enumerate().any(|(i, x)| {
            if i != last {
                x.len() > 50 || x.contains('\n')
            } else {
                sum > 100
            }
        })
    };

    if vertical {
        let mut out = format!("{}\n", open);

        for el in &elem_text {
            for line in el.lines() {
                out += &format!("{}{}\n", tabs(ind + 4), line);
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
        out += last_elem_lines.next().unwrap();

        for line in last_elem_lines {
            out += &format!("\n{}{}", tabs(ind), line);
        }

        out.push(closing);
        out
    }
}

impl SpwnFmt for DictDef {
    fn fmt(&self, ind: Indent) -> String {
        match self {
            DictDef::Def((name, expr)) => format!("{}{}: {}", tabs(ind), name, expr.fmt(ind)),
            DictDef::Extract(expr) => format!("{}..{}", tabs(ind), expr.fmt(ind)),
        }
    }
}

// fn trim_start_tabs(string: &str) -> (&str, Indent) {
//     //https://doc.rust-lang.org/src/core/str/mod.rs.html#4082-4090
//     let mut ind = 0;
//     for (i, c) in string.chars().enumerate() {
//         match c {
//             '\t' => ind += 4,
//             ' ' => ind += 1,
//             _ => return (unsafe { string.get_unchecked(i..string.len()) }, ind),
//         }
//     }
//     ("", ind)
// }

// fn indent_comment(comment: &str, ind: Indent) -> String {
//     let mut in_comment = false;
//     let mut current_off = 0;
//     let mut out = String::new();
//     for line in comment.lines() {
//         let (trimmed, ind_offset) = trim_start_tabs(line);
//         if !in_comment {
//             if trimmed.starts_with("//") {
//                 out += &format!("{}{}\r\n", tabs(ind), trimmed);
//             } else if trimmed.starts_with("/*") {
//                 in_comment = true;
//                 current_off = ind_offset;
//                 out += &format!("{}{}\r\n", tabs(ind), trimmed);
//             }
//         } else {
//             out += &format!("{}{}\r\n", tabs(ind_offset - current_off), trimmed);
//         }

//         if line.trim_end().ends_with("*/") {
//             in_comment = false
//         }
//     }
//     out
// }

/*#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        println!(
            "{}",
            indent_comment(
                &String::from(
                    "
//hello
                /*
                a = {
                    b = 2
                    c = 3
                    a = {
                        b = 2
                        c = 3
                        a = {
                            b = 2
                            c = 3
                        }
                    }
                }
                */

//bruh
//bruh
//bruh
        "
                ),
                0
            )
        )
    }
}*/

impl SpwnFmt for Statement {
    fn fmt(&self, ind: Indent) -> String {
        let mut out = String::new();
        // if let Some(comment) = &self.comment.0 {
        //     //out += "[stmt pre]";

        //     out += &indent_comment(comment, ind);
        //     //
        //     if !comment.ends_with('\n') {
        //         out += "\n";
        //     }
        //     out += &tabs(ind);
        // }
        out += &if self.arrow {
            format!("-> {}\n", self.body.fmt(ind))
        } else {
            format!("{}\n", self.body.fmt(ind))
        };
        // if let Some(comment) = &self.comment.1 {
        //     if !comment.starts_with('\n') {
        //         out += "\n";
        //     }

        //     //out += &tabs(ind);
        //     //out += "[stmt post]";
        //     out += &indent_comment(comment, ind);
        // }
        out
    }
}

impl SpwnFmt for StatementBody {
    fn fmt(&self, ind: Indent) -> String {
        match self {
            //StatementBody::Definition(def) => format!("{}", def.fmt(ind)),
            StatementBody::Call(call) => call.fmt(ind),
            StatementBody::Expr(x) => x.fmt(ind),
            StatementBody::TypeDef(x) => format!("type {}", x),
            StatementBody::Return(x) => match x {
                Some(expr) => format!("return {}", expr.fmt(ind)),
                None => "return".to_string(),
            },
            StatementBody::Definition(x) => x.fmt(ind),
            StatementBody::Impl(x) => x.fmt(ind),
            StatementBody::If(x) => x.fmt(ind),
            StatementBody::For(x) => x.fmt(ind),
            StatementBody::While(_) => "While loop lol".to_string(),
            StatementBody::Error(x) => x.fmt(ind),
            StatementBody::Extract(x) => format!("extract {}", x.fmt(ind)),
            StatementBody::Break => String::from("break"),
            StatementBody::Continue => String::from("continue"),
        }
    }
}

//for object def
impl SpwnFmt for (Expression, Expression) {
    fn fmt(&self, ind: Indent) -> String {
        format!("{}: {}", self.0.fmt(ind), self.1.fmt(ind))
    }
}

impl SpwnFmt for ArrayDef {
    fn fmt(&self, ind: Indent) -> String {
        match &self.operator {
            Some(ArrayPrefix::Collect) => format!("*{}", self.value.fmt(ind)),
            Some(ArrayPrefix::Spread) => format!("..{}", self.value.fmt(ind)),
            None => self.value.fmt(ind),
        }
    }
}

impl SpwnFmt for ValueBody {
    fn fmt(&self, ind: Indent) -> String {
        use ValueBody::*;
        match self {
            Id(x) => x.fmt(ind),
            Number(x) => format!("{}", x),
            CmpStmt(x) => format!("!{{\n{}\n{}}}", x.fmt(ind + 4), tabs(ind)),
            Dictionary(x) => element_list(x, '{', '}', ind),
            Array(x) => element_list(x, '[', ']', ind),
            Symbol(x) => x.to_string(),
            Bool(x) => format!("{}", x),
            Expression(x) => format!("({})", x.fmt(ind)),
            Str(x) => format!("\"{}\"", x.inner),
            Import(x, f) => format!("import{} {:?}", if *f { "!" } else { "" }, x),
            Obj(x) => {
                (match x.mode {
                    ObjectMode::Object => "obj".to_string(),
                    ObjectMode::Trigger => "trigger".to_string(),
                }) + &element_list(&x.props, '{', '}', ind)
            }
            Macro(x) => x.fmt(ind),
            Resolved(_) => "<val>".to_string(),
            TypeIndicator(x) => format!("@{}", x),
            Null => "null".to_string(),
            SelfVal => "self".to_string(),
            Ternary(t) => if !t.is_pattern {
                format!(
                    "{} if is {} else {}",
                    t.if_expr.fmt(ind),
                    t.condition.fmt(ind),
                    t.else_expr.fmt(ind)
                )
            } else {
                format!(
                    "{} if {} else {}",
                    t.if_expr.fmt(ind),
                    t.condition.fmt(ind),
                    t.else_expr.fmt(ind)
                )
            }
            ListComp(c) => format!(
                "{} for {} in {}",
                c.body.fmt(ind),
                c.symbol,
                c.iterator.fmt(ind)
            ),
            Switch(_, _) => "switch".to_string(),
        }
    }
}

impl SpwnFmt for ValueLiteral {
    fn fmt(&self, ind: Indent) -> String {
        self.body.fmt(ind)
    }
}

impl SpwnFmt for IdClass {
    fn fmt(&self, _ind: Indent) -> String {
        match self {
            IdClass::Group => "g",
            IdClass::Color => "c",
            IdClass::Item => "i",
            IdClass::Block => "b",
        }
        .to_string()
    }
}

impl SpwnFmt for Path {
    fn fmt(&self, ind: Indent) -> String {
        match self {
            Path::Member(def) => format!(".{}", def),
            Path::Associated(def) => format!("::{}", def),
            Path::NSlice(_def) => "[its a slice ok]".to_string(),
            Path::Constructor(dict) => format!("::{}", element_list(dict, '{', '}', ind)),
            Path::Index(call) => format!("[{}]", call.fmt(ind)),
            Path::Call(x) => element_list(x, '(', ')', ind),
            Path::Increment => "++".to_string(),
            Path::Decrement => "--".to_string(),
        }
    }
}

impl SpwnFmt for Argument {
    fn fmt(&self, ind: Indent) -> String {
        if let Some(symbol) = &self.symbol {
            format!("{} = {}", symbol, self.value.fmt(ind))
        } else {
            self.value.fmt(ind)
        }
    }
}

impl SpwnFmt for Call {
    fn fmt(&self, ind: Indent) -> String {
        format!("{}!", self.function.fmt(ind))
    }
}

impl SpwnFmt for For {
    fn fmt(&self, ind: Indent) -> String {
        format!(
            "for {} in {} {{\n{}\n{}}}",
            self.symbol.fmt(ind),
            self.array.fmt(ind),
            CompoundStatement {
                statements: self.body.clone()
            }
            .fmt(ind + 4),
            tabs(ind)
        )
    }
}

impl SpwnFmt for Variable {
    fn fmt(&self, ind: Indent) -> String {
        let mut out = String::new();

        // if let Some(comment) = &self.comment.0 {
        //     //out += "[var pre]";
        //     out += &indent_comment(comment, ind);

        //     if comment.ends_with('\n') {
        //         out += &tabs(ind);
        //     }
        // }

        if let Some(op) = &self.operator {
            out += &op.fmt(ind);
        }

        out += &self.value.fmt(ind);

        for p in &self.path {
            out += &p.fmt(ind).to_string();
        }

        // if let Some(comment) = &self.comment.1 {
        //     //out += "[var post]";

        //     out += &indent_comment(comment, ind);

        //     /*if comment.ends_with("\n") {
        //         out += &tabs(ind);
        //     }*/
        // }

        out
    }
}

impl SpwnFmt for Expression {
    fn fmt(&self, ind: Indent) -> String {
        let mut out = String::new();
        for (i, op) in self.operators.iter().enumerate() {
            if let Operator::Range = op {
                out += &format!("{}{}", self.values[i].fmt(ind), (*op).fmt(ind));
            } else {
                out += &format!("{} {} ", self.values[i].fmt(ind), (*op).fmt(ind));
            }
        }

        out += &self.values.last().unwrap().fmt(ind);

        out
    }
}

impl SpwnFmt for Id {
    fn fmt(&self, ind: Indent) -> String {
        if self.unspecified {
            format!("?{}", self.class_name.fmt(ind))
        } else {
            format!("{}{}", self.number, self.class_name.fmt(ind))
        }
    }
}

impl SpwnFmt for Operator {
    fn fmt(&self, _ind: Indent) -> String {
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
            Operator::IntDividedBy => "/%",
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
            Operator::IntDivide => "/%=",
            Operator::As => "as",
            Operator::In => "in",
            Operator::Either => "|",
            Operator::Both => "&",
            Operator::Exponate => "^=",
            Operator::Modulate => "%=",
            Operator::Swap => "<=>",
            Operator::Is => "is",
        }
        .to_string()
    }
}

impl SpwnFmt for UnaryOperator {
    fn fmt(&self, _ind: Indent) -> String {
        match self {
            UnaryOperator::Not => "!",
            UnaryOperator::Minus => "-",
            UnaryOperator::Decrement => "--",
            UnaryOperator::Increment => "++",
            UnaryOperator::EqPattern => "==",
            UnaryOperator::NotEqPattern => "!=",
            UnaryOperator::MorePattern => ">",
            UnaryOperator::LessPattern => "<",
            UnaryOperator::MoreOrEqPattern => ">=",
            UnaryOperator::LessOrEqPattern => "<=",
            UnaryOperator::InPattern => "in",
        }
        .to_string()
    }
}

impl SpwnFmt for Definition {
    fn fmt(&self, ind: Indent) -> String {
        format!(
            "{}{}{}",
            if self.mutable { "let " } else { "" },
            self.symbol.fmt(ind),
            if let Some(value) = &self.value {
                format!(" = {}", value.fmt(ind))
            } else {
                String::new()
            }
        )
    }
}

impl SpwnFmt for Error {
    fn fmt(&self, ind: Indent) -> String {
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
    fn fmt(&self, ind: Indent) -> String {
        let mut out = String::new();

        for s in &self.statements {
            out += &format!("{}{}", tabs(ind), s.fmt(ind));
        }

        trim_newline(&mut out);

        out
    }
}

impl SpwnFmt for Implementation {
    fn fmt(&self, ind: Indent) -> String {
        format!("impl {} ", self.symbol.fmt(ind)) + &element_list(&self.members, '{', '}', ind)
    }
}

impl SpwnFmt for If {
    fn fmt(&self, ind: Indent) -> String {
        let mut out = format!(
            "if {} {{\n{}\n{}}}",
            self.condition.fmt(ind),
            CompoundStatement {
                statements: self.if_body.clone()
            }
            .fmt(ind + 4),
            tabs(ind)
        );

        if let Some(body) = &self.else_body {
            out += &format!(
                " else {{\n{}\n{}}}",
                CompoundStatement {
                    statements: body.clone()
                }
                .fmt(ind + 4),
                tabs(ind)
            );
        }

        out
    }
}

impl SpwnFmt for ArgDef {
    fn fmt(&self, ind: Indent) -> String {
        let (name, value, tag, typ, _, _) = self;

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
    fn fmt(&self, ind: Indent) -> String {
        let mut out = String::new();

        out += &self.properties.fmt(ind);

        out += &element_list(&self.args, '(', ')', ind);
        out += &format!(" {{\n{}\n{}}}", &self.body.fmt(ind + 4), tabs(ind));
        out
    }
}

impl SpwnFmt for (String, Vec<Argument>) {
    fn fmt(&self, ind: Indent) -> String {
        self.0.clone() + &element_list(&self.1, '(', ')', ind)
    }
}

impl SpwnFmt for Attribute {
    fn fmt(&self, ind: Indent) -> String {
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
