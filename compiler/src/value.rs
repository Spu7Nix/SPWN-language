use crate::builtins::*;
use crate::compiler::import_module;
use crate::compiler::merge_all_contexts;

use errors::compiler_info::CodeArea;
use errors::compiler_info::CompilerInfo;
use errors::create_error;
use parser::ast;
use shared::BreakType;
use shared::SpwnSource;
use shared::StoredValue;
use slyce::Slice as Slyce;

use crate::{compiler_types::*, context::*, globals::Globals, leveldata::*, value_storage::*};
use shared::FileRange;
//use std::boxed::Box;

use internment::LocalIntern;

use fnv::FnvHashMap;
use std::hash::Hash;



use errors::RuntimeError;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Group(Group),
    Color(Color),
    Block(Block),
    Item(Item),
    Number(f64),
    Bool(bool),
    TriggerFunc(TriggerFunction),
    Dict(FnvHashMap<LocalIntern<String>, StoredValue>),
    Macro(Box<Macro>),
    Str(String),
    Array(Vec<StoredValue>),
    Obj(Vec<(u16, ObjParam)>, ast::ObjectMode),
    Builtins,
    BuiltinFunction(Builtin),
    TypeIndicator(TypeId),
    Range(i32, i32, usize), //start, end, step
    Pattern(Pattern),
    Null,
}

pub type Slice = (Option<isize>, Option<isize>, Option<isize>);

const MAX_DICT_EL_DISPLAY: usize = 10;

#[derive(Clone, Debug, PartialEq)]
pub struct Macro {
    pub args: Vec<MacroArgDef>,
    pub def_variables: FnvHashMap<LocalIntern<String>, StoredValue>,
    pub def_file: LocalIntern<SpwnSource>,
    pub body: Vec<ast::Statement>,
    pub tag: ast::Attribute,
    pub arg_pos: FileRange,
    pub ret_pattern: Option<StoredValue>,
}

impl Hash for Macro {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        //self.args.hash(state);
        for i in &self.def_variables {
            i.hash(state);
        }
        self.def_file.hash(state);
        //self.body.hash(state);
        //self.tag.hash(state);
        self.arg_pos.hash(state);
        self.ret_pattern.hash(state);
        /*
            i omitted the stuff that has ast inside cuz it
            was too deep of a rabbit hoke to derive Hash for
        */
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MacroArgDef {
    pub name: LocalIntern<String>,
    pub default: Option<StoredValue>,
    pub attribute: ast::Attribute,
    pub pattern: Option<StoredValue>,
    pub position: FileRange,
    pub as_ref: bool,
}
// impl Macro {
//     pub fn get_arg_area(&self) -> CodeArea {
//         assert!(!self.args.is_empty());
//         let first = self.args.first().unwrap().4;
//         let last = self.args.last().unwrap().4;
//         assert_eq!(first.file, last.file);
//         CodeArea {
//             pos: (first.pos.0, last.pos.1),
//             file: first.file,
//         }
//     }
// }
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct TriggerFunction {
    pub start_group: Group,
    //pub all_groups: Vec<Group>,
}
#[derive(Clone, Debug, PartialEq, Hash)]
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

macro_rules! type_id {
    (group) => {0};
    (color) => {1};
    (block) => {2};
    (item) => {3};
    (number) => {4};
    (bool) => {5};
    (trigger_function) => {6};
    (dictionary) => {7};
    (macro) => {8};
    (string) => {9};
    (array) => {10};
    (object) => {11};
    (spwn) => {12};
    (builtin) => {13};
    (type_indicator) => {14};
    (NULL) => {15};
    (trigger) => {16};
    (range) => {17};
    (pattern) => {18};
    (object_key) => {19};
    (epsilon) => {20};
}

pub(crate) use type_id;

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
            Value::Dict(d) => match d.get(&globals.TYPE_MEMBER_NAME) {
                Some(member) => match globals.stored_values[*member] {
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

    // pub fn direct_references(&self) -> Vec<StoredValue> {
    //     match self {
    //         Value::Array(a) => {
    //             return a.iter().copied().collect()
    //         }
    //         Value::Dict(a) => {
    //             return a.values().copied().collect()
    //         }
    //         Value::Macro(m) => {

    //             let mut out = Vec::new();
    //             out.extend(m.args.iter().filter_map(|a| a.1));
    //             out.extend(m.args.iter().filter_map(|a| a.3));

    //             out.extend(m.def_variables.values());

    //             out
    //         }
    //         _ => Vec::new(),
    //     }
    // }

    pub fn hash<H: std::hash::Hasher>(&self, state: &mut H, globals: &Globals) {
        match self {
            Value::Group(v) => v.hash(state),
            Value::Color(v) => v.hash(state),
            Value::Block(v) => v.hash(state),
            Value::Item(v) => v.hash(state),
            Value::Number(v) => ((v * 100000.0) as usize).hash(state),
            Value::Bool(v) => v.hash(state),
            Value::TriggerFunc(v) => v.hash(state),
            Value::Dict(v) => {
                for (k, el) in v {
                    k.hash(state);
                    globals.stored_values[*el].hash(state, globals);
                }
            },
            Value::Macro(v) => v.hash(state),
            Value::Str(v) => v.hash(state),
            Value::Array(v) => {
                for i in v {
                    globals.stored_values[*i].hash(state, globals);
                }
            },
            Value::Obj(v, m) => {
                for i in v {
                    i.hash(state);
                }
                m.hash(state);
            },
            Value::Builtins => "spwn".hash(state),
            Value::BuiltinFunction(v) => v.hash(state),
            Value::TypeIndicator(v) => v.hash(state),
            Value::Range(s, e, st) => {
                s.hash(state);
                e.hash(state);
                st.hash(state);
            },
            Value::Pattern(v) => v.hash(state),
            Value::Null => "null".hash(state),
        }
    }

    pub fn get_type_str(&self, globals: &Globals) -> String {
        let t = self.to_num(globals);
        find_key_for_value(&globals.type_ids, t).unwrap().clone()
    }

    pub fn matches_pat(
        &self,
        pat_val: &Value,
        info: &CompilerInfo,
        globals: &mut Globals,
        context: &Context,
    ) -> Result<bool, RuntimeError> {
        let pat = if let Value::Pattern(p) = convert_type(pat_val, type_id!(pattern), info, globals, context)? {
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

                        _ => Err(RuntimeError::CustomError(create_error(
                            info.clone(),
                            "arrays with multiple elements cannot be used as patterns (yet)",
                            &[],
                            None,
                        ))),
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }
    pub fn to_str_full<F, E>(&self, globals: &mut Globals, mut display_inner: F) -> Result<String, E> where F: FnMut(&Self, &mut Globals) -> Result<String, E> {
        Ok(match self {
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
                if let Some(n) = d.get(&globals.TYPE_MEMBER_NAME) {
                    let val = globals.stored_values[*n].clone();
                    out += &display_inner(&val, globals)?;
                    d.remove(&globals.TYPE_MEMBER_NAME);
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

                    let stored_val = display_inner(&globals.stored_values[*val].clone(), globals)?;
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
                        out += &arg.name;
                        if let Some(val) = arg.pattern {
                            out += &format!(": {}", display_inner(&globals.stored_values[val].clone(), globals)?)
                        };
                        if let Some(val) = arg.default {
                            out += &format!(" = {}", display_inner(&globals.stored_values[val].clone(), globals)?)
                        };
                        out += ", ";
                    }
                    out.pop();
                    out.pop();
                }
                out + ") { /* code omitted */ }"
            }
            Value::Str(s) => format!("'{}'", s),
            Value::Array(a) => {
                if a.is_empty() {
                    "[]".to_string()
                } else {
                    let mut out = String::from("[");
                    for val in a {
                        out += &display_inner(&globals.stored_values[*val].clone(), globals)?;
                        out += ", ";
                    }
                    out.pop();
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
            Value::BuiltinFunction(n) => format!("<built-in-function: {}>", String::from(*n)),
            Value::Null => "Null".to_string(),
            Value::TypeIndicator(id) => format!(
                "@{}",
                find_key_for_value(&globals.type_ids, *id)
                    .unwrap_or(&String::from("[TYPE NOT FOUND]"))
            ),

            Value::Pattern(p) => match p {
                Pattern::Type(t) => Value::TypeIndicator(*t).to_str(globals),
                Pattern::Either(p1, p2) => format!(
                    "{} | {}",
                    display_inner(&Value::Pattern(*p1.clone()), globals)?,
                    display_inner(&Value::Pattern(*p2.clone()), globals)?
                ),
                Pattern::Array(a) => {
                    if a.is_empty() {
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
                    }
                }
            },
        })
    
    }
    pub fn to_str(&self, globals: &mut Globals) -> String {
        self.to_str_full(globals, |val, globals| -> Result<String, ()> { Ok(val.to_str(globals)) }).unwrap()
    }
    pub fn display(&self, full_context: &mut FullContext, globals: &mut Globals, info: &CompilerInfo) -> Result<String, RuntimeError> {
        display_val(self.clone(), full_context, globals, info)
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
        return Ok(val.clone());
    }

    if typ == 9 {
        return Ok(Value::Str(val.to_str(globals)));
    }

    Ok(match (val, typ) {
        
        (Value::Number(n), 0) => Value::Group(Group::new(*n as u16)),
        (Value::Number(n), 1) => Value::Color(Color::new(*n as u16)),
        (Value::Number(n), 2) => Value::Block(Block::new(*n as u16)),
        (Value::Number(n), 3) => Value::Item(Item::new(*n as u16)),
        (Value::Number(n), 4) => Value::Number(*n),
        (Value::Number(n), 5) => Value::Bool(*n != 0.0),

            
        

        

        (Value::Group(g), 4) => Value::Number(match g.id {
            Id::Specific(n) => n as f64,
            _ => return Err(RuntimeError::CustomError(create_error(
                info.clone(),
                "This group isn't known at this time, and can therefore not be converted to a number!",
                &[],
                None,
            ))) 
            
        }),
        

        (Value::Color(g), 4) => Value::Number(match g.id {
            Id::Specific(n) => n as f64,
            _ => return Err(RuntimeError::CustomError(create_error(
                info.clone(),
                "This color isn't known at this time, and can therefore not be converted to a number!",
                &[],
                None,
            ))) 
            
        }),

        (Value::Block(g), 4) => Value::Number(match g.id {
            Id::Specific(n) => n as f64,
            _ => return Err(RuntimeError::CustomError(create_error(
                info.clone(),
                "This block ID isn't known at this time, and can therefore not be converted to a number!",
                &[],
                None,
            ))) 
            
        }),

        (Value::Item(g), 4) => Value::Number(match g.id {
            Id::Specific(n) => n as f64,
            _ => return Err(RuntimeError::CustomError(create_error(
                info.clone(),
                "This item ID isn't known at this time, and can therefore not be converted to a number!",
                &[],
                None,
            ))) 
            
        }),

    

        (Value::Bool(b),4) => Value::Number(if *b { 1.0 } else { 0.0 }),
        

    

        (Value::TriggerFunc(f),0) => Value::Group(f.start_group),
            

        (Value::Range(start, end, step), 10) => {
            Value::Array(if start < end {
                (*start..*end).step_by(*step).map(|x|
                    store_const_value(Value::Number(x as f64),  globals, context.start_group, info.position)).collect::<Vec<StoredValue>>()
            } else {
                (*end..*start).step_by(*step).rev().map(|x|
                    store_const_value(Value::Number(x as f64),  globals, context.start_group, info.position)).collect::<Vec<StoredValue>>()
            })
        },

    
        (Value::Str(s), 4) => {
            let out: std::result::Result<f64, _> = s.parse();
            match out {
                Ok(n) => Value::Number(n),
                _ => {
                    
                    return Err(RuntimeError::CustomError(create_error(
                        info.clone(),
                        &format!("Cannot convert string '{}' to @number", s),
                        &[],
                        None,
                    ))) 
                }
            }
        },
        (Value::Str(s), 10) => {
            Value::Array(s.chars().map(|x| store_const_value(Value::Str(x.to_string()),  globals, context.start_group, info.position)).collect::<Vec<StoredValue>>())
        },
        

    
        (Value::Array(arr), 18) => {
            // pattern
            let mut new_vec = Vec::new();
            for el in arr {
                new_vec.push(match globals.stored_values[*el].clone() {
                    Value::Pattern(p) => p,
                    a => if let Value::Pattern(p) = convert_type(&a, type_id!(pattern), info, globals, context)? {
                        p
                    } else {
                        unreachable!()
                    },
                })
            }
            Value::Pattern(Pattern::Array(new_vec))
        }

        
    
        (Value::TypeIndicator(t),18) => {

            Value::Pattern(Pattern::Type(*t))
        }

            

        _ => {
            

            return Err(RuntimeError::CustomError(create_error(
                info.clone(),
                &format!(
                    "'{}' can't be converted to '{}'!",
                     find_key_for_value(&globals.type_ids, val.to_num(globals)).unwrap(), find_key_for_value(&globals.type_ids, typ).unwrap(),
                ),
                &[],
                None,
            ))) 
        }
    })
}

//copied from https://stackoverflow.com/questions/59401720/how-do-i-find-the-key-for-a-value-in-a-hashmap
pub fn find_key_for_value(
    map: &FnvHashMap<String, (u16, CodeArea)>,
    value: u16,
) -> Option<&String> {
    map.iter()
        .find_map(|(key, val)| if val.0 == value { Some(key) } else { None })
}

pub fn slice_array(
    input: &[StoredValue],
    slices_: Vec<Slice>, //note: slices are in *reverse order*
    globals: &mut Globals,
    info: CompilerInfo,
    context: &Context,
) -> Result<Vec<StoredValue>, RuntimeError> {
    let mut slices = slices_;

    let current_slice = slices.pop().unwrap();
    let s = Slyce {
        start: current_slice.0.into(),
        end: current_slice.1.into(),
        step: current_slice.2,
    };

    let sliced = s.apply(input).copied().collect::<Vec<_>>();

    let mut result = Vec::<StoredValue>::new();

    for i in &sliced {
        if !slices.is_empty() {
            let val = match globals.stored_values[*i].clone() {
                Value::Array(arr) => {
                    slice_array(&arr, slices.clone(), globals, info.clone(), context)?
                }
                _ => {
                    return Err(RuntimeError::CustomError(create_error(
                        info,
                        "Cannot slice nonconforming multidimensional array",
                        &[],
                        None,
                    )));
                }
            };

            let stored_arr = store_const_value(
                Value::Array(val),
                globals,
                context.start_group,
                info.position,
            );
            result.push(stored_arr);
        } else {
            return Ok(sliced);
        }
    }
    Ok(result)
}

use crate::compiler_types::EvalExpression;
use crate::compiler_types::ToTriggerFunc;

pub fn macro_to_value(
    m: &ast::Macro,
    contexts: &mut FullContext,
    globals: &mut Globals,
    info: CompilerInfo,
    //mut define_new: bool,
    constant: bool,
) -> Result<(), RuntimeError> {
    globals.push_new_preserved();
    // todo: add check for context split on pattern and default vals
    for full_context in contexts.iter() {
        let fn_context = full_context.inner().start_group;
        let mut args: Vec<MacroArgDef> = Vec::new();

        for (name, default, attr, pat, pos, as_ref) in m.args.iter() {
            let def_val = match default {
                Some(e) => {
                    e.eval(full_context, globals, info.clone(), constant)?;

                    if full_context.inner().start_group != fn_context {
                        return Err(RuntimeError::ContextChangeError {
                            message: "A macro argument default value can't change the trigger function context".to_string(),
                            info,
                            context_changes: full_context.inner().fn_context_change_stack.clone()
                        });
                    }

                    let out = clone_value(
                        full_context.inner().return_value,
                        globals,
                        full_context.inner().start_group,
                        true,
                        info.position,
                    );

                    globals.push_preserved_val(out);

                    Some(out)
                }
                None => None,
            };
            let pat = match pat {
                Some(e) => {
                    e.eval(full_context, globals, info.clone(), constant)?;

                    if full_context.inner().start_group != fn_context {
                        return Err(RuntimeError::ContextChangeError {
                            message: "A macro argument pattern can't change the trigger function context".to_string(),
                            info,
                            context_changes: full_context.inner().fn_context_change_stack.clone()
                        });
                    }

                    globals.push_preserved_val(full_context.inner().return_value);

                    Some(full_context.inner().return_value)
                }
                None => None,
            };
            args.push(MacroArgDef {
                name: *name,
                default: def_val,
                attribute: attr.clone(),
                pattern: pat,
                position: *pos,
                as_ref: *as_ref,
            });
        }

        let ret_pattern = if let Some(expr) = &m.ret_type {
            expr.eval(full_context, globals, info.clone(), constant)?;
        
            if full_context.inner().start_group != fn_context {
                return Err(RuntimeError::ContextChangeError {
                    message: "A macro return pattern can't change the trigger function context".to_string(),
                    info,
                    context_changes: full_context.inner().fn_context_change_stack.clone()
                });
            }

            let out = full_context.inner().return_value;

            globals.push_preserved_val(out);
            Some(out)
        } else {
            None
        };

        full_context.inner().return_value = store_const_value(
            Value::Macro(Box::new(Macro {
                args,
                body: m.body.statements.clone(),
                def_variables: full_context
                    .inner()
                    .get_variables()
                    .iter()
                    .map(|(name, s)| (*name, s.last().unwrap().val))
                    .collect(),
                def_file: info.position.file,
                arg_pos: m.arg_pos,
                tag: m.properties.clone(),
                ret_pattern
            })),
            globals,
            full_context.inner().start_group,
            info.position,
        );
    }

    globals.pop_preserved();

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DefineResult {
    AlreadyDefined(bool), // bool: redefinable
    Ok, // wasn't defined before, but is now
}
// the actual value comes in context.return_value

// bruh moment
pub trait VariableFuncs {
    fn to_value(
        &self,
        contexts: &mut FullContext,
        globals: &mut Globals,
        info: CompilerInfo,
        constant: bool,
    ) -> Result<(), RuntimeError>;

    //fn is_undefinable(&self, context: &Context, globals: &mut Globals, dstruct_allowed: bool) -> bool;

    fn try_define(
        &self,
        contexts: &mut FullContext,
        globals: &mut Globals,
        info: &CompilerInfo,
        mutable: bool,
        layer: i16
    ) -> Result<DefineResult, RuntimeError>;
}

impl VariableFuncs for ast::Variable {
    fn to_value(
        &self,
        contexts: &mut FullContext,
        globals: &mut Globals,
        mut info: CompilerInfo,
        constant: bool,
    ) -> Result<(), RuntimeError> {
        contexts.reset_return_vals(globals);
        info.position.pos = self.pos;

        use ast::IdClass;
        for full_context in contexts.iter() {
            match &self.value.body {
                ast::ValueBody::Resolved(r) => full_context.inner().return_value = *r,
                ast::ValueBody::SelfVal => {
                    if let Some(val) = full_context.inner().get_variable(globals.SELF_MEMBER_NAME) {
                        full_context.inner().return_value = val
                    } else {
                        return Err(RuntimeError::UndefinedErr {
                            undefined: globals.SELF_MEMBER_NAME.to_string(),
                            desc: "variable".to_string(),
                            info,
                        });
                    }
                }
                ast::ValueBody::Id(id) => {
                    full_context.inner().return_value = store_const_value(
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
                        globals,
                        full_context.inner().start_group,
                        info.position,
                    )
                }
                ast::ValueBody::Number(num) => {
                    full_context.inner().return_value = store_const_value(
                        Value::Number(*num),
                        globals,
                        full_context.inner().start_group,
                        info.position,
                    )
                }
                ast::ValueBody::Dictionary(dict) => {
                    eval_dict(dict.clone(), full_context, globals, info.clone(), constant)?
                }
                ast::ValueBody::CmpStmt(cmp_stmt) => {
                    cmp_stmt.to_trigger_func(full_context, globals, info.clone(), None)?
                }

                ast::ValueBody::Expression(expr) => {
                    expr.eval(full_context, globals, info.clone(), constant)?
                }

                ast::ValueBody::Bool(b) => {
                    full_context.inner().return_value = store_const_value(
                        Value::Bool(*b),
                        globals,
                        full_context.inner().start_group,
                        info.position,
                    )
                }
                ast::ValueBody::Symbol(string) => {
                    if string.as_ref() == "$" {
                        full_context.inner().return_value = globals.BUILTIN_STORAGE;
                    } else {
                        match full_context.inner().get_variable(*string) {
                            Some(value) => full_context.inner().return_value = value,
                            None => {
                                let mut similar_names = Vec::new();
                                let mut extracts = Vec::new();
                                for (name, v) in full_context.inner().get_variables() {
                                    let dist = distance::damerau_levenshtein(name, string);
                                    if distance::damerau_levenshtein(name, string) < 3 {
                                        similar_names.push((name.to_string(), dist));
                                    }
                                    let val = v.last().unwrap().val;

                                    if let Value::Dict(d) = &globals.stored_values[val] {
                                        for key in d.keys() {
                                            if key == string {
                                                extracts.push(name.to_string());
                                                break;
                                            }
                                        }

                                        similar_names.extend(d.keys().filter_map(|key| {
                                            let dist = distance::damerau_levenshtein(key, string);
                                            if dist < 2 {
                                                Some((format!("{}.{}", name, key), dist + 1))
                                            } else {
                                                None
                                            }
                                        }))
                                    }
                                }
                                for key in crate::builtins::BUILTIN_NAMES {
                                    if *key == string.as_str() {
                                        extracts.push("$".to_string());
                                        break;
                                    }
                                }
                                for key in crate::builtins::BUILTIN_NAMES {
                                    let dist = distance::damerau_levenshtein(key, string);
                                    if dist < 3 {
                                        similar_names.push((format!("$.{}", key), dist + 1));
                                    }
                                }
                                similar_names.sort_by_key(|a| a.1);
                                let msg = &format!(
                                    "Maybe you meant {}{}",
                                    match similar_names.len() {
                                        0 => String::new(),
                                        1 => format!("`{}`", similar_names[0].0),
                                        _ => format!(
                                            "{} or `{}`",
                                            similar_names[..(similar_names.len() - 1)]
                                                .iter()
                                                .map(|a| format!("`{}`", a.0))
                                                .collect::<Vec<_>>()
                                                [..std::cmp::min(5, similar_names.len() - 1)]
                                                .join(", "),
                                            similar_names.last().unwrap().0
                                        ),
                                    },
                                    if !extracts.is_empty() {
                                        format!(", or maybe you forgot to add {} to the top of your file?", match extracts.len() {
                                        1 => format!("`extract {}`", extracts[0]),
                                        _ => format!("{} or `extract {}`", extracts[..(extracts.len() - 1)].iter().map(|a| format!("`extract {}`", a)).collect::<Vec<_>>()[..std::cmp::min(5, extracts.len() - 1)].join(", "), extracts.last().unwrap())
                                    })
                                    } else {
                                        "?".to_string()
                                    }
                                );

                                let note: Option<&str> = if similar_names.is_empty() {
                                    None
                                } else {
                                    Some(msg)
                                };
                                return Err(RuntimeError::CustomError(create_error(
                                    info.clone(),
                                    &format!("`{}` is not defined in this scope", string),
                                    &[(
                                        CodeArea {
                                            pos: self.pos,
                                            ..info.position
                                        },
                                        &format!("`{}` is not defined", string),
                                    )],
                                    note,
                                )));
                            }
                        }
                    }
                }
                ast::ValueBody::Str(s) => {
                    full_context.inner().return_value = store_const_value(
                        Value::Str(s.inner.clone()),
                        globals,
                        full_context.inner().start_group,
                        info.position,
                    )
                }

                ast::ValueBody::ListComp(comp) => {
                    globals.push_new_preserved();
                    comp.iterator
                        .eval(full_context, globals, info.clone(), true)?;

                    let i_name = comp.symbol;

                    for context in full_context.iter() {
                        let (_, val) = context.inner_value();

                        globals.push_preserved_val(val);

                        context.inner().return_value = store_const_value(
                            Value::Array(vec![]),
                            globals,
                            context.inner().start_group,
                            info.position,
                        );

                        match globals.stored_values[val].clone() {
                            // what are we iterating
                            Value::Array(arr) => {
                                // its an array!

                                for element in arr {
                                    context.set_variable_and_clone(
                                        i_name,
                                        element,
                                        -1, // so that it gets removed at the end of the scope
                                        true,
                                        globals,
                                        globals.get_area(element),
                                    );

                                    for con_iter in context.iter() {
                                        con_iter.enter_scope(); // mini scope sandwich

                                        let item_list = globals.stored_values
                                            [con_iter.inner().return_value]
                                            .clone();

                                        match &comp.condition {
                                            Some(cond) => {
                                                cond.eval(con_iter, globals, info.clone(), true)?;
                                                for cond_ctx in con_iter.iter() {
                                                    globals.push_preserved_val(
                                                        cond_ctx.inner().return_value,
                                                    );
                                                    match &globals.stored_values
                                                        [cond_ctx.inner().return_value]
                                                    {
                                                        Value::Bool(b) => {
                                                            if *b {
                                                                comp.body.eval(
                                                                    cond_ctx,
                                                                    globals,
                                                                    info.clone(),
                                                                    true,
                                                                )?;
                                                                for expr_ctx in cond_ctx.iter() {
                                                                    let mut local_list =
                                                                        item_list.clone();
                                                                    if let Value::Array(ref mut a) =
                                                                        local_list
                                                                    {
                                                                        a.push(
                                                                            expr_ctx
                                                                                .inner()
                                                                                .return_value,
                                                                        );
                                                                    } else {
                                                                        unreachable!();
                                                                    }

                                                                    expr_ctx.inner().return_value =
                                                                        store_const_value(
                                                                            local_list,
                                                                            globals,
                                                                            expr_ctx
                                                                                .inner()
                                                                                .start_group,
                                                                            info.position,
                                                                        );
                                                                    globals.push_preserved_val(
                                                                        expr_ctx
                                                                            .inner()
                                                                            .return_value,
                                                                    );
                                                                }
                                                            } else {
                                                                cond_ctx.inner().return_value =
                                                                    store_const_value(
                                                                        item_list.clone(),
                                                                        globals,
                                                                        cond_ctx
                                                                            .inner()
                                                                            .start_group,
                                                                        info.position,
                                                                    );
                                                                globals.push_preserved_val(
                                                                    cond_ctx.inner().return_value,
                                                                );
                                                            }
                                                        }
                                                        a => {
                                                            return Err(RuntimeError::TypeError {
                                                                expected: "bool".to_string(),
                                                                found: a.get_type_str(globals),
                                                                val_def: globals.get_area(val),
                                                                info,
                                                            })
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                                comp.body.eval(
                                                    con_iter,
                                                    globals,
                                                    info.clone(),
                                                    true,
                                                )?;
                                                for expr_ctx in con_iter.iter() {
                                                    let mut local_list = item_list.clone();
                                                    if let Value::Array(ref mut a) = local_list {
                                                        a.push(expr_ctx.inner().return_value);
                                                    } else {
                                                        unreachable!();
                                                    }

                                                    expr_ctx.inner().return_value =
                                                        store_const_value(
                                                            local_list,
                                                            globals,
                                                            expr_ctx.inner().start_group,
                                                            info.position,
                                                        );
                                                    globals.push_preserved_val(
                                                        expr_ctx.inner().return_value,
                                                    );
                                                }
                                            }
                                        }

                                        con_iter.exit_scope();
                                    }
                                }
                                //println!("{:?}", out);
                            }
                            Value::Dict(d) => {
                                // its a dict!

                                for (k, v) in d {
                                    for c in context.iter() {
                                        let fn_context = c.inner().start_group;
                                        let key = store_val_m(
                                            Value::Str(k.as_ref().clone()),
                                            globals,
                                            fn_context,
                                            true,
                                            globals.get_area(v),
                                        );
                                        let val = clone_value(
                                            v,
                                            globals,
                                            fn_context,
                                            true,
                                            globals.get_area(v),
                                        );
                                        // reset all variables per context
                                        (*c.inner()).new_variable(
                                            i_name,
                                            store_const_value(
                                                Value::Array(vec![key, val]),
                                                globals,
                                                fn_context,
                                                globals.get_area(v),
                                            ),
                                            -1,
                                        );
                                    }

                                    for con_iter in context.iter() {
                                        con_iter.enter_scope(); // mini scope sandwich

                                        let item_list = globals.stored_values
                                            [con_iter.inner().return_value]
                                            .clone();

                                        match &comp.condition {
                                            Some(cond) => {
                                                cond.eval(con_iter, globals, info.clone(), true)?;
                                                for cond_ctx in con_iter.iter() {
                                                    match &globals.stored_values
                                                        [cond_ctx.inner().return_value]
                                                    {
                                                        Value::Bool(b) => {
                                                            if *b {
                                                                comp.body.eval(
                                                                    cond_ctx,
                                                                    globals,
                                                                    info.clone(),
                                                                    true,
                                                                )?;
                                                                for expr_ctx in cond_ctx.iter() {
                                                                    let mut local_list =
                                                                        item_list.clone();
                                                                    if let Value::Array(ref mut a) =
                                                                        local_list
                                                                    {
                                                                        a.push(
                                                                            expr_ctx
                                                                                .inner()
                                                                                .return_value,
                                                                        );
                                                                    } else {
                                                                        unreachable!();
                                                                    }

                                                                    expr_ctx.inner().return_value =
                                                                        store_const_value(
                                                                            local_list,
                                                                            globals,
                                                                            expr_ctx
                                                                                .inner()
                                                                                .start_group,
                                                                            info.position,
                                                                        );
                                                                }
                                                            } else {
                                                                cond_ctx.inner().return_value =
                                                                    store_const_value(
                                                                        item_list.clone(),
                                                                        globals,
                                                                        cond_ctx
                                                                            .inner()
                                                                            .start_group,
                                                                        info.position,
                                                                    );
                                                            }
                                                        }
                                                        a => {
                                                            return Err(RuntimeError::TypeError {
                                                                expected: "bool".to_string(),
                                                                found: a.get_type_str(globals),
                                                                val_def: globals.get_area(val),
                                                                info,
                                                            })
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                                comp.body.eval(
                                                    con_iter,
                                                    globals,
                                                    info.clone(),
                                                    true,
                                                )?;
                                                for expr_ctx in con_iter.iter() {
                                                    let mut local_list = item_list.clone();
                                                    if let Value::Array(ref mut a) = local_list {
                                                        a.push(expr_ctx.inner().return_value);
                                                    } else {
                                                        unreachable!();
                                                    }

                                                    expr_ctx.inner().return_value =
                                                        store_const_value(
                                                            local_list,
                                                            globals,
                                                            expr_ctx.inner().start_group,
                                                            info.position,
                                                        );
                                                }
                                            }
                                        }

                                        con_iter.exit_scope();
                                    }
                                }
                            }
                            Value::Str(s) => {
                                for ch in s.chars() {
                                    context.set_variable_and_store(
                                        i_name,
                                        Value::Str(ch.to_string()),
                                        -1, // so that it gets removed at the end of the scope
                                        true,
                                        globals,
                                        info.position,
                                    );

                                    for con_iter in context.iter() {
                                        con_iter.enter_scope(); // mini scope sandwich

                                        let item_list = globals.stored_values
                                            [con_iter.inner().return_value]
                                            .clone();

                                        match &comp.condition {
                                            Some(cond) => {
                                                cond.eval(con_iter, globals, info.clone(), true)?;
                                                for cond_ctx in con_iter.iter() {
                                                    match &globals.stored_values
                                                        [cond_ctx.inner().return_value]
                                                    {
                                                        Value::Bool(b) => {
                                                            if *b {
                                                                comp.body.eval(
                                                                    cond_ctx,
                                                                    globals,
                                                                    info.clone(),
                                                                    true,
                                                                )?;
                                                                for expr_ctx in cond_ctx.iter() {
                                                                    let mut local_list =
                                                                        item_list.clone();
                                                                    if let Value::Array(ref mut a) =
                                                                        local_list
                                                                    {
                                                                        a.push(
                                                                            expr_ctx
                                                                                .inner()
                                                                                .return_value,
                                                                        );
                                                                    } else {
                                                                        unreachable!();
                                                                    }

                                                                    expr_ctx.inner().return_value =
                                                                        store_const_value(
                                                                            local_list,
                                                                            globals,
                                                                            expr_ctx
                                                                                .inner()
                                                                                .start_group,
                                                                            info.position,
                                                                        );
                                                                }
                                                            } else {
                                                                cond_ctx.inner().return_value =
                                                                    store_const_value(
                                                                        item_list.clone(),
                                                                        globals,
                                                                        cond_ctx
                                                                            .inner()
                                                                            .start_group,
                                                                        info.position,
                                                                    );
                                                            }
                                                        }
                                                        a => {
                                                            return Err(RuntimeError::TypeError {
                                                                expected: "bool".to_string(),
                                                                found: a.get_type_str(globals),
                                                                val_def: globals.get_area(val),
                                                                info,
                                                            })
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                                comp.body.eval(
                                                    con_iter,
                                                    globals,
                                                    info.clone(),
                                                    true,
                                                )?;
                                                for expr_ctx in con_iter.iter() {
                                                    let mut local_list = item_list.clone();
                                                    if let Value::Array(ref mut a) = local_list {
                                                        a.push(expr_ctx.inner().return_value);
                                                    } else {
                                                        unreachable!();
                                                    }

                                                    expr_ctx.inner().return_value =
                                                        store_const_value(
                                                            local_list,
                                                            globals,
                                                            expr_ctx.inner().start_group,
                                                            info.position,
                                                        );
                                                }
                                            }
                                        }

                                        con_iter.exit_scope();
                                    }
                                }
                            }

                            Value::Range(start, end, step) => {
                                let mut normal = (start..end).step_by(step);
                                let mut rev = (end..start).step_by(step).rev();
                                let range: &mut dyn Iterator<Item = i32> =
                                    if start < end { &mut normal } else { &mut rev };

                                for num in range {
                                    context.set_variable_and_store(
                                        i_name,
                                        Value::Number(num as f64),
                                        -1, // so that it gets removed at the end of the scope
                                        true,
                                        globals,
                                        info.position,
                                    );

                                    for con_iter in context.iter() {
                                        con_iter.enter_scope(); // mini scope sandwich

                                        let item_list = globals.stored_values
                                            [con_iter.inner().return_value]
                                            .clone();

                                        match &comp.condition {
                                            Some(cond) => {
                                                cond.eval(con_iter, globals, info.clone(), true)?;
                                                for cond_ctx in con_iter.iter() {
                                                    match &globals.stored_values
                                                        [cond_ctx.inner().return_value]
                                                    {
                                                        Value::Bool(b) => {
                                                            if *b {
                                                                comp.body.eval(
                                                                    cond_ctx,
                                                                    globals,
                                                                    info.clone(),
                                                                    true,
                                                                )?;
                                                                for expr_ctx in cond_ctx.iter() {
                                                                    let mut local_list =
                                                                        item_list.clone();
                                                                    if let Value::Array(ref mut a) =
                                                                        local_list
                                                                    {
                                                                        a.push(
                                                                            expr_ctx
                                                                                .inner()
                                                                                .return_value,
                                                                        );
                                                                    } else {
                                                                        unreachable!();
                                                                    }

                                                                    expr_ctx.inner().return_value =
                                                                        store_const_value(
                                                                            local_list,
                                                                            globals,
                                                                            expr_ctx
                                                                                .inner()
                                                                                .start_group,
                                                                            info.position,
                                                                        );
                                                                }
                                                            } else {
                                                                cond_ctx.inner().return_value =
                                                                    store_const_value(
                                                                        item_list.clone(),
                                                                        globals,
                                                                        cond_ctx
                                                                            .inner()
                                                                            .start_group,
                                                                        info.position,
                                                                    );
                                                            }
                                                        }
                                                        a => {
                                                            return Err(RuntimeError::TypeError {
                                                                expected: "bool".to_string(),
                                                                found: a.get_type_str(globals),
                                                                val_def: globals.get_area(val),
                                                                info,
                                                            })
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                                comp.body.eval(
                                                    con_iter,
                                                    globals,
                                                    info.clone(),
                                                    true,
                                                )?;
                                                for expr_ctx in con_iter.iter() {
                                                    let mut local_list = item_list.clone();
                                                    if let Value::Array(ref mut a) = local_list {
                                                        a.push(expr_ctx.inner().return_value);
                                                    } else {
                                                        unreachable!();
                                                    }

                                                    expr_ctx.inner().return_value =
                                                        store_const_value(
                                                            local_list,
                                                            globals,
                                                            expr_ctx.inner().start_group,
                                                            info.position,
                                                        );
                                                }
                                            }
                                        }

                                        con_iter.exit_scope();
                                    }
                                }
                            }

                            a => {
                                return Err(RuntimeError::TypeError {
                                    expected: "array, dictionary, string or range".to_string(),
                                    found: a.get_type_str(globals),
                                    val_def: globals.get_area(val),
                                    info,
                                })
                            }
                        }

                        /*context.inner().return_value = store_const_value(
                            Value::Array(output),
                            globals,
                            context.inner().start_group,
                            info.position
                        );*/
                    }
                    globals.pop_preserved();
                }

                ast::ValueBody::Array(a) => {
                    //let combinations = all_combinations(a.iter().map(|ref x| x.value.clone()).collect::<Vec<_>>(), full_context, globals, info.clone(), constant)?;

                    let combinations: Vec<(_, _)> = reduce_combinations(
                        a.clone(), 
                        full_context, 
                        |item: &ast::ArrayDef, ctx, list: Vec<StoredValue>, globals| {
                            let mut added = Vec::new();
                            match item.operator {
                                Some(ast::ArrayPrefix::Collect) => {
                                    let expr = &item.value;
                                    if expr.values.len() == 1 && matches!(expr.values[0].value.body, ast::ValueBody::Array(_)) {
                                        // ..[a, b]

                                    } else {
                                        // ..a
                                        expr.eval(ctx, globals, info.clone(), constant)?;
                                        for expr_context in ctx.iter() {
                                            let evaled_expr = expr_context.inner().return_value;
                                            match &globals.stored_values[evaled_expr] {
                                                Value::Array(ar) => {

                                                },
                                                a => {
                                                    return Err(RuntimeError::TypeError {
                                                        expected: "array".to_string(),
                                                        found: a.get_type_str(globals),
                                                        val_def: globals.get_area(evaled_expr),
                                                        info: info.clone(),
                                                    })
                                                }
                                            }
                                        }
                                    }
                                },
                                _ => {
                                    // a
                                    item.value.eval(ctx, globals, info.clone(), constant)?;

                                    for full_context in ctx.iter() {
                                        let result = full_context.inner().return_value;
                                        let mut updated_list = list.clone();

                                        updated_list.push(result);
                                        globals.push_preserved_val(result);
                                        added.push((updated_list, full_context));
                                    }
                                }
                            }
                            Ok(added)
                        },
                        globals,
                    )?;
                    //panic!("fix soon");

                    for (arr, fc) in combinations {
                        fc.inner().return_value = store_const_value(
                            Value::Array(
                                arr.iter()
                                    .enumerate()
                                    .map(|(i, v)| {
                                        clone_value(
                                            *v,
                                            globals,
                                            fc.inner().start_group,
                                            true, // will be changed
                                            CodeArea {
                                                pos: a[i].value.get_pos(),
                                                ..info.position
                                            },
                                        )
                                    })
                                    .collect(),
                            ),
                            globals,
                            fc.inner().start_group,
                            info.position,
                        )
                    }
                }
                ast::ValueBody::Import(i, f) => {
                    //let mut new_contexts = context.clone();
                    import_module(i, full_context, globals, info.clone(), *f)?;
                }

                ast::ValueBody::TypeIndicator(name) => {
                    full_context.inner().return_value = match globals.type_ids.get(name) {
                        Some(id) => store_const_value(
                            Value::TypeIndicator(id.0),
                            globals,
                            full_context.inner().start_group,
                            info.position,
                        ),
                        None => {
                            return Err(RuntimeError::UndefinedErr {
                                undefined: name.clone(),
                                info,
                                desc: "type".to_string(),
                            });
                        }
                    };
                }

                ast::ValueBody::Ternary(t) => {
                    t.condition
                        .eval(full_context, globals, info.clone(), constant)?;

                    for context in full_context.iter() {
                        // through every conditional context
                        match &globals.stored_values[context.inner().return_value] {
                            Value::Bool(b) => {
                                let answer = if *b { &t.if_expr } else { &t.else_expr };

                                answer.eval(context, globals, info.clone(), constant)?;
                            }
                            a => {
                                return Err(RuntimeError::TypeError {
                                    expected: "boolean".to_string(),
                                    found: a.get_type_str(globals),
                                    val_def: globals.get_area(context.inner().return_value),
                                    info,
                                })
                            }
                        }
                    }
                }

                ast::ValueBody::Switch(expr, cases) => {
                    expr.eval(full_context, globals, info.clone(), constant)?;

                    for full_context in full_context.iter() {
                        let val1 = full_context.inner().return_value;

                        for case in cases {
                            match &case.typ {
                                ast::CaseType::Value(v) => {
                                    v.eval(full_context, globals, info.clone(), constant)?;
                                    for full_context in full_context.iter() {
                                        let val2 = full_context.inner().return_value;
                                        handle_operator(
                                            val1,
                                            val2,
                                            Builtin::EqOp,
                                            full_context,
                                            globals,
                                            &info,
                                        )?;

                                        // lets loop through all those result values
                                        for full_context in full_context.iter() {
                                            match &globals.stored_values
                                                [full_context.inner().return_value]
                                            {
                                                Value::Bool(b) => {
                                                    if *b {
                                                        case.body.eval(
                                                            full_context,
                                                            globals,
                                                            info.clone(),
                                                            constant,
                                                        )?;
                                                        for c in full_context.iter() {
                                                            c.inner().broken = Some((
                                                                BreakType::Switch(
                                                                    c.inner().return_value,
                                                                ),
                                                                CodeArea::new(),
                                                            ))
                                                        }
                                                    }
                                                }
                                                a => {
                                                    // if the == operator for that type doesn't output a boolean, it can't be
                                                    // used in a switch statement
                                                    return Err(RuntimeError::TypeError {
                                                        expected: "boolean".to_string(),
                                                        found: a.get_type_str(globals),
                                                        val_def: globals.get_area(
                                                            full_context.inner().return_value,
                                                        ),
                                                        info,
                                                    });
                                                }
                                            };
                                        }
                                    }
                                }
                                ast::CaseType::Pattern(p) => {
                                    p.eval(full_context, globals, info.clone(), constant)?;

                                    for full_context in full_context.iter() {
                                        let pat_val = globals.stored_values
                                            [full_context.inner().return_value]
                                            .clone();
                                        let b = globals.stored_values[val1].clone().matches_pat(
                                            &pat_val,
                                            &info,
                                            globals,
                                            full_context.inner(),
                                        )?;

                                        if b {
                                            case.body.eval(
                                                full_context,
                                                globals,
                                                info.clone(),
                                                constant,
                                            )?;
                                            for c in full_context.iter() {
                                                c.inner().broken = Some((
                                                    BreakType::Switch(c.inner().return_value),
                                                    CodeArea::new(),
                                                ))
                                            }
                                        }
                                    }
                                }

                                ast::CaseType::Default => {
                                    //this should be the last case, so we just return the body

                                    case.body.eval(
                                        full_context,
                                        globals,
                                        info.clone(),
                                        constant,
                                    )?;
                                    for c in full_context.iter() {
                                        c.inner().broken = Some((
                                            BreakType::Switch(c.inner().return_value),
                                            CodeArea::new(),
                                        ))
                                    }
                                }
                            }
                        }
                        for c in full_context.with_breaks() {
                            match c.inner().broken {
                                Some((BreakType::Switch(v), _)) => {
                                    c.inner().return_value = v;
                                    c.inner().broken = None;
                                }
                                None => {
                                    c.inner().return_value = globals.NULL_STORAGE;
                                }
                                _ => (),
                            }
                        }
                    }
                }
                ast::ValueBody::Obj(o) => {
                    // parsing an obj

                    let mut all_expr: Vec<ast::Expression> = Vec::new(); // all expressions

                    for prop in &o.props {
                        // iterate through obj properties

                        all_expr.push(prop.0.clone()); // this is the object key expression
                        all_expr.push(prop.1.clone()); // this is the object value expression
                    }
                    let new_info = info.clone();

                    let combinations =
                        all_combinations(all_expr, full_context, globals, new_info, constant)?; // evaluate all expressions gathered

                    for (expressions, context) in combinations {
                        let mut obj: Vec<(u16, ObjParam)> = Vec::new();
                        for i in 0..(o.props.len()) {
                            let o_key = expressions[i * 2];
                            let o_val = expressions[i * 2 + 1];
                            // hopefully self explanatory

                            let disallowed_message = "You are not allowed to set the group ID(s) or the spawn triggered state of a @trigger. Use obj instead";

                            let (key, pattern) = match &globals.stored_values[o_key] {
                                // key = int of the id, pattern = what type should be expected from the value
                                Value::Number(n) => {
                                    // number, i have no clue why people would use this over an obj_key
                                    let out = convert_to_int(*n, &info)? as u16;

                                    if o.mode == ast::ObjectMode::Trigger
                                        && (out == 57 || out == 62)
                                    {
                                        return Err(RuntimeError::CustomError(create_error(
                                            info,
                                            disallowed_message,
                                            &[],
                                            None,
                                        )));
                                    }

                                    (out, None)
                                }
                                Value::Dict(d) => {
                                    // this is specifically for object_key dicts
                                    let gotten_type = d.get(&globals.TYPE_MEMBER_NAME);
                                    if gotten_type == None
                                        || globals.stored_values[*gotten_type.unwrap()]
                                            != Value::TypeIndicator(19)
                                    {
                                        // 19 = object_key??
                                        return Err(RuntimeError::TypeError {
                                            expected: "number or @object_key".to_string(),
                                            found: globals.get_type_str(o_key),
                                            val_def: globals.get_area(o_key),
                                            info,
                                        });
                                    }
                                    let id = d.get(&globals.OBJ_KEY_ID);
                                    if id == None {
                                        return Err(RuntimeError::CustomError(create_error(
                                            info,
                                            "object key has no 'id' member",
                                            &[],
                                            None,
                                        )));
                                    }
                                    let pattern = d.get(&globals.OBJ_KEY_PATTERN);
                                    if pattern == None {
                                        return Err(RuntimeError::CustomError(create_error(
                                            info,
                                            "object key has no 'pattern' member",
                                            &[],
                                            None,
                                        )));
                                    }

                                    (
                                        match &globals.stored_values[*id.unwrap()] {
                                            // check if the ID is actually an int. it should be
                                            Value::Number(n) => {
                                                let out = convert_to_int(*n, &info)? as u16;

                                                if o.mode == ast::ObjectMode::Trigger
                                                    && (out == 57 || out == 62)
                                                {
                                                    // group ids and stuff on triggers
                                                    return Err(RuntimeError::CustomError(
                                                        create_error(
                                                            info,
                                                            disallowed_message,
                                                            &[],
                                                            None,
                                                        ),
                                                    ));
                                                }
                                                out
                                            }
                                            _ => {
                                                return Err(RuntimeError::TypeError {
                                                    expected: "number".to_string(),
                                                    found: globals.get_type_str(*id.unwrap()),
                                                    val_def: globals.get_area(*id.unwrap()),
                                                    info,
                                                })
                                            }
                                        },
                                        Some((
                                            globals.stored_values[*pattern.unwrap()].clone(),
                                            *pattern.unwrap(),
                                        )),
                                    )
                                }
                                a => {
                                    return Err(RuntimeError::TypeError {
                                        expected: "number or @object_key".to_string(),
                                        found: a.get_type_str(globals),
                                        val_def: globals.get_area(o_key),
                                        info,
                                    })
                                }
                            };

                            obj.push((
                                key,
                                {   // parse the value
                                    let val = globals.stored_values[o_val].clone();

                                    if let Some(pat) = pattern { // check if pattern is actually enforced (not null)
                                        if !val.matches_pat(&pat.0, &info, globals, context.inner())? {
                                            return Err(RuntimeError::PatternMismatchError {
                                                pattern: pat.0.to_str(globals),
                                                val: val.get_type_str(globals),
                                                val_def: globals.get_area(o_val),
                                                pat_def: globals.get_area(pat.1),
                                                info
                                            });
                                            
                                        }
                                    }
                                    let err = Err(RuntimeError::CustomError(create_error(
                                        info.clone(),
                                        &format!(
                                            "{} is not a valid object value",
                                            val.to_str(globals)
                                        ),
                                        &[],
                                        None,
                                    )));

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
                                                    _ => return Err(RuntimeError::CustomError(create_error(
                                                        info,
                                                        "Arrays in object parameters can only contain groups",
                                                        &[],
                                                        None,
                                                    )))
                                                })
                                            }

                                            out
                                        }),
                                        Value::Dict(d) => {
                                            if let Some(t) = d.get(&globals.TYPE_MEMBER_NAME) {
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

                        context.inner().return_value = store_const_value(
                            Value::Obj(obj, o.mode),
                            globals,
                            context.inner().start_group,
                            info.position,
                        );
                    }
                }

                ast::ValueBody::Macro(m) => {
                    macro_to_value(m, full_context, globals, info.clone(), constant)?;
                }
                //ast::ValueLiteral::Resolved(r) => out.push((r.clone(), context)),
                ast::ValueBody::Null => full_context.inner().return_value = globals.NULL_STORAGE,
            };
        }
        let mut path_iter = self.path.iter();
        for c in contexts.iter() {
            (*c.inner()).return_value2 = globals.NULL_STORAGE;
        }
        globals.push_new_preserved();
        for full_context in contexts.iter() {
            globals.push_preserved_val(full_context.inner().return_value);
        }

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
                    for full_context in contexts.iter() {
                        let v = full_context.inner().return_value;
                        (*full_context.inner()).return_value2 = v;
                        let val = globals.stored_values[v].clone(); // this is the object we are getting member of

                        (*full_context.inner()).return_value =
                            match val.member(*m, full_context.inner(), globals, info.clone()) {
                                Some(m) => m,
                                None => {
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: m.as_ref().clone(),
                                        info,
                                        desc: "member".to_string(),
                                    });
                                }
                            };
                    }
                }

                ast::Path::Associated(a) => {
                    for full_context in contexts.iter() {
                        let v = full_context.inner().return_value;
                        (*full_context.inner()).return_value2 = v;
                        let val = globals.stored_values[v].clone(); // this is the object we are getting member of
                        (*full_context.inner()).return_value = match &val {
                            Value::TypeIndicator(t) => match globals.implementations.get(t) {
                                Some(imp) => match imp.get(a) {
                                    Some((val, _)) => {
                                        if let Value::Macro(m) = &globals.stored_values[*val] {
                                            if !m.args.is_empty()
                                                && m.args[0].name == globals.SELF_MEMBER_NAME
                                            {
                                                return Err(RuntimeError::CustomError(create_error(
                                                        info,
                                                        "Cannot access method (macro with a \"self\" argument) using \"::\"",
                                                        &[],
                                                        None,
                                                    )));
                                            }
                                        }
                                        *val
                                    }
                                    None => {
                                        let type_name =
                                            find_key_for_value(&globals.type_ids, *t).unwrap();
                                        return Err(RuntimeError::UndefinedErr {
                                            undefined: a.as_ref().clone(),
                                            info,
                                            desc: format!("associated member of @{}", type_name),
                                        });
                                    }
                                },
                                None => {
                                    let type_name =
                                        find_key_for_value(&globals.type_ids, *t).unwrap();
                                    return Err(RuntimeError::UndefinedErr {
                                        undefined: a.as_ref().clone(),
                                        info,
                                        desc: format!("associated member of @{}", type_name),
                                    });
                                }
                            },
                            a => {
                                return Err(RuntimeError::TypeError {
                                    expected: "type indicator".to_string(),
                                    found: a.get_type_str(globals),
                                    val_def: globals.get_area(v),
                                    info,
                                })
                            }
                        };
                    }
                }

                ast::Path::NSlice(slices) => {
                    //TODO: nslice
                    let mut expr_vec = Vec::<ast::Expression>::new();

                    for slice in slices {
                        let null_expr = ast::Expression {
                            operators: Vec::new(),
                            values: vec![ast::Variable {
                                value: ast::ValueLiteral {
                                    body: ast::ValueBody::Null,
                                },
                                operator: None,
                                pos: (0, 0),
                                //comment: (None, None),
                                path: Vec::new(),
                                tag: ast::Attribute::new(),
                            }],
                        };
                        expr_vec.push(slice.left.clone().unwrap_or_else(|| null_expr.clone()));
                        expr_vec.push(slice.right.clone().unwrap_or_else(|| null_expr.clone()));
                        expr_vec.push(slice.step.clone().unwrap_or_else(|| null_expr.clone()));
                    }

                    for full_context in contexts.iter() {
                        let val_ptr = full_context.inner().return_value;
                        (*full_context.inner()).return_value2 = val_ptr;
                        let val = globals.stored_values[val_ptr].clone(); // this is the object we are indexing

                        let combinations = all_combinations(
                            expr_vec.clone(),
                            full_context,
                            globals,
                            info.clone(),
                            true,
                        )?;

                        let mut sorted_nslices = Vec::<(Vec<Slice>, &mut FullContext)>::new();
                        let conv_slice = |v| -> Result<Option<isize>, RuntimeError> {
                            return match &globals.stored_values[v] {
                                Value::Number(n) => {
                                    if (n.floor() - *n).abs() > f64::EPSILON {
                                        return Err(RuntimeError::CustomError(create_error(
                                            info.clone(),
                                            &format!("Cannot slice with non-integer number {}.", n),
                                            &[],
                                            None,
                                        )));
                                    }
                                    Ok(Some(*n as isize))
                                }
                                Value::Null => Ok(None),
                                _ => {
                                    return Err(RuntimeError::TypeError {
                                        expected: "@number".to_string(),
                                        found: globals.get_type_str(v),
                                        val_def: globals.get_area(v),
                                        info: info.clone(),
                                    });
                                }
                            };
                        };

                        for (parsed_slices, context) in combinations {
                            let mut sorted_nslice = Vec::<Slice>::new();
                            let mut count: usize = 0;
                            loop {
                                if count >= parsed_slices.len() {
                                    break;
                                } else if count + 2 >= parsed_slices.len() {
                                    panic!("this is not very bueno {}", parsed_slices.len());
                                }
                                let mut sorted_slice: Slice = (None, None, None);
                                sorted_slice.0 = conv_slice(parsed_slices[count])?;
                                count += 1;
                                sorted_slice.1 = conv_slice(parsed_slices[count])?;
                                count += 1;
                                sorted_slice.2 = conv_slice(parsed_slices[count])?;
                                count += 1;

                                sorted_nslice.push(sorted_slice);
                            }
                            sorted_nslices.push((sorted_nslice, context));
                        }

                        match val {
                            Value::Array(arr) => {
                                for nslice in sorted_nslices {
                                    //println!("slices {:?}", nslice);
                                    let mut nslice_0 = nslice.0;
                                    nslice_0.reverse();
                                    let sliced = slice_array(
                                        &arr,
                                        nslice_0,
                                        globals,
                                        info.clone(),
                                        nslice.1.inner(),
                                    )?;

                                    let stored_arr = store_const_value(
                                        Value::Array(sliced),
                                        globals,
                                        nslice.1.inner().start_group,
                                        info.position,
                                    );

                                    nslice.1.inner().return_value = stored_arr;
                                }
                            }
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    expected: "@array".to_string(),
                                    found: globals.get_type_str(val_ptr),
                                    val_def: globals.get_area(val_ptr),
                                    info,
                                })
                            }
                        }
                    }
                }

                ast::Path::Index(i) => {
                    for full_context in contexts.iter() {
                        let val_ptr = full_context.inner().return_value;
                        (*full_context.inner()).return_value2 = val_ptr;
                        let val = globals.stored_values[val_ptr].clone(); // this is the object we are indexing

                        i.eval(full_context, globals, info.clone(), constant)?;

                        for full_context in full_context.iter() {
                            let index_ptr = full_context.inner().return_value;
                            match &val {
                                Value::Array(arr) => {
                                    match &globals.stored_values[index_ptr] {
                                        Value::Number(n) => {
                                            let len = arr.len();
                                            if (*n) < 0.0 && (-*n) as usize > len {
                                                return Err(RuntimeError::CustomError(create_error(
                                                    info,
                                                    &format!("Index too low! Index is {}, but length is {}.", n, len),
                                                    &[],
                                                    None,
                                                )));
                                            }

                                            if *n as usize >= len {
                                                return Err(RuntimeError::CustomError(create_error(
                                                    info,
                                                    &format!("Index too high! Index is {}, but length is {}.", n, len),
                                                    &[],
                                                    None,
                                                )));
                                            }

                                            if *n < 0.0 {
                                                (*full_context.inner()).return_value =
                                                    arr[len - (-n as usize)]
                                            } else {
                                                (*full_context.inner()).return_value =
                                                    arr[*n as usize]
                                            }
                                        }
                                        _ => {
                                            return Err(RuntimeError::TypeError {
                                                expected: "number".to_string(),
                                                found: globals.get_type_str(index_ptr),
                                                val_def: globals.get_area(index_ptr),
                                                info,
                                            })
                                        }
                                    }
                                }
                                Value::Dict(d) => match &globals.stored_values[index_ptr] {
                                    Value::Str(s) => {
                                        let intern = LocalIntern::new(s.clone());
                                        if !d.contains_key(&intern) {
                                            return Err(RuntimeError::UndefinedErr {
                                                undefined: s.to_string(),
                                                info,
                                                desc: "dictionary key".to_string(),
                                            });
                                        }
                                        full_context.inner().return_value = d[&intern];
                                    }
                                    _ => {
                                        return Err(RuntimeError::TypeError {
                                            expected: "string".to_string(),
                                            found: globals.get_type_str(index_ptr),
                                            val_def: globals.get_area(index_ptr),
                                            info,
                                        })
                                    }
                                },

                                Value::Obj(o, _) => {
                                    match &globals.stored_values[index_ptr] {
                                        Value::Dict(d) => {
                                            let gotten_type = d.get(&globals.TYPE_MEMBER_NAME);
                                            if gotten_type == None
                                                || globals.stored_values[*gotten_type.unwrap()]
                                                    != Value::TypeIndicator(19)
                                            {
                                                // 19 = object_key??
                                                return Err(RuntimeError::TypeError {
                                                    expected: "number or @object_key".to_string(),
                                                    found: globals.get_type_str(index_ptr),
                                                    val_def: globals.get_area(index_ptr),
                                                    info,
                                                });
                                            }

                                            let id = d.get(&globals.OBJ_KEY_ID);
                                            if id == None {
                                                return Err(RuntimeError::CustomError(
                                                    create_error(
                                                        info,
                                                        "object key has no 'id' member",
                                                        &[],
                                                        None,
                                                    ),
                                                ));
                                            }
                                            let okey = match &globals.stored_values[*id.unwrap()] {
                                                // check if the ID is actually an int. it should be
                                                Value::Number(n) => *n as u16,
                                                _ => {
                                                    return Err(RuntimeError::TypeError {
                                                        expected: "number".to_string(),
                                                        found: globals.get_type_str(*id.unwrap()),
                                                        val_def: globals.get_area(*id.unwrap()),
                                                        info,
                                                    })
                                                }
                                            };

                                            let mut contains = false;
                                            for iter in o.iter() {
                                                if iter.0 == okey {
                                                    contains = true;

                                                    let out_val = match &iter.1 {
                                                        // its just converting value to objparam basic level stuff
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
                                                                let stored = store_const_value(
                                                                    Value::Group(*s),
                                                                    globals,
                                                                    full_context
                                                                        .inner()
                                                                        .start_group,
                                                                    info.position,
                                                                );
                                                                out.push(stored);
                                                            }
                                                            Value::Array(out)
                                                        }

                                                        ObjParam::Epsilon => {
                                                            let mut map = FnvHashMap::<
                                                                LocalIntern<String>,
                                                                StoredValue,
                                                            >::default(
                                                            );
                                                            let stored = store_const_value(
                                                                Value::TypeIndicator(20),
                                                                globals,
                                                                full_context.inner().start_group,
                                                                info.position,
                                                            );
                                                            map.insert(
                                                                globals.TYPE_MEMBER_NAME,
                                                                stored,
                                                            );
                                                            Value::Dict(map)
                                                        }
                                                    };
                                                    let stored = store_const_value(
                                                        out_val,
                                                        globals,
                                                        full_context.inner().start_group,
                                                        info.position,
                                                    );
                                                    full_context.inner().return_value = stored;
                                                    break;
                                                }
                                            }

                                            if !contains {
                                                return Err(RuntimeError::CustomError(
                                                    create_error(
                                                        info,
                                                        "Cannot find key in object",
                                                        &[],
                                                        None,
                                                    ),
                                                ));
                                            }
                                        }
                                        _ => {
                                            return Err(RuntimeError::TypeError {
                                                expected: "number or @object_key".to_string(),
                                                found: globals.get_type_str(index_ptr),
                                                val_def: globals.get_area(index_ptr),
                                                info,
                                            })
                                        }
                                    }
                                }
                                Value::Str(s) => {
                                    let arr: Vec<char> = s.chars().collect();

                                    match &globals.stored_values[index_ptr] {
                                        Value::Number(n) => {
                                            let len = arr.len();
                                            if (*n) < 0.0 && (-*n) as usize >= len {
                                                return Err(RuntimeError::CustomError(create_error(
                                                    info,
                                                    &format!("Index too low! Index is {}, but length is {}.", n, len),
                                                    &[],
                                                    None,
                                                )));
                                            }

                                            if *n as usize >= len {
                                                return Err(RuntimeError::CustomError(create_error(
                                                    info,
                                                    &format!("Index too high! Index is {}, but length is {}.", n, len),
                                                    &[],
                                                    None,
                                                )));
                                            }

                                            if *n < 0.0 {
                                                (*full_context.inner()).return_value =
                                                    store_const_value(
                                                        Value::Str(
                                                            arr[len - (-n as usize)].to_string(),
                                                        ),
                                                        globals,
                                                        full_context.inner().start_group,
                                                        info.position,
                                                    );
                                            } else {
                                                (*full_context.inner()).return_value =
                                                    store_const_value(
                                                        Value::Str(arr[*n as usize].to_string()),
                                                        globals,
                                                        full_context.inner().start_group,
                                                        info.position,
                                                    );
                                            }
                                        }
                                        _ => {
                                            return Err(RuntimeError::TypeError {
                                                expected: "number".to_string(),
                                                found: globals.get_type_str(index_ptr),
                                                val_def: globals.get_area(index_ptr),
                                                info,
                                            })
                                        }
                                    }
                                }
                                _a => {
                                    return Err(RuntimeError::TypeError {
                                        expected: "indexable type".to_string(),
                                        found: globals.get_type_str(val_ptr),
                                        val_def: globals.get_area(val_ptr),
                                        info,
                                    })
                                }
                            }
                        }
                    }
                }

                ast::Path::Increment => {
                    for full_context in contexts.iter() {
                        let val_ptr = full_context.inner().return_value;
                        (*full_context.inner()).return_value2 = val_ptr;
                        handle_unary_operator(
                            val_ptr,
                            Builtin::IncrOp,
                            full_context,
                            globals,
                            &info,
                        )?;
                    }
                }

                ast::Path::Decrement => {
                    for full_context in contexts.iter() {
                        let val_ptr = full_context.inner().return_value;
                        (*full_context.inner()).return_value2 = val_ptr;
                        handle_unary_operator(
                            val_ptr,
                            Builtin::DecrOp,
                            full_context,
                            globals,
                            &info,
                        )?;
                    }
                }

                ast::Path::Constructor(defs) => {
                    for full_context in contexts.iter() {
                        let val_ptr = full_context.inner().return_value;
                        (*full_context.inner()).return_value2 = val_ptr;

                        match globals.stored_values[val_ptr].clone() {
                            Value::TypeIndicator(_) => {
                                let mut new_defs = defs.clone();
                                new_defs.push(ast::DictDef::Def((
                                    globals.TYPE_MEMBER_NAME,
                                    ast::ValueBody::Resolved(val_ptr)
                                        .to_variable(globals.get_area(val_ptr).pos)
                                        .to_expression(),
                                )));
                                ast::ValueBody::Dictionary(new_defs.clone())
                                    .to_variable(info.position.pos)
                                    .to_value(full_context, globals, info.clone(), constant)?;
                            }
                            _a => {
                                return Err(RuntimeError::TypeError {
                                    expected: "type indicator".to_string(),
                                    found: globals.get_type_str(val_ptr),
                                    val_def: globals.get_area(val_ptr),
                                    info,
                                })
                            }
                        }
                    }
                }

                ast::Path::Call(args) => {
                    for full_context in contexts.iter() {
                        let val_ptr = full_context.inner().return_value;

                        match globals.stored_values[val_ptr].clone() {
                            Value::Macro(m) => {
                                let parent = full_context.inner().return_value2;
                                execute_macro(
                                    (*m, args.clone()),
                                    full_context,
                                    globals,
                                    parent,
                                    info.clone(),
                                )?;
                            }

                            Value::TypeIndicator(_) => {
                                if args.len() != 1 {
                                    // cast takes 1 argument only

                                    return Err(RuntimeError::CustomError(create_error(
                                        info,
                                        &format!(
                                            "casting takes one argument, but {} were provided",
                                            args.len()
                                        ),
                                        &[],
                                        None,
                                    )));
                                }

                                args[0].value.eval(
                                    full_context,
                                    globals,
                                    info.clone(),
                                    constant,
                                )?;

                                // go through each context, c = context
                                for full_context in full_context.iter() {
                                    handle_operator(
                                        full_context.inner().return_value,
                                        val_ptr,
                                        Builtin::AsOp,
                                        full_context,
                                        globals,
                                        &info,
                                    )?; // just use the "as" operator
                                }
                            }

                            Value::BuiltinFunction(name) => {
                                let evaled_args = all_combinations(
                                    args.iter().map(|x| x.value.clone()).collect(),
                                    full_context,
                                    globals,
                                    info.clone(),
                                    constant,
                                )?;

                                globals.push_new_preserved();
                                for (arg_values, _) in &evaled_args {
                                    for val in arg_values {
                                        globals.push_preserved_val(*val)
                                    }
                                }

                                for (args, context) in evaled_args {
                                    built_in_function(name, args, info.clone(), globals, context)?;
                                }

                                globals.pop_preserved();
                            }
                            _a => {
                                return Err(RuntimeError::TypeError {
                                    expected: "macro, built-in function or type indicator"
                                        .to_string(),
                                    found: globals.get_type_str(val_ptr),
                                    val_def: globals.get_area(val_ptr),
                                    info,
                                })
                            }
                        }
                        for full_context in full_context.iter() {
                            (*full_context.inner()).return_value2 = val_ptr;
                        }
                    }
                }
            };

            for full_context in contexts.iter() {
                globals.push_preserved_val(full_context.inner().return_value);
                globals.push_preserved_val(full_context.inner().return_value2);
            }
        }

        globals.pop_preserved();

        use ast::UnaryOperator;
        if let Some(o) = &self.operator {
            for full_context in contexts.iter() {
                let val_ptr = full_context.inner().return_value;
                match o {
                    UnaryOperator::Minus => {
                        handle_unary_operator(
                            val_ptr,
                            Builtin::NegOp,
                            full_context,
                            globals,
                            &info,
                        )?
                    }

                    UnaryOperator::Increment => {
                        handle_unary_operator(
                            val_ptr,
                            Builtin::PreIncrOp,
                            full_context,
                            globals,
                            &info,
                        )?
                    }

                    UnaryOperator::Decrement => {
                        handle_unary_operator(
                            val_ptr,
                            Builtin::PreDecrOp,
                            full_context,
                            globals,
                            &info,
                        )?
                    }

                    UnaryOperator::Not => {
                        handle_unary_operator(
                            val_ptr,
                            Builtin::NotOp,
                            full_context,
                            globals,
                            &info,
                        )?
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
            for c in contexts.iter() {
                if let Value::Macro(m) = &mut globals.stored_values[c.inner().return_value] {
                    m.tag.tags.extend(self.tag.tags.clone())
                }
            }
        }

        merge_all_contexts(contexts, globals, true);

        Ok(())
    }

    // writes to return_value2
    fn try_define(
        &self,
        contexts: &mut FullContext,
        globals: &mut Globals,
        info: &CompilerInfo,
        mutable: bool,
        layer: i16
    ) -> Result<DefineResult, RuntimeError> {
        use ariadne::Fmt;
        use parser::fmt::SpwnFmt;
        let mut results = Vec::new();
        for full_context in contexts.iter() {
            let mut defined = true;
            let mut redefinable = false;

            let value = match &self.operator {
                None => store_val_m(
                    Value::Null,
                    globals,
                    full_context.inner().start_group,
                    !mutable,
                    info.position,
                ),
                Some(a) => {
                    return Err(RuntimeError::CustomError(create_error(
                        info.clone(),
                        &format!(
                            "Cannot use operator `{:?}` when defining a variable",
                            a.fmt(0)
                        ),
                        &[],
                        None,
                    )))
                }
            };

            let mut current_ptr = match &self.value.body {
                ast::ValueBody::Symbol(a) => {
                    if let (Some(ptr), false) = (
                        full_context.inner().get_variable(*a),
                        mutable && self.path.is_empty(),
                    ) {
                        if full_context.inner().is_redefinable(*a) == Some(true) {
                            redefinable = true;
                        }
                        ptr
                    } else {
                        // define or redefine
                        full_context.inner().new_variable(*a, value, layer);
                        defined = false;
                        value
                    }
                }

                ast::ValueBody::TypeIndicator(t) => {
                    if let Some(typ) = globals.type_ids.get(t) {
                        store_const_value(
                            Value::TypeIndicator(typ.0),
                            globals,
                            full_context.inner().start_group,
                            info.position,
                        )
                    } else {
                        return Err(RuntimeError::CustomError(create_error(
                            info.clone(),
                            &format!("Use a type statement to define a new type: type @{}", t),
                            &[],
                            None,
                        )));
                    }
                }

                ast::ValueBody::SelfVal => {
                    if let Some(ptr) = full_context.inner().get_variable(globals.SELF_MEMBER_NAME) {
                        ptr
                    } else {
                        return Err(RuntimeError::UndefinedErr {
                            undefined: globals.SELF_MEMBER_NAME.to_string(),
                            desc: "variable".to_string(),
                            info: info.clone(),
                        });
                    }
                }

                a => {
                    return Err(RuntimeError::CustomError(create_error(
                        info.clone(),
                        &format!("Expected symbol or type-indicator, found {}", a.fmt(0)),
                        &[],
                        None,
                    )))
                }
            };

            for p in &self.path {
                if !defined {
                    return Err(RuntimeError::CustomError(create_error(
                        info.clone(),
                        &format!(
                            "You cannot have the extention `value{}` when `value` is undefined",
                            p.fmt(0).fg(ariadne::Color::Red)
                        ),
                        &[],
                        None,
                    )));
                }
                redefinable = false;

                match p {
                    ast::Path::Member(m) => {
                        let val = globals.stored_values[current_ptr].clone();
                        match val.member(*m, full_context.inner(), globals, info.clone()) {
                            Some(s) => current_ptr = s,
                            None => {
                                if !globals.is_mutable(current_ptr) {
                                    return Err(RuntimeError::MutabilityError {
                                        val_def: globals.get_area(current_ptr),
                                        info: info.clone(),
                                    });
                                }
                                let stored =
                                    globals.stored_values.map.get_mut(current_ptr).unwrap();
                                if let Value::Dict(d) = &mut stored.val {
                                    (*d).insert(*m, value);
                                    defined = false;
                                    current_ptr = value;
                                } else {
                                    return Err(RuntimeError::CustomError(create_error(
                                        info.clone(),
                                        "Cannot edit members of a non-dictionary value",
                                        &[],
                                        None,
                                    )));
                                }
                            }
                        };
                    }
                    ast::Path::Index(i) => {
                        // keep previous return value
                        let prev_ret = full_context.inner().return_value;
                        globals.push_new_preserved();
                        globals.push_preserved_val(prev_ret);

                        i.eval(full_context, globals, info.clone(), true)?;

                        if let FullContext::Split(_, _) = full_context {
                            return Err(RuntimeError::CustomError(create_error(
                                info.clone(),
                                "Index definition values that split the context are not supported",
                                &[],
                                None,
                            )));
                        }

                        let first_context_eval = full_context.inner().return_value;
                        (*full_context.inner()).return_value = prev_ret;
                        globals.pop_preserved();

                        match &globals.stored_values[current_ptr] {
                            Value::Dict(d) => {
                                if let Value::Str(st) =
                                    globals.stored_values[first_context_eval].clone()
                                {
                                    let intern = LocalIntern::new(st);
                                    match d.get(&intern) {
                                        Some(a) => current_ptr = *a,
                                        None => {
                                            let stored = globals
                                                .stored_values
                                                .map
                                                .get_mut(current_ptr)
                                                .unwrap();
                                            if !stored.mutable {
                                                return Err(RuntimeError::MutabilityError {
                                                    val_def: stored.def_area,
                                                    info: info.clone(),
                                                });
                                            }
                                            let fn_context = full_context.inner().start_group;
                                            if stored.fn_context != fn_context {
                                                return Err(
                                                    RuntimeError::ContextChangeMutateError {
                                                        val_def: stored.def_area,
                                                        info: info.clone(),
                                                        context_changes: full_context
                                                            .inner()
                                                            .fn_context_change_stack
                                                            .clone(),
                                                    },
                                                );
                                            }

                                            if let Value::Dict(d) = &mut stored.val {
                                                (*d).insert(intern, value);
                                                defined = false;
                                                current_ptr = value;
                                            } else {
                                                unreachable!();
                                            }
                                        }
                                    };
                                } else {
                                    return Err(RuntimeError::TypeError {
                                        expected: "string".to_string(),
                                        found: globals.get_type_str(first_context_eval),
                                        val_def: globals.get_area(first_context_eval),
                                        info: info.clone(),
                                    });
                                }
                            }
                            Value::Array(a) => {
                                if let Value::Number(n) =
                                    globals.stored_values[first_context_eval].clone()
                                {
                                    if n > (a.len() - 1) as f64 || -n > a.len() as f64 {
                                        return Err(RuntimeError::CustomError(create_error(
                                            info.clone(),
                                            &format!(
                                                "Index {} is out of range of array (length {})",
                                                n,
                                                a.len()
                                            ),
                                            &[],
                                            None,
                                        )));
                                    } else {
                                        let i = convert_to_int(n, info)?;
                                        if i < 0 {
                                            current_ptr = a[a.len() - (-i as usize)];
                                        } else {
                                            current_ptr = a[i as usize];
                                        }
                                    }
                                } else {
                                    return Err(RuntimeError::TypeError {
                                        expected: "number".to_string(),
                                        found: globals.get_type_str(first_context_eval),
                                        val_def: globals.get_area(first_context_eval),
                                        info: info.clone(),
                                    });
                                }
                            }

                            Value::Str(_) => {
                                return Err(RuntimeError::CustomError(create_error(
                                    info.clone(),
                                    "Assigning a character in a string is not suppored",
                                    &[],
                                    Some("Consider making a new string"),
                                )));
                            }
                            _ => {
                                return Err(RuntimeError::CustomError(create_error(
                                    info.clone(),
                                    &format!("The expression `value{} = ...` is only allowed when `value` is a dictionary or an array", "[ ... ]".fg(ariadne::Color::Red)),
                                    &[],
                                    None,
                                )));
                            }
                        }
                    }
                    ast::Path::Associated(m) => {
                        match &globals.stored_values[current_ptr] {
                            Value::TypeIndicator(t) => {
                                match (*globals).implementations.get_mut(t) {
                                    Some(imp) => {
                                        if let Some((val, _)) = imp.get(m) {
                                            current_ptr = *val;
                                        } else {
                                            (*imp).insert(*m, (value, true));
                                            defined = false;
                                            current_ptr = value;
                                        }
                                    }
                                    None => {
                                        let mut new_imp = FnvHashMap::default();
                                        new_imp.insert(*m, (value, true));
                                        (*globals).implementations.insert(*t, new_imp);
                                        defined = false;
                                        current_ptr = value;
                                    }
                                }
                            }
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    expected: "type indicator".to_string(),
                                    found: globals.get_type_str(current_ptr),
                                    val_def: globals.get_area(current_ptr),
                                    info: info.clone(),
                                });
                            }
                        };
                    }
                    _ => {
                        return Err(RuntimeError::CustomError(create_error(
                            info.clone(),
                            &format!(
                                "The expression `value{} = ...` is not allowed",
                                p.fmt(0).fg(ariadne::Color::Red)
                            ),
                            &[],
                            None,
                        )));
                    }
                }
            }

            if defined {
                results.push(DefineResult::AlreadyDefined(redefinable))
            } else {
                results.push(DefineResult::Ok)
            }
            (*full_context.inner()).return_value2 = current_ptr;
        }
        let mut iter = results.into_iter();
        let out = iter.next().unwrap();
        if iter.any(|a| a != out) {
            return Err(RuntimeError::CustomError(create_error(
                info.clone(),
                "This definition expression is executed in a split context, where some contexts make it an assign expression, while others make it a definition, which is not allowed",
                &[],
                None,
            )));
        }
        Ok(out)
    }
}

pub fn display_val(
    val: Value,
    full_context: &mut FullContext,
    globals: &mut Globals,
    info: &CompilerInfo,
) -> Result<String, RuntimeError> {
    
    assert!(matches!(full_context, FullContext::Single(_)));
    let stored = store_const_value(val, globals, full_context.inner().start_group, Default::default());

    handle_unary_operator(stored, Builtin::DisplayOp, full_context, globals, info)?;
    Ok(match globals.stored_values[full_context.inner().return_value].clone() {
        Value::Str(s) => s,
        a => {
            display_val(a, full_context, globals, info)?
        }
    })
    
}