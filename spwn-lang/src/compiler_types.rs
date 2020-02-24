use crate::ast;
use crate::levelstring::*;
use crate::native::*;
//use std::boxed::Box;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::compiler::{compile_scope, import_module, next_free};

pub type Returns = Vec<(Value, Context)>;
pub type Implementations = HashMap<String, HashMap<String, Value>>;

#[derive(PartialEq, Debug, Clone)]
pub struct Context {
    pub x: u32,
    pub y: u16,
    pub start_group: Group,
    pub spawn_triggered: bool,
    pub variables: HashMap<String, Value>,
    pub implementations: Implementations,
}
#[derive(Debug, Clone)]
pub struct CompilerInfo {
    pub depth: u8,
    pub path: Vec<String>,
}

impl CompilerInfo {
    pub fn next(&self, name: &str) -> CompilerInfo {
        let mut new_path = self.path.clone();
        new_path.push(name.to_string());
        CompilerInfo {
            depth: self.depth + 1,
            path: new_path,
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
    Dict(HashMap<String, Value>),
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
            .to_value(context, globals, info.clone());
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
                    new_acum.push((
                        //doing the operation
                        match self.operators[i].as_ref() {
                            "||" | "&&" => {
                                //boolean operations
                                if let Value::Bool(b) = val {
                                    if let Value::Bool(a) = acum_val {
                                        match self.operators[i].as_ref() {
                                            "||" => Value::Bool(a || b),
                                            "&&" => Value::Bool(a && b),
                                            _ => unreachable!(),
                                        }
                                    } else {
                                        panic!("Right side must be boolean")
                                    }
                                } else {
                                    panic!("Both sides must be boolean")
                                }
                            }
                            ">" | "<" | ">=" | "<=" | "/" | "*" | "+" | "-" => {
                                //number operations
                                if let Value::Number(num) = val {
                                    if let Value::Number(a) = acum_val {
                                        match self.operators[i].as_ref() {
                                            ">" => Value::Bool(a > num),
                                            "<" => Value::Bool(a < num),
                                            ">=" => Value::Bool(a >= num),
                                            "<=" => Value::Bool(a <= num),
                                            "/" => Value::Number(a / num),
                                            "*" => Value::Number(a * num),
                                            "+" => Value::Number(a + num),
                                            "-" => Value::Number(a - num),
                                            _ => unreachable!(),
                                        }
                                    } else {
                                        panic!("Right side must be number")
                                    }
                                } else {
                                    panic!("Both sides must be numbers")
                                }
                            }
                            //any
                            "==" => Value::Bool(val == acum_val),
                            "!=" => Value::Bool(val != acum_val),
                            "->" => Value::Array(
                                (match acum_val {
                                    Value::Number(n) => n as i32,
                                    _ => panic!("Both sides must be numbers"),
                                }..match val {
                                    Value::Number(n) => n as i32,
                                    _ => panic!("Both sides must be numbers"),
                                })
                                    .map(|x| Value::Number(x as f64))
                                    .collect(),
                            ),
                            _ => unreachable!(),
                        },
                        c2,
                    ));
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
    info: CompilerInfo,
) -> (Returns, Returns) {
    // second returns is for any compound statements in the args
    let (evaled_args, inner_returns) = all_combinations(
        args.iter().map(|x| x.value.clone()).collect(),
        context,
        globals,
        info.clone(),
    );
    let mut new_contexts: Vec<Context> = Vec::new();

    for (arg_values, mut new_context) in evaled_args {
        new_context.variables = m.def_context.variables.clone();
        let mut new_variables: HashMap<String, Value> = HashMap::new();

        //parse each argument given into a local macro variable
        for (i, arg) in args.iter().enumerate() {
            match &arg.symbol {
                Some(name) => {
                    if m.args.iter().any(|e| e.0 == *name) {
                        new_variables.insert(name.clone(), arg_values[i].clone());
                    } else {
                        panic!("This function has no argument with this name!")
                    }
                }
                None => {
                    new_variables.insert(m.args[i].0.clone(), arg_values[i].clone());
                }
            }
        }
        //insert defaults and check non-optional arguments
        for arg in m.args.iter() {
            if !new_variables.contains_key(&arg.0) {
                match &arg.1 {
                    Some(default) => {
                        new_variables.insert(arg.0.clone(), default.clone());
                    }

                    None => panic!("Non-optional argument '{}' not satisfied!", arg.0),
                }
            }
        }

        new_context.variables.extend(new_variables);

        new_contexts.push(new_context);
    }
    (
        compile_scope(&m.body, new_contexts, globals, info.next("macro body")).1,
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
        info,
    );
    inner_returns.extend(returns);
    let mut out = Returns::new();
    for expressions in evaled {
        let mut expr_index: usize = 0;
        let mut dict_out: HashMap<String, Value> = HashMap::new();
        for def in dict.clone() {
            match def {
                ast::DictDef::Def(d) => {
                    dict_out.insert(d.0.clone(), expressions.0[expr_index].clone());
                }
                ast::DictDef::Extract(_) => {
                    dict_out.extend(match &expressions.0[expr_index] {
                        Value::Dict(d) => d.clone(),
                        _ => panic!("Cannot extract from this value"),
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
        let mut out: Returns = Vec::new();
        let mut inner_returns = Returns::new();

        match &self.value {
            ast::ValueLiteral::ID(id) => out.push((
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
            ast::ValueLiteral::Number(num) => out.push((Value::Number(*num), context)),
            ast::ValueLiteral::Dictionary(dict) => {
                let (new_out, new_inner_returns) =
                    eval_dict(dict.clone(), context, globals, info.next("dictionary"));
                out = new_out;
                inner_returns = new_inner_returns;
            }
            ast::ValueLiteral::CmpStmt(cmp_stmt) => {
                let (evaled, returns) = cmp_stmt.to_scope(&context, globals, info.clone());
                inner_returns.extend(returns);
                out.push((Value::Func(evaled), context));
            }

            ast::ValueLiteral::Expression(expr) => {
                let (evaled, returns) = expr.eval(context, globals, info.clone());
                inner_returns.extend(returns);
                out.extend(evaled.iter().cloned());
            }

            ast::ValueLiteral::Bool(b) => out.push((Value::Bool(*b), context)),
            ast::ValueLiteral::Symbol(string) => match context.variables.get(string) {
                Some(value) => out.push(((*value).clone(), context)),
                None => panic!(format!(
                    "The variable {} does not exist in this Func.",
                    string
                )),
            },
            ast::ValueLiteral::Str(s) => out.push((Value::Str(s.clone()), context)),
            ast::ValueLiteral::Array(a) => {
                let (evaled, returns) =
                    all_combinations(a.clone(), context, globals, info.next("array"));
                inner_returns.extend(returns);
                out = evaled
                    .iter()
                    .map(|x| (Value::Array(x.0.clone()), x.1.clone()))
                    .collect();
            }
            ast::ValueLiteral::Import(i) => {
                let mut new_context = context.clone();
                let (val, imp) = import_module(i, globals, info.clone());
                new_context.implementations.extend(imp);
                out.push((val, new_context));
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
                    for i in 0..(o.len() - 1) {
                        let v = expressions[i * 2].clone();
                        let v2 = expressions[i * 2 + 1].clone();

                        obj.push((
                            match v {
                                Value::Number(n) => n as u16,
                                _ => panic!("Expected number as object property"),
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
                                x => panic!("{:?} is not a valid object value", x),
                            },
                        ))
                    }
                    out.push((Value::Obj(obj), context));
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

                    out.push((
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
            ast::ValueLiteral::Null => out.push((Value::Null, context)),
        };
        let mut path_iter = self.path.iter();
        let mut parent = out.clone();
        for p in &mut path_iter {
            let stored = out.clone();

            match &p {
                ast::Path::Member(m) => {
                    out = out
                        .iter()
                        .map(|x| (x.0.member(m.clone(), &x.1), x.1.clone()))
                        .collect();
                }

                ast::Path::Index(i) => {
                    let mut new_out: Returns = Vec::new();

                    for (prev_v, prev_c) in out.clone() {
                        match prev_v {
                            Value::Array(arr) => {
                                let (evaled, returns) = i.eval(prev_c, globals, info.next("index"));
                                inner_returns.extend(returns);
                                for index in evaled {
                                    match index.0 {
                                        Value::Number(n) => {
                                            new_out.push((arr[n as usize].clone(), index.1));
                                        }
                                        _ => panic!("Index must be a number"),
                                    }
                                }
                            }
                            _ => panic!("Cannot index this"),
                        }
                    }
                }

                ast::Path::Call(args) => {
                    for (v, cont) in out.clone() {
                        match v {
                            Value::Macro(m) => {
                                let (evaled, returns) =
                                    execute_macro((m, args.clone()), cont, globals, info.clone());
                                inner_returns.extend(returns);
                                out = evaled;
                            }
                            _ => panic!("not a macro"),
                        }
                    }
                }
            };

            parent = stored;
        }

        if let Some(o) = &self.operator {
            for final_value in &mut out {
                match o.as_ref() {
                    "-" => {
                        if let Value::Number(n) = final_value.0 {
                            *final_value = (Value::Number(-n), final_value.1.clone());
                        } else {
                            panic!("Cannot make non number type negative!")
                        }
                    }

                    "!" => {
                        if let Value::Bool(b) = final_value.0 {
                            *final_value = (Value::Bool(!b), final_value.1.clone());
                        } else {
                            panic!("Cannot nagate non-boolean value")
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
