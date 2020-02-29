use crate::ast;
use crate::levelstring::*;
use crate::native::*;
//use std::boxed::Box;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::compiler::{compile_scope, import_module, next_free};

pub type Returns = Vec<(Value, Context)>;
pub type Implementations = HashMap<String, HashMap<String, u32>>;
pub type StoredValue = u32; //index to stored value in globals.stored_values
pub fn store_value(val: Value, globals: &mut Globals) -> StoredValue {
    (*globals).stored_values.push(val);
    (globals.stored_values.len() as u32) - 1
}

#[derive(PartialEq, Debug, Clone)]
pub struct Context {
    pub x: u32,
    pub y: u16,
    pub start_group: Group,
    pub spawn_triggered: bool,
    pub variables: HashMap<String, StoredValue>,
    pub implementations: Implementations,
}
#[derive(Debug, Clone)]
pub struct CompilerInfo {
    pub depth: u8,
    pub path: Vec<String>,
    pub line: (usize, usize),
}

impl CompilerInfo {
    pub fn next(&self, name: &str) -> CompilerInfo {
        let mut new_path = self.path.clone();
        new_path.push(name.to_string());
        CompilerInfo {
            depth: self.depth + 1,
            path: new_path,
            line: self.line,
        }
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            x: 0,
            y: 1500,
            start_group: Group { id: 0 },
            spawn_triggered: false,
            variables: HashMap::new(),
            //return_val: Box::new(Value::Null),
            implementations: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Macro {
    pub args: Vec<(String, Option<Value>)>,
    pub def_context: Context,
    pub body: Vec<ast::Statement>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Group(Group),
    Color(Color),
    Block(Block),
    Item(Item),
    Number(f64),
    Bool(bool),
    Func(Group),
    Dict(HashMap<String, StoredValue>),
    Macro(Macro),
    Str(String),
    Array(Vec<Value>),
    Obj(Vec<(u16, String)>),
    Null,
}

#[derive(Clone)]
pub struct Globals {
    pub closed_groups: Vec<u16>,
    pub closed_colors: Vec<u16>,
    pub closed_blocks: Vec<u16>,
    pub closed_items: Vec<u16>,
    pub path: PathBuf,
    pub obj_list: Vec<GDObj>,

    pub highest_x: u32,
    pub stored_values: Vec<Value>,
}

pub fn compile_error(msg: &str, info: CompilerInfo) -> String {
    format!(
        "
{}
-~-~-~-~-~-~-~-~-~-~-~
line: {}, position: {}
Error: \"{}\"

",
        info.path.join("->"),
        info.line.0,
        info.line.1,
        msg
    )
}

/*#[derive(Debug)]
pub enum ValSuccess {
    Literal(Value),
    Evaluatable((Value, Vec<Context>)),
}

use ValSuccess::{Evaluatable, Literal};*/

/*fn add_contexts(evaled: Returns, new_contexts: &mut Vec<Context>) -> Vec<Value> {
    let v = evaled.iter().map(|x| x.0.clone()).collect();
    let c = evaled.iter().map(|x| x.1.clone());
    (*new_contexts).extend(c);
    v
}*/

/*fn into_tuple_vec<T1, T2>(vec1: Vec<T1>, vec2: Vec<T2>) -> Vec<(T1, T2)> {
    if vec1.len() != vec2.len() {
        panic!("not equal length of vectors");
    }
    let mut out: Vec<(T1, T2)> = Vec::new();

    for i in 0..vec1.len() {
        out.push((vec1[i].clone(), vec2[i].clone()));
    }
    out
}*/
impl ast::Expression {
    pub fn eval(
        &self,
        context: Context,
        globals: &mut Globals,
        info: CompilerInfo,
    ) -> (Returns, Returns) {
        //second returns is in case there are any values in the expression that includes a return statement
        let mut vals = self.values.iter();
        let first_value = vals
            .next()
            .unwrap()
            .to_value(context.clone(), globals, info.clone());
        let mut acum = first_value.0;
        let mut inner_returns = first_value.1;

        if self.operators.is_empty() {
            //if only variable
            return (acum, inner_returns);
        }

        for (i, var) in vals.enumerate() {
            let mut new_acum: Returns = Vec::new();
            //every value in acum will be operated with the value of var in the corresponding context
            for (acum_val, c) in acum {
                //what the value in acum becomes
                let evaled = var.to_value(c, globals, info.clone());
                inner_returns.extend(evaled.1);
                for (val, c2) in evaled.0 {
                    let vals: Returns = match self.operators[i].as_ref() {
                        "||" => {
                            if let Some(Value::Macro(m)) =
                                acum_val.member("_or_".to_string(), &context, globals, info.clone())
                            {
                                let (values, _) = execute_macro(
                                    (
                                        m,
                                        vec![ast::Argument {
                                            symbol: None,
                                            value: ast::Expression {
                                                values: vec![ast::Variable {
                                                    value: ast::ValueLiteral::Resolved(val),
                                                    path: Vec::new(),
                                                    operator: None,
                                                }],
                                                operators: Vec::new(),
                                            },
                                        }],
                                    ),
                                    c2,
                                    globals,
                                    acum_val.clone(),
                                    info.next("logical or"),
                                );
                                values
                            } else {
                                if let Value::Bool(b) = &acum_val {
                                    if let Value::Bool(b2) = &val {
                                        vec![(Value::Bool(*b || *b2), c2)]
                                    } else {
                                        panic!(compile_error("This type has no _or_ macro", info))
                                    }
                                } else {
                                    panic!(compile_error(
                                        "This type has no _or_ macro",
                                        info.clone()
                                    ))
                                }
                            }
                        }
                        "&&" => {
                            if let Some(Value::Macro(m)) = acum_val.member(
                                "_and_".to_string(),
                                &context,
                                globals,
                                info.clone(),
                            ) {
                                let (values, _) = execute_macro(
                                    (
                                        m,
                                        vec![ast::Argument {
                                            symbol: None,
                                            value: ast::Expression {
                                                values: vec![ast::Variable {
                                                    value: ast::ValueLiteral::Resolved(val),
                                                    path: Vec::new(),
                                                    operator: None,
                                                }],
                                                operators: Vec::new(),
                                            },
                                        }],
                                    ),
                                    c2,
                                    globals,
                                    acum_val.clone(),
                                    info.next("logical and"),
                                );
                                values
                            } else {
                                if let Value::Bool(b) = &acum_val {
                                    if let Value::Bool(b2) = &val {
                                        vec![(Value::Bool(*b && *b2), c2)]
                                    } else {
                                        panic!(compile_error("This type has no _and_ macro", info))
                                    }
                                } else {
                                    panic!(compile_error(
                                        "This type has no _and_ macro",
                                        info.clone()
                                    ))
                                }
                            }
                        }
                        ">" => {
                            if let Some(Value::Macro(m)) = acum_val.member(
                                "_more_than_".to_string(),
                                &context,
                                globals,
                                info.clone(),
                            ) {
                                let (values, _) = execute_macro(
                                    (
                                        m,
                                        vec![ast::Argument {
                                            symbol: None,
                                            value: ast::Expression {
                                                values: vec![ast::Variable {
                                                    value: ast::ValueLiteral::Resolved(val),
                                                    path: Vec::new(),
                                                    operator: None,
                                                }],
                                                operators: Vec::new(),
                                            },
                                        }],
                                    ),
                                    c2,
                                    globals,
                                    acum_val.clone(),
                                    info.next("comparison"),
                                );
                                values
                            } else {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = val {
                                        vec![(Value::Bool(n > &n2), c2)]
                                    } else {
                                        panic!(compile_error(
                                            "This type has no _more_than_ macro",
                                            info
                                        ))
                                    }
                                } else {
                                    panic!(compile_error(
                                        "This type has no _more_than_ macro",
                                        info.clone()
                                    ))
                                }
                            }
                        }
                        "<" => {
                            if let Some(Value::Macro(m)) = acum_val.member(
                                "_less_than_".to_string(),
                                &context,
                                globals,
                                info.clone(),
                            ) {
                                let (values, _) = execute_macro(
                                    (
                                        m,
                                        vec![ast::Argument {
                                            symbol: None,
                                            value: ast::Expression {
                                                values: vec![ast::Variable {
                                                    value: ast::ValueLiteral::Resolved(val),
                                                    path: Vec::new(),
                                                    operator: None,
                                                }],
                                                operators: Vec::new(),
                                            },
                                        }],
                                    ),
                                    c2,
                                    globals,
                                    acum_val.clone(),
                                    info.next("comparison"),
                                );
                                values
                            } else {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = &val {
                                        vec![(Value::Bool(n < n2), c2)]
                                    } else {
                                        panic!(compile_error(
                                            "This type has no _less_than_ macro",
                                            info
                                        ))
                                    }
                                } else {
                                    panic!(compile_error(
                                        "This type has no _less_than_ macro",
                                        info.clone()
                                    ))
                                }
                            }
                        }
                        ">=" => {
                            if let Some(Value::Macro(m)) = acum_val.member(
                                "_more_or_equal_".to_string(),
                                &context,
                                globals,
                                info.clone(),
                            ) {
                                let (values, _) = execute_macro(
                                    (
                                        m,
                                        vec![ast::Argument {
                                            symbol: None,
                                            value: ast::Expression {
                                                values: vec![ast::Variable {
                                                    value: ast::ValueLiteral::Resolved(val),
                                                    path: Vec::new(),
                                                    operator: None,
                                                }],
                                                operators: Vec::new(),
                                            },
                                        }],
                                    ),
                                    c2,
                                    globals,
                                    acum_val.clone(),
                                    info.next("comparison"),
                                );
                                values
                            } else {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = &val {
                                        vec![(Value::Bool(n >= n2), c2)]
                                    } else {
                                        panic!(compile_error(
                                            "This type has no _more_or_equal_ macro",
                                            info
                                        ))
                                    }
                                } else {
                                    panic!(compile_error(
                                        "This type has no _more_or_equal_ macro",
                                        info.clone()
                                    ))
                                }
                            }
                        }
                        "<=" => {
                            if let Some(Value::Macro(m)) = acum_val.member(
                                "_less_or_equal_".to_string(),
                                &context,
                                globals,
                                info.clone(),
                            ) {
                                let (values, _) = execute_macro(
                                    (
                                        m,
                                        vec![ast::Argument {
                                            symbol: None,
                                            value: ast::Expression {
                                                values: vec![ast::Variable {
                                                    value: ast::ValueLiteral::Resolved(val),
                                                    path: Vec::new(),
                                                    operator: None,
                                                }],
                                                operators: Vec::new(),
                                            },
                                        }],
                                    ),
                                    c2,
                                    globals,
                                    acum_val.clone(),
                                    info.next("comparison"),
                                );
                                values
                            } else {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = &val {
                                        vec![(Value::Bool(n <= n2), c2)]
                                    } else {
                                        panic!(compile_error(
                                            "This type has no _less_or_equal_ macro",
                                            info
                                        ))
                                    }
                                } else {
                                    panic!(compile_error(
                                        "This type has no _less_or_equal_ macro",
                                        info.clone()
                                    ))
                                }
                            }
                        }
                        "/" => {
                            if let Some(Value::Macro(m)) = acum_val.member(
                                "_divided_by_".to_string(),
                                &context,
                                globals,
                                info.clone(),
                            ) {
                                let (values, _) = execute_macro(
                                    (
                                        m,
                                        vec![ast::Argument {
                                            symbol: None,
                                            value: ast::Expression {
                                                values: vec![ast::Variable {
                                                    value: ast::ValueLiteral::Resolved(val),
                                                    path: Vec::new(),
                                                    operator: None,
                                                }],
                                                operators: Vec::new(),
                                            },
                                        }],
                                    ),
                                    c2,
                                    globals,
                                    acum_val.clone(),
                                    info.next("divide"),
                                );
                                values
                            } else {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = &val {
                                        vec![(Value::Number(n / n2), c2)]
                                    } else {
                                        panic!(compile_error(
                                            "This type has no _divided_by_ macro",
                                            info
                                        ))
                                    }
                                } else {
                                    panic!(compile_error(
                                        "This type has no _divided_by_ macro",
                                        info.clone()
                                    ))
                                }
                            }
                        }
                        "*" => {
                            if let Some(Value::Macro(m)) = acum_val.member(
                                "_times_".to_string(),
                                &context,
                                globals,
                                info.clone(),
                            ) {
                                let (values, _) = execute_macro(
                                    (
                                        m,
                                        vec![ast::Argument {
                                            symbol: None,
                                            value: ast::Expression {
                                                values: vec![ast::Variable {
                                                    value: ast::ValueLiteral::Resolved(val),
                                                    path: Vec::new(),
                                                    operator: None,
                                                }],
                                                operators: Vec::new(),
                                            },
                                        }],
                                    ),
                                    c2,
                                    globals,
                                    acum_val.clone(),
                                    info.next("multiply"),
                                );
                                values
                            } else {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = &val {
                                        vec![(Value::Number(n * n2), c2)]
                                    } else {
                                        panic!(compile_error(
                                            "This type has no _times_ macro",
                                            info
                                        ))
                                    }
                                } else {
                                    panic!(compile_error(
                                        "This type has no _times_ macro",
                                        info.clone()
                                    ))
                                }
                            }
                        }
                        "+" => {
                            if let Some(Value::Macro(m)) = acum_val.member(
                                "_plus_".to_string(),
                                &context,
                                globals,
                                info.clone(),
                            ) {
                                let (values, _) = execute_macro(
                                    (
                                        m,
                                        vec![ast::Argument {
                                            symbol: None,
                                            value: ast::Expression {
                                                values: vec![ast::Variable {
                                                    value: ast::ValueLiteral::Resolved(val),
                                                    path: Vec::new(),
                                                    operator: None,
                                                }],
                                                operators: Vec::new(),
                                            },
                                        }],
                                    ),
                                    c2,
                                    globals,
                                    acum_val.clone(),
                                    info.next("plus"),
                                );
                                values
                            } else {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = &val {
                                        vec![(Value::Number(n + n2), c2)]
                                    } else {
                                        panic!(compile_error("This type has no _plus_ macro", info))
                                    }
                                } else {
                                    panic!(compile_error(
                                        "This type has no _plus_ macro",
                                        info.clone()
                                    ))
                                }
                            }
                        }
                        "-" => {
                            if let Some(Value::Macro(m)) = acum_val.member(
                                "_minus_".to_string(),
                                &context,
                                globals,
                                info.clone(),
                            ) {
                                let (values, _) = execute_macro(
                                    (
                                        m,
                                        vec![ast::Argument {
                                            symbol: None,
                                            value: ast::Expression {
                                                values: vec![ast::Variable {
                                                    value: ast::ValueLiteral::Resolved(val),
                                                    path: Vec::new(),
                                                    operator: None,
                                                }],
                                                operators: Vec::new(),
                                            },
                                        }],
                                    ),
                                    c2,
                                    globals,
                                    acum_val.clone(),
                                    info.next("subtract"),
                                );
                                values
                            } else {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = &val {
                                        vec![(Value::Number(n - n2), c2)]
                                    } else {
                                        panic!(compile_error(
                                            "This type has no _minus_ macro",
                                            info
                                        ))
                                    }
                                } else {
                                    panic!(compile_error(
                                        "This type has no _minus_ macro",
                                        info.clone()
                                    ))
                                }
                            }
                        }
                        "==" => vec![(Value::Bool(val == acum_val), c2)],
                        "!=" => vec![(Value::Bool(val != acum_val), c2)],
                        "->" => vec![(
                            Value::Array(
                                (match &acum_val {
                                    Value::Number(n) => *n as i32,
                                    _ => panic!(compile_error("Both sides must be numbers", info)),
                                }..match val {
                                    Value::Number(n) => n as i32,
                                    _ => panic!(compile_error("Both sides must be numbers", info)),
                                })
                                    .map(|x| Value::Number(x as f64))
                                    .collect(),
                            ),
                            c2,
                        )],
                        _ => unreachable!(),
                    };
                    new_acum.extend(vals);
                }
            }
            acum = new_acum;
        }
        (acum, inner_returns)
    }
}

pub fn execute_macro(
    (m, args): (Macro, Vec<ast::Argument>),
    context: Context,
    globals: &mut Globals,
    parent: Value,
    info: CompilerInfo,
) -> (Returns, Returns) {
    // second returns is for any compound statements in the args
    let (evaled_args, inner_returns) = all_combinations(
        args.iter().map(|x| x.value.clone()).collect(),
        context.clone(),
        globals,
        info.clone(),
    );
    let mut new_contexts: Vec<Context> = Vec::new();

    for (arg_values, mut new_context) in evaled_args {
        new_context.variables = m.def_context.variables.clone();
        let mut new_variables: HashMap<String, StoredValue> = HashMap::new();

        //parse each argument given into a local macro variable
        for (i, arg) in args.iter().enumerate() {
            match &arg.symbol {
                Some(name) => {
                    if m.args.iter().any(|e| e.0 == *name) {
                        new_variables
                            .insert(name.clone(), store_value(arg_values[i].clone(), globals));
                    } else {
                        panic!(compile_error(
                            "This macro has no argument with this name!",
                            info
                        ))
                    }
                }
                None => {
                    new_variables.insert(
                        m.args[if m.args[0].0 == "self" { i + 1 } else { i }]
                            .0
                            .clone(),
                        store_value(arg_values[i].clone(), globals),
                    );
                }
            }
        }
        //insert defaults and check non-optional arguments
        let mut m_args_iter = m.args.iter();
        if m.args[0].0 == "self" {
            new_variables.insert("self".to_string(), store_value(parent.clone(), globals));
            m_args_iter.next();
        }
        for arg in m_args_iter {
            if !new_variables.contains_key(&arg.0) {
                match &arg.1 {
                    Some(default) => {
                        new_variables.insert(arg.0.clone(), store_value(default.clone(), globals));
                    }

                    None => panic!(compile_error(
                        &format!("Non-optional argument '{}' not satisfied!", arg.0),
                        info
                    )),
                }
            }
        }

        new_context.variables.extend(new_variables);

        new_contexts.push(new_context);
    }
    let compiled = compile_scope(&m.body, new_contexts, globals, info.next("macro body"));
    let returns = if compiled.1.is_empty() {
        compiled
            .0
            .iter()
            .map(|x| (Value::Null, x.clone()))
            .collect()
    } else {
        compiled.1
    };
    (
        returns
            .iter()
            .map(|x| {
                (
                    x.0.clone(),
                    Context {
                        variables: context.variables.clone(),
                        ..x.1.clone()
                    },
                )
            })
            .collect(),
        inner_returns,
    )
}

fn all_combinations(
    a: Vec<ast::Expression>,
    context: Context,
    globals: &mut Globals,
    info: CompilerInfo,
) -> (Vec<(Vec<Value>, Context)>, Returns) {
    let mut out: Vec<(Vec<Value>, Context)> = Vec::new();
    let mut inner_returns = Returns::new();
    if a.is_empty() {
        //if there are so value, there is only one combination
        out.push((Vec::new(), context));
    } else {
        let mut a_iter = a.iter();
        //starts with all the combinations of the first expr
        let (start_values, start_returns) =
            a_iter.next().unwrap().eval(context, globals, info.clone());
        out.extend(
            start_values
                .iter()
                .map(|x| (vec![x.0.clone()], x.1.clone())),
        );
        inner_returns.extend(start_returns);
        //for the rest of the expressions
        for expr in a_iter {
            //all the new combinations end up in this
            let mut new_out: Vec<(Vec<Value>, Context)> = Vec::new();
            //run through all the lists in out
            for (inner_arr, c) in out.iter() {
                //for each one, run through all the returns in that context
                let (values, returns) = expr.eval(c.clone(), globals, info.clone());
                inner_returns.extend(returns);
                for (v, c2) in values.iter() {
                    //push a new list with each return pushed to it
                    new_out.push((
                        {
                            let mut new_arr = inner_arr.clone();
                            new_arr.push(v.clone());
                            new_arr
                        },
                        c2.clone(),
                    ));
                }
            }
            //set out to this new one and repeat
            out = new_out;
        }
    }
    (out, inner_returns)
}
pub fn eval_dict(
    dict: Vec<ast::DictDef>,
    context: Context,
    globals: &mut Globals,
    info: CompilerInfo,
) -> (Returns, Returns) {
    let mut inner_returns = Returns::new();
    let (evaled, returns) = all_combinations(
        dict.iter()
            .map(|def| match def {
                ast::DictDef::Def(d) => d.1.clone(),
                ast::DictDef::Extract(e) => e.clone(),
            })
            .collect(),
        context,
        globals,
        info.clone(),
    );
    inner_returns.extend(returns);
    let mut out = Returns::new();
    for expressions in evaled {
        let mut expr_index: usize = 0;
        let mut dict_out: HashMap<String, StoredValue> = HashMap::new();
        for def in dict.clone() {
            match def {
                ast::DictDef::Def(d) => {
                    dict_out.insert(
                        d.0.clone(),
                        store_value(expressions.0[expr_index].clone(), globals),
                    );
                }
                ast::DictDef::Extract(_) => {
                    dict_out.extend(match &expressions.0[expr_index] {
                        Value::Dict(d) => d.clone(),
                        _ => panic!(compile_error("Cannot extract from this value", info)),
                    });
                }
            };
            expr_index += 1;
        }
        out.push((Value::Dict(dict_out), expressions.1));
    }
    (out, inner_returns)
}

impl ast::Variable {
    pub fn to_value(
        &self,
        context: Context,
        globals: &mut Globals,
        info: CompilerInfo,
    ) -> (Returns, Returns) {
        // TODO: Check if this variable has native functions called on it, and if not set this to false
        let mut start_val: Vec<(Value, Context)> = Vec::new();
        let mut inner_returns = Returns::new();

        match &self.value {
            ast::ValueLiteral::Resolved(r) => start_val.push((r.clone(), context)),
            ast::ValueLiteral::ID(id) => start_val.push((
                match id.class_name.as_ref() {
                    "g" => {
                        if id.unspecified {
                            Value::Group(Group {
                                id: next_free(&mut globals.closed_groups),
                            })
                        } else {
                            Value::Group(Group { id: id.number })
                        }
                    }
                    "c" => {
                        if id.unspecified {
                            Value::Color(Color {
                                id: next_free(&mut globals.closed_colors),
                            })
                        } else {
                            Value::Color(Color { id: id.number })
                        }
                    }
                    "b" => {
                        if id.unspecified {
                            Value::Block(Block {
                                id: next_free(&mut globals.closed_blocks),
                            })
                        } else {
                            Value::Block(Block { id: id.number })
                        }
                    }
                    "i" => {
                        if id.unspecified {
                            Value::Item(Item {
                                id: next_free(&mut globals.closed_items),
                            })
                        } else {
                            Value::Item(Item { id: id.number })
                        }
                    }
                    _ => unreachable!(),
                },
                context,
            )),
            ast::ValueLiteral::Number(num) => start_val.push((Value::Number(*num), context)),
            ast::ValueLiteral::Dictionary(dict) => {
                let (new_out, new_inner_returns) =
                    eval_dict(dict.clone(), context, globals, info.next("dictionary"));
                start_val = new_out;
                inner_returns = new_inner_returns;
            }
            ast::ValueLiteral::CmpStmt(cmp_stmt) => {
                let (evaled, returns) = cmp_stmt.to_scope(&context, globals, info.clone());
                inner_returns.extend(returns);
                start_val.push((Value::Func(evaled), context));
            }

            ast::ValueLiteral::Expression(expr) => {
                let (evaled, returns) = expr.eval(context, globals, info.clone());
                inner_returns.extend(returns);
                start_val.extend(evaled.iter().cloned());
            }

            ast::ValueLiteral::Bool(b) => start_val.push((Value::Bool(*b), context)),
            ast::ValueLiteral::Symbol(string) => match context.variables.get(string) {
                Some(value) => {
                    start_val.push((((*globals).stored_values[*value as usize]).clone(), context))
                }
                None => panic!(compile_error(
                    &format!("The variable \"{}\" does not exist in this Func.", string),
                    info
                )),
            },
            ast::ValueLiteral::Str(s) => start_val.push((Value::Str(s.clone()), context)),
            ast::ValueLiteral::Array(a) => {
                let (evaled, returns) =
                    all_combinations(a.clone(), context, globals, info.next("array"));
                inner_returns.extend(returns);
                start_val = evaled
                    .iter()
                    .map(|x| (Value::Array(x.0.clone()), x.1.clone()))
                    .collect();
            }
            ast::ValueLiteral::Import(i) => {
                let mut new_context = context.clone();
                let (val, imp) = import_module(i, globals, info.clone());
                new_context.implementations.extend(imp);
                start_val.push((val, new_context));
            }
            ast::ValueLiteral::Obj(o) => {
                let mut all_expr: Vec<ast::Expression> = Vec::new();
                for prop in o {
                    all_expr.push(prop.0.clone());
                    all_expr.push(prop.1.clone());
                }
                let (evaled, returns) =
                    all_combinations(all_expr, context, globals, info.next("object"));
                inner_returns.extend(returns);
                for (expressions, context) in evaled {
                    let mut obj: Vec<(u16, String)> = Vec::new();
                    for i in 0..(o.len()) {
                        let v = expressions[i * 2].clone();
                        let v2 = expressions[i * 2 + 1].clone();

                        obj.push((
                            match v {
                                Value::Number(n) => n as u16,
                                _ => panic!(compile_error(
                                    "Expected number as object property",
                                    info
                                )),
                            },
                            match v2 {
                                Value::Number(n) => n.to_string(),
                                Value::Str(s) => s,
                                Value::Func(g) => g.id.to_string(),

                                Value::Group(g) => g.id.to_string(),
                                Value::Color(c) => c.id.to_string(),
                                Value::Block(b) => b.id.to_string(),
                                Value::Item(i) => i.id.to_string(),

                                Value::Bool(b) => {
                                    if b {
                                        "1".to_string()
                                    } else {
                                        "0".to_string()
                                    }
                                }

                                //Value::Array(a) => {} TODO: Add this
                                x => panic!(compile_error(
                                    &format!("{:?} is not a valid object value", x),
                                    info
                                )),
                            },
                        ))
                    }
                    start_val.push((Value::Obj(obj), context));
                }
            }

            ast::ValueLiteral::Macro(m) => {
                let mut all_expr: Vec<ast::Expression> = Vec::new();
                for arg in &m.args {
                    if let Some(e) = &arg.1 {
                        all_expr.push(e.clone());
                    }
                }
                let (argument_possibilities, returns) =
                    all_combinations(all_expr, context, globals, info.next("macro argument"));
                inner_returns.extend(returns);
                for defaults in argument_possibilities {
                    let mut args: Vec<(String, Option<Value>)> = Vec::new();
                    let mut expr_index = 0;
                    for arg in m.args.iter() {
                        args.push((
                            arg.0.clone(),
                            match &arg.1 {
                                Some(_) => {
                                    expr_index += 1;
                                    Some(defaults.0[expr_index - 1].clone())
                                }
                                None => None,
                            },
                        ));
                    }

                    start_val.push((
                        Value::Macro(Macro {
                            args,
                            body: m.body.statements.clone(),
                            def_context: defaults.1.clone(),
                        }),
                        defaults.1,
                    ))
                }
            }
            //ast::ValueLiteral::Resolved(r) => out.push((r.clone(), context)),
            ast::ValueLiteral::Null => start_val.push((Value::Null, context)),
        };

        let mut path_iter = self.path.iter();
        let mut with_parent: Vec<(Value, Context, Value)> = start_val
            .iter()
            .map(|x| (x.0.clone(), x.1.clone(), Value::Null))
            .collect();
        for p in &mut path_iter {
            match &p {
                ast::Path::Member(m) => {
                    with_parent = with_parent
                        .iter()
                        .map(|x| {
                            (
                                x.0.member(m.clone(), &x.1, globals, info.clone()).expect(
                                    &compile_error(
                                        &format!(
                                            "'{}' does not have member '{}'!",
                                            match x.0.member(
                                                "TYPE".to_string(),
                                                &x.1,
                                                globals,
                                                info.clone()
                                            ) {
                                                Some(Value::Str(s)) => s,
                                                _ => unreachable!(),
                                            },
                                            m
                                        ),
                                        info.clone(),
                                    ),
                                ),
                                x.1.clone(),
                                x.0.clone(),
                            )
                        })
                        .collect();
                }

                ast::Path::Index(i) => {
                    let mut new_out: Vec<(Value, Context, Value)> = Vec::new();

                    for (prev_v, prev_c, _) in with_parent.clone() {
                        match prev_v.clone() {
                            Value::Array(arr) => {
                                let (evaled, returns) = i.eval(prev_c, globals, info.next("index"));
                                inner_returns.extend(returns);
                                for index in evaled {
                                    match index.0 {
                                        Value::Number(n) => {
                                            new_out.push((
                                                arr[n as usize].clone(),
                                                index.1,
                                                prev_v.clone(),
                                            ));
                                        }
                                        _ => panic!(compile_error("Index must be a number", info)),
                                    }
                                }
                            }
                            _ => panic!(compile_error("Cannot index this", info)),
                        }
                    }
                }

                ast::Path::Call(args) => {
                    for (v, cont, parent) in with_parent.clone().iter() {
                        match v {
                            Value::Macro(m) => {
                                let (evaled, returns) = execute_macro(
                                    (m.clone(), args.clone()),
                                    cont.clone(),
                                    globals,
                                    parent.clone(),
                                    info.clone(),
                                );
                                inner_returns.extend(returns);
                                with_parent = evaled
                                    .iter()
                                    .map(|x| (x.0.clone(), x.1.clone(), v.clone()))
                                    .collect();
                            }
                            _ => panic!(compile_error("not a macro", info)),
                        }
                    }
                }
            };
        }

        let mut out: Returns = with_parent
            .iter()
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect();

        if let Some(o) = &self.operator {
            for final_value in &mut out {
                match o.as_ref() {
                    "-" => {
                        if let Value::Number(n) = final_value.0 {
                            *final_value = (Value::Number(-n), final_value.1.clone());
                        } else {
                            panic!(compile_error("Cannot make non number type negative!", info))
                        }
                    }

                    "!" => {
                        if let Value::Bool(b) = final_value.0 {
                            *final_value = (Value::Bool(!b), final_value.1.clone());
                        } else {
                            panic!(compile_error("Cannot nagate non-boolean value", info))
                        }
                    }

                    _ => unreachable!(),
                }
            }
        }

        (out, inner_returns)
    }
}

impl ast::CompoundStatement {
    fn to_scope(
        &self,
        context: &Context,
        globals: &mut Globals,
        info: CompilerInfo,
    ) -> (Group, Returns) {
        //create the function context
        let mut new_context = context.clone();

        new_context.spawn_triggered = true;

        //pick a start group
        let start_group = Group {
            id: next_free(&mut globals.closed_groups),
        };

        new_context.start_group = start_group;

        (
            start_group,
            compile_scope(
                &self.statements,
                vec![new_context],
                globals,
                info.next("function body"),
            )
            .1,
        )
    }
}
