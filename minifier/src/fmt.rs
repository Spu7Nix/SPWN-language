use parser::ast::*;
use shared::ImportType;

use crate::shared::MinifyOptions;
use crate::set_traits;

pub fn fmt(statements: Vec<Statement>, opts: &MinifyOptions) -> String {
    let mut minifed = String::new();

    for s in statements.iter() {
        minifed += &s.clone().fmt(&opts);
    }

    return minifed;
}

fn list(vec: &[impl MinOpt], braces: [&str; 2], opts: &MinifyOptions) -> String {
    return format!("{1}{0}{2}", vec.iter().map(|v| v.fmt(opts)).collect::<Vec<_>>().join(","), braces[0], braces[1]);
}

fn optimise(value: String) -> String {
    return format!("{}{}", match value.chars().nth(0) {
        Some('{') => "",
        Some('(') => "",
        Some('[') => "",
        Some(_) => " ",
        None => unreachable!(),
    }, &value);
}

set_traits! {
    trait MinOpt {
        fn fmt(&self, opts: &MinifyOptions) -> String;
    }

    [Statement]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return if self.arrow {
            format!("->{};", self.body.fmt(opts))
        } else {
            format!("{};", self.body.fmt(opts))
        };
    }

    [StatementBody]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return match self {
            StatementBody::Call(sb) => sb.fmt(opts),
            StatementBody::Expr(sb) => sb.fmt(opts),
            StatementBody::TypeDef(sb) => format!("type@{}", sb),
            StatementBody::Return(sb) => match sb {
                Some(expr) => format!("return{}", optimise(expr.fmt(opts))),
                None => "return".to_string(),
            },
            StatementBody::Definition(sb) => sb.fmt(opts),
            StatementBody::Impl(sb) => sb.fmt(opts),
            StatementBody::If(sb) => sb.fmt(opts),
            StatementBody::For(sb) => sb.fmt(opts),
            StatementBody::While(sb) => sb.fmt(opts),
            StatementBody::Error(sb) => sb.fmt(opts),
            StatementBody::Extract(sb) => format!("extract{}", optimise(sb.fmt(opts))),
            StatementBody::Break => "break".to_string(),
            StatementBody::Continue => "continue".to_string(),
        };
    }

    [Call]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return format!("{}!", self.function.fmt(opts));
    }

    [Variable]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        let mut out = String::new();

        if let Some(op) = &self.operator {
            match op {
                UnaryOperator::Range => out += "0",
                _ => {},
            }
            out += &op.fmt(opts);
        }

        out += &self.value.fmt(opts);

        for p in &self.path {
            out += &p.fmt(opts).to_string();
        }

        return out;
    }

    [ValueLiteral]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return self.body.fmt(opts);
    }

    [ValueBody]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        use ValueBody::*;

        return match self {
            Id(vb) => vb.fmt(opts),
            Number(vb) => format!("{}", vb),
            CmpStmt(vb) => format!("!{{{}}}", vb.fmt(opts)),
            Dictionary(vb) => list(vb, ["{", "}"], opts),
            Array(vb) => list(vb, ["[", "]"], opts),
            Symbol(vb) => vb.to_string(),
            Bool(vb) => format!("{}", vb),
            Expression(vb) => format!("{}", vb.fmt(opts)),
            Str(vb) => format!("\"{}\"", vb.inner.escape_default()),
            Import(vb, f) => {
                format!("import{} {}", if *f { "!" } else { "" },
                    (match vb {
                        ImportType::Script(sc) => format!("{:?}", sc),
                        ImportType::Lib(x) => x.to_string(),
                    })
                )
            },
            Obj(vb) => {
                (match vb.mode {
                    ObjectMode::Object => "obj".to_string(),
                    ObjectMode::Trigger => "trigger".to_string(),
                }) + &list(&vb.props, ["{", "}"], opts)
            }
            Macro(vb) => vb.fmt(opts),
            TypeIndicator(vb) => format!("@{}", vb),
            Ternary(t) => format!(
                "{} if{} else{}",
                t.if_expr.fmt(opts),
                optimise(t.condition.fmt(opts)),
                optimise(t.else_expr.fmt(opts))
            ),
            ListComp(c) => format!(
                "{} for {} in {}",
                c.body.fmt(opts),
                c.symbol,
                c.iterator.fmt(opts)
            ),
            Switch(vb, cases) => format!("switch{}{}", optimise(vb.fmt(opts)), list(cases, ["{", "}"], opts)),
            Null => "null".to_string(),
            SelfVal => "self".to_string(),

            Resolved(_) => unreachable!(),
        };
    }

    [Path]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return match self {
            Path::Member(def) => format!(".{}", def),
            Path::Associated(def) => format!("::{}", def),
            Path::NSlice(def) => list(def, ["[", "]"], opts),
            Path::Constructor(dict) => format!("::{}", list(dict, ["{", "}"], opts)),
            Path::Index(call) => format!("[{}]", call.fmt(opts)),
            Path::Call(x) => list(x, ["(", ")"], opts),
            Path::Increment => "++".to_string(),
            Path::Decrement => "--".to_string(),
        };
    }

    [Case]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return format!("{}{}", match &self.typ {
            CaseType::Value(expr) => format!("case{}:", optimise(expr.fmt(opts))),
            CaseType::Pattern(pat) => format!("{}:", pat.fmt(opts)),
            CaseType::Default => "else:".to_string(),
        }, 
        self.body.fmt(opts));
    }

    [Slice]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        let mut out = String::new();

        if let Some(left) = &self.left {
            out += &format!("{}", left.fmt(opts));
        }
        if let Some(step) = &self.step {
            out += &format!(":{}", step.fmt(opts));
        }
        if let Some(right) = &self.right {
            out += &format!(":{}", right.fmt(opts));
        }

        return out;
    }

    [IdClass]
    fn fmt(&self, _: &MinifyOptions) -> String {
        return match self {
            IdClass::Group => "g",
            IdClass::Color => "c",
            IdClass::Item => "i",
            IdClass::Block => "b",
        }.to_string();
    }

    [Argument]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return if let Some(symbol) = &self.symbol {
            format!("{}={}", symbol, self.value.fmt(opts))
        } else {
            self.value.fmt(opts)
        };
    }

    [For]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return format!(
            "for {} in {}{{{}}}",
            self.symbol.fmt(opts),
            self.array.fmt(opts),
            CompoundStatement {
                statements: self.body.clone()
            }.fmt(opts),
        );
    }

    [While]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return format!(
            "while{}{{{}}}",
            optimise(self.condition.fmt(opts)),
            CompoundStatement {
                statements: self.body.clone()
            }.fmt(opts)
        );
    }

    [Operator]
    fn fmt(&self, _: &MinifyOptions) -> String {
        return match self {
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
            Operator::Either => "|",
            Operator::Exponate => "^=",
            Operator::Modulate => "%=",
            Operator::Swap => "<=>",
            //only ones that needs spaces since theyre keywords
            Operator::Has => " has ",
            Operator::As => " as ",
        }.to_string();
    }

    [Expression]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        let mut out = String::new();
        
        for (i, op) in self.operators.iter().enumerate() {
            out += &format!("{}{}", self.values[i].fmt(opts), (*op).fmt(opts));
        }
        out += &self.values.last().unwrap().fmt(opts);

        return out;
    }

    [Id]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return if self.unspecified {
            format!("?{}", self.class_name.fmt(opts))
        } else {
            format!("{}{}", self.number, self.class_name.fmt(opts))
        }; 
    }

    [UnaryOperator]
    fn fmt(&self, _: &MinifyOptions) -> String {
        return match self {
            UnaryOperator::Not => "!",
            UnaryOperator::Minus => "-",
            UnaryOperator::Range => "..",
            UnaryOperator::Decrement => "--",
            UnaryOperator::Increment => "++",
        }.to_string();
    }

    [Definition]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return format!(
            "{}{}{}",
            if self.mutable { "let " } else { "" },
            self.symbol.fmt(opts),
            if let Some(value) = &self.value {
                format!("={}", value.fmt(opts))
            } else {
                String::new()
            }
        );
    }

    [Error]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return format!("throw {}", self.message.fmt(opts));
    }

    [CompoundStatement]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        let mut out = String::new();

        for s in &self.statements {
            out += &s.fmt(opts);
        }

        return out;
    }

    [Implementation]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return format!("impl{}", self.symbol.fmt(opts)) + &list(&self.members, ["{", "}"], opts);
    }

    [If]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        let mut out = format!(
            "if{}{{{}}}",
            optimise(self.condition.fmt(opts)),
            CompoundStatement {
                statements: self.if_body.clone()
            }.fmt(opts),
        );

        if let Some(body) = &self.else_body {
            out += &format!(
                "else{{{}}}",
                CompoundStatement {
                    statements: body.clone()
                }.fmt(opts),
            );
        }

        return out;
    }

    [(Expression, Expression)]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return format!("{}:{}", self.0.fmt(opts), self.1.fmt(opts));
    }

    [DictDef]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return match self {
            DictDef::Def((name, expr)) => format!("{}:{}", name, expr.fmt(opts)),
            DictDef::Extract(expr) => format!("..{}", expr.fmt(opts)),
        }
    }

    [Macro]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        let mut out = String::new();

        out += &self.properties.fmt(opts);

        out += &list(&self.args, ["(", ")"], opts);
        out += &format!("{{{}}}", &self.body.fmt(opts));

        return out;  
    }

    [ArgDef]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        let (name, value, tag, typ, _, _) = self;

        let mut out = tag.fmt(opts);
        out += name;

        if let Some(expr) = typ {
            if !opts.clear_types {
                out += &format!(":{}", expr.fmt(opts));
            }
        }
        if let Some(expr) = value {
            out += &format!("={}", expr.fmt(opts));
        }

        return out;
    }

    [(String, Vec<Argument>)]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        return self.0.clone() + &list(&self.1, ["(", ")"], opts);
    }

    [Attribute]
    fn fmt(&self, opts: &MinifyOptions) -> String {
        if self.tags.is_empty() || opts.keep.is_empty() {
            return String::new();
        }

        let tags = self.tags.iter().filter_map(|t| {
            if opts.keep.contains(&t.0) { 
                Some(format!("{}{}", t.0, list(&t.1, ["(", ")"], opts)))
            } else { None }
        })
        .collect::<Vec<_>>();

        return format!("#[{}]", tags.join(","));
    }
}