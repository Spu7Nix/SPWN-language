//! Defining all native types (and functions?)

use crate::ast::ObjectMode;
use crate::compiler::RuntimeError;
use crate::compiler_types::*;
use crate::levelstring::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub type ArbitraryID = u16;
pub type SpecificID = u16;
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ID {
    Specific(SpecificID),
    Arbitrary(ArbitraryID), // will be given specific ids at the end of compilation
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Group {
    pub id: ID,
}

impl Group {
    pub fn new(id: SpecificID) -> Self {
        //creates new specific group
        Group {
            id: ID::Specific(id),
        }
    }

    pub fn next_free(counter: &mut ArbitraryID) -> Self {
        //creates new specific group
        (*counter) += 1;
        Group {
            id: ID::Arbitrary(*counter),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Color {
    pub id: ID,
}

impl Color {
    pub fn new(id: SpecificID) -> Self {
        //creates new specific color
        Self {
            id: ID::Specific(id),
        }
    }

    pub fn next_free(counter: &mut ArbitraryID) -> Self {
        //creates new specific color
        (*counter) += 1;
        Self {
            id: ID::Arbitrary(*counter),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub id: ID,
}

impl Block {
    pub fn new(id: SpecificID) -> Self {
        //creates new specific block
        Self {
            id: ID::Specific(id),
        }
    }

    pub fn next_free(counter: &mut ArbitraryID) -> Self {
        //creates new specific block
        (*counter) += 1;
        Self {
            id: ID::Arbitrary(*counter),
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub id: ID,
}

impl Item {
    pub fn new(id: SpecificID) -> Self {
        //creates new specific item id
        Self {
            id: ID::Specific(id),
        }
    }

    pub fn next_free(counter: &mut ArbitraryID) -> Self {
        //creates new specific item id
        (*counter) += 1;
        Self {
            id: ID::Arbitrary(*counter),
        }
    }
}

pub fn context_trigger(context: &Context, uid_counter: &mut usize) -> GDObj {
    let mut params = HashMap::new();
    params.insert(57, ObjParam::Group(context.start_group));
    (*uid_counter) += 1;
    GDObj {
        params: HashMap::new(),
        func_id: context.func_id,
        mode: ObjectMode::Trigger,
        unique_id: *uid_counter,
    }
}

pub const TYPE_MEMBER_NAME: &str = "type";
impl Value {
    pub fn member(
        &self,
        member: String,
        context: &Context,
        globals: &mut Globals,
    ) -> Option<StoredValue> {
        let get_impl = |t: u16, m: String| match globals.implementations.get(&t) {
            Some(imp) => match imp.get(&m) {
                Some(mem) => Some(mem.0),
                None => None,
            },
            None => None,
        };
        if member == TYPE_MEMBER_NAME {
            Some(match self {
                Value::Dict(dict) => match dict.get(TYPE_MEMBER_NAME) {
                    Some(value) => *value,
                    None => store_value(
                        Value::TypeIndicator(self.to_num(globals)),
                        1,
                        globals,
                        context,
                    ),
                },

                _ => store_value(
                    Value::TypeIndicator(self.to_num(globals)),
                    1,
                    globals,
                    context,
                ),
            })
        } else {
            match self {
                Value::Func(f) => {
                    if member == "group" {
                        return Some(store_value(
                            Value::Group(f.start_group),
                            1,
                            globals,
                            context,
                        ));
                    }
                }

                Value::Str(a) => {
                    if member == "length" {
                        return Some(store_value(
                            Value::Number(a.len() as f64),
                            1,
                            globals,
                            context,
                        ));
                    }
                }
                Value::Array(a) => {
                    if member == "length" {
                        return Some(store_const_value(
                            Value::Number(a.len() as f64),
                            1,
                            globals,
                            context,
                        ));
                    }
                }
                Value::Range(start, end, step) => match member.as_ref() {
                    "start" => {
                        return Some(store_const_value(
                            Value::Number(*start as f64),
                            1,
                            globals,
                            context,
                        ))
                    }
                    "end" => {
                        return Some(store_const_value(
                            Value::Number(*end as f64),
                            1,
                            globals,
                            context,
                        ))
                    }
                    "step_size" => {
                        return Some(store_const_value(
                            Value::Number(*step as f64),
                            1,
                            globals,
                            context,
                        ))
                    }
                    _ => (),
                },
                _ => (),
            };

            let my_type = self.to_num(globals);

            match self {
                Value::Builtins => Some(store_value(
                    Value::BuiltinFunction(member),
                    1,
                    globals,
                    context,
                )),
                Value::Dict(dict) => match dict.get(&member) {
                    Some(value) => Some(*value),
                    None => get_impl(my_type, member),
                },
                Value::Func(f) => {
                    if &member == "start_group" {
                        Some(store_value(
                            Value::Group(f.start_group),
                            1,
                            globals,
                            context,
                        ))
                    } else {
                        get_impl(my_type, member)
                    }
                }
                _ => get_impl(my_type, member),
            }
        }
    }
}

pub const BUILTIN_LIST: &[&str] = &[
    "print",
    "sin",
    "cos",
    "tan",
    "asin",
    "acos",
    "atan",
    "floor",
    "ceil",
    "add",
    "current_context",
    "append",
    "pop",
    "remove_index",
    "readfile",
    "substr",
    "matches",
    "b64encrypt",
    "b64decrypt",
    //operators
    "_or_",
    "_and_",
    "_more_than_",
    "_less_than_",
    "_more_or_equal_",
    "_less_or_equal_",
    "_divided_by_",
    "_times_",
    "_mod_",
    "_pow_",
    "_plus_",
    "_minus_",
    "_equal_",
    "_not_equal_",
    "_assign_",
    "_as_",
    "_add_",
    "_subtract_",
    "_exponate_",
    "_modulate_",
    "_multiply_",
    "_divide_",
    "_either_",
    "_range_",
];

const CANNOT_CHANGE_ERROR: &str = "
Cannot change a variable that was defined in another group/function context
(consider using a counter)
";

pub fn built_in_function(
    name: &str,
    arguments: Vec<StoredValue>,
    info: CompilerInfo,
    globals: &mut Globals,
    context: &Context,
) -> Result<Value, RuntimeError> {
    Ok(match name {
        "print" => {
            let mut out = String::new();
            for val in arguments {
                out += &globals.stored_values[val].to_str(globals);
            }
            //out.pop();
            println!("{}", out);
            Value::Null
        }

        "matches" => {
            if arguments.len() != 2 {
                return Err(RuntimeError::BuiltinError {
                    message: "expected two arguments: the type to be checked and the pattern"
                        .to_string(),
                    info,
                });
            }
            let val = globals.stored_values[arguments[0]].clone();
            let pattern = globals.stored_values[arguments[1]].clone();
            Value::Bool(val.matches_pat(&pattern, &info, globals, context)?)
        }

        "b64encrypt" => {
            if arguments.len() != 1 {
                return Err(RuntimeError::BuiltinError {
                    message: "expected one argument: string to be encrypted".to_string(),
                    info,
                });
            }

            let val = globals.stored_values[arguments[0]].clone();
            match val {
                Value::Str(s) => {
                    let encrypted = base64::encode(&s.as_bytes());
                    Value::Str(encrypted)
                }
                _ => {
                    return Err(RuntimeError::BuiltinError {
                        message: "expected one argument: string to be encrypted".to_string(),
                        info,
                    })
                }
            }
        }
        "b64decrypt" => {
            if arguments.len() != 1 {
                return Err(RuntimeError::BuiltinError {
                    message: "expected one argument: string to be encrypted".to_string(),
                    info,
                });
            }

            let val = globals.stored_values[arguments[0]].clone();
            match val {
                Value::Str(s) => {
                    let decrypted = match base64::decode(&s) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(RuntimeError::BuiltinError {
                                message: format!("Base 64 error: {}", e),
                                info,
                            })
                        }
                    };
                    Value::Str(String::from_utf8_lossy(&decrypted).to_string())
                }
                _ => {
                    return Err(RuntimeError::BuiltinError {
                        message: "expected one argument: string to be encrypted".to_string(),
                        info,
                    })
                }
            }
        }

        // "is_in_use" => {
        //     if arguments.len() != 1 {
        //         return Err(RuntimeError::BuiltinError {
        //             message: "expected one argument: The ID to check for".to_string(),
        //             info,
        //         });
        //     }
        //     let obj_prop = match globals.stored_values[arguments[0]] {
        //         Value::Group(g) => ObjParam::Group(g),
        //         Value::Color(c) => ObjParam::Color(c),
        //         Value::Block(b) => ObjParam::Block(b),
        //         Value::Item(i) => ObjParam::Item(i),
        //         _ => {
        //             return Err(RuntimeError::BuiltinError {
        //                 message: "value given was not an ID (group, color, block or item ID)"
        //                     .to_string(),
        //                 info,
        //             })
        //         }
        //     };
        //     let mut out = Value::Bool(false);
        //     for (obj, _) in &globals.func_ids[context.func_id].obj_list {
        //         for val in obj.params.values() {
        //             if val == &obj_prop {
        //                 out = Value::Bool(true);
        //             }
        //         }
        //     }
        //     out
        // }
        "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "floor" | "ceil" => {
            if arguments.len() != 1 {
                return Err(RuntimeError::BuiltinError {
                    message: "expected one argument".to_string(),
                    info,
                });
            }

            match &globals.stored_values[arguments[0]] {
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

                a => {
                    return Err(RuntimeError::BuiltinError {
                        message: format!("Expected number, found {}", a.to_str(globals)),
                        info,
                    })
                }
            }
        }

        "add" => {
            if arguments.len() != 1 {
                return Err(RuntimeError::BuiltinError {
                    message: "Expected one argument".to_string(),
                    info,
                });
            }

            match &globals.stored_values[arguments[0]] {
                Value::Obj(obj, mode) => {
                    let c_t = context_trigger(context, &mut globals.uid_counter);
                    let mut obj_map = HashMap::<u16, ObjParam>::new();

                    for p in obj {
                        obj_map.insert(p.0, p.1.clone());
                    }
                    match mode {
                        ObjectMode::Object => {
                            if context.start_group.id != ID::Specific(0) {
                                return Err(RuntimeError::BuiltinError {
                                    message: String::from("you cannot add an obj type object in a function context. Consider moving this add function call to another context, or changing it to a trigger type"), 
                                    info
                                });
                            }
                            (*globals).uid_counter += 1;
                            let obj = GDObj {
                                params: obj_map,
                                func_id: context.func_id,
                                mode: ObjectMode::Object,
                                unique_id: globals.uid_counter,
                            };
                            (*globals).objects.push(obj)
                        }
                        ObjectMode::Trigger => {
                            let obj = GDObj {
                                params: obj_map,
                                mode: ObjectMode::Trigger,
                                ..c_t
                            }
                            .context_parameters(context);
                            (*globals).trigger_order += 1;
                            (*globals).func_ids[context.func_id]
                                .obj_list
                                .push((obj, globals.trigger_order))
                        }
                    };
                }

                a => {
                    return Err(RuntimeError::BuiltinError {
                        message: format!("Expected object, found {}", a.to_str(globals)),
                        info,
                    })
                }
            }

            Value::Null
        }

        "append" => {
            if arguments.len() != 2 {
                return Err(RuntimeError::BuiltinError {
                    message: "Expected two arguments, the first one being an array and the other being the value to append.".to_string(),
                    info,
                });
            }
            if !globals.is_mutable(arguments[0]) {
                return Err(RuntimeError::BuiltinError {
                    message: String::from("This array is not mutable"),
                    info,
                });
            }
            //set lifetime to the lifetime of the array

            let cloned = clone_value(
                arguments[1],
                globals.stored_values.map.get(&arguments[0]).unwrap().3,
                globals,
                context,
                globals.is_mutable(arguments[1]),
            );

            let typ = globals.get_type_str(arguments[0]);

            match &mut globals.stored_values[arguments[0]] {
                Value::Array(arr) => (*arr).push(cloned),

                _ => {
                    return Err(RuntimeError::BuiltinError {
                        message: format!("Expected array, found @{}", typ),
                        info,
                    })
                }
            }

            Value::Null
        }

        "readfile" => {
            if arguments.len() != 1 {
                return Err(RuntimeError::BuiltinError {
                    message: "Expected file name".to_string(),
                    info,
                });
            }
            let val = globals.stored_values[arguments[0]].clone();
            match val {
                Value::Str(s) => {
                    let path = Path::new(&s);
                    if !path.exists() {
                        return Err(RuntimeError::BuiltinError {
                            message: "Path doesn't exist".to_string(),
                            info,
                        });
                    }
                    let ret = fs::read_to_string(s);
                    let rval = match ret {
                        Ok(file) => file,
                        Err(_) => {
                            return Err(RuntimeError::BuiltinError {
                                message: "File cannot be opened".to_string(),
                                info,
                            });
                        }
                    };
                    Value::Str(rval)
                }
                _ => {
                    return Err(RuntimeError::BuiltinError {
                        message: "Expected one argument: string as path".to_string(),
                        info,
                    });
                }
            }
        }

        "pop" => {
            if arguments.len() != 1 {
                return Err(RuntimeError::BuiltinError {
                    message: "Expected one arguments, the array or string to pop from".to_string(),
                    info,
                });
            }
            if !globals.is_mutable(arguments[0]) {
                return Err(RuntimeError::BuiltinError {
                    message: String::from("This value is not mutable"),
                    info,
                });
            }

            let typ = globals.get_type_str(arguments[0]);

            match &mut globals.stored_values[arguments[0]] {
                Value::Array(arr) => match (*arr).pop() {
                    Some(val) => globals.stored_values[val].clone(),
                    None => Value::Null,
                },
                Value::Str(s) => match (*s).pop() {
                    Some(val) => Value::Str(val.to_string()),
                    None => Value::Null,
                },
                _ => {
                    return Err(RuntimeError::BuiltinError {
                        message: format!("Expected array or string, found @{}", typ),
                        info,
                    })
                }
            }
        }

        "substr" => {
            if arguments.len() != 3 {
                return Err(RuntimeError::BuiltinError {
                    message: "Expected three arguments: string to be sliced, a start index, and an end index".to_string(),
                    info,
                });
            }

            let val = match globals.stored_values[arguments[0]].clone() {
                Value::Str(s) => s,
                _ => {
                    let typ = globals.get_type_str(arguments[0]);
                    return Err(RuntimeError::BuiltinError {
                        message: format!("Expected string, found @{}", typ),
                        info,
                    });
                }
            };

            let start_index = match globals.stored_values[arguments[1]] {
                Value::Number(n) => n as usize,
                _ => {
                    let typ = globals.get_type_str(arguments[1]);
                    return Err(RuntimeError::BuiltinError {
                        message: format!("Expected number as start index, found @{}", typ),
                        info,
                    });
                }
            };

            let end_index = match globals.stored_values[arguments[2]] {
                Value::Number(n) => n as usize,
                _ => {
                    let typ = globals.get_type_str(arguments[2]);
                    return Err(RuntimeError::BuiltinError {
                        message: format!("Expected number as start index, found @{}", typ),
                        info,
                    });
                }
            };

            if start_index >= end_index {
                return Err(RuntimeError::BuiltinError {
                    message: "Start index is larger than end index".to_string(),
                    info,
                });
            }
            if end_index > val.len() {
                return Err(RuntimeError::BuiltinError {
                    message: "End index is larger than string".to_string(),
                    info,
                });
            }
            Value::Str(val.as_str()[start_index..end_index].to_string())
        }

        "remove_index" => {
            if arguments.len() != 2 {
                return Err(RuntimeError::BuiltinError {
                    message: "Expected two arguments, the array or string to remove from and the index of the element to be removed".to_string(),
                    info,
                });
            }
            if !globals.is_mutable(arguments[0]) {
                return Err(RuntimeError::BuiltinError {
                    message: String::from("This value is not mutable"),
                    info,
                });
            }

            let typ = globals.get_type_str(arguments[0]);

            let index = match globals.stored_values[arguments[1]] {
                Value::Number(n) => n as usize,
                _ => {
                    let typ = globals.get_type_str(arguments[1]);
                    return Err(RuntimeError::BuiltinError {
                        message: format!("Expected number as index, found @{}", typ),
                        info,
                    });
                }
            };

            match &mut globals.stored_values[arguments[0]] {
                Value::Array(arr) => {
                    let out = (*arr).remove(index);
                    globals.stored_values[out].clone()
                }

                Value::Str(s) => Value::Str((*s).remove(index).to_string()),
                _ => {
                    return Err(RuntimeError::BuiltinError {
                        message: format!("Expected array or string, found @{}", typ),
                        info,
                    })
                }
            }
        }

        "current_context" => Value::Str(format!("{:?}", context)),

        "_or_" | "_and_" | "_more_than_" | "_less_than_" | "_more_or_equal_"
        | "_less_or_equal_" | "_divided_by_" | "_times_" | "_mod_" | "_pow_" | "_plus_"
        | "_minus_" | "_equal_" | "_not_equal_" | "_assign_" | "_as_" | "_add_" | "_subtract_"
        | "_multiply_" | "_divide_" | "_intdivide_" |"_either_" | "_exponate_" | "_modulate_" | "_range_" => {
            if arguments.len() != 2 {
                return Err(RuntimeError::BuiltinError {
                    message: "Expected two arguments".to_string(),
                    info,
                });
            }
            let acum_val = arguments[0];
            let val = arguments[1];
            let c2 = &context;

            let a_type = globals.get_type_str(acum_val);
            let b_type = globals.get_type_str(val);

            let acum_val_fn_context = globals.get_val_fn_context(acum_val, info.clone())?;
            let mutable = globals.is_mutable(acum_val);
            let val_b = globals.stored_values[val].clone();
            let val_a = &mut globals.stored_values[acum_val];

            fn mutable_err(info: CompilerInfo, attempted_op_macro: &str) -> RuntimeError {
                RuntimeError::RuntimeError {
                    message: format!(
                        "
This value is not mutable! 
Consider defining it with 'let', or implementing a '{}' macro on its type.",
                        attempted_op_macro
                    ),
                    info,
                }
            }

            match name {
                "_range_" => {
                    let start = match val_a {
                        Value::Number(n) => convert_to_int(*n, &info)?,
                        _ => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!("expected @number, found @{}", b_type),
                                info,
                            })
                        }
                    };
                    match val_b {
                        Value::Number(end) => Value::Range(start, convert_to_int(end, &info)?, 1),
                        Value::Range(step, end, old_step) => {
                            if old_step != 1 {
                                return Err(RuntimeError::RuntimeError {
                                message: "Range operator cannot be used on a range that already has a non-default stepsize"
                                    .to_string(),
                                info,
                            });
                            }
                            Value::Range(
                                start,
                                end,
                                if step < 0 {
                                    return Err(RuntimeError::RuntimeError {
                                        message: "cannot have a stepsize less than 0".to_string(),
                                        info,
                                    });
                                } else {
                                    step as usize
                                },
                            )
                        }
                        _ => {
                            println!("{:?}", val_a);
                            return Err(RuntimeError::RuntimeError {
                                message: format!("expected @number, found @{}", a_type),
                                info,
                            });
                        }
                    }
                }
                "_or_" => match (val_a, val_b) {
                    (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a || b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "bool and bool".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_and_" => match (val_a, val_b) {
                    (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a && b),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "bool and bool".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_more_than_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Bool(*a > b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_less_than_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Bool(*a < b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_more_or_equal_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Bool(*a >= b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_less_or_equal_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Bool(*a <= b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_divided_by_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(*a / b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_intdivide_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(((*a as i32) / (b as i32)).into()),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_times_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(*a * b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_mod_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(*a % b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_pow_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(a.powf(b)),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_plus_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(*a + b),
                    (Value::Str(a), Value::Str(b)) => Value::Str(a.clone() + &b),
                    (Value::Array(a), Value::Array(b)) => {
                        Value::Array([a.as_slice(), b.as_slice()].concat())
                    }

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number, array and array or string and string"
                                .to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_minus_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(*a - b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info,
                        })
                    }
                },
                "_equal_" => Value::Bool(value_equality(acum_val, val, globals)),
                "_not_equal_" => Value::Bool(!value_equality(acum_val, val, globals)),
                "_assign_" => {
                    //println!("hi1");
                    if !mutable {
                        return Err(mutable_err(info, "_assign_"));
                    }
                    if acum_val_fn_context != c2.start_group {
                        return Err(RuntimeError::RuntimeError {
                            message: CANNOT_CHANGE_ERROR.to_string(),
                            info,
                        });
                    }

                    if globals.stored_values[acum_val] == Value::Null
                        && globals.stored_values.map.get(&acum_val).unwrap().2
                    {
                        //println!("hi");
                        globals.stored_values[acum_val] = globals.stored_values[val].clone();
                        globals.stored_values.set_mutability(acum_val, true);
                        globals.stored_values[acum_val].clone()
                    } else {
                        //println!("{:?}", globals.stored_values[acum_val]);
                        globals.stored_values[acum_val] = globals.stored_values[val].clone();
                        globals.stored_values[acum_val].clone()
                    }
                }
                "_as_" => match globals.stored_values[val] {
                    Value::TypeIndicator(t) => convert_type(
                        &globals.stored_values[acum_val].clone(),
                        t,
                        &info,
                        globals,
                        &context,
                    )?,

                    _ => {
                        return Err(RuntimeError::RuntimeError {
                            message: "Expected a type-indicator to convert to!".to_string(),
                            info,
                        });
                    }
                },
                "_either_" => Value::Pattern(Pattern::Either(
                    if let Value::Pattern(p) = convert_type(
                        &globals.stored_values[acum_val].clone(),
                        18,
                        &info,
                        globals,
                        &context,
                    )? {
                        Box::new(p)
                    } else {
                        unreachable!()
                    },
                    if let Value::Pattern(p) = convert_type(
                        &globals.stored_values[val].clone(),
                        18,
                        &info,
                        globals,
                        &context,
                    )? {
                        Box::new(p)
                    } else {
                        unreachable!()
                    },
                )),
                "_add_" => {
                    if !mutable {
                        return Err(mutable_err(info, "_add_"));
                    }
                    if acum_val_fn_context != c2.start_group {
                        return Err(RuntimeError::RuntimeError {
                            message: CANNOT_CHANGE_ERROR.to_string(),
                            info,
                        });
                    }

                    match (val_a, val_b) {
                        (Value::Number(a), Value::Number(b)) => (*a) += b,
                        (Value::Str(a), Value::Str(b)) => (*a) += &b,
                        (Value::Array(a), Value::Array(b)) => (*a).extend(&b),

                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "number and number, array and array or string and string"
                                    .to_string(),
                                found: format!("{} and {}", a_type, b_type),
                                info,
                            })
                        }
                    };
                    Value::Null
                }
                "_subtract_" => {
                    if !mutable {
                        return Err(mutable_err(info, "_subtract_"));
                    }
                    if acum_val_fn_context != c2.start_group {
                        return Err(RuntimeError::RuntimeError {
                            message: CANNOT_CHANGE_ERROR.to_string(),
                            info,
                        });
                    }

                    match (val_a, val_b) {
                        (Value::Number(a), Value::Number(b)) => (*a) -= b,

                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "number and number".to_string(),
                                found: format!("{} and {}", a_type, b_type),
                                info,
                            })
                        }
                    };
                    Value::Null
                }
                "_multiply_" => {
                    if !mutable {
                        return Err(mutable_err(info, "_multiply_"));
                    }
                    if acum_val_fn_context != c2.start_group {
                        return Err(RuntimeError::RuntimeError {
                            message: CANNOT_CHANGE_ERROR.to_string(),
                            info,
                        });
                    }

                    match (val_a, val_b) {
                        (Value::Number(a), Value::Number(b)) => (*a) *= b,

                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "number and number".to_string(),
                                found: format!("{} and {}", a_type, b_type),
                                info,
                            })
                        }
                    };
                    Value::Null
                }
                "_exponate_" => {
                    if !mutable {
                        return Err(mutable_err(info, "_exponate_"));
                    }
                    if acum_val_fn_context != c2.start_group {
                        return Err(RuntimeError::RuntimeError {
                            message: CANNOT_CHANGE_ERROR.to_string(),
                            info,
                        });
                    }

                    match (val_a, val_b) {
                        (Value::Number(a), Value::Number(b)) => (*a) = a.powf(b),

                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "number and number".to_string(),
                                found: format!("{} and {}", a_type, b_type),
                                info,
                            })
                        }
                    };
                    Value::Null
                }
                "_modulate_" => {
                    if !mutable {
                        return Err(mutable_err(info, "_modulate_"));
                    }
                    if acum_val_fn_context != c2.start_group {
                        return Err(RuntimeError::RuntimeError {
                            message: CANNOT_CHANGE_ERROR.to_string(),
                            info,
                        });
                    }

                    match (val_a, val_b) {
                        (Value::Number(a), Value::Number(b)) => (*a) %= b,

                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "number and number".to_string(),
                                found: format!("{} and {}", a_type, b_type),
                                info,
                            })
                        }
                    };
                    Value::Null
                }
                "_divide_" => {
                    if !mutable {
                        return Err(mutable_err(info, "_divide_"));
                    }
                    if acum_val_fn_context != c2.start_group {
                        return Err(RuntimeError::RuntimeError {
                            message: CANNOT_CHANGE_ERROR.to_string(),
                            info,
                        });
                    }

                    match (val_a, val_b) {
                        (Value::Number(a), Value::Number(b)) => (*a) /= b,

                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "number and number".to_string(),
                                found: format!("{} and {}", a_type, b_type),
                                info,
                            })
                        }
                    };
                    Value::Null
                }
                _ => unreachable!(),
            }
        }

        a => {
            return Err(RuntimeError::RuntimeError {
                message: format!("Nonexistant builtin-function: {}", a),
                info,
            })
        }
    })
}
