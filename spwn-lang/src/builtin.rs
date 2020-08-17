//! Defining all native types (and functions?)

use crate::compiler_types::*;
use crate::levelstring::*;
//use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Group {
    pub id: u16,
}

#[derive(Debug, Copy, Clone, PartialEq)]

pub struct Color {
    pub id: u16,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Block {
    pub id: u16,
}

impl Block {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Item {
    pub id: u16,
}

pub fn context_trigger(context: Context, globals: &mut Globals, info: CompilerInfo) -> GDObj {
    GDObj {
        obj_id: 0,
        groups: vec![context.start_group],
        target: Group { id: 0 },
        spawn_triggered: context.spawn_triggered,
        params: Vec::new(),
        func_id: info.func_id,
    }
}

const TYPE_MEMBER_NAME: &str = "TYPE";
impl Value {
    pub fn member(
        &self,
        member: String,
        context: &Context,
        globals: &mut Globals,
        _: CompilerInfo,
    ) -> Option<Value> {
        //println!("{:?}", context.implementations);
        let get_impl = |t: String, m: String| match context.implementations.get(&(t)) {
            Some(imp) => match imp.get(&m) {
                Some(mem) => Some((*globals).stored_values[*mem as usize].clone()),
                None => None,
            },
            None => None,
        };
        let my_type = match self {
            Value::Dict(dict) => match dict.get(TYPE_MEMBER_NAME) {
                Some(value) => match (*globals).stored_values[*value as usize].clone() {
                    Value::Str(s) => s,
                    _ => unreachable!(),
                },
                None => "dictionary".to_string(),
            },

            Value::Func(f) => {
                if member == "group" {
                    return Some(Value::Group(f.start_group));
                }
                "function".to_string()
            }
            Value::Group(_) => "group".to_string(),
            Value::Color(_) => "color".to_string(),
            Value::Block(_) => "block".to_string(),
            Value::Item(_) => "item".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Macro(_) => "macro".to_string(),
            Value::Str(a) => {
                if member == "length" {
                    return Some(Value::Number(a.len() as f64));
                }
                "string".to_string()
            }
            Value::Array(a) => {
                if member == "length" {
                    return Some(Value::Number(a.len() as f64));
                }
                "array".to_string()
            }
            Value::Obj(_) => "object".to_string(),
            Value::Builtins => {
                if member == "TYPE" {
                    "SPWN".to_string()
                } else {
                    return Some(Value::BuiltinFunction(member));
                }
            }
            Value::BuiltinFunction(_) => "built-in function".to_string(),
            Value::Null => "null".to_string(),
        };

        if member == TYPE_MEMBER_NAME {
            return Some(Value::Str(my_type.to_string()));
        } else {
            match self {
                Value::Dict(dict) => match dict.get(&member) {
                    Some(value) => Some((*globals).stored_values[*value as usize].clone()),
                    None => get_impl(my_type.to_string(), member).clone(),
                },
                Value::Func(f) => {
                    if &member == "start_group" {
                        Some(Value::Group(f.start_group))
                    } else {
                        get_impl(my_type.to_string(), member).clone()
                    }
                }
                _ => get_impl(my_type.to_string(), member).clone(),
            }
        }
    }
}

pub fn built_in_function(
    name: &str,
    arguments: Vec<Value>,
    info: CompilerInfo,
    globals: &mut Globals,
) -> Value {
    match name {
        "print" => {
            let mut out = String::new();
            for val in arguments {
                out += &format!("{} ", val);
            }
            out.pop();
            println!("{}", out);
            Value::Null
        }

        "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "floor" | "ceil" => {
            if arguments.len() != 1 {
                panic!(compile_error("Expected one argument", info))
            }

            match &arguments[0] {
                Value::Number(n) => Value::Number(match name {
                    "sin" => n.sin(),
                    "cos" => n.cos(),
                    "tan" => n.tan(),
                    "asin" => n.asin(),
                    "acos" => n.acos(),
                    "atan" => n.atan(),
                    "floor" => n.floor(),
                    "ceil" => n.ceil(),

                    _ => unreachable!(),
                }),

                a => panic!(compile_error(
                    &format!("Expected number, found: {}", a),
                    info
                )),
            }
        }

        "add" => {
            //add object

            Value::Null
        }

        _ => panic!(compile_error(
            &format!("Nonexistant built-in-function: {}", name),
            info
        )),
    }
}
