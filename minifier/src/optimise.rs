use parser::ast::*;

use std::collections::HashMap;
use std::char;
use internment::LocalIntern;

use crate::shared::MinifyOptions;
use crate::set_traits;

fn gen_var_name(mut n: usize, mut s: String) -> String {
    let mut r = n % 26;
    r = if r > 0 { r } else { 26 };
    n = (n - r) / 26;
    s.push(char::from_u32((64 + r) as u32).unwrap());

    if n > 26 {
        s = gen_var_name(n, s);
    }
    else if n > 0 {
        s.push(char::from_u32((64 + n) as u32).unwrap());
    }

    return s;
}


pub fn pre_optimise(mut statements: Vec<Statement>, opts: MinifyOptions) -> Vec<Statement> {

    let mut know_vars: Vars = HashMap::new();

    for (i, s) in statements.clone().iter().enumerate() {
        statements[i] = s.clone().replace_vars(&mut know_vars);
    }

    return statements;
}

type Vars = HashMap<String, String>;

fn get_or_gen(curr: String, vars: &mut Vars) -> String {
    if !vars.contains_key(&curr) {
        vars.insert(curr.clone(), gen_var_name(vars.keys().len() + 1, String::new()));

        return vars.get(&curr).unwrap().to_owned();
    }

    return vars.get(&curr).unwrap().to_owned();
}

set_traits! {
    trait MinOpt {
        fn replace_vars(self, vars: &mut Vars) -> Self;
    }

    [Statement]
    fn replace_vars(mut self, vars: &mut Vars) -> Self {
        self.body = self.body.replace_vars(vars);
        return self;
    }

    [StatementBody]
    fn replace_vars(mut self, vars: &mut Vars) -> Self {
        return match self {
            StatementBody::Call(sb) => Self::Call(sb.replace_vars(vars)),
            // StatementBody::Expr(sb) => sb.replace_vars(vars),
            StatementBody::TypeDef(name) => Self::TypeDef(get_or_gen(name, vars)),
            // StatementBody::Return(sb) => match sb {
            //     Some(expr) => expr.replace_vars(vars),
            //     None => self,
            // },
            // StatementBody::Definition(sb) => sb.replace_vars(vars),
            // StatementBody::Impl(sb) => sb.replace_vars(vars),
            // StatementBody::If(sb) => sb.replace_vars(vars),
            // StatementBody::For(sb) => sb.replace_vars(vars),
            // StatementBody::While(sb) => sb.replace_vars(vars),
            
            //StatementBody::Extract(x) => format!("extract {}", x.fmt(ind)),

            _ => self,
        }
    }

    [Call]
    fn replace_vars(mut self, vars: &mut Vars) -> Self {
        self.function = self.function.replace_vars(vars);
        return self;
    }

    [Variable]
    fn replace_vars(mut self, vars: &mut Vars) -> Self {
        self.value = self.value.replace_vars(vars);

        self.path = self.path.iter().map(|p| p.clone().replace_vars(vars))
                        .collect::<Vec<_>>();

        return self;
    }

    [ValueLiteral]
    fn replace_vars(mut self, vars: &mut Vars) -> Self {
        self.body = self.body.replace_vars(vars);
        return self;
    }

    [ValueBody]
    fn replace_vars(mut self, vars: &mut Vars) -> Self {
        self
    }

    [Path]
    fn replace_vars(self, vars: &mut Vars) -> Self {
        match self {
            Path::Member(name) => Self::Member(LocalIntern::new(get_or_gen(name.to_string(), vars))),
            // Path::Associated(def) => format!("::{}", def),
            // Path::NSlice(_def) => "[its a slice ok]".to_string(),
            // Path::Constructor(dict) => format!("::{}", element_list(dict, '{', '}', ind)),
            // Path::Index(call) => format!("[{}]", call.fmt(ind)),
            // Path::Call(x) => element_list(x, '(', ')', ind),

            _ => self
        }
    }


    [IdClass]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [Argument]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [For]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [Expression]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [Id]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [Operator]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [UnaryOperator]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [Definition]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [Error]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [CompoundStatement]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [Implementation]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [If]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [ArgDef]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [Macro]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [(String, Vec<Argument>)]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [Attribute]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [(Expression, Expression)]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
    [DictDef]
    fn replace_vars(self, vars: &mut Vars) -> Self {self}
}