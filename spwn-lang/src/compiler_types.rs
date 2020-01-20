use crate::ast;
use crate::levelstring::*;
use crate::native::*;
use std::boxed::Box;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::compiler::{compile_scope, import_module, next_free};

#[derive(Clone, Debug, PartialEq)]
pub struct Context {
    pub x: u32,
    pub y: u16,
    pub added_groups: Vec<Group>,
    pub spawn_triggered: bool,
    pub variables: HashMap<String, Value>,
    pub ret: Option<Box<Return>>,
}

impl Context {
    pub fn move_down(&self) -> Context {
        Context {
            y: self.y - 30,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Return {
    pub context: Context,
    pub call_statement: ast::Statement,
    pub statements: Vec<ast::Statement>,
}

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

pub enum ValSuccess {
    Literal(Value),
    Evaluatable(ast::Expression),
}

use ValSuccess::{Literal, Evaluatable};

impl ast::Expression {
    pub fn eval(&self, context: &Context, globals: &mut Globals) -> ValSuccess {
        let mut vals = self.values.iter();
        let mut acum = match vals.next().unwrap().to_value(context, globals) {
            Literal(l) => l,
            Evaluatable(e) => return Evaluatable(e)
        };

        if self.operators.is_empty() {
            return Literal(acum);
        }

        for (i, var) in vals.enumerate() {
            let val = match var.to_value(context, globals) {
                Literal(l) => l,
                Evaluatable(e) => return Evaluatable(e)
            };

            match self.operators[i].as_ref() {
                "||" | "&&" => {
                    //boolean operations
                    if let Value::Bool(b) = val {
                        if let Value::Bool(a) = acum {
                            match self.operators[i].as_ref() {
                                "||" => acum = Value::Bool(a || b),
                                "&&" => acum = Value::Bool(a && b),
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
                        if let Value::Number(a) = acum {
                            match self.operators[i].as_ref() {
                                ">" => acum = Value::Bool(a > num),
                                "<" => acum = Value::Bool(a < num),
                                ">=" => acum = Value::Bool(a >= num),
                                "<=" => acum = Value::Bool(a <= num),
                                "/" => acum = Value::Number(a / num),
                                "*" => acum = Value::Number(a * num),
                                "+" => acum = Value::Number(a + num),
                                "-" => acum = Value::Number(a - num),
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
                    acum = Value::Bool(val == acum);
                }

                "!=" => {
                    acum = Value::Bool(val != acum);
                }

                _ => unreachable!(),
            }
        }

        Literal(acum)
    }
}



impl ast::Variable {
    pub fn to_value(&self, context: &Context, globals: &mut Globals) -> ValSuccess {
        // TODO: Check if this variable has native functions called on it, and if not set this to false

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
            ast::ValueLiteral::Dictionary(dict) => Value::Dict(dict.read(&context, globals)),
            ast::ValueLiteral::CmpStmt(cmp_stmt) => {
                Value::Scope(cmp_stmt.to_scope(context, globals))
            }

            ast::ValueLiteral::Expression(expr) => match expr.eval(&context, globals) {
                Literal(l) => l,
                Evaluatable(e) => return Evaluatable(e),
            },

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

                for val in a.iter() {
                    arr.push(match val.eval(&context, globals) {
                        Literal(l) => l,
                        Evaluatable(e) => return Evaluatable(e),
                    });
                }
                Value::Array(arr)
            }
            ast::ValueLiteral::Import(i) => import_module(i, globals),
            ast::ValueLiteral::Obj(o) => Value::Obj(
                o.iter()
                    .map(|prop| {
                        (
                            match prop.0.eval(&context, globals) {
                                Literal(Value::Number(n)) => n as u16,
                                _ => panic!("Expected Literal number as object property (too lazy to change this rn)"),
                            },
                            match prop.1.eval(&context, globals) {
                                Literal(Value::Number(n)) => n.to_string(),
                                Literal(Value::Str(s)) => s,
                                //Value::Array(a) => {} TODO: Add this
                                _ => panic!("Not a valid object value"),
                            },
                        )
                    })
                    .collect(),
            ),

            ast::ValueLiteral::Macro(m) => Value::Macro(Macro {
                args: {
                    let mut args: Vec<(String, Option<Value>)> = Vec::new();
                    for arg in m.args.iter() {
                        
                        args.push((arg.0.clone(),
                        match &arg.1 {
                            Some(expr) => Some(match expr.eval(&context, globals) {
                                Literal(l) => l,
                                Evaluatable(e) => return Evaluatable(e),
                            }),
                            None => None,
                        }));
                        
                        
                    }

                    args
                    
                },
                def_context: context.clone(),
                body: m.body.statements.clone(),
            }),

            ast::ValueLiteral::Null => Value::Null,
            ast::ValueLiteral::PLACEHOLDER => unreachable!()
        };

        for p in self.path.iter() {
            match p {
                ast::Path::Member(m) => final_value = member(final_value, m.clone()),

                ast::Path::Index(i) => {
                    final_value = match final_value {
                        Value::Array(a) => a[match i.eval(context, globals) {
                            Literal(l) => {
                                match l {
                                    Value::Number(n) => n,
                                    _ => panic!("Expected number"),
                                }
                            },

                            Evaluatable(e) => return Evaluatable(e)
                            
                        } as usize]
                            .clone(),
                        _ => panic!("Can't index non-iteratable variable"),
                    }
                }

                ast::Path::Call(c) => unimplemented!(),
            }
        }

        Literal(final_value)
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

        compile_scope(
            &self.statements,
            new_context,
            start_group,
            globals,
            Value::Null,
        )
    }
}

impl ast::Dictionary {
    fn read(&self, context: &Context, globals: &mut Globals) -> HashMap<String, Value> {
        compile_scope(
            &self.members,
            context.clone(),
            Group { id: 0 },
            globals,
            Value::Null,
        )
        .members
    }
}
