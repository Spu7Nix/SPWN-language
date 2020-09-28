//! Defining all native types (and functions?)

use crate::ast::ObjectMode;
use crate::compiler::RuntimeError;
use crate::compiler_types::*;
use crate::levelstring::*;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Group {
    pub id: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]

pub struct Color {
    pub id: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: u16,
}

pub fn context_trigger(context: Context, info: CompilerInfo) -> GDObj {
    let mut params = HashMap::new();
    params.insert(57, context.start_group.id.to_string());
    params.insert(
        62,
        if context.spawn_triggered {
            String::from("1")
        } else {
            String::from("0")
        },
    );
    GDObj {
        params: HashMap::new(),
        func_id: info.func_id,
        mode: ObjectMode::Trigger,
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
        let get_impl = |t: u16, m: String| match context.implementations.get(&t) {
            Some(imp) => match imp.get(&m) {
                Some(mem) => Some(*mem),
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
                        return Some(store_value(
                            Value::Number(a.len() as f64),
                            1,
                            globals,
                            context,
                        ));
                    }
                }
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
                    None => get_impl(my_type, member).clone(),
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
                        get_impl(my_type, member).clone()
                    }
                }
                _ => get_impl(my_type, member).clone(),
            }
        }
    }
}

pub const BUILTIN_LIST: [&str; 31] = [
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
    "_multiply_",
    "_divide_",
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

        "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "floor" | "ceil" => {
            if arguments.len() != 1 {
                return Err(RuntimeError::BuiltinError {
                    message: "Expected one error".to_string(),
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
                    let c_t = context_trigger(context.clone(), info.clone());
                    let mut obj_map = HashMap::<u16, String>::new();

                    for p in obj {
                        obj_map.insert(p.0, p.1.clone());
                    }

                    (*globals).func_ids[info.func_id].obj_list.push(match mode {
                        ObjectMode::Object => GDObj {
                            params: obj_map.clone(),
                            func_id: info.func_id,
                            mode: ObjectMode::Object,
                        },
                        ObjectMode::Trigger => GDObj {
                            params: obj_map.clone(),
                            mode: ObjectMode::Trigger,
                            ..c_t
                        }
                        .context_parameters(context.clone()),
                    });
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

        "current_context" => Value::Str(format!("{:?}", context)),

        "_or_" | "_and_" | "_more_than_" | "_less_than_" | "_more_or_equal_"
        | "_less_or_equal_" | "_divided_by_" | "_times_" | "_mod_" | "_pow_" | "_plus_"
        | "_minus_" | "_equal_" | "_not_equal_" | "_assign_" | "_as_" | "_add_" | "_subtract_"
        | "_multiply_" | "_divide_" => {
            if arguments.len() != 2 {
                return Err(RuntimeError::BuiltinError {
                    message: "Expected two arguments".to_string(),
                    info,
                });
            }
            let acum_val = arguments[0];
            let val = arguments[1];
            let c2 = &context;

            let a_type = globals.get_type_str(val);
            let b_type = globals.get_type_str(acum_val);

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
                "_or_" => match (val_a, val_b) {
                    (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a || b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "bool and bool".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_and_" => match (val_a, val_b) {
                    (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a && b),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "bool and bool".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_more_than_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Bool(*a > b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_less_than_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Bool(*a < b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_more_or_equal_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Bool(*a >= b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_less_or_equal_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Bool(*a <= b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_divided_by_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(*a / b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_times_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(*a * b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_mod_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(*a % b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_pow_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(a.powf(b)),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_plus_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(*a + b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_minus_" => match (val_a, val_b) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(*a - b),

                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "number and number".to_string(),
                            found: format!("{} and {}", a_type, b_type),
                            info: info.clone(),
                        })
                    }
                },
                "_equal_" => {
                    Value::Bool(globals.stored_values[acum_val] == globals.stored_values[val])
                }
                "_not_equal_" => {
                    Value::Bool(globals.stored_values[acum_val] != globals.stored_values[val])
                }
                "_assign_" => {
                    if !mutable {
                        return Err(mutable_err(info, "_assign_"));
                    }
                    if acum_val_fn_context != c2.start_group {
                        return Err(RuntimeError::RuntimeError {
                            message: CANNOT_CHANGE_ERROR.to_string(),
                            info: info.clone(),
                        });
                    }

                    globals.stored_values[acum_val] = globals.stored_values[val].clone();
                    globals.stored_values[acum_val].clone()
                }
                "_as_" => match globals.stored_values[val] {
                    Value::TypeIndicator(t) => convert_type(
                        globals.stored_values[acum_val].clone(),
                        t,
                        info.clone(),
                        globals,
                    )?,

                    _ => {
                        return Err(RuntimeError::RuntimeError {
                            message: "Expected a type-indicator to convert to!".to_string(),
                            info: info.clone(),
                        });
                    }
                },
                "_add_" => {
                    if !mutable {
                        return Err(mutable_err(info, "_add_"));
                    }
                    if acum_val_fn_context != c2.start_group {
                        return Err(RuntimeError::RuntimeError {
                            message: CANNOT_CHANGE_ERROR.to_string(),
                            info: info.clone(),
                        });
                    }

                    match (val_a, val_b) {
                        (Value::Number(a), Value::Number(b)) => (*a) += b,

                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "number and number".to_string(),
                                found: format!("{} and {}", a_type, b_type),
                                info: info.clone(),
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
                            info: info.clone(),
                        });
                    }

                    match (val_a, val_b) {
                        (Value::Number(a), Value::Number(b)) => (*a) -= b,

                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "number and number".to_string(),
                                found: format!("{} and {}", a_type, b_type),
                                info: info.clone(),
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
                            info: info.clone(),
                        });
                    }

                    match (val_a, val_b) {
                        (Value::Number(a), Value::Number(b)) => (*a) *= b,

                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "number and number".to_string(),
                                found: format!("{} and {}", a_type, b_type),
                                info: info.clone(),
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
                            info: info.clone(),
                        });
                    }

                    match (val_a, val_b) {
                        (Value::Number(a), Value::Number(b)) => (*a) /= b,

                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "number and number".to_string(),
                                found: format!("{} and {}", a_type, b_type),
                                info: info.clone(),
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
