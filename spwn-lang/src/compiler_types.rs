use crate::ast;
use crate::levelstring::*;
use crate::native::*;
use std::boxed::Box;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::compiler::{compile_scope, import_module, next_free};

#[derive(Clone, PartialEq)]
pub struct Context {
    pub x: u32,
    pub y: u16,
    pub start_group: Group,
    pub spawn_triggered: bool,
    pub variables: HashMap<String, Value>,
    pub return_val: Box<Value>,
}
use std::fmt;
impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CONTEXT")
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
            return_val: Box::new(Value::Null),
        }
    }
}

/*#[derive(Clone, Debug, PartialEq)]
pub struct Return {
    pub context: Context,
    pub statements: Vec<ast::Statement>,
}*/

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub group: Group,
    pub members: HashMap<String, Value>,
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
    Scope(Scope),
    Dict(HashMap<String, Value>),
    Macro(Macro),
    Str(String),
    Array(Vec<Value>),
    Obj(Vec<(u16, String)>),
    Null,
}

impl Value {
    pub fn to_variable(&self) -> ast::Variable {
        ast::Variable {
            value: ast::ValueLiteral::Resolved(self.clone()),
            path: Vec::new(),
            operator: None,
        }
    }
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

    pub implementations: HashMap<String, HashMap<String, Value>>,
}

/*#[derive(Debug)]
pub enum ValSuccess {
    Literal(Value),
    Evaluatable((Value, Vec<Context>)),
}

use ValSuccess::{Evaluatable, Literal};*/

fn add_contexts(evaled: Vec<(Value, Context)>, new_contexts: &mut Vec<Context>) -> Vec<Value> {
    let v = evaled.iter().map(|x| x.0).collect();
    let c = evaled.iter().map(|x| x.1);
    (*new_contexts).extend(c);
    v
}

fn into_tuple_vec<T1, T2>(vec1: Vec<T1>, vec2: Vec<T2>) -> Vec<(T1, T2)> {
    if vec1.len() != vec2.len() {
        panic!("not equal length of vectors");
    }
    let out: Vec<(T1, T2)> = Vec::new();

    for i in 0..vec1.len() {
        out.push((vec1[i], vec2[i]));
    }
    out
}
impl ast::Expression {
    pub fn eval(&self, contexts: Vec<Context>, globals: &mut Globals) -> Vec<(Value, Context)> {
        let mut new_contexts: Vec<Context> = contexts;
        let mut vals = self.values.iter();
        let mut acum = add_contexts(
            vals.next().unwrap().to_value(contexts, globals),
            &mut new_contexts,
        );

        if self.operators.is_empty() {
            return into_tuple_vec(acum, new_contexts);
        }

        for (i, var) in vals.enumerate() {
            acum = acum.iter().map(|tup| {
                let acum_val = tup;
                
                match self.operators[i].as_ref() {
                    "||" | "&&" => {
                        //boolean operations
                        if let Value::Bool(b) = val {
                            if let Value::Bool(a) = acum_val {
                                match self.operators[i].as_ref() {
                                    "||" => acum_val = Value::Bool(a || b),
                                    "&&" => acum_val = Value::Bool(a && b),
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
                                    ">" => acum_val = Value::Bool(a > num),
                                    "<" => acum_val = Value::Bool(a < num),
                                    ">=" => acum_val = Value::Bool(a >= num),
                                    "<=" => acum_val = Value::Bool(a <= num),
                                    "/" => acum_val = Value::Number(a / num),
                                    "*" => acum_val = Value::Number(a * num),
                                    "+" => acum_val = Value::Number(a + num),
                                    "-" => acum_val = Value::Number(a - num),
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
                    "==" => {
                        acum_val = Value::Bool(val == acum_val);
                    }
                    "!=" => {
                        acum_val = Value::Bool(val != acum_val);
                    }
                    "->" => {
                        acum_val = Value::Array(
                            (match acum_val {
                                Value::Number(n) => n as i32,
                                _ => panic!("Both sides must be numbers"),
                            }..match val {
                                Value::Number(n) => n as i32,
                                _ => panic!("Both sides must be numbers"),
                            })
                                .map(|x| Value::Number(x as f64))
                                .collect(),
                        )
                    }
                    _ => unreachable!(),
                }
                ()
            }).collect();
        }

        (acum, new_contexts)
    }
}

pub fn execute_macro(
    (m, args): (Macro, Vec<(Option<String>, Value)>),
    context: &Context,
    globals: &mut Globals,
) -> Vec<Context> {
    let mut new_context = context.clone();
    new_context.variables = m.def_context.variables;

    let mut new_variables: HashMap<String, Value> = HashMap::new();

    //parse each argument given into a local macro variable
    for (i, arg) in args.iter().enumerate() {
        match &arg.0 {
            Some(name) => {
                if m.args.iter().any(|e| e.0 == *name) {
                    new_variables.insert(name.clone(), arg.1.clone());
                } else {
                    panic!("This function has no argument with this name!")
                }
            }
            None => {
                new_variables.insert(m.args[i].0.clone(), arg.1.clone());
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

    //new_context.y -= 30;
    compile_scope(&m.body, vec![new_context], globals).1
}

impl ast::Variable {
    pub fn to_value(&self, contexts: Vec<Context>, globals: &mut Globals) -> Vec<(Value, Context)> {
        // TODO: Check if this variable has native functions called on it, and if not set this to false
        let mut new_contexts: Vec<Context> = Vec::new();

        let mut final_value = match &self.value {
            ast::ValueLiteral::ID(id) => match id.class_name.as_ref() {
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
            ast::ValueLiteral::Number(num) => Value::Number(*num),
            ast::ValueLiteral::Dictionary(dict) => Value::Dict(dict.read(*context, globals)),
            ast::ValueLiteral::CmpStmt(cmp_stmt) => {
                Value::Scope(cmp_stmt.to_scope(context, globals))
            }

            ast::ValueLiteral::Expression(expr) => {
                add_contexts(expr.eval(context, globals), &mut new_contexts)
            }

            ast::ValueLiteral::Bool(b) => Value::Bool(*b),
            ast::ValueLiteral::Symbol(string) => match context.variables.get(string) {
                Some(value) => (value).clone(),
                None => panic!(format!(
                    "The variable {} does not exist in this scope.",
                    string
                )),
            },
            ast::ValueLiteral::Str(s) => Value::Str(s.clone()),
            ast::ValueLiteral::Array(a) => {
                let mut arr: Vec<Value> = Vec::new();
                let mut a_iter = a.iter();
                for val in &mut a_iter {
                    arr.push(add_contexts(val.eval(context, globals), &mut new_contexts));
                }
                Value::Array(arr)
            }
            ast::ValueLiteral::Import(i) => import_module(i, globals),
            ast::ValueLiteral::Obj(o) => Value::Obj({
                let mut obj: Vec<(u16, String)> = Vec::new();
                let mut o_iter = o.iter();
                for prop in &mut o_iter {
                    let v = add_contexts(prop.0.eval(context, globals), &mut new_contexts);
                    let v2 = add_contexts(prop.1.eval(context, globals), &mut new_contexts);

                    obj.push((
                        match v {
                            Value::Number(n) => n as u16,
                            _ => panic!("Expected number as object property"),
                        },
                        match v2 {
                            Value::Number(n) => n.to_string(),
                            Value::Str(s) => s,
                            Value::Scope(s) => s.group.id.to_string(),

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
                obj
            }),

            ast::ValueLiteral::Macro(m) => Value::Macro(Macro {
                args: {
                    let mut args: Vec<(String, Option<Value>)> = Vec::new();
                    for arg in m.args.iter() {
                        args.push((
                            arg.0.clone(),
                            match &arg.1 {
                                Some(expr) => {
                                    let (v, c) = expr.eval(context, globals);
                                    new_contexts.extend(c.iter().cloned());
                                    Some(v)
                                }
                                None => None,
                            },
                        ));
                    }

                    args
                },
                def_context: context.clone(),
                body: m.body.statements.clone(),
            }),

            ast::ValueLiteral::Resolved(r) => r.clone(),

            ast::ValueLiteral::Null => Value::Null,
        };
        let mut path_iter = self.path.iter();
        let mut parent = Value::Null;
        for p in &mut path_iter {
            let stored = final_value.clone();
            match p {
                ast::Path::Member(m) => final_value = final_value.member(m.clone(), globals),

                ast::Path::Index(i) => {
                    let v = add_contexts(i.eval(context, globals), &mut new_contexts);
                    final_value = match &final_value {
                        Value::Array(a) => a[match v {
                            Value::Number(n) => n,
                            _ => panic!("Expected number"),
                        } as usize]
                            .clone(),
                        _ => panic!("Can't index non-iteratable variable"),
                    }
                }

                ast::Path::Call(c) => {
                    let m = match final_value.clone() {
                        Value::Macro(m) => m,
                        _ => panic!("Not a macro!"),
                    };
                    let mut vals: Vec<(Option<String>, Value)> = Vec::new();
                    let mut c_iter = c.iter();
                    if !m.args.is_empty() && m.args[0].0 == "self" {
                        vals.push((None, parent))
                    }
                    for arg in &mut c_iter {
                        let arg_value = (
                            arg.symbol,
                            add_contexts(arg.value.eval(context, globals), &mut new_contexts),
                        );
                        vals.push(arg_value);
                    }
                    final_value = add_contexts(
                        execute_macro((m, vals), context, globals),
                        &mut new_contexts,
                    );
                }
            }
            parent = stored;
        }

        if let Some(o) = &self.operator {
            match o.as_ref() {
                "-" => {
                    if let Value::Number(n) = &final_value {
                        final_value = Value::Number(-n);
                    } else {
                        panic!("Cannot make non number type negative!")
                    }
                }

                "!" => {
                    if let Value::Bool(b) = &final_value {
                        final_value = Value::Bool(!b);
                    } else {
                        panic!("Cannot make non boolean type oposite!")
                    }
                }

                _ => unreachable!(),
            }
        }

        (final_value, new_contexts)
    }
}

impl ast::CompoundStatement {
    fn to_scope(&self, context: &Context, globals: &mut Globals) -> Scope {
        //create the function context
        let mut new_context = context.clone();

        new_context.spawn_triggered = true;

        //pick a start group
        let start_group = Group {
            id: next_free(&mut globals.closed_groups),
        };

        compile_scope(&self.statements, vec![new_context], globals).0
    }
}

impl ast::Dictionary {
    fn read(&self, context: Context, globals: &mut Globals) -> HashMap<String, Value> {
        compile_scope(&self.members, vec![context], globals)
            .0
            .members
    }
}
