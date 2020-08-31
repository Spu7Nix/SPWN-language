use crate::ast;
use crate::builtin::*;
use crate::levelstring::*;
//use std::boxed::Box;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::compiler::{compile_scope, import_module, next_free, RuntimeError};

pub type TypeID = u16;

pub type Returns = Vec<(Value, Context)>;
pub type Implementations = HashMap<TypeID, HashMap<String, u32>>;
pub type StoredValue = u32; //index to stored value in globals.stored_values
pub fn store_value(val: Value, globals: &mut Globals) -> StoredValue {
    (*globals).stored_values.push(val);
    (globals.stored_values.len() as u32) - 1
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Context {
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
    pub func_id: usize,
}

impl CompilerInfo {
    pub fn next(
        &self,
        name: &str,
        globals: &mut Globals,
        use_in_organization: bool,
    ) -> CompilerInfo {
        let mut new_path = self.path.clone();
        new_path.push(name.to_string());

        if use_in_organization {
            (*globals).func_ids.push(FunctionID {
                name: name.to_string(),
                parent: Some(self.func_id),
                obj_list: Vec::new(),
                width: None,
            });
        }

        CompilerInfo {
            depth: self.depth + 1,
            path: new_path,
            line: self.line,
            func_id: if use_in_organization {
                (*globals).func_ids.len() - 1
            } else {
                self.func_id
            },
        }
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
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
pub struct Function {
    pub start_group: Group,
    //pub all_groups: Vec<Group>,
}

#[derive(Clone, Debug, PartialEq)]

pub enum Value {
    Group(Group),
    Color(Color),
    Block(Block),
    Item(Item),
    Number(f64),
    Bool(bool),
    Func(Function),
    Dict(HashMap<String, StoredValue>),
    Macro(Macro),
    Str(String),
    Array(Vec<Value>),
    Obj(Vec<(u16, String)>),
    Builtins,
    BuiltinFunction(String),
    TypeIndicator(TypeID),
    Null,
}

impl Value {
    //numeric representation of value
    pub fn to_num(&self, globals: &Globals) -> TypeID {
        match self {
            Value::Group(_) => 0,
            Value::Color(_) => 1,
            Value::Block(_) => 2,
            Value::Item(_) => 3,
            Value::Number(_) => 4,
            Value::Bool(_) => 5,
            Value::Func(_) => 6,
            Value::Dict(d) => match d.get(TYPE_MEMBER_NAME) {
                Some(member) => match globals.stored_values[*member as usize] {
                    Value::TypeIndicator(t) => t,
                    _ => unreachable!(),
                },

                None => 7,
            },
            Value::Macro(_) => 8,
            Value::Str(_) => 9,
            Value::Array(_) => 10,
            Value::Obj(_) => 11,
            Value::Builtins => 12,
            Value::BuiltinFunction(_) => 13,
            Value::TypeIndicator(_) => 14,
            Value::Null => 15,
        }
    }
}

//copied from https://stackoverflow.com/questions/59401720/how-do-i-find-the-key-for-a-value-in-a-hashmap
fn find_key_for_value<'a>(map: &'a HashMap<String, u16>, value: u16) -> Option<&'a String> {
    map.iter()
        .find_map(|(key, &val)| if val == value { Some(key) } else { None })
}

//use std::fmt;

const MAX_DICT_EL_DISPLAY: u16 = 10;

impl Value {
    pub fn to_str(&self, globals: &Globals) -> String {
        match self {
            Value::Group(g) => (g.id.to_string() + "g"),
            Value::Color(c) => (c.id.to_string() + "c"),
            Value::Block(b) => (b.id.to_string() + "b"),
            Value::Item(i) => (i.id.to_string() + "i"),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Func(f) => format!("<function {}g>", f.start_group.id),
            Value::Dict(d) => {
                let mut out = String::from("{\n");
                let mut count = 0;
                let mut d_iter = d.iter();
                for (key, val) in &mut d_iter {
                    count += 1;

                    if count > MAX_DICT_EL_DISPLAY {
                        out += &format!("... ({} more)  ", d_iter.count());
                        break;
                    }
                    let stored_val = (*globals).stored_values[*val as usize].to_str(globals);
                    out += &format!("{}: {},\n", key, stored_val);
                }
                out.pop();
                out.pop();

                out += "\n}"; //why do i have to do this twice? idk

                out
            }
            Value::Macro(_) => "<macro>".to_string(),
            Value::Str(s) => s.clone(),
            Value::Array(a) => {
                if a.is_empty() {
                    "[]".to_string()
                } else {
                    let mut out = String::from("[");
                    for val in a {
                        out += &val.to_str(globals);
                        out += ",";
                    }
                    out.pop();
                    out += "]";

                    out
                }
            }
            Value::Obj(o) => {
                let mut out = String::new();
                for (key, val) in o {
                    out += &format!("{},{},", key, val);
                }
                out.pop();
                out += ";";
                out
            }
            Value::Builtins => "SPWN".to_string(),
            Value::BuiltinFunction(n) => format!("<built-in-function: {}>", n),
            Value::Null => "Null".to_string(),
            Value::TypeIndicator(id) => format!(
                "@{}",
                match find_key_for_value(&globals.type_ids, *id) {
                    Some(name) => name,
                    None => "[TYPE NOT FOUND]",
                }
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionID {
    pub parent: Option<usize>, //index of parent id, if none it is a top-level id
    pub width: Option<u32>,    //width of this id, is none when its not calculated yet
    pub name: String,          //name of this id, used for the label
    pub obj_list: Vec<GDObj>,  //list of objects in this function id
}

#[derive(Clone)]
pub struct Globals {
    pub closed_groups: Vec<u16>,
    pub closed_colors: Vec<u16>,
    pub closed_blocks: Vec<u16>,
    pub closed_items: Vec<u16>,
    pub path: PathBuf,

    pub lowest_y: HashMap<u32, u16>,
    pub stored_values: Vec<Value>,

    pub type_ids: HashMap<String, u16>,
    pub type_id_count: u16,

    pub func_ids: Vec<FunctionID>,
}

fn handle_operator<F>(
    value1: Value,
    value2: Value,
    op: F,
    macro_name: &str,
    op_name: &str,
    context: Context,
    globals: &mut Globals,
    info: CompilerInfo,
) -> Result<Returns, RuntimeError>
where
    F: FnOnce(Value, Value) -> Result<Value, RuntimeError>,
{
    Ok(
        if let Some(Value::Macro(m)) =
            value1.member(macro_name.to_string(), &context, globals, info.clone())
        {
            let new_info = info.next(op_name, globals, false);
            let (values, _) = execute_macro(
                (
                    m,
                    vec![ast::Argument {
                        symbol: None,
                        value: ast::Expression {
                            values: vec![ast::Variable {
                                value: ast::ValueLiteral::Resolved(value2),
                                path: Vec::new(),
                                operator: None,
                            }],
                            operators: Vec::new(),
                        },
                    }],
                ),
                context,
                globals,
                value1.clone(),
                new_info,
            )?;
            values
        } else {
            vec![(op(value1, value2)?, context)]
        },
    )
}

impl ast::Expression {
    pub fn eval(
        &self,
        context: Context,
        globals: &mut Globals,
        info: CompilerInfo,
    ) -> Result<(Returns, Returns), RuntimeError> {
        //second returns is in case there are any values in the expression that includes a return statement
        let mut vals = self.values.iter();
        let first_value = vals
            .next()
            .unwrap()
            .to_value(context.clone(), globals, info.clone())?;
        let mut acum = first_value.0;
        let mut inner_returns = first_value.1;

        let operator_macro_desc: String = String::from("operator macro member");

        if self.operators.is_empty() {
            //if only variable
            return Ok((acum, inner_returns));
        }

        for (i, var) in vals.enumerate() {
            let mut new_acum: Returns = Vec::new();
            //every value in acum will be operated with the value of var in the corresponding context
            for (acum_val, c) in acum {
                //what the value in acum becomes
                let evaled = var.to_value(c, globals, info.clone())?;
                inner_returns.extend(evaled.1);

                use ast::Operator::*;

                for (val, c2) in evaled.0 {
                    let vals: Returns = match self.operators[i] {
                        Or => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| -> Result<Value, RuntimeError> {
                                if let Value::Bool(b) = &acum_val {
                                    if let Value::Bool(b2) = &val {
                                        Ok(Value::Bool(*b || *b2))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_or_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_or_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_or_",
                            "logical or",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        And => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| {
                                if let Value::Bool(b) = &acum_val {
                                    if let Value::Bool(b2) = &val {
                                        Ok(Value::Bool(*b && *b2))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_and_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_and_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_and_",
                            "logical and",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        More => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = val {
                                        Ok(Value::Bool(n > &n2))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_more_than_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_more_than_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_more_than_",
                            "comparison",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Less => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = val {
                                        Ok(Value::Bool(n < &n2))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_less_than_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_less_than_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_less_than_",
                            "comparison",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        MoreOrEqual => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = val {
                                        Ok(Value::Bool(n >= &n2))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_more_or_equal_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_more_or_equal_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_more_or_equal_",
                            "comparison",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        LessOrEqual => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = val {
                                        Ok(Value::Bool(n <= &n2))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_less_or_equal_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_less_or_equal_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_less_or_equal_",
                            "comparison",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Divide => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = val {
                                        Ok(Value::Number(n / &n2))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_divided_by_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_divided_by_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_divided_by_",
                            "divide",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Multiply => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = val {
                                        Ok(Value::Number(n * &n2))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_times_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_times_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_times_",
                            "multiply",
                            c2,
                            globals,
                            info.clone(),
                        )?,

                        Modulo => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = val {
                                        Ok(Value::Number(n % &n2))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_mod_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_mod_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_mod_",
                            "modulo",
                            c2,
                            globals,
                            info.clone(),
                        )?,

                        Power => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = val {
                                        Ok(Value::Number(n.powf(n2)))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_pow_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_pow_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_pow_",
                            "power",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Plus => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = val {
                                        Ok(Value::Number(n + &n2))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_plus_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_plus_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_plus_",
                            "add",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Minus => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| {
                                if let Value::Number(n) = &acum_val {
                                    if let Value::Number(n2) = val {
                                        Ok(Value::Number(n - &n2))
                                    } else {
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: "_minus_".to_string(),
                                            info: info.clone(),
                                            desc: operator_macro_desc.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: "_minus_".to_string(),
                                        info: info.clone(),
                                        desc: operator_macro_desc.clone(),
                                    });
                                }
                            },
                            "_minus_",
                            "subtract",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Equal => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| Ok(Value::Bool(acum_val == val)),
                            "_equal_",
                            "logical equal",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        NotEqual => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val| Ok(Value::Bool(acum_val != val)),
                            "_not_equal_",
                            "logical not_equal",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Range => vec![(
                            Value::Array({
                                let start = match &acum_val.clone() {
                                    Value::Number(n) => *n as i32,
                                    _ => {
                                        return Err(RuntimeError::RuntimeError {
                                            message: "Both sides of range must be Numbers"
                                                .to_string(),
                                            info: info.clone(),
                                        })
                                    }
                                };
                                let end = match val {
                                    Value::Number(n) => n as i32,
                                    _ => {
                                        return Err(RuntimeError::RuntimeError {
                                            message: "Both sides of range must be Numbers"
                                                .to_string(),
                                            info: info.clone(),
                                        })
                                    }
                                };
                                if start < end {
                                    (start..end).collect::<Vec<i32>>()
                                } else {
                                    (end..start).rev().collect::<Vec<i32>>()
                                }
                                .into_iter()
                                .map(|x| Value::Number(x as f64))
                                .collect()
                            }),
                            c2,
                        )],
                    };
                    new_acum.extend(vals);
                }
            }
            acum = new_acum;
        }
        Ok((acum, inner_returns))
    }
}

pub fn execute_macro(
    (m, args): (Macro, Vec<ast::Argument>),
    context: Context,
    globals: &mut Globals,
    parent: Value,
    info: CompilerInfo,
) -> Result<(Returns, Returns), RuntimeError> {
    let mut inner_inner_returns = Vec::new();
    let mut new_contexts: Vec<Context> = Vec::new();
    if !m.args.is_empty() {
        // second returns is for any compound statements in the args
        let (evaled_args, inner_returns) = all_combinations(
            args.iter().map(|x| x.value.clone()).collect(),
            context.clone(),
            globals,
            info.clone(),
        )?;
        inner_inner_returns.extend(inner_returns);

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
                            return Err(RuntimeError::UndefinedErr {
                                undefined: name.clone(),
                                info: info.clone(),
                                desc: "macro argument".to_string(),
                            });
                        }
                    }
                    None => {
                        if (if m.args[0].0 == "self" { i + 1 } else { i }) > m.args.len() - 1 {
                            return Err(RuntimeError::RuntimeError {
                                message: "Too many arguments!".to_string(),
                                info: info.clone(),
                            });
                        }
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
                            new_variables
                                .insert(arg.0.clone(), store_value(default.clone(), globals));
                        }

                        None => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!(
                                    "Non-optional argument '{}' not satisfied!",
                                    arg.0
                                ),
                                info: info.clone(),
                            })
                        }
                    }
                }
            }

            new_context.variables.extend(new_variables);

            new_contexts.push(new_context);
        }
    } else {
        let mut new_context = context.clone();
        new_context.variables = m.def_context.variables.clone();
        /*let mut new_variables: HashMap<String, StoredValue> = HashMap::new();

        if m.args[0].0 == "self" {
            new_variables.insert("self".to_string(), store_value(parent.clone(), globals));
        }

        new_context.variables.extend(new_variables);*/

        new_contexts.push(new_context);
    }
    let new_info = info.next("macro body", globals, false);
    let compiled = compile_scope(&m.body, new_contexts, globals, new_info)?;

    let returns = if compiled.1.is_empty() {
        compiled
            .0
            .iter()
            .map(|x| (Value::Null, x.clone()))
            .collect()
    } else {
        if compiled.1.len() > 1 {
            let mut return_vals = Vec::<(Value, u8, Vec<Context>)>::new();
            for (val, c) in compiled.1 {
                let mut found = false;
                for val2 in &mut return_vals {
                    if val == val2.0 {
                        (*val2).1 += 1;
                        (*val2).2.push(c.clone());
                        found = true;
                        break;
                    }
                }
                if !found {
                    return_vals.push((val, 1, vec![c]));
                }
            }

            let mut rets = Returns::new();

            for (val, count, c) in return_vals {
                if count > 1 {
                    let mut new_context = context.clone();
                    new_context.spawn_triggered = true;
                    //pick a start group
                    let start_group = Group {
                        id: next_free(
                            &mut globals.closed_groups,
                            ast::IDClass::Group,
                            info.clone(),
                        )?,
                    };

                    for cont in c {
                        let obj = GDObj {
                            obj_id: 1268,
                            groups: vec![cont.start_group],
                            target: start_group,

                            ..context_trigger(cont.clone(), globals, info.clone())
                        }
                        .context_parameters(cont.clone());
                        (*globals).func_ids[info.func_id].obj_list.push(obj);
                    }

                    new_context.start_group = start_group;

                    rets.push((val, new_context))
                } else {
                    rets.push((val, c[0].clone()))
                }
                //compact the returns down to one function with a return

                //create the function context
            }
            rets
        } else {
            compiled.1
        }
    };

    Ok((
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
        inner_inner_returns,
    ))
}

fn all_combinations(
    a: Vec<ast::Expression>,
    context: Context,
    globals: &mut Globals,
    info: CompilerInfo,
) -> Result<(Vec<(Vec<Value>, Context)>, Returns), RuntimeError> {
    let mut out: Vec<(Vec<Value>, Context)> = Vec::new();
    let mut inner_returns = Returns::new();
    if a.is_empty() {
        //if there are so value, there is only one combination
        out.push((Vec::new(), context));
    } else {
        let mut a_iter = a.iter();
        //starts with all the combinations of the first expr
        let (start_values, start_returns) =
            a_iter
                .next()
                .unwrap()
                .eval(context, globals, info.clone())?;
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
                let (values, returns) = expr.eval(c.clone(), globals, info.clone())?;
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
    Ok((out, inner_returns))
}
pub fn eval_dict(
    dict: Vec<ast::DictDef>,
    context: Context,
    globals: &mut Globals,
    info: CompilerInfo,
) -> Result<(Returns, Returns), RuntimeError> {
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
    )?;
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
                        a => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!(
                                    "Cannot extract from this value: {}",
                                    a.to_str(globals)
                                ),
                                info: info.clone(),
                            })
                        }
                    });
                }
            };
            expr_index += 1;
        }
        out.push((Value::Dict(dict_out), expressions.1));
    }
    Ok((out, inner_returns))
}

impl ast::Variable {
    pub fn to_value(
        &self,
        context: Context,
        globals: &mut Globals,
        info: CompilerInfo,
    ) -> Result<(Returns, Returns), RuntimeError> {
        // TODO: Check if this variable has native functions called on it, and if not set this to false
        let mut start_val: Vec<(Value, Context)> = Vec::new();
        let mut inner_returns = Returns::new();

        use ast::IDClass;

        match &self.value {
            ast::ValueLiteral::Resolved(r) => start_val.push((r.clone(), context)),
            ast::ValueLiteral::ID(id) => start_val.push((
                match id.class_name {
                    IDClass::Group => {
                        if id.unspecified {
                            Value::Group(Group {
                                id: next_free(
                                    &mut globals.closed_groups,
                                    ast::IDClass::Group,
                                    info.clone(),
                                )?,
                            })
                        } else {
                            Value::Group(Group { id: id.number })
                        }
                    }
                    IDClass::Color => {
                        if id.unspecified {
                            Value::Color(Color {
                                id: next_free(
                                    &mut globals.closed_colors,
                                    ast::IDClass::Color,
                                    info.clone(),
                                )?,
                            })
                        } else {
                            Value::Color(Color { id: id.number })
                        }
                    }
                    IDClass::Block => {
                        if id.unspecified {
                            Value::Block(Block {
                                id: next_free(
                                    &mut globals.closed_blocks,
                                    ast::IDClass::Block,
                                    info.clone(),
                                )?,
                            })
                        } else {
                            Value::Block(Block { id: id.number })
                        }
                    }
                    IDClass::Item => {
                        if id.unspecified {
                            Value::Item(Item {
                                id: next_free(
                                    &mut globals.closed_items,
                                    ast::IDClass::Item,
                                    info.clone(),
                                )?,
                            })
                        } else {
                            Value::Item(Item { id: id.number })
                        }
                    }
                },
                context,
            )),
            ast::ValueLiteral::Number(num) => start_val.push((Value::Number(*num), context)),
            ast::ValueLiteral::Dictionary(dict) => {
                let new_info = info.next("dictionary", globals, false);
                let (new_out, new_inner_returns) =
                    eval_dict(dict.clone(), context, globals, new_info)?;
                start_val = new_out;
                inner_returns = new_inner_returns;
            }
            ast::ValueLiteral::CmpStmt(cmp_stmt) => {
                let (evaled, returns) = cmp_stmt.to_scope(&context, globals, info.clone())?;
                inner_returns.extend(returns);
                start_val.push((Value::Func(evaled), context));
            }

            ast::ValueLiteral::Expression(expr) => {
                let (evaled, returns) = expr.eval(context, globals, info.clone())?;
                inner_returns.extend(returns);
                start_val.extend(evaled.iter().cloned());
            }

            ast::ValueLiteral::Bool(b) => start_val.push((Value::Bool(*b), context)),
            ast::ValueLiteral::Symbol(string) => {
                if string == "$" {
                    start_val.push((Value::Builtins, context));
                } else {
                    match context.variables.get(string) {
                        Some(value) => start_val
                            .push((((*globals).stored_values[*value as usize]).clone(), context)),
                        None => {
                            return Err(RuntimeError::UndefinedErr {
                                undefined: string.clone(),
                                info: info.clone(),
                                desc: "variable".to_string(),
                            })
                        }
                    }
                }
            }
            ast::ValueLiteral::Str(s) => start_val.push((Value::Str(s.clone()), context)),
            ast::ValueLiteral::Array(a) => {
                let new_info = info.next("array", globals, false);
                let (evaled, returns) = all_combinations(a.clone(), context, globals, new_info)?;
                inner_returns.extend(returns);
                start_val = evaled
                    .iter()
                    .map(|x| (Value::Array(x.0.clone()), x.1.clone()))
                    .collect();
            }
            ast::ValueLiteral::Import(i) => {
                let mut new_context = context.clone();
                let (val, imp) = import_module(i, globals, info.clone())?;
                new_context.implementations.extend(imp);
                start_val.push((val, new_context));
            }

            ast::ValueLiteral::TypeIndicator(name) => {
                start_val.push((
                    match globals.type_ids.get(name) {
                        Some(id) => Value::TypeIndicator(*id),
                        None => {
                            return Err(RuntimeError::UndefinedErr {
                                undefined: name.clone(),
                                info: info.clone(),
                                desc: "type".to_string(),
                            });
                        }
                    },
                    context,
                ));
            }
            ast::ValueLiteral::Obj(o) => {
                let mut all_expr: Vec<ast::Expression> = Vec::new();
                for prop in o {
                    all_expr.push(prop.0.clone());
                    all_expr.push(prop.1.clone());
                }
                let new_info = info.next("object", globals, false);
                let (evaled, returns) = all_combinations(all_expr, context, globals, new_info)?;
                inner_returns.extend(returns);
                for (expressions, context) in evaled {
                    let mut obj: Vec<(u16, String)> = Vec::new();
                    for i in 0..(o.len()) {
                        let v = expressions[i * 2].clone();
                        let v2 = expressions[i * 2 + 1].clone();

                        obj.push((
                            match v {
                                Value::Number(n) => n as u16,
                                a => {
                                    return Err(RuntimeError::RuntimeError {
                                        message: format!(
                                            "Expected number type as object key, found: {}",
                                            a.to_str(globals)
                                        ),
                                        info: info.clone(),
                                    })
                                }
                            },
                            match v2 {
                                Value::Number(n) => n.to_string(),
                                Value::Str(s) => s,
                                Value::Func(g) => g.start_group.id.to_string(),

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
                                x => {
                                    return Err(RuntimeError::RuntimeError {
                                        message: format!(
                                            "{} is not a valid object value",
                                            x.to_str(globals)
                                        ),
                                        info: info.clone(),
                                    })
                                }
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
                let new_info = info.next("macro argument", globals, false);
                let (argument_possibilities, returns) =
                    all_combinations(all_expr, context, globals, new_info)?;
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
                    for x in &mut with_parent {
                        *x = (
                            match x.0.member(m.clone(), &x.1, globals, info.clone()) {
                                Some(m) => m,
                                None => {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: m.clone(),
                                        info: info.clone(),
                                        desc: String::from("property"),
                                    })
                                }
                            },
                            x.1.clone(),
                            x.0.clone(),
                        )
                    }
                }

                ast::Path::Index(i) => {
                    let mut new_out: Vec<(Value, Context, Value)> = Vec::new();

                    for (prev_v, prev_c, _) in with_parent.clone() {
                        match prev_v.clone() {
                            Value::Array(arr) => {
                                let new_info = info.next("index", globals, false);
                                let (evaled, returns) = i.eval(prev_c, globals, new_info)?;
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
                                        a => {
                                            return Err(RuntimeError::RuntimeError {
                                                message: format!(
                                                    "expected number in index, found {}",
                                                    a.to_str(globals)
                                                ),
                                                info: info.clone(),
                                            })
                                        }
                                    }
                                }
                            }
                            a => {
                                return Err(RuntimeError::RuntimeError {
                                    message: format!(
                                        "Cannot index this type: {}",
                                        a.to_str(globals)
                                    ),
                                    info: info.clone(),
                                })
                            }
                        }
                    }

                    with_parent = new_out
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
                                )?;
                                inner_returns.extend(returns);
                                with_parent = evaled
                                    .iter()
                                    .map(|x| (x.0.clone(), x.1.clone(), v.clone()))
                                    .collect();
                            }

                            Value::BuiltinFunction(name) => {
                                let (evaled_args, returns) = all_combinations(
                                    args.iter().map(|x| x.value.clone()).collect(),
                                    cont.clone(),
                                    globals,
                                    info.clone(),
                                )?;
                                inner_returns.extend(returns);

                                let mut all_values = Returns::new();

                                for (args, context) in evaled_args {
                                    let evaled = built_in_function(
                                        name,
                                        args.clone(),
                                        info.clone(),
                                        globals,
                                        context.clone(),
                                    )?;
                                    all_values.push((evaled, context))
                                }

                                with_parent = all_values
                                    .iter()
                                    .map(|x| (x.0.clone(), x.1.clone(), v.clone()))
                                    .collect();
                            }
                            a => {
                                return Err(RuntimeError::RuntimeError {
                                    message: format!(
                                        "Cannot call this type with arguments: {}",
                                        a.to_str(globals)
                                    ),
                                    info: info.clone(),
                                })
                            }
                        }
                    }
                }
            };
        }

        let mut out: Returns = with_parent
            .iter()
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect();

        use ast::UnaryOperator;
        if let Some(o) = &self.operator {
            for final_value in &mut out {
                match o {
                    UnaryOperator::Minus => {
                        if let Value::Number(n) = final_value.0 {
                            *final_value = (Value::Number(-n), final_value.1.clone());
                        } else {
                            return Err(RuntimeError::RuntimeError {
                                message: "Cannot make non-number type negative".to_string(),
                                info: info.clone(),
                            });
                        }
                    }

                    UnaryOperator::Not => {
                        if let Value::Bool(b) = final_value.0 {
                            *final_value = (Value::Bool(!b), final_value.1.clone());
                        } else {
                            return Err(RuntimeError::RuntimeError {
                                message: "Cannot negate non-boolean type".to_string(),
                                info: info.clone(),
                            });
                        }
                    }

                    UnaryOperator::Range => {
                        if let Value::Number(n) = final_value.0 {
                            *final_value = (
                                Value::Array(
                                    (if n > 0.0 {
                                        0..(n as i32)
                                    } else {
                                        (n as i32)..0
                                    })
                                    .map(|x| Value::Number(x as f64))
                                    .collect(),
                                ),
                                final_value.1.clone(),
                            );
                        } else {
                            return Err(RuntimeError::RuntimeError {
                                message: "Expected number in range".to_string(),
                                info: info.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok((out, inner_returns))
    }
}

impl ast::CompoundStatement {
    fn to_scope(
        &self,
        context: &Context,
        globals: &mut Globals,
        info: CompilerInfo,
    ) -> Result<(Function, Returns), RuntimeError> {
        //create the function context
        let mut new_context = context.clone();

        new_context.spawn_triggered = true;

        //pick a start group
        let start_group = Group {
            id: next_free(
                &mut globals.closed_groups,
                ast::IDClass::Group,
                info.clone(),
            )?,
        };

        new_context.start_group = start_group;
        let new_info = info.next("function body", globals, true);
        let (_, inner_returns) =
            compile_scope(&self.statements, vec![new_context], globals, new_info)?;

        Ok((Function { start_group }, inner_returns))
    }
}
