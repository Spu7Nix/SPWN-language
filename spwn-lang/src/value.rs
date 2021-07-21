use crate::ast;
use crate::{compiler_types::*, context::*, globals::Globals, levelstring::*, value_storage::*};
use crate::compiler_info::CompilerInfo;
use crate::compiler::import_module;
use crate::builtin::*;
//use std::boxed::Box;

use std::collections::HashMap;
use std::path::PathBuf;
use smallvec::smallvec;

use crate::compiler::RuntimeError;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Group(Group),
    Color(Color),
    Block(Block),
    Item(Item),
    Number(f64),
    Bool(bool),
    TriggerFunc(TriggerFunction),
    Dict(HashMap<String, StoredValue>),
    Macro(Box<Macro>),
    Str(String),
    Array(Vec<StoredValue>),
    Obj(Vec<(u16, ObjParam)>, ast::ObjectMode),
    Builtins,
    BuiltinFunction(String),
    TypeIndicator(TypeId),
    Range(i32, i32, usize), //start, end, step
    Pattern(Pattern),
    Null,
}

const MAX_DICT_EL_DISPLAY: usize = 10;

#[derive(Clone, Debug, PartialEq)]
pub struct Macro {
    //             name         default val      tag          pattern
    pub args: Vec<(
        String,
        Option<StoredValue>,
        ast::Attribute,
        Option<StoredValue>,
    )>,
    pub def_context: Context,
    pub def_file: PathBuf,
    pub body: Vec<ast::Statement>,
    pub tag: ast::Attribute,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TriggerFunction {
    pub start_group: Group,
    //pub all_groups: Vec<Group>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Pattern {
    Type(TypeId),
    Array(Vec<Pattern>),
    Either(Box<Pattern>, Box<Pattern>),
}

pub fn value_equality(val1: StoredValue, val2: StoredValue, globals: &Globals) -> bool {
    match (&globals.stored_values[val1], &globals.stored_values[val2]) {
        (Value::Array(a1), Value::Array(a2)) => {
            if a1.len() != a2.len() {
                return false;
            }

            for i in 0..a1.len() {
                if !value_equality(a1[i], a2[i], globals) {
                    return false;
                }
            }
            true
        }
        (Value::Dict(d1), Value::Dict(d2)) => {
            if d1.len() != d2.len() {
                return false;
            }

            for key in d1.keys() {
                if let Some(val1) = d2.get(key) {
                    if let Some(val2) = d1.get(key) {
                        if !value_equality(*val1, *val2, globals) {
                            return false;
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    return false;
                }
            }
            true
        }
        (a, b) => a == b,
    }
}

impl Value {
    //numeric representation of value
    pub fn to_num(&self, globals: &Globals) -> TypeId {
        match self {
            Value::Group(_) => 0,
            Value::Color(_) => 1,
            Value::Block(_) => 2,
            Value::Item(_) => 3,
            Value::Number(_) => 4,
            Value::Bool(_) => 5,
            Value::TriggerFunc(_) => 6,
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
            Value::Obj(_, mode) => match mode {
                ast::ObjectMode::Object => 11,
                ast::ObjectMode::Trigger => 16,
            },
            Value::Builtins => 12,
            Value::BuiltinFunction(_) => 13,
            Value::TypeIndicator(_) => 14,
            Value::Null => 15,
            Value::Range(_, _, _) => 17,
            Value::Pattern(_) => 18,
        }
    }

    pub fn matches_pat(
        &self,
        pat_val: &Value,
        info: &CompilerInfo,
        globals: &mut Globals,
        context: &Context,
    ) -> Result<bool, RuntimeError> {
        let pat = if let Value::Pattern(p) = convert_type(pat_val, 18, info, globals, context)? {
            p
        } else {
            unreachable!()
        };
        match pat {
            Pattern::Either(p1, p2) => {
                Ok(
                    self.matches_pat(&Value::Pattern(*p1), info, globals, context)?
                        || self.matches_pat(&Value::Pattern(*p2), info, globals, context)?,
                )
            }
            Pattern::Type(t) => Ok(self.to_num(globals) == t),
            Pattern::Array(a_pat) => {
                if let Value::Array(a_val) = self {
                    match a_pat.len() {
                        0 => Ok(true),

                        1 => {
                            for el in a_val {
                                let val = globals.stored_values[*el].clone();
                                if !val.matches_pat(
                                    &Value::Pattern(a_pat[0].clone()),
                                    info,
                                    globals,
                                    context,
                                )? {
                                    return Ok(false);
                                }
                            }
                            Ok(true)
                        }

                        _ => Err(RuntimeError::RuntimeError {
                            message: String::from(
                                "arrays with multiple elements cannot be used as patterns (yet)",
                            ),
                            info: info.clone(),
                        }),
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }
    pub fn to_str(&self, globals: &Globals) -> String {
        match self {
            Value::Group(g) => {
                (if let Id::Specific(id) = g.id {
                    id.to_string()
                } else {
                    "?".to_string()
                }) + "g"
            }
            Value::Color(c) => {
                (if let Id::Specific(id) = c.id {
                    id.to_string()
                } else {
                    "?".to_string()
                }) + "c"
            }
            Value::Block(b) => {
                (if let Id::Specific(id) = b.id {
                    id.to_string()
                } else {
                    "?".to_string()
                }) + "b"
            }
            Value::Item(i) => {
                (if let Id::Specific(id) = i.id {
                    id.to_string()
                } else {
                    "?".to_string()
                }) + "i"
            }
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::TriggerFunc(_) => "!{ /* trigger function */ }".to_string(),
            Value::Range(start, end, stepsize) => {
                if *stepsize != 1 {
                    format!("{}..{}..{}", start, stepsize, end)
                } else {
                    format!("{}..{}", start, end)
                }
            }
            Value::Dict(dict_in) => {
                let mut out = String::new();
                
                let mut d = dict_in.clone();
                if let Some(n) = d.get(TYPE_MEMBER_NAME) {
                    let val = &globals.stored_values[*n];
                    out += &val.to_str(globals);
                    d.remove(TYPE_MEMBER_NAME);
                    out += "::";
                }
                out += "{";
                let mut d_iter = d.iter();
                for (count, (key, val)) in (&mut d_iter).enumerate() {
                    

                    if count > MAX_DICT_EL_DISPLAY {
                        let left = d_iter.count();
                        if left > 0 {
                            out += &format!("... ({} more)  ", left);
    
                        }
                        break;
                        
                    }
                    
                    let stored_val = (*globals).stored_values[*val as usize].to_str(globals);
                    out += &format!("{}: {},", key, stored_val);
                }
                if !d.is_empty() {
                    out.pop();
                }
                

                out += "}"; //why do i have to do this twice? idk

                out
            }
            Value::Macro(m) => {
                let mut out = String::from("(");
                if !m.args.is_empty() {
                    for arg in m.args.iter() {
                        out += &arg.0;
                        if let Some(val) = arg.3 {
                            out += &format!(
                                ": {}",
                                globals.stored_values[val].to_str(globals),
                            )
                        };
                        if let Some(val) = arg.1 {
                            out += &format!(" = {}", globals.stored_values[val].to_str(globals))
                        };
                        out += ", ";
                    }
                    out.pop();
                    out.pop();
                }
                out + ") { /* code omitted */ }"
            }
            Value::Str(s) => s.clone(),
            Value::Array(a) => {
                if a.is_empty() {
                    "[]".to_string()
                } else {
                    let mut out = String::from("[");
                    for val in a {
                        out += &globals.stored_values[*val].to_str(globals);
                        out += ",";
                    }
                    out.pop();
                    out += "]";

                    out
                }
            }
            Value::Obj(o, _) => {
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
                find_key_for_value(&globals.type_ids, *id).unwrap_or(&String::from("[TYPE NOT FOUND]"))
            ),

            Value::Pattern(p) => match p {
                Pattern::Type(t) => Value::TypeIndicator(*t).to_str(globals),
                Pattern::Either(p1, p2) => format!("{} | {}", Value::Pattern(*p1.clone()).to_str(globals), Value::Pattern(*p2.clone()).to_str(globals)),
                Pattern::Array(a) => if a.is_empty() {
                    "[]".to_string()
                } else {
                    let mut out = String::from("[");
                    for p in a {
                        out += &Value::Pattern(p.clone()).to_str(globals);
                        out += ",";
                    }
                    out.pop();
                    out += "]";

                    out
                },
            },
        }
    }
}

pub fn convert_type(
    val: &Value,
    typ: TypeId,
    info: &CompilerInfo,
    globals: &mut Globals,
    context: &Context,
) -> Result<Value, RuntimeError> {

    if val.to_num(globals) == typ {
        return Ok(val.clone())
    }

    if typ == 9 {
        return Ok(Value::Str(val.to_str(globals)));
    }

    Ok(match val {
        Value::Number(n) => match typ {
            0 => Value::Group(Group::new(*n as u16)),
            1 => Value::Color(Color::new(*n as u16)),
            2 => Value::Block(Block::new(*n as u16)),
            3 => Value::Item(Item::new(*n as u16)),
            4 => Value::Number(*n),
            5 => Value::Bool(*n != 0.0),

            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Number can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info: info.clone(),
                })
            }
        },

        Value::Group(g) => match typ {
            
            4 => Value::Number(match g.id {
                Id::Specific(n) => n as f64,
                _ => return Err(RuntimeError::RuntimeError {
                    message: "This group isn\'t known at this time, and can therefore not be converted to a number!".to_string(),
                    info: info.clone(),
                })
            }),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Group can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info: info.clone(),
                })
            }
        },

        Value::Color(c) => match typ {
            
            4 => Value::Number(match c.id {
                Id::Specific(n) => n as f64,
                _ => return Err(RuntimeError::RuntimeError {
                    message: "This color isn\'t known at this time, and can therefore not be converted to a number!".to_string(),
                    info: info.clone(),
                })
            }),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Color can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info: info.clone(),
                })
            }
        },

        Value::Block(b) => match typ {
            
            4 => Value::Number(match b.id {
                Id::Specific(n) => n as f64,
                _ => return Err(RuntimeError::RuntimeError {
                    message: "This block ID isn\'t known at this time, and can therefore not be converted to a number!".to_string(),
                    info: info.clone(),
                })
            }),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Block ID can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info: info.clone(),
                })
            }
        },

        Value::Item(i) => match typ {
            
            4 => Value::Number(match i.id {
                Id::Specific(n) => n as f64,
                _ => return Err(RuntimeError::RuntimeError {
                    message: "This item ID isn\'t known at this time, and can therefore not be converted to a number!".to_string(),
                    info: info.clone(),
                })
            }),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Item ID can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info: info.clone(),
                })
            }
        },

        Value::Bool(b) => match typ {
            
            4 => Value::Number(if *b { 1.0 } else { 0.0 }),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Boolean can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info: info.clone(),
                })
            }
        },

        Value::TriggerFunc(f) => match typ {
            
            0 => Value::Group(f.start_group),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Trigger function can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info: info.clone(),
                })
            }
        },

        Value::Range(start, end, step) => match typ {
            10 => {
                Value::Array(if start < end { 
                    (*start..*end).step_by(*step).map(|x| 
                        store_value(Value::Number(x as f64), 1, globals, &context)).collect::<Vec<StoredValue>>() 
                } else { 
                    (*end..*start).step_by(*step).rev().map(|x| 
                        store_value(Value::Number(x as f64), 1, globals, &context)).collect::<Vec<StoredValue>>()
                })
            },
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Range can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info: info.clone(),
                })
            }
        },

        Value::Str(s) => match typ {
            4 => {
                let out: std::result::Result<f64, _> = s.parse();
                match out {
                    Ok(n) => Value::Number(n),
                    _ => {
                        return Err(RuntimeError::RuntimeError {
                            message: format!("Cannot convert '{}' to @number", s),
                            info: info.clone()
                        })
                    }
                }
            },
            10 => {
                Value::Array(s.chars().map(|x| store_value(Value::Str(x.to_string()), 1, globals, &context)).collect::<Vec<StoredValue>>())
            },
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "String can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info: info.clone(),
                })
            }
        },

        Value::Array(arr) => match typ {
            18 => {
                // pattern
                let mut new_vec = Vec::new();
                for el in arr {
                    new_vec.push(match globals.stored_values[*el].clone() {
                        Value::Pattern(p) => p,
                        a => if let Value::Pattern(p) = convert_type(&a, 18, info, globals, context)? {
                            p
                        } else {
                            unreachable!()
                        },
                    })
                }
                Value::Pattern(Pattern::Array(new_vec))
            }

            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Array can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info: info.clone(),
                })
            }

        }
        Value::TypeIndicator(t) =>  match typ {
            18 => {
                
                Value::Pattern(Pattern::Type(*t))
            }

            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Type-Indicator can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info: info.clone(),
                })
            }
        }

        _ => {
            return Err(RuntimeError::RuntimeError {
                message: format!(
                    "'{}' can't be converted to '{}'!",
                     find_key_for_value(&globals.type_ids, val.to_num(globals)).unwrap(), find_key_for_value(&globals.type_ids, typ).unwrap(),
                ),
                info: info.clone(),
            })
        }
    })

}

//copied from https://stackoverflow.com/questions/59401720/how-do-i-find-the-key-for-a-value-in-a-hashmap
pub fn find_key_for_value(map: &HashMap<String, (u16, PathBuf, (usize, usize))>, value: u16) -> Option<&String> {
    map.iter()
        .find_map(|(key, val)| if val.0 == value { Some(key) } else { None })
}

pub fn macro_to_value(
    m:&ast::Macro,
    context: &Context,
    globals: &mut Globals,
    info: CompilerInfo,
    //mut define_new: bool,
    constant: bool,
    
) -> Result<(Returns, Returns), RuntimeError> {
    let mut all_expr: Vec<ast::Expression> = Vec::new();
    let mut start_val = Returns::new();
    let mut inner_returns = Returns::new();
    for arg in &m.args {
        if let Some(e) = &arg.1 {
            all_expr.push(e.clone());
        }

        if let Some(e) = &arg.3 {
            all_expr.push(e.clone());
        }
    }
    let new_info = info.clone();
    let (argument_possibilities, returns) =
        all_combinations(all_expr, &context, globals, new_info, constant)?;
    inner_returns.extend(returns);
    for defaults in argument_possibilities {
        let mut args: Vec<(String, Option<StoredValue>, ast::Attribute, Option<StoredValue>)> =
            Vec::new();
        let mut expr_index = 0;
        
        for arg in m.args.iter() {
            let def_val = match &arg.1 {
                Some(_) => {
                    expr_index += 1;
                    Some(
                        clone_value(defaults.0[expr_index - 1], 1, globals, defaults.1.start_group, true)
                    )
                }
                None => None,
            };
            let pat = match &arg.3 {
                Some(_) => {
                    expr_index += 1;
                    Some(defaults.0[expr_index - 1])
                }
                None => None,
            };
            args.push((
                arg.0.clone(),
                def_val,
                arg.2.clone(),
                pat,
            ));
        }

        
            start_val.push((
                store_const_value(
                    Value::Macro(Box::new(Macro {
                        args,
                        body: m.body.statements.clone(),
                        def_context: defaults.1.clone(),
                        def_file: info.current_file.clone(),
                        tag: m.properties.clone(),
                    })),
                    1,
                    globals,
                    &context,
                ),
                defaults.1,
            ))
        

        
    }
    Ok((start_val, inner_returns))
}


impl ast::Variable {
    pub fn to_value(
        &self,
        mut context: Context,
        globals: &mut Globals,
        mut info: CompilerInfo,
        //mut define_new: bool,
        constant: bool,
    ) -> Result<(Returns, Returns), RuntimeError> {
        info.pos = self.pos;
        
        let mut start_val = Returns::new();
        let mut inner_returns = Returns::new();

        //let mut defined = true;
        if let Some(UnaryOperator::Let) = self.operator {
            let val = self.define(&mut context, globals, &info)?;
            start_val = smallvec![(val, context)];
            return Ok((start_val, inner_returns));
        }

        use ast::IdClass;
 
        

        match &self.value.body {
            ast::ValueBody::Resolved(r) => start_val.push((*r, context.clone())),
            ast::ValueBody::SelfVal => {
                if let Some(val) = context.variables.get("self") {
                    start_val.push((*val, context.clone()))
                } else {
                    return Err(RuntimeError::RuntimeError {
                        message: "\"self\" is not defined!".to_string(),
                        info,
                    });
                }
            }
            ast::ValueBody::Id(id) => start_val.push((
                store_const_value(
                    match id.class_name {
                        IdClass::Group => {
                            if id.unspecified {
                                Value::Group(Group::next_free(&mut globals.closed_groups))
                            } else {
                                Value::Group(Group::new(id.number))
                            }
                        }
                        IdClass::Color => {
                            if id.unspecified {
                                Value::Color(Color::next_free(&mut globals.closed_colors))
                            } else {
                                Value::Color(Color::new(id.number))
                            }
                        }
                        IdClass::Block => {
                            if id.unspecified {
                                Value::Block(Block::next_free(&mut globals.closed_blocks))
                            } else {
                                Value::Block(Block::new(id.number))
                            }
                        }
                        IdClass::Item => {
                            if id.unspecified {
                                Value::Item(Item::next_free(&mut globals.closed_items))
                            } else {
                                Value::Item(Item::new(id.number))
                            }
                        }
                    },
                    1,
                    globals,
                    &context,
                ),
                context.clone(),
            )),
            ast::ValueBody::Number(num) => start_val.push((
                store_const_value(Value::Number(*num), 1, globals, &context),
                context.clone(),
            )),
            ast::ValueBody::Dictionary(dict) => {
                let new_info = info.clone();
                let (new_out, new_inner_returns) =
                    eval_dict(dict.clone(), &context, globals, new_info, constant)?;
                start_val = new_out;
                inner_returns = new_inner_returns;
            }
            ast::ValueBody::CmpStmt(cmp_stmt) => {
                let (evaled, returns) = cmp_stmt.to_scope(&context, globals, info.clone(), None)?;
                inner_returns.extend(returns);
                start_val.push((
                    store_const_value(Value::TriggerFunc(evaled), 1, globals, &context),
                    context.clone(),
                ));
            }

            ast::ValueBody::Expression(expr) => {
                let (evaled, returns) = expr.eval(&context, globals, info.clone(), constant)?;
                inner_returns.extend(returns);
                start_val.extend(evaled.iter().cloned());
            }

            ast::ValueBody::Bool(b) => start_val.push((
                store_const_value(Value::Bool(*b), 1, globals, &context),
                context.clone(),
            )),
            ast::ValueBody::Symbol(string) => {
                if string == "$" {
                    start_val.push((0, context.clone()));
                } else {
                    match context.variables.get(string) {
                        Some(value) => start_val.push((*value, context.clone())),
                        None => {
                            return Err(RuntimeError::UndefinedErr {
                                undefined: string.clone(),
                                info,
                                desc: "variable".to_string(),
                            });
                        }
                    }
                }
            }
            ast::ValueBody::Str(s) => start_val.push((
                store_const_value(Value::Str(s.clone()), 1, globals, &context),
                context.clone(),
            )),
            ast::ValueBody::Array(a) => {
                let new_info = info.clone();
                let (evaled, returns) =
                    all_combinations(a.clone(), &context, globals, new_info, constant)?;
                inner_returns.extend(returns);
                start_val = evaled
                    .iter()
                    .map(|x| {
                        (
                            store_value(Value::Array(x.0.clone()), 1, globals, &context),
                            x.1.clone(),
                        )
                    })
                    .collect();
            }
            ast::ValueBody::Import(i, f) => {
                //let mut new_contexts = context.clone();
                start_val = import_module(i, &context, globals, info.clone(), *f)?;
            }

            ast::ValueBody::TypeIndicator(name) => {
                start_val.push((
                    match globals.type_ids.get(name) {
                        Some(id) => {
                            store_const_value(Value::TypeIndicator(id.0), 1, globals, &context)
                        }
                        None => {
                            return Err(RuntimeError::UndefinedErr {
                                undefined: name.clone(),
                                info,
                                desc: "type".to_string(),
                            });
                        }
                    },
                    context.clone(),
                ));
            }

            ast::ValueBody::Ternary(t) => {
                
                let (evaled, returns) = t.condition.eval(&context, globals, info.clone(), constant)?;
                // contexts of the conditional

                inner_returns.extend(returns);

                for (condition, context) in evaled { // through every conditional context
                    match &globals.stored_values[condition] {
                        Value::Bool(b) => {
                            let answer = if *b {&t.if_expr} else {&t.else_expr};

                            let (evaled, returns) = answer.eval(&context, globals, info.clone(), constant)?;
                            inner_returns.extend(returns);
                            start_val.extend(evaled);
                        }
                        a => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!("Expected boolean condition in ternary statement, found {}", a.to_str(globals)),
                                info,

                            })
                        }
                    }
                }
            }

            ast::ValueBody::Switch(expr, cases) => {
                // ok so in spwn you have to always assume every expression will split the context, that is,
                // output multiple values in multiple contexts. This is called context splitting. A list of 
                // values and contexts (Vec<(Value, Context)>) is called bundled together in a type called Returns
                let (evaled, returns) = expr.eval(&context, globals, info.clone(), constant)?;
                //inner returns are return statements that are inside the expression, for example in a function/trigger context/ whatever we call it now
                inner_returns.extend(returns);

                // now we loop through every value the first expression outputted
                for (val1, context) in evaled {
                    //lets store the current contexts we are working with in a vector, starting with only the context
                    // outputted from the first expression
                    let mut contexts = vec![context.clone()];


                    for case in cases {
                        // if there are no contexts left to deal with, we can leave the loop
                        if contexts.is_empty() {
                            break
                        }

                        match &case.typ {
                            ast::CaseType::Value(v) => {
                                // in this type of case we want to check if the original expression is
                                // equal to some value. for this, we use the == operator

                                // lets first evaluate the value we will compare to
                                // remember, we have to evaluate it in all the contexts we are working with
                                let mut all_values = Vec::new();
                                for c in &contexts {
                                    let (evaled, returns) = v.eval(c, globals, info.clone(), constant)?;
                                    inner_returns.extend(returns);
                                    all_values.extend(evaled);
                                }

                                // lets clear the contexts list for now, as we will refill it
                                // with new contexts from the next few evaluations
                                contexts.clear();
                                
                                // looping through all the values of the expression we just evaled
                                for (val2, c) in all_values {

                                    // lets compare the two values with the == operator
                                    // since this is an expression in itself, we also have to assume
                                    // this will output multiple values
                                    let results = handle_operator(
                                        val1, 
                                        val2, 
                                        "_equal_", 
                                        &c, 
                                        globals, 
                                        &info
                                    )?;

                                    // lets loop through all those result values
                                    for (r, c) in results {
                                        if let Value::Bool(b) = globals.stored_values[r] {
                                            if b {
                                                // if the two values match, we output this value to the output "start val"
                                                // we can't break here, because the two values might only match in this one context,
                                                // and there may be more contexts left to check
                                                let (evaled, returns) = case.body.eval(&c, globals, info.clone(), constant)?;
                                                inner_returns.extend(returns);
                                                start_val.extend(evaled);
                                            } else {
                                                // if they dont match, we keep going through the cases in this context
                                                contexts.push(c)
                                            }
                                        } else {
                                            // if the == operator for that type doesn't output a boolean, it can't be
                                            // used in a switch statement
                                            return Err(RuntimeError::RuntimeError {
                                                message: "== operator returned non-boolean value".to_string(),
                                                info,
                                                
                                            });
                                        }
                                    }
                                }

                            }
                            ast::CaseType::Pattern(p) => {
                                // this is pretty much the same as the one before, except that we use .matches_pat
                                // to check instead of ==
                                let mut all_patterns = Vec::new();
                                for c in &contexts {
                                    let (evaled, returns) = p.eval(c, globals, info.clone(), constant)?;
                                    inner_returns.extend(returns);
                                    all_patterns.extend(evaled);
                                }
                                contexts.clear();
                                
                                for (pat, c) in all_patterns {
                                    let pat_val = globals.stored_values[pat].clone();
                                    let b = globals.stored_values[val1].clone().matches_pat(&pat_val, &info, globals, &context)?;

                                    if b {
                                        let (evaled, returns) = case.body.eval(&c, globals, info.clone(), constant)?;
                                        inner_returns.extend(returns);
                                        start_val.extend(evaled);
                                    } else {
                                        contexts.push(c)
                                    }
                                        
                                }
                            }

                            ast::CaseType::Default => {
                                //this should be the last case, so we just return the body
                                for c in &contexts {
                                    let (evaled, returns) = case.body.eval(&c, globals, info.clone(), constant)?;
                                    inner_returns.extend(returns);
                                    start_val.extend(evaled);
                                }
                            }
                        }
                        
                    }
                }



            }
            ast::ValueBody::Obj(o) => { // parsing an obj

                let mut all_expr: Vec<ast::Expression> = Vec::new(); // all expressions

                for prop in &o.props { // iterate through obj properties

                    all_expr.push(prop.0.clone()); // this is the object key expression
                    all_expr.push(prop.1.clone()); // this is the object value expression
                }
                let new_info = info.clone();

                let (evaled, returns) =
                    all_combinations(all_expr, &context, globals, new_info, constant)?; // evaluate all expressions gathered
                inner_returns.extend(returns);
                for (expressions, context) in evaled {
                    let mut obj: Vec<(u16, ObjParam)> = Vec::new();
                    for i in 0..(o.props.len()) {

                        let o_key = expressions[i * 2]; 
                        let o_val = expressions[i * 2 + 1];
                        // hopefully self explanatory

                        let (key, pattern) = match &globals.stored_values[o_key] {
                        // key = int of the id, pattern = what type should be expected from the value

                            Value::Number(n) => { // number, i have no clue why people would use this over an obj_key
                                let out = *n as u16;

                                if o.mode == ast::ObjectMode::Trigger && (out == 57 || out == 62) {
                                    return Err(RuntimeError::RuntimeError {
                                        message: "You are not allowed to set the group ID(s) or the spawn triggered state of a @trigger. Use obj instead".to_string(),
                                        info,
                                    })
                                }

                                (out, None)
                            },
                            Value::Dict(d) => { // this is specifically for object_key dicts
                                let gotten_type = d.get(TYPE_MEMBER_NAME);
                                if gotten_type == None ||  globals.stored_values[*gotten_type.unwrap()] != Value::TypeIndicator(19) { // 19 = object_key??
                                    return Err(RuntimeError::RuntimeError {
                                        message: "expected either @number or @object_key as object key".to_string(),
                                        info,
                                    })
                                }
                                let id = d.get("id");
                                if id == None {
                                    return Err(RuntimeError::RuntimeError { // object_key has an ID member for the key basically
                                        message: "object key has no 'id' member".to_string(),
                                        info,
                                    })
                                }
                                let pattern = d.get("pattern");
                                if pattern == None {
                                    return Err(RuntimeError::RuntimeError { // same with pattern, for the expected type
                                        message: "object key has no 'pattern' member".to_string(),
                                        info,
                                    })
                                }

                                (match &globals.stored_values[*id.unwrap()] { // check if the ID is actually an int. it should be
                                    Value::Number(n) => {
                                        let out = *n as u16;

                                        if o.mode == ast::ObjectMode::Trigger && (out == 57 || out == 62) { // group ids and stuff on triggers
                                            return Err(RuntimeError::RuntimeError {
                                                message: "You are not allowed to set the group ID(s) or the spawn triggered state of a @trigger. Use obj instead".to_string(),
                                                info,
                                            })
                                        }
                                        out
                                    }
                                    _ => return Err(RuntimeError::RuntimeError {
                                        message: format!("object key's id has to be @number, found {}", globals.get_type_str(*id.unwrap())),
                                        info,
                                    })
                                }, Some(globals.stored_values[*pattern.unwrap()].clone()))
                                
                            }
                            a => {
                                return Err(RuntimeError::RuntimeError {
                                    message: format!(
                                        "expected either @number or @object_key as object key, found: {}",
                                        a.to_str(globals)
                                    ),
                                    info,
                                })
                            }
                        };

                        obj.push((
                            key,
                            {   // parse the value
                                let val = globals.stored_values[o_val].clone();

                                if let Some(pat) = pattern { // check if pattern is actually enforced (not null)
                                    if !val.matches_pat(&pat, &info, globals, &context)? {
                                        return Err(RuntimeError::RuntimeError {
                                            message: format!(
                                                "key required value to match {}, found {}",
                                                pat.to_str(globals), val.to_str(globals)
                                            ),
                                            info,
                                        })
                                    }
                                }
                                let err = Err(RuntimeError::RuntimeError {
                                    message: format!(
                                        "{} is not a valid object value",
                                        val.to_str(globals)
                                    ),
                                    info: info.clone(),
                                });
                                
                                match &val { // its just converting value to objparam basic level stuff
                                    Value::Number(n) => {
                                        
                                        ObjParam::Number(*n)
                                    },
                                    Value::Str(s) => ObjParam::Text(s.clone()),
                                    Value::TriggerFunc(g) => ObjParam::Group(g.start_group),

                                    Value::Group(g) => ObjParam::Group(*g),
                                    Value::Color(c) => ObjParam::Color(*c),
                                    Value::Block(b) => ObjParam::Block(*b),
                                    Value::Item(i) => ObjParam::Item(*i),

                                    Value::Bool(b) => ObjParam::Bool(*b),

                                    Value::Array(a) => ObjParam::GroupList({
                                        let mut out = Vec::new();
                                        for s in a {
                                            out.push(match globals.stored_values[*s] {
                                                Value::Group(g) => g,
                                                _ => return Err(RuntimeError::RuntimeError {
                                                    message: "Arrays in object parameters can only contain groups".to_string(),
                                                    info,
                                                })
                                            })
                                        }
                                        
                                        out
                                    }),
                                    Value::Dict(d) => {
                                        if let Some(t) = d.get(TYPE_MEMBER_NAME) {
                                            if let Value::TypeIndicator(t) = globals.stored_values[*t] {
                                                if t == 20 { // type indicator number 20 is epsilon ig
                                                    ObjParam::Epsilon
                                                } else {
                                                    return err;
                                                }
                                            } else {
                                                return err;
                                            }
                                        } else {
                                            return err;
                                        }
                                    }
                                    _ => {
                                        return err;
                                    }
                                }
                        
                            },
                        ))
                    }
                    
                    start_val.push((
                        store_const_value(Value::Obj(obj, o.mode), 1, globals, &context),
                        context,
                    ));
                }
            }

            ast::ValueBody::Macro(m) => {
                let (vals, inner_ret) = macro_to_value(m, &context, globals, info.clone(), constant)?;
                start_val.extend(vals); inner_returns.extend(inner_ret);
            }
            //ast::ValueLiteral::Resolved(r) => out.push((r.clone(), context)),
            ast::ValueBody::Null => start_val.push((1, context.clone())),
        };

        let mut path_iter = self.path.iter();
        let mut with_parent: Vec<(StoredValue, Context, StoredValue)> =
            start_val.iter().map(|x| (x.0, x.1.clone(), 1)).collect();
        for p in &mut path_iter {
            // if !defined {
            //     use crate::fmt::SpwnFmt;
            //     return Err(RuntimeError::RuntimeError {
            //         message: format!("Cannot run {} on an undefined value", p.fmt(0)),
            //         info,
            //     });
            // }
            match &p {
                ast::Path::Member(m) => {
                    for x in &mut with_parent {
                        let val = globals.stored_values[x.0].clone(); // this is the object we are getting member of
                        *x = ( 
                            match val.member(m.clone(), &x.1, globals) {
                                Some(m) => m,
                                None => {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: m.clone(),
                                        info,
                                        desc: "member".to_string(),
                                    });
                                }
                            },
                            x.1.clone(),
                            x.0,
                        )
                    }
                }

                ast::Path::Associated(a) => {
                    for x in &mut with_parent {
                        *x = (
                            match &globals.stored_values[x.0] {
                                Value::TypeIndicator(t) => match globals.implementations.get(&t) {
                                    Some(imp) => match imp.get(a) {
                                        Some((val, _)) => {
                                            if let Value::Macro(m) = &globals.stored_values[*val] {
                                                if !m.args.is_empty() && m.args[0].0 == "self" {
                                                    return Err(RuntimeError::RuntimeError {
                                                        message: "Cannot access method (macro with a \"self\" argument) using \"::\"".to_string(),
                                                        info,
                                                    });
                                                }
                                            }
                                            *val
                                        }
                                        None => {
                                            let type_name =
                                                find_key_for_value(&globals.type_ids, *t).unwrap();
                                            return Err(RuntimeError::RuntimeError {
                                                message: format!(
                                                    "No {} property on type @{}",
                                                    a, type_name
                                                ),
                                                info,
                                            });
                                        }
                                    },
                                    None => {
                                        let type_name =
                                            find_key_for_value(&globals.type_ids, *t).unwrap();
                                        return Err(RuntimeError::RuntimeError {
                                            message: format!(
                                                "No values are implemented on @{}",
                                                type_name
                                            ),
                                            info,
                                        });
                                    }
                                },
                                a => {
                                    return Err(RuntimeError::RuntimeError {
                                        message: format!(
                                            "Expected type indicator, found: {}",
                                            a.to_str(globals)
                                        ),
                                        info,
                                    })
                                }
                            },
                            x.1.clone(),
                            x.0,
                        )
                    }
                }

                ast::Path::Index(i) => {
                    let mut new_out: Vec<(StoredValue, Context, StoredValue)> = Vec::new();

                    for (prev_v, prev_c, _) in with_parent.clone() {
                        
                        match globals.stored_values[prev_v].clone() {
                            Value::Array(arr)  => {
                                
                                let (evaled, returns) =
                                    i.eval(&prev_c, globals, info.clone(), constant)?;
                                inner_returns.extend(returns);
                                for index in evaled {
                                    match &globals.stored_values[index.0] {
                                        Value::Number(n) => {
                                            let len = arr.len();
                                            if (*n) < 0.0 && (-*n) as usize >= len {
                                                return Err(RuntimeError::RuntimeError {
                                                    message: format!("Index too low! Index is {}, but length is {}.", n, len),
                                                    info,
                                                });
                                            }
                                            
                                            if *n as usize >= len {
                                                return Err(RuntimeError::RuntimeError {
                                                    message: format!("Index too high! Index is {}, but length is {}.", n, len),
                                                    info,
                                                });
                                            }

                                            if *n < 0.0 {
                                                new_out.push((arr[len - (-n as usize)], index.1, prev_v));
                                            } else {
                                                new_out.push((arr[*n as usize], index.1, prev_v));
                                            }

                                            
                                        }
                                        _ => {
                                            return Err(RuntimeError::RuntimeError {
                                                message: format!(
                                                    "expected @number in index, found @{}",
                                                    globals.get_type_str(index.0)
                                                ),
                                                info,
                                            })
                                        }
                                    }
                                }
                            }
                            Value::Dict(d)  => {
                                
                                let (evaled, returns) =
                                    i.eval(&prev_c, globals, info.clone(), constant)?;
                                inner_returns.extend(returns);
                                for index in evaled {
                                    match &globals.stored_values[index.0] {
                                        Value::Str(s) => {
                                            if !d.contains_key(s) {
                                                return Err(RuntimeError::RuntimeError {
                                                    message: format!("Cannot find key '{}' in dictionary",s),
                                                    info,
                                                })
                                            }
                                            new_out.push((d[s], index.1, prev_v));  
                                        }
                                        _ => {
                                            return Err(RuntimeError::RuntimeError {
                                                message: format!(
                                                    "expected @string in index, found @{}",
                                                    globals.get_type_str(index.0)
                                                ),
                                                info,
                                            })
                                        }
                                    }
                                }
                            }

                            Value::Obj(o, _) => {

                                let (evaled, returns) =
                                    i.eval(&prev_c, globals, info.clone(), constant)?;
                                inner_returns.extend(returns);
                                for index in evaled {
                                    match &globals.stored_values[index.0] {
                                        Value::Dict(d) => {
                                            let gotten_type = d.get(TYPE_MEMBER_NAME);
                                            if gotten_type == None ||  globals.stored_values[*gotten_type.unwrap()] != Value::TypeIndicator(19) { // 19 = object_key??
                                                return Err(RuntimeError::RuntimeError {
                                                    message: "expected either @number or @object_key in index".to_string(),
                                                    info,
                                                })
                                            }

                                            let id = d.get("id");
                                            if id == None {
                                                return Err(RuntimeError::RuntimeError { // object_key has an ID member for the key basically
                                                    message: "object key has no 'id' member".to_string(),
                                                    info,
                                                })
                                            }
                                            let okey = match &globals.stored_values[*id.unwrap()] { // check if the ID is actually an int. it should be
                                                Value::Number(n) => {
                                                    *n as u16
                                                }
                                                _ => return Err(RuntimeError::RuntimeError {
                                                    message: format!("object key's id has to be @number, found {}", globals.get_type_str(*id.unwrap())),
                                                    info,
                                                })
                                            };

                                            let mut contains = false;
                                            for iter in o.iter() {
                                                if iter.0 == okey {
                                                    contains = true;

                                                    let out_val = match &iter.1 { // its just converting value to objparam basic level stuff
                                                        ObjParam::Number(n) => Value::Number(*n),
                                                        ObjParam::Text(s) => Value::Str(s.clone()),

                                                        ObjParam::Group(g) => Value::Group(*g),
                                                        ObjParam::Color(c) => Value::Color(*c),
                                                        ObjParam::Block(b) => Value::Block(*b),
                                                        ObjParam::Item(i) => Value::Item(*i),

                                                        ObjParam::Bool(b) => Value::Bool(*b),

                                                        ObjParam::GroupList(g) => {
                                                            let mut out = Vec::new();
                                                            for s in g {
                                                                let stored = store_const_value(Value::Group(*s), 1, globals, &index.1);
                                                                out.push(stored);
                                                            }
                                                            Value::Array(out)
                                                        },
                                                        
                                                        ObjParam::Epsilon => {
                                                            let mut map = HashMap::<String, StoredValue>::new();
                                                            let stored = store_const_value(Value::TypeIndicator(20), 1, globals, &index.1);
                                                            map.insert(TYPE_MEMBER_NAME.to_string(), stored);
                                                            Value::Dict(map)
                                                        }
                                                    };
                                                    let stored = store_const_value(out_val, globals.stored_values.map.get(&prev_v).unwrap().lifetime, globals, &index.1);
                                                    new_out.push((stored, index.1, prev_v));
                                                    break;
                                                }
                                            }

                                            if !contains {
                                                return Err(RuntimeError::RuntimeError {
                                                    message: "Cannot find key in object".to_string(),
                                                    info,
                                                });
                                            }

                                        }
                                        _ => {
                                            return Err(RuntimeError::RuntimeError {
                                                message: format!(
                                                    "expected @object_key or @number in index, found @{}",
                                                    globals.get_type_str(index.0)
                                                ),
                                                info,
                                            })
                                        }
                                    }
                                }

                            }
                            Value::Str(s)  => {
                                let arr: Vec<char> = s.chars().collect();
                                
                                let (evaled, returns) =
                                    i.eval(&prev_c, globals, info.clone(), constant)?;
                                inner_returns.extend(returns);
                                for index in evaled {
                                    match &globals.stored_values[index.0] {
                                        Value::Number(n) => {
                                            let len = arr.len();
                                            if (*n) < 0.0 && (-*n) as usize >= len {
                                                return Err(RuntimeError::RuntimeError {
                                                    message: format!("Index too low! Index is {}, but length is {}.", n, len),
                                                    info,
                                                });
                                            }
                                            
                                            if *n as usize >= len {
                                                return Err(RuntimeError::RuntimeError {
                                                    message: format!("Index too high! Index is {}, but length is {}.", n, len),
                                                    info,
                                                });
                                            }

                                            let val = if *n < 0.0 {
                                                Value::Str(arr[len - (-n as usize)].to_string())
                                               
                                            } else {
                                                Value::Str(arr[*n as usize].to_string())
                                            };
                                            let stored = store_const_value(val, 1, globals, &index.1);

                                            new_out.push((stored, index.1, prev_v));
                                            
                                        }
                                        _ => {
                                            return Err(RuntimeError::RuntimeError {
                                                message: format!(
                                                    "expected @number in index, found @{}",
                                                    globals.get_type_str(index.0)
                                                ),
                                                info,
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
                                    info,
                                })
                            }
                        }
                    }

                    with_parent = new_out
                }

                ast::Path::Increment => {
                    for (prev_v,prev_c, _) in &mut with_parent {
                        let is_mutable = globals.stored_values.map[&prev_v].mutable;
                        match &mut globals.stored_values[*prev_v] {
                            Value::Number(n) => {
                                *n += 1.0;
                                *prev_v = store_val_m(Value::Number(*n - 1.0),1, globals, prev_c, is_mutable);
                            }
                            _ => {
                                return Err(RuntimeError::RuntimeError {
                                    message: "Cannot increment this type".to_string(),
                                    info,
                                })
                            }
                        }
                    } 
                }

                ast::Path::Decrement => {
                    for (prev_v,prev_c, _) in &mut with_parent {
                        let is_mutable = globals.stored_values.map[&prev_v].mutable;
                        match &mut globals.stored_values[*prev_v] {
                            Value::Number(n) => {
                                *n -= 1.0;                          
                                *prev_v = store_val_m(Value::Number(*n + 1.0),1, globals, prev_c, is_mutable);
                            }
                            _ => {
                                return Err(RuntimeError::RuntimeError {
                                    message: "Cannot decrement this type".to_string(),
                                    info,
                                })
                            }
                        }
                    } 
                }

                ast::Path::Constructor(defs) => {
                    let mut new_out: Vec<(StoredValue, Context, StoredValue)> = Vec::new();

                    for (prev_v, prev_c, _) in &with_parent {
                        match globals.stored_values[*prev_v].clone() {
                            Value::TypeIndicator(t) => {
                                let (dicts, returns) = ast::ValueBody::Dictionary(defs.clone())
                                    .to_variable()
                                    .to_value(prev_c.clone(), globals, info.clone(), constant)?;
                                inner_returns.extend(returns);
                                for dict in &dicts {
                                    let stored_type =
                                        store_value(Value::TypeIndicator(t), 1, globals, &context);
                                    if let Value::Dict(map) = &mut globals.stored_values[dict.0] {
                                        (*map).insert(TYPE_MEMBER_NAME.to_string(), stored_type);
                                    } else {
                                        unreachable!()
                                    }

                                    new_out.push((dict.0, dict.1.clone(), *prev_v));
                                }
                            }
                            a => {
                                return Err(RuntimeError::RuntimeError {
                                message: format!(
                                    "Attempted to construct on a value that is not a type indicator: {}",
                                    a.to_str(globals)
                                ),
                                info,
                            });
                            }
                        }
                    }
                    with_parent = new_out
                }

                ast::Path::Call(args) => {
                    for (v, cont, parent) in with_parent.clone().iter() {
                        match globals.stored_values[*v].clone() {
                            Value::Macro(m) => {
                                let (evaled, returns) = execute_macro(
                                    (*m, args.clone()),
                                    cont,
                                    globals,
                                    *parent,
                                    info.clone(),
                                )?;
                                inner_returns.extend(returns);
                                with_parent =
                                    evaled.iter().map(|x| (x.0, x.1.clone(), *v)).collect();
                            }

                            Value::TypeIndicator(_) => {
                                if args.len() != 1 { // cast takes 1 argument only
                                    return Err(RuntimeError::RuntimeError {
                                        message: format!("casting takes one argument, but {} were provided", args.len()),
                                        info,
                                    })
                                }

                                // one value for each context
                                let mut all_values = Returns::new();

                                //find out whats in the thing we are casting first, its a tuple because contexts and stuff
                                let (evaled, returns) = args[0].value.eval(cont, globals, info.clone(), constant)?; 

                                //return statements are weird in spwn 
                                inner_returns.extend(returns);

                                // go through each context, c = context
                                for (val, c) in evaled {
                                    let evaled = handle_operator(val, *v, "_as_", &c, globals, &info)?; // just use the "as" operator
                                    all_values.extend(evaled);
                                }
                                
                                with_parent =
                                all_values.iter().map(|x| (x.0, x.1.clone(), *v)).collect(); // not sure but it looks important
                            }

                            Value::BuiltinFunction(name) => {
                                let (evaled_args, returns) = all_combinations(
                                    args.iter().map(|x| x.value.clone()).collect(),
                                    cont,
                                    globals,
                                    info.clone(),
                                    constant,
                                )?;
                                inner_returns.extend(returns);

                                let mut all_values = Returns::new();

                                for (args, context) in evaled_args {
                                    let evaled = built_in_function(
                                        &name,
                                        args,
                                        info.clone(),
                                        globals,
                                        &context,
                                    )?;
                                    all_values
                                        .push((store_value(evaled, 1, globals, &context), context))
                                }

                                with_parent =
                                    all_values.iter().map(|x| (x.0, x.1.clone(), *v)).collect();
                            }
                            a => {
                                return Err(RuntimeError::RuntimeError {
                                    message: format!(
                                        "Cannot call ( ... ) on '{}'",
                                        a.to_str(globals)
                                    ),
                                    info,
                                })
                            }
                        }
                    }
                }
            };
        }

        let mut out: Returns = with_parent.iter().map(|x| (x.0, x.1.clone())).collect();

        use ast::UnaryOperator;
        if let Some(o) = &self.operator {
            for final_value in &mut out {
                match o {
                    UnaryOperator::Minus => {
                        if let Value::Number(n) = globals.stored_values[final_value.0] {
                            *final_value = (
                                store_value(Value::Number(-n), 1, globals, &context),
                                final_value.1.clone(),
                            );
                        } else {
                            return Err(RuntimeError::RuntimeError {
                                message: "Cannot make non-number type negative".to_string(),
                                info,
                            });
                        }
                    }

                    UnaryOperator::Increment => {
                        if let Value::Number(n) = &mut globals.stored_values[final_value.0] {
                            *n += 1.0;
                        } else {
                            return Err(RuntimeError::RuntimeError {
                                message: "Cannot increment non-number type".to_string(),
                                info,
                            });
                        }
                    }

                    UnaryOperator::Decrement => {
                        if let Value::Number(n) = &mut globals.stored_values[final_value.0] {
                            *n -= 1.0;
                        } else {
                            return Err(RuntimeError::RuntimeError {
                                message: "Cannot decrement non-number type".to_string(),
                                info,
                            });
                        }
                    }

                    UnaryOperator::Not => {
                        if let Value::Bool(b) = globals.stored_values[final_value.0] {
                            *final_value = (
                                store_value(Value::Bool(!b), 1, globals, &context),
                                final_value.1.clone(),
                            );
                        } else {
                            return Err(RuntimeError::RuntimeError {
                                message: "Cannot negate non-boolean type".to_string(),
                                info,
                            });
                        }
                    }

                    UnaryOperator::Let => (),

                    UnaryOperator::Range => {
                        if let Value::Number(n) = globals.stored_values[final_value.0] {
                            let end = convert_to_int(n, &info)?;
                            *final_value = (
                                store_value(
                                    Value::Range(0, end, 1),
                                    1,
                                    globals,
                                    &context,
                                ),
                                final_value.1.clone(),
                            );
                        } else {
                            return Err(RuntimeError::RuntimeError {
                                message: "Expected number in range".to_string(),
                                info,
                            });
                        }
                    }
                }
            }
        }

        // if self
        //         .tag
        //         .tags
        //         .iter()
        //         .any(|x| x.0 == "allow_context_change")
        // {
            
        //     for (val, _) in &out {
        //         (*globals
        //             .stored_values
        //             .map
        //             .get_mut(val)
        //             .expect("index not found"))
        //             .allow_context_change = true;

                
        //     }
        // }
        if !self.tag.tags.is_empty() {
            for (val, _) in &out {
                if let Value::Macro(m) = &mut globals.stored_values[*val] { m.tag.tags.extend(self.tag.tags.clone()) }
            }
        }

        Ok((out, inner_returns))
    }

    
    pub fn is_undefinable(&self, context: &Context, globals: &mut Globals) -> bool {
        //use crate::fmt::SpwnFmt;
        // if self.operator == Some(ast::UnaryOperator::Let) {
        //     return true
        // }
        
        // println!("hello? {}", self.fmt(0));
        let mut current_ptr = match &self.value.body {
            ast::ValueBody::Symbol(a) => {
                
                if let Some(ptr) = context.variables.get(a) {
                    if self.path.is_empty() {
                        //redefine
                        if globals.is_mutable(*ptr) {
                            return true
                        }
                        if globals.stored_values[*ptr].clone()
                        .member(String::from("_assign_"), &context, globals).is_some() {
                            // if it has assign operator implemented
                            return true
                        }
                        return false
                    }
                    *ptr
                } else {
                    return false;
                }
            }

            ast::ValueBody::TypeIndicator(t) => {
                if let Some(typ) = globals.type_ids.get(t) {
                    store_const_value(Value::TypeIndicator(typ.0), 1, globals, context)
                } else {
                    return false;
                }
            }

            ast::ValueBody::SelfVal => {
                if let Some(ptr) = context.variables.get("self") {
                    *ptr
                } else {
                    return false;
                }
            }

            _ => return true,
        };

        for p in &self.path {
            match p {
                ast::Path::Member(m) => {
                    
                    if let Value::Dict(d) = &globals.stored_values[current_ptr] {
                        match d.get(m) {
                            Some(s) => current_ptr = *s,
                            None => return false,
                        }
                        
                    } else {
                        
                        return true;
                    }
                }
                ast::Path::Associated(m) => match &globals.stored_values[current_ptr] {
                    Value::TypeIndicator(t) => match globals.implementations.get(t) {
                        Some(imp) => {
                            if let Some(val) = imp.get(m) {
                                current_ptr = val.0;
                            } else {
                                return false;
                            }
                        }
                        None => return false,
                    },
                    _ => {
                        return true;
                    }
                },
                ast::Path::Index(i) => {
                    if i.values.len() == 1 {
                        if let ast::ValueBody::Str(s) = &i.values[0].value.body {
                            match &globals.stored_values[current_ptr] {
                                Value::Dict(d)  => {
                                    return d.get(s).is_some()
                                    
                                }
                                _ => return true,
                            }
                        } else {
                            return true
                        }
                    } else {
                        return true
                    }
                    
                }
                _ => return true,
            }
        }

        true
    }
    
    pub fn define(
        &self,
        //value: StoredValue,
        context: &mut Context,
        globals: &mut Globals,
        info: &CompilerInfo,
        
    ) -> Result<StoredValue, RuntimeError> {
        // when None, the value is already defined
        use crate::fmt::SpwnFmt;
        let mut defined = true;
        
        

        let value = match &self.operator {
            Some(ast::UnaryOperator::Let) => store_value(Value::Null, 1, globals, context),
            None => store_const_value(Value::Null, 1, globals, context),
            a => {
                return Err(RuntimeError::RuntimeError {
                    message: format!("Cannot use operator {:?} when defining a variable", a),
                    info: info.clone(),
                })
            }
        };

        let mut current_ptr = match &self.value.body {
            ast::ValueBody::Symbol(a) => {
                if let Some(ptr) = context.variables.get_mut(a) {
                    if self.path.is_empty() {
                        //redefine
                        *ptr = value;
                        return Ok(value)
                    }
                    *ptr
                } else {
                    (*context).variables.insert(a.clone(), value);
                    defined = false;
                    value
                }
            }

            ast::ValueBody::TypeIndicator(t) => {
                if let Some(typ) = globals.type_ids.get(t) {
                    store_const_value(Value::TypeIndicator(typ.0), 1, globals, context)
                } else {
                    return Err(RuntimeError::RuntimeError {
                        message: format!("Use a type statement to define a new type: type {}", t),
                        info: info.clone(),
                    });
                }
            }

            ast::ValueBody::SelfVal => {
                if let Some(ptr) = context.variables.get("self") {
                    *ptr
                } else {
                    return Err(RuntimeError::RuntimeError {
                        message: String::from("\"self\" is not defined!"),
                        info: info.clone(),
                    });
                }
            }

            a => {
                return Err(RuntimeError::RuntimeError {
                    message: format!("Expected symbol or type-indicator, found {}", a.fmt(0)),
                    info: info.clone(),
                })
            }
        };

        

        for p in &self.path {
            (*globals.stored_values.map.get_mut(&value).unwrap()).lifetime = globals.get_lifetime(current_ptr);
            if !defined {
                return Err(RuntimeError::RuntimeError {
                    message: format!("Cannot run {} on an undefined value", p.fmt(0)),
                    info: info.clone(),
                });
            }

            match p {
                ast::Path::Member(m) => {
                    let val = globals.stored_values[current_ptr].clone();
                    match val.member(m.clone(), &context, globals) {
                        Some(s) => current_ptr = s,
                        None => {
                            let stored = globals.stored_values.map.get_mut(&current_ptr).unwrap();
                            if !stored.mutable {
                                return Err(RuntimeError::RuntimeError {
                                    message: "Cannot edit members of a constant value".to_string(),
                                    info: info.clone(),
                                });
                            }
                            if let Value::Dict(d) = &mut stored.val {
                                (*d).insert(m.clone(), value);
                                defined = false;
                                current_ptr = value;
                            } else {
                                return Err(RuntimeError::RuntimeError {
                                    message: "Cannot edit members of a non-dictionary value"
                                        .to_string(),
                                    info: info.clone(),
                                });
                            }
                        }
                    };
                }
                ast::Path::Index(i) => {
                    let (evaled, _) = i.eval(&context, globals, info.clone(), true)?;
                    let first_context_eval = evaled[0].0;
                    match &globals.stored_values[current_ptr] {
                        Value::Dict(d)  => {
                            if evaled.len() > 1 {
                                println!("Warning: context splitting inside of an index definition. Use $.dict_add for better results");
                            }
                            if let Value::Str(st) = globals.stored_values[first_context_eval].clone() {

                                match d.get(&st) {
                                    Some(_) => current_ptr = first_context_eval,
                                    None => {
                                        let stored = globals.stored_values.map.get_mut(&current_ptr).unwrap();
                                        if !stored.mutable {
                                            return Err(RuntimeError::RuntimeError {
                                                message: "Cannot edit members of a constant value".to_string(),
                                                info: info.clone(),
                                            });
                                        }
                                        if let Value::Dict(d) = &mut stored.val {
                                            (*d).insert(st.to_string(), value);
                                            defined = false;
                                            current_ptr = value;
                                        } else {
                                            unreachable!();
                                        }
                                    }
                                };
                            } else {
                                return Err(RuntimeError::RuntimeError {
                                    message: "Only string indexes are supported for dicts".to_string(),
                                    info: info.clone(),
                                });
                            }
                        }
                        _ => {
                            return Err(RuntimeError::RuntimeError {
                                message: "Other values are not supported yet".to_string(),
                                info: info.clone()
                            })
                        },
                    }
                }
                ast::Path::Associated(m) => {
                    match &globals.stored_values[current_ptr] {
                        Value::TypeIndicator(t) => match (*globals).implementations.get_mut(t) {
                            Some(imp) => {
                                if let Some((val,_)) = imp.get(m) {
                                    current_ptr = *val;
                                } else {
                                    (*imp).insert(m.clone(), (value, true));
                                    defined = false;
                                    current_ptr = value;
                                }
                            }
                            None => {
                                let mut new_imp = HashMap::new();
                                new_imp.insert(m.clone(), (value, true));
                                (*globals).implementations.insert(*t, new_imp);
                                defined = false;
                                current_ptr = value;
                            }
                        },
                        a => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!(
                                    "Expected a type-indicator to define an implementation on, found {}",
                                    a.to_str(globals)
                                ),
                                info: info.clone(),
                            });
                        }
                    };
                }
                _ => {
                    return Err(RuntimeError::RuntimeError {
                        message: format!("Cannot run {} in a definition expression", p.fmt(0)),
                        info: info.clone(),
                    })
                }
            }
        }
        
        if defined {
            Err(RuntimeError::RuntimeError {
                message: format!("{} is already defined!", self.fmt(0)),
                info: info.clone(),
            })
        } else {
            Ok(current_ptr)
        }
    }
}