///types and functions used by the compiler
use crate::ast;
use crate::builtin::*;
use crate::levelstring::*;

use crate::parser::FileRange;
//use std::boxed::Box;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::compiler::{compile_scope, import_module, RuntimeError, BUILTIN_STORAGE, NULL_STORAGE};

pub type TypeID = u16;

pub type Implementations = HashMap<TypeID, HashMap<String, StoredValue>>;
pub type StoredValue = usize; //index to stored value in globals.stored_values

pub struct ValStorage {
    pub map: HashMap<usize, (Value, Group, bool, u16)>, //val, fn context, mutable, lifetime
}

/*
LIFETIME:

value gets deleted when lifetime reaches 0
deeper scope => lifetime++
shallower scopr => lifetime--
*/

impl std::ops::Index<usize> for ValStorage {
    type Output = Value;

    fn index(&self, i: usize) -> &Self::Output {
        &self
            .map
            .get(&i)
            .unwrap_or_else(|| panic!("index {} not found", i))
            .0
    }
}

impl std::ops::IndexMut<usize> for ValStorage {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.map.get_mut(&i).unwrap().0
    }
}

impl ValStorage {
    pub fn new() -> Self {
        ValStorage {
            map: vec![
                (BUILTIN_STORAGE, (Value::Builtins, Group::new(0), false, 1)),
                (NULL_STORAGE, (Value::Null, Group::new(0), false, 1)),
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }

    pub fn increment_lifetimes(&mut self) {
        for (_, val) in self.map.iter_mut() {
            (*val).3 += 1;
        }
    }

    pub fn decrement_lifetimes(&mut self) {
        for (_, val) in self.map.iter_mut() {
            (*val).3 -= 1;
        }
    }

    pub fn clean_up(&mut self) {
        let mut to_be_removed = Vec::new();
        for (index, val) in self.map.iter() {
            if val.3 == 0 {
                to_be_removed.push(*index)
            }
        }
        for index in to_be_removed {
            self.map.remove(&index);
        }
    }

    pub fn increment_single_lifetime(&mut self, index: usize, amount: u16) {
        (*self.map.get_mut(&index).unwrap()).3 += amount;
        match self[index].clone() {
            Value::Array(a) => {
                for e in a {
                    self.increment_single_lifetime(e, amount)
                }
            }
            Value::Dict(a) => {
                for (_, e) in a {
                    self.increment_single_lifetime(e, amount)
                }
            }
            Value::Macro(m) => {
                for (_, e, _, e2) in m.args {
                    if let Some(val) = e {
                        self.increment_single_lifetime(val, amount)
                    }
                    if let Some(val) = e2 {
                        self.increment_single_lifetime(val, amount)
                    }
                }

                for (_, v) in m.def_context.variables.iter() {
                    self.increment_single_lifetime(*v, amount)
                }
            }
            _ => (),
        };
    }
}

pub fn store_value(
    val: Value,
    lifetime: u16,
    globals: &mut Globals,
    context: &Context,
) -> StoredValue {
    let index = globals.val_id;
    (*globals)
        .stored_values
        .map
        .insert(index, (val, context.start_group, true, lifetime));
    (*globals).val_id += 1;
    index
}

pub fn clone_value(
    index: usize,
    lifetime: u16,
    globals: &mut Globals,
    context: &Context,
    constant: bool,
) -> StoredValue {
    let mut old_val = globals.stored_values[index].clone();

    match &mut old_val {
        Value::Array(arr) => {
            old_val = Value::Array(
                arr.iter()
                    .map(|x| clone_value(*x, lifetime, globals, context, constant))
                    .collect(),
            );
        }

        Value::Dict(arr) => {
            old_val = Value::Dict(
                arr.iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            clone_value(*v, lifetime, globals, context, constant),
                        )
                    })
                    .collect(),
            );
        }

        Value::Macro(m) => {
            for arg in &mut m.args {
                if let Some(def_val) = &mut arg.1 {
                    (*def_val) = clone_value(*def_val, lifetime, globals, context, constant);
                }

                if let Some(def_val) = &mut arg.3 {
                    (*def_val) = clone_value(*def_val, lifetime, globals, context, constant);
                }
            }

            for (_, v) in m.def_context.variables.iter_mut() {
                (*v) = clone_value(*v, lifetime, globals, context, constant)
            }
        }
        _ => (),
    };

    //clone all inner values
    //do the thing
    //bing bang
    //profit
    if constant {
        store_const_value(old_val, lifetime, globals, context)
    } else {
        store_value(old_val, lifetime, globals, context)
    }
}

pub fn store_const_value(
    val: Value,
    lifetime: u16,
    globals: &mut Globals,
    context: &Context,
) -> StoredValue {
    let index = globals.val_id;
    (*globals)
        .stored_values
        .map
        .insert(index, (val, context.start_group, false, lifetime));
    (*globals).val_id += 1;
    index
}

pub type FnIDPtr = usize;

pub type Returns = Vec<(StoredValue, Context)>;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Context {
    pub start_group: Group,
    //pub spawn_triggered: bool,
    pub variables: HashMap<String, StoredValue>,
    //pub self_val: Option<StoredValue>,
    pub implementations: Implementations,

    pub func_id: FnIDPtr,

    // info stores the info for the break statement if the context is "broken"
    // broken doesn't mean something is wrong with it, it just means
    // a break statement has been used :)
    pub broken: Option<CompilerInfo>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerInfo {
    pub depth: u8,
    pub path: Vec<String>,
    pub current_file: PathBuf,
    pub pos: FileRange,
}

impl CompilerInfo {
    pub fn new() -> Self {
        CompilerInfo {
            depth: 0,
            path: vec!["main scope".to_string()],
            current_file: PathBuf::new(),
            pos: ((0, 0), (0, 0)),
        }
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            start_group: Group::new(0),
            //spawn_triggered: false,
            variables: HashMap::new(),
            //return_val: Box::new(Value::Null),
            implementations: HashMap::new(),
            //self_val: None,
            func_id: 0,
            broken: None,
        }
    }

    pub fn next_fn_id(&self, globals: &mut Globals) -> Context {
        (*globals).func_ids.push(FunctionID {
            parent: Some(self.func_id),
            obj_list: Vec::new(),
            width: None,
        });

        let mut out = self.clone();
        out.func_id = globals.func_ids.len() - 1;
        out
    }

}

//will merge one set of context, returning false if no mergable contexts were found
pub fn merge_contexts(contexts: &mut Vec<Context>, globals: &mut Globals) -> bool {
    
    let mut mergable_ind = Vec::<usize>::new();
    let mut ref_c = 0;
    loop {
        if ref_c >= contexts.len() {
            return false;
        }
        for (i, c) in contexts.iter().enumerate() {
            if i == ref_c {
                continue;
            }
            let ref_c = &contexts[ref_c];

            if (ref_c.broken == None) != (c.broken == None) {
                continue;
            }
            let mut not_eq = false;

            //check variables are equal
            for (key, val) in &c.variables {
                if globals.stored_values[ref_c.variables[key]] != globals.stored_values[*val] {
                    not_eq = true;
                    break;
                }
            }
            if not_eq {
                continue;
            }
            //check implementations are equal
            for (key, val) in &c.implementations {
                for (key2, val) in val {
                    if globals.stored_values[ref_c.implementations[key][key2]] != globals.stored_values[*val] {
                        not_eq = true;
                        break;
                    }
                }
            }
            if not_eq {
                continue;
            }

            //everything is equal, add to list
            mergable_ind.push(i);
        }
        if mergable_ind.is_empty() {
            ref_c += 1;
        } else {
            break
        }
    }

    let new_group = Group::next_free(&mut globals.closed_groups);
    //add spawn triggers
    let mut add_spawn_trigger = |context: &Context| {
        let mut params = HashMap::new();
        params.insert(
            51,
            ObjParam::Group(new_group),
        );
        params.insert(1, ObjParam::Number(1268.0));

        (*globals).func_ids[context.func_id].obj_list.push(
            GDObj {
                params,

                ..context_trigger(&context)
            }
            .context_parameters(&context),
        )
    };
    add_spawn_trigger(&contexts[ref_c]);
    for i in mergable_ind.iter() {
        add_spawn_trigger(&contexts[*i])
    }
    
    (*contexts)[ref_c].start_group = new_group;
    (*contexts)[ref_c].next_fn_id(globals);
    
    for i in mergable_ind.iter().rev() {
        (*contexts).swap_remove(*i);
    }

    true
    
}

#[derive(Clone, Debug, PartialEq)]
pub struct Macro {
    //             name         default val      tag          pattern
    pub args: Vec<(String, Option<StoredValue>, ast::Tag, Option<StoredValue>)>,
    pub def_context: Context,
    pub def_file: PathBuf,
    pub body: Vec<ast::Statement>,
    pub tag: ast::Tag,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub start_group: Group,
    //pub all_groups: Vec<Group>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Pattern {
    Type(TypeID),
    Array(Vec<Pattern>),
    Either(Box<Pattern>, Box<Pattern>),
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
    Macro(Box<Macro>),
    Str(String),
    Array(Vec<StoredValue>),
    Obj(Vec<(u16, ObjParam)>, ast::ObjectMode),
    Builtins,
    BuiltinFunction(String),
    TypeIndicator(TypeID),
    Range(i32, i32, usize), //start, end, step
    Pattern(Pattern),
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

    pub fn matches_pat(&self, pat_val: &Value, info: &CompilerInfo, globals: &mut Globals, context: &Context) -> Result<bool, RuntimeError> {
        let pat = if let Value::Pattern(p) = convert_type(pat_val, 18, info, globals, context)? {p} else {unreachable!()};
        match pat {
            Pattern::Either(p1, p2) => Ok(self.matches_pat(&Value::Pattern(*p1), info, globals, context)? || self.matches_pat(&Value::Pattern(*p2), info,globals, context)?),
            Pattern::Type(t) => Ok(self.to_num(globals) == t),
            Pattern::Array(a_pat) => {
                if let Value::Array(a_val) = self {
                    match a_pat.len() {
                        0 => Ok(true),
    
                        1 => {
                            for el in a_val {
                                let val = globals.stored_values[*el].clone();
                                if !val.matches_pat(&Value::Pattern(a_pat[0].clone()), info, globals, context)? {
                                    return Ok(false)
                                }
                            }
                            Ok(true)
                        }

                        _ => Err(RuntimeError::RuntimeError {
                            message: String::from("arrays with multiple elements cannot be used as patterns (yet)"),
                            info: info.clone(),
                        })
                    }
                } else {
                    Ok(false)
                }
                
            }
        }
    }
}

//copied from https://stackoverflow.com/questions/59401720/how-do-i-find-the-key-for-a-value-in-a-hashmap
pub fn find_key_for_value(map: &HashMap<String, (u16, PathBuf, (usize, usize))>, value: u16) -> Option<&String> {
    map.iter()
        .find_map(|(key, val)| if val.0 == value { Some(key) } else { None })
}






pub fn convert_type(
    val: &Value,
    typ: TypeID,
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
                ID::Specific(n) => n as f64,
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
                ID::Specific(n) => n as f64,
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
                ID::Specific(n) => n as f64,
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
                ID::Specific(n) => n as f64,
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

        Value::Func(f) => match typ {
            
            0 => Value::Group(f.start_group),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Function can't be converted to '{}'!",
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
                    find_key_for_value(&globals.type_ids, typ).unwrap(), find_key_for_value(&globals.type_ids, val.to_num(globals)).unwrap()
                ),
                info: info.clone(),
            })
        }
    })
}

//use std::fmt;

const MAX_DICT_EL_DISPLAY: u16 = 10;

impl Value {
    pub fn to_str(&self, globals: &Globals) -> String {
        match self {
            Value::Group(g) => {
                (if let ID::Specific(id) = g.id {
                    id.to_string()
                } else {
                    "?".to_string()
                }) + "g"
            }
            Value::Color(c) => {
                (if let ID::Specific(id) = c.id {
                    id.to_string()
                } else {
                    "?".to_string()
                }) + "c"
            }
            Value::Block(b) => {
                (if let ID::Specific(id) = b.id {
                    id.to_string()
                } else {
                    "?".to_string()
                }) + "b"
            }
            Value::Item(i) => {
                (if let ID::Specific(id) = i.id {
                    id.to_string()
                } else {
                    "?".to_string()
                }) + "i"
            }
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Func(f) => format!("{{ /*function {}g*/ }}", {
                if let ID::Specific(id) = f.start_group.id {
                    id.to_string()
                } else {
                    "?".to_string()
                }
            }),
            Value::Range(start, end, stepsize) => {
                if *stepsize != 1 {
                    format!("{}..{}..{}", start, stepsize, end)
                } else {
                    format!("{}..{}", start, end)
                }
            }
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
                for (key, _val) in o {
                    out += &format!("{},{},", key, "val");
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

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionID {
    pub parent: Option<usize>, //index of parent id, if none it is a top-level id
    pub width: Option<u32>,    //width of this id, is none when its not calculated yet
    //pub name: String,          //name of this id, used for the label
    pub obj_list: Vec<GDObj>, //list of objects in this function id
}

pub struct Globals {
    //counters for arbitrary groups
    pub closed_groups: u16,
    pub closed_colors: u16,
    pub closed_blocks: u16,
    pub closed_items: u16,

    pub path: PathBuf,

    pub lowest_y: HashMap<u32, u16>,
    pub stored_values: ValStorage,
    pub val_id: usize,

    pub type_ids: HashMap<String, (u16, PathBuf, (usize, usize))>,
    pub type_id_count: u16,

    pub func_ids: Vec<FunctionID>,
    pub objects: Vec<GDObj>,

    pub statement_counter: HashMap<(PathBuf, (usize, usize)), u128>,
}

impl Globals {
    pub fn get_val_fn_context(
        &self,
        p: StoredValue,
        info: CompilerInfo,
    ) -> Result<Group, RuntimeError> {
        match self.stored_values.map.get(&p) {
            Some(val) => Ok(val.1),
            None => Err(RuntimeError::RuntimeError {
                message: "Pointer points to no data!".to_string(),
                info,
            }),
        }
    }
    pub fn is_mutable(&self, p: StoredValue) -> bool {
        match self.stored_values.map.get(&p) {
            Some(val) => val.2,
            None => unreachable!(),
        }
    }

    pub fn get_type_str(&self, p: StoredValue) -> String {
        let val = &self.stored_values[p];
        let typ = match val {
            Value::Dict(d) => {
                if let Some(s) = d.get(TYPE_MEMBER_NAME) {
                    match self.stored_values[*s] {
                        Value::TypeIndicator(t) => t,
                        _ => unreachable!(),
                    }
                } else {
                    val.to_num(self)
                }
            }
            _ => val.to_num(self),
        };
        find_key_for_value(&self.type_ids, typ).unwrap().clone()
    }
}

impl Globals {
    pub fn new(path: PathBuf) -> Self {
        let storage = ValStorage::new();
        let mut globals = Globals {
            closed_groups: 0,
            closed_colors: 0,
            closed_blocks: 0,
            closed_items: 0,
            path,

            lowest_y: HashMap::new(),

            type_ids: HashMap::new(),
            type_id_count: 0,

            val_id: storage.map.len(),
            stored_values: storage,
            func_ids: vec![FunctionID {
                parent: None,
                width: None,
                obj_list: Vec::new(),
            }],
            objects: Vec::new(),
            statement_counter: HashMap::new(),
        };

        

        let mut add_type = |name: &str, id: u16| {
            globals.type_ids.insert(String::from(name), (id, PathBuf::new(), (0,0)))
        };

        add_type("group", 0);
        add_type("color", 1);
        add_type("block", 2);
        add_type("item", 3);
        add_type("number", 4);
        add_type("bool", 5);
        add_type("function", 6);
        add_type("dictionary", 7);
        add_type("macro", 8);
        add_type("string", 9);
        add_type("array", 10);
        add_type("object", 11);
        add_type("spwn", 12);
        add_type("builtin", 13);
        add_type("type_indicator", 14);
        add_type("null", 15);
        add_type("trigger", 16);
        add_type("range", 17);
        add_type("pattern", 18);
        add_type("object_key", 19);

        globals.type_id_count = globals.type_ids.len() as u16;

        globals
    }
}

fn handle_operator(
    value1: StoredValue,
    value2: StoredValue,
    macro_name: &str,
    context: &Context,
    globals: &mut Globals,
    info: CompilerInfo,
) -> Result<Returns, RuntimeError> {
    Ok(
        if let Some(val) =
            globals.stored_values[value1]
                .clone()
                .member(macro_name.to_string(), &context, globals)
        {
            if let Value::Macro(m) = globals.stored_values[val].clone() {
                let new_info = info.clone();
                if m.args.is_empty() {
                    return Err(RuntimeError::RuntimeError {
                        message: String::from("Expected at least one argument in operator macro"),
                        info: new_info,
                    });
                }
                let val2 = globals.stored_values[value2].clone();

                if let Some(target_typ) = m.args[0].3 {
                    let pat = &globals.stored_values[target_typ].clone();

                    if  !val2.matches_pat(pat, &info, globals, context)? {
                        //if types dont match, act as if there is no macro at all
                        return Ok(vec![(
                            store_value(
                                built_in_function(
                                    macro_name,
                                    vec![value1, value2],
                                    info,
                                    globals,
                                    &context,
                                )?,
                                1,
                                globals,
                                &context,
                            ),
                            context.clone(),
                        )]);
                    }
                }

                let (values, _) = execute_macro(
                    (
                        *m,
                        //copies argument so the original value can't be mutated
                        //prevents side effects and shit
                        vec![ast::Argument::from(store_value(val2, 1, globals, &context))],
                    ),
                    context,
                    globals,
                    value1,
                    new_info,
                )?;
                values
            } else {
                vec![(
                    store_value(
                        built_in_function(
                            macro_name,
                            vec![value1, value2],
                            info,
                            globals,
                            &context,
                        )?,
                        1,
                        globals,
                        &context,
                    ),
                    context.clone(),
                )]
            }
        } else {
            vec![(
                store_value(
                    built_in_function(macro_name, vec![value1, value2], info, globals, &context)?,
                    1,
                    globals,
                    &context,
                ),
                context.clone(),
            )]
        },
    )
}

fn convert_to_int(num: f64, info: &CompilerInfo) -> Result<i32, RuntimeError> {
    let rounded = num.round();
    if (num - rounded).abs() > 0.000000001 {
        return Err(RuntimeError::RuntimeError {
            message: format!("expected integer, found {}", num),
            info: info.clone(),
        });
    }
    Ok(rounded as i32)
}

impl ast::Expression {
    pub fn eval(
        &self,
        context: &Context,
        globals: &mut Globals,
        info: CompilerInfo,
        constant: bool,
    ) -> Result<(Returns, Returns), RuntimeError> {
        //second returns is in case there are any values in the expression that includes a return statement
        let mut vals = self.values.iter();
        let first_value =
            vals.next()
                .unwrap()
                .to_value(context.clone(), globals, info.clone(), constant)?;
        let mut acum = first_value.0;
        let mut inner_returns = first_value.1;

        if self.operators.is_empty() {
            //if only variable
            return Ok((acum, inner_returns));
        }

        for (i, var) in vals.enumerate() {
            let mut new_acum: Returns = Vec::new();
            //every value in acum will be operated with the value of var in the corresponding context
            for (acum_val, c) in acum {
                //what the value in acum becomes
                let evaled = var.to_value(c, globals, info.clone(), constant)?;
                inner_returns.extend(evaled.1);

                use ast::Operator::*;

                for (val, c2) in &evaled.0 {
                    //let val_fn_context = globals.get_val_fn_context(val, info.clone());
                    let vals: Returns = match self.operators[i] {
                        Or => handle_operator(acum_val, *val, "_or_", c2, globals, info.clone())?,
                        And => handle_operator(acum_val, *val, "_and_", c2, globals, info.clone())?,
                        More => handle_operator(
                            acum_val,
                            *val,
                            "_more_than_",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Less => handle_operator(
                            acum_val,
                            *val,
                            "_less_than_",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        MoreOrEqual => handle_operator(
                            acum_val,
                            *val,
                            "_more_or_equal_",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        LessOrEqual => handle_operator(
                            acum_val,
                            *val,
                            "_less_or_equal_",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Slash => handle_operator(
                            acum_val,
                            *val,
                            "_divided_by_",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Star => {
                            handle_operator(acum_val, *val, "_times_", c2, globals, info.clone())?
                        }

                        Modulo => {
                            handle_operator(acum_val, *val, "_mod_", c2, globals, info.clone())?
                        }

                        Power => {
                            handle_operator(acum_val, *val, "_pow_", c2, globals, info.clone())?
                        }
                        Plus => {
                            handle_operator(acum_val, *val, "_plus_", c2, globals, info.clone())?
                        }
                        Minus => {
                            handle_operator(acum_val, *val, "_minus_", c2, globals, info.clone())?
                        }
                        Equal => {
                            handle_operator(acum_val, *val, "_equal_", c2, globals, info.clone())?
                        }
                        NotEqual => handle_operator(
                            acum_val,
                            *val,
                            "_not_equal_",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        
                        Either => handle_operator(acum_val,
                            *val,
                            "_either_",
                            c2,
                            globals,
                            info.clone()
                        )?,
                        Range => vec![(
                            store_value(
                                {
                                    let end = match globals.stored_values[*val] {
                                        Value::Number(n) => convert_to_int(n, &info)?,
                                        _ => {
                                            return Err(RuntimeError::RuntimeError {
                                                message: "Both sides of range must be Numbers"
                                                    .to_string(),
                                                info,
                                            })
                                        }
                                    };
                                    match globals.stored_values[acum_val] {
                                        Value::Number(start) => {
                                            Value::Range(convert_to_int(start, &info)?, end, 1)
                                        }
                                        Value::Range(start, step, old_step) => {
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
                                                        message:
                                                            "cannot have a stepsize less than 0"
                                                                .to_string(),
                                                        info,
                                                    });
                                                } else {
                                                    step as usize
                                                },
                                            )
                                        }
                                        _ => {
                                            return Err(RuntimeError::RuntimeError {
                                                message: "Both sides of range must be Numbers"
                                                    .to_string(),
                                                info,
                                            })
                                        }
                                    }

                                    // if start < end {
                                    //     (start..end).collect::<Vec<i32>>()
                                    // } else {
                                    //     (end..start).rev().collect::<Vec<i32>>()
                                    // }
                                    // .into_iter()
                                    // .map(|x| store_value(Value::Number(x as f64), 1, globals, &c2))
                                    // .collect()
                                },
                                1,
                                globals,
                                &c2,
                            ),
                            c2.clone(),
                        )],
                        //MUTABLE ONLY
                        //ADD CHECk
                        Assign => {
                            handle_operator(acum_val, *val, "_assign_", c2, globals, info.clone())?
                        }

                        As => handle_operator(acum_val, *val, "_as_", c2, globals, info.clone())?,

                        Add => handle_operator(acum_val, *val, "_add_", c2, globals, info.clone())?,

                        Subtract => handle_operator(
                            acum_val,
                            *val,
                            "_subtract_",
                            c2,
                            globals,
                            info.clone(),
                        )?,

                        Multiply => handle_operator(
                            acum_val,
                            *val,
                            "_multiply_",
                            c2,
                            globals,
                            info.clone(),
                        )?,

                        Divide => {
                            handle_operator(acum_val, *val, "_divide_", c2, globals, info.clone())?
                        }
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
    context: &Context,
    globals: &mut Globals,
    parent: StoredValue,
    info: CompilerInfo,
) -> Result<(Returns, Returns), RuntimeError> {
    let mut inner_inner_returns = Vec::new();
    let mut new_contexts: Vec<Context> = Vec::new();
    if !m.args.is_empty() {
        // second returns is for any compound statements in the args
        let (evaled_args, inner_returns) = all_combinations(
            args.iter().map(|x| x.value.clone()).collect(),
            context,
            globals,
            info.clone(),
            true,
        )?;
        inner_inner_returns.extend(inner_returns);

        for (arg_values, mut new_context) in evaled_args {
            new_context.variables = m.def_context.variables.clone();
            let mut new_variables: HashMap<String, StoredValue> = HashMap::new();

            //parse each argument given into a local macro variable
            //index of arg if no arg is specified
            let mut def_index = if m.args[0].0 == "self" { 1 } else { 0 };
            for (i, arg) in args.iter().enumerate() {
                match &arg.symbol {
                    Some(name) => {
                        let arg_def = m.args.iter().enumerate().find(|e| e.1 .0 == *name);
                        if let Some((_arg_i, arg_def)) = arg_def {
                            //type check!!
                            //maybe make type check function
                            if let Some(t) = arg_def.3 {
                                let val = globals.stored_values[arg_values[i]].clone();
                                let pat = globals.stored_values[t].clone();

                                if !val.matches_pat(&pat, &info, globals, context)? {
                                    return Err(RuntimeError::TypeError {
                                        expected: pat.to_str(globals),
                                        found: val.to_str(globals),
                                        info,
                                    });
                                }
                            };

                            new_variables.insert(name.clone(), arg_values[i]);
                        } else {
                            return Err(RuntimeError::UndefinedErr {
                                undefined: name.clone(),
                                info,
                                desc: "macro argument".to_string(),
                            });
                        }
                    }
                    None => {
                        if (def_index) > m.args.len() - 1 {
                            return Err(RuntimeError::RuntimeError {
                                message: "Too many arguments!".to_string(),
                                info,
                            });
                        }

                        //type check!!
                        if let Some(t) = m.args[def_index].3 {
                            let val = globals.stored_values[arg_values[i]].clone();
                            let pat = globals.stored_values[t].clone();

                            if !val.matches_pat(&pat, &info, globals, context)? {
                                return Err(RuntimeError::TypeError {
                                    expected: pat.to_str(globals),
                                    found: val.to_str(globals),
                                    info,
                                });
                            }
                        };

                        new_variables.insert(
                            m.args[def_index].0.clone(),
                            clone_value(arg_values[i], 1, globals, &context, true),
                        );
                        def_index += 1;
                    }
                }
            }
            //insert defaults and check non-optional arguments
            let mut m_args_iter = m.args.iter();
            if m.args[0].0 == "self" {
                if globals.stored_values[parent] == Value::Null {
                    return Err(RuntimeError::RuntimeError {
                        message: "
This macro requires a parent (a \"self\" value), but it seems to have been called alone (or on a null value).
Should be used like this: value.macro(arguments)".to_string(), info
                    });
                }
                //self doesn't need to be cloned, as it is a referance (kinda)
                new_context.variables.insert("self".to_string(), parent);
                m_args_iter.next();
            }
            for arg in m_args_iter {
                if !new_variables.contains_key(&arg.0) {
                    match &arg.1 {
                        Some(default) => {
                            new_variables.insert(
                                arg.0.clone(),
                                clone_value(*default, 1, globals, &context, true),
                            );
                        }

                        None => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!(
                                    "Non-optional argument '{}' not satisfied!",
                                    arg.0
                                ),
                                info,
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
    let mut new_info = info;
    new_info.current_file = m.def_file;
    let mut compiled = compile_scope(&m.body, new_contexts, globals, new_info)?;

    // stop break chain
    for c in &mut compiled.0 {
        (*c).broken = None;
    }

    let returns = if compiled.1.is_empty() {
        compiled.0.iter().map(|x| (1, x.clone())).collect()
    } else if compiled.1.len() > 1 {
        let mut return_vals = Vec::<(Value, u8, Vec<Context>)>::new();
        for (val, c) in compiled.1 {
            let mut found = false;
            for val2 in &mut return_vals {
                if globals.stored_values[val] == val2.0 {
                    (*val2).1 += 1;
                    (*val2).2.push(c.clone());
                    found = true;
                    break;
                }
            }
            if !found {
                return_vals.push((globals.stored_values[val].clone(), 1, vec![c]));
            }
        }

        let mut rets = Returns::new();

        for (val, count, c) in return_vals {
            if count > 1 {
                let mut new_context = context.clone();
                //new_context.spawn_triggered = true;
                //pick a start group
                let start_group = Group::next_free(&mut globals.closed_groups);

                for cont in c {
                    let mut params = HashMap::new();
                    params.insert(1, ObjParam::Number(1268.0));
                    params.insert(51, ObjParam::Group(start_group));
                    let obj = GDObj {
                        params,

                        ..context_trigger(&cont)
                    }
                    .context_parameters(&cont);
                    (*globals).func_ids[context.func_id].obj_list.push(obj);
                }

                new_context.start_group = start_group;

                rets.push((store_value(val, 1, globals, &context), new_context))
            } else {
                rets.push((store_value(val, 1, globals, &context), c[0].clone()))
            }
            //compact the returns down to one function with a return

            //create the function context
        }
        rets
    } else {
        compiled.1
    };

    Ok((
        returns
            .iter()
            .map(|x| {
                //set mutable to false
                (*globals.stored_values.map.get_mut(&x.0).unwrap()).2 = false;
                (
                    x.0,
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
type ReturnsList = Vec<(Vec<StoredValue>, Context)>;
fn all_combinations(
    a: Vec<ast::Expression>,
    context: &Context,
    globals: &mut Globals,
    info: CompilerInfo,
    constant: bool,
) -> Result<(ReturnsList, Returns), RuntimeError> {
    let mut out = ReturnsList::new();
    let mut inner_returns = Returns::new();
    if a.is_empty() {
        //if there are so value, there is only one combination
        out.push((Vec::new(), context.clone()));
    } else {
        let mut a_iter = a.iter();
        //starts with all the combinations of the first expr
        let (start_values, start_returns) =
            a_iter
                .next()
                .unwrap()
                .eval(context, globals, info.clone(), constant)?;
        out.extend(start_values.iter().map(|x| (vec![x.0], x.1.clone())));
        inner_returns.extend(start_returns);
        //for the rest of the expressions
        for expr in a_iter {
            //all the new combinations end up in this
            let mut new_out: Vec<(Vec<StoredValue>, Context)> = Vec::new();
            //run through all the lists in out
            for (inner_arr, c) in out.iter() {
                //for each one, run through all the returns in that context
                let (values, returns) = expr.eval(c, globals, info.clone(), constant)?;
                inner_returns.extend(returns);
                for (v, c2) in values.iter() {
                    //push a new list with each return pushed to it
                    new_out.push((
                        {
                            let mut new_arr = inner_arr.clone();
                            new_arr.push(*v);
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
    context: &Context,
    globals: &mut Globals,
    info: CompilerInfo,
    constant: bool,
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
        constant,
    )?;
    inner_returns.extend(returns);
    let mut out = Returns::new();
    for expressions in evaled {
        let mut dict_out: HashMap<String, StoredValue> = HashMap::new();
        for (expr_index, def) in dict.iter().enumerate() {
            match def {
                ast::DictDef::Def(d) => {
                    dict_out.insert(d.0.clone(), expressions.0[expr_index]);
                }
                ast::DictDef::Extract(_) => {
                    dict_out.extend(
                        match globals.stored_values[expressions.0[expr_index]].clone() {
                            Value::Dict(d) => d.clone(),
                            a => {
                                return Err(RuntimeError::RuntimeError {
                                    message: format!(
                                        "Cannot extract from this value: {}",
                                        a.to_str(globals)
                                    ),
                                    info,
                                })
                            }
                        },
                    );
                }
            };
        }
        out.push((
            store_value(Value::Dict(dict_out), 1, globals, &context),
            expressions.1,
        ));
    }
    Ok((out, inner_returns))
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
        
        let mut start_val = Returns::new();
        let mut inner_returns = Returns::new();

        //let mut defined = true;
        if let Some(UnaryOperator::Let) = self.operator {
            let val = self.define(&mut context, globals, &info)?;
            start_val = vec![(val, context)];
            return Ok((start_val, inner_returns));
        }

        use ast::IDClass;

        info.pos = self.pos;

        match &self.value.body {
            ast::ValueBody::Resolved(r) => start_val.push((*r, context.clone())),
            ast::ValueBody::SelfVal => {
                if let Some(val) = context.variables.get("self") {
                    start_val.push((*val, context.clone()))
                } else {
                    return Err(RuntimeError::RuntimeError {
                        message: "Cannot use \"self\" outside of macros!".to_string(),
                        info,
                    });
                }
            }
            ast::ValueBody::ID(id) => start_val.push((
                store_const_value(
                    match id.class_name {
                        IDClass::Group => {
                            if id.unspecified {
                                Value::Group(Group::next_free(&mut globals.closed_groups))
                            } else {
                                Value::Group(Group::new(id.number))
                            }
                        }
                        IDClass::Color => {
                            if id.unspecified {
                                Value::Color(Color::next_free(&mut globals.closed_colors))
                            } else {
                                Value::Color(Color::new(id.number))
                            }
                        }
                        IDClass::Block => {
                            if id.unspecified {
                                Value::Block(Block::next_free(&mut globals.closed_blocks))
                            } else {
                                Value::Block(Block::new(id.number))
                            }
                        }
                        IDClass::Item => {
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
                    store_const_value(Value::Func(evaled), 1, globals, &context),
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
            ast::ValueBody::Import(i) => {
                //let mut new_contexts = context.clone();
                start_val = import_module(i, &context, globals, info.clone())?;
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
            ast::ValueBody::Obj(o) => {
                let mut all_expr: Vec<ast::Expression> = Vec::new();
                for prop in &o.props {
                    all_expr.push(prop.0.clone());
                    all_expr.push(prop.1.clone());
                }
                let new_info = info.clone();
                let (evaled, returns) =
                    all_combinations(all_expr, &context, globals, new_info, constant)?;
                inner_returns.extend(returns);
                for (expressions, context) in evaled {
                    let mut obj: Vec<(u16, ObjParam)> = Vec::new();
                    for i in 0..(o.props.len()) {
                        let v = expressions[i * 2];
                        let v2 = expressions[i * 2 + 1];


                        let (key, pattern) = match &globals.stored_values[v] {
                            Value::Number(n) => {
                                let out = *n as u16;

                                if o.mode == ast::ObjectMode::Trigger && (out == 57 || out == 62) {
                                    return Err(RuntimeError::RuntimeError {
                                        message: "You are not allowed to set the group ID(s) or the spawn triggered state of a @trigger. Use obj instead".to_string(),
                                        info,
                                    })
                                }

                                (out, None)
                            },
                            Value::Dict(d) => {
                                let gotten_type = d.get(TYPE_MEMBER_NAME);
                                if gotten_type == None ||  globals.stored_values[*gotten_type.unwrap()] != Value::TypeIndicator(19) {
                                    return Err(RuntimeError::RuntimeError {
                                        message: "expected either @number or @object_key as object key".to_string(),
                                        info,
                                    })
                                }
                                let id = d.get("id");
                                if id == None {
                                    return Err(RuntimeError::RuntimeError {
                                        message: "object key has no 'id' member".to_string(),
                                        info,
                                    })
                                }
                                let pattern = d.get("pattern");
                                if pattern == None {
                                    return Err(RuntimeError::RuntimeError {
                                        message: "object key has no 'pattern' member".to_string(),
                                        info,
                                    })
                                }

                                (match &globals.stored_values[*id.unwrap()] {
                                    Value::Number(n) => {
                                        let out = *n as u16;

                                        if o.mode == ast::ObjectMode::Trigger && (out == 57 || out == 62) {
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
                            {   
                                let val = globals.stored_values[v2].clone();

                                if let Some(pat) = pattern {
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
                                
                                match &val {
                                    Value::Number(n) => ObjParam::Number(*n),
                                    Value::Str(s) => ObjParam::Text(s.clone()),
                                    Value::Func(g) => ObjParam::Group(g.start_group),

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
                                    x => {
                                        return Err(RuntimeError::RuntimeError {
                                            message: format!(
                                                "{} is not a valid object value",
                                                x.to_str(globals)
                                            ),
                                            info,
                                        })
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
                let mut all_expr: Vec<ast::Expression> = Vec::new();
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
                    let mut args: Vec<(String, Option<StoredValue>, ast::Tag, Option<StoredValue>)> =
                        Vec::new();
                    let mut expr_index = 0;
                    
                    for arg in m.args.iter() {
                        let def_val = match &arg.1 {
                            Some(_) => {
                                expr_index += 1;
                                Some(
                                    clone_value(defaults.0[expr_index - 1], 1, globals, &defaults.1, true)
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
                        let val = globals.stored_values[x.0].clone();
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
                                Value::TypeIndicator(t) => match x.1.implementations.get(&t) {
                                    Some(imp) => match imp.get(a) {
                                        Some(val) => {
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
                            Value::Array(arr) => {
                                let new_info = info.clone();
                                let (evaled, returns) =
                                    i.eval(&prev_c, globals, new_info, constant)?;
                                inner_returns.extend(returns);
                                for index in evaled {
                                    match &globals.stored_values[index.0] {
                                        Value::Number(n) => {
                                            let len = arr.len();
                                            if (*n) < 0.0 {
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

                                            new_out.push((arr[*n as usize], index.1, prev_v));
                                        }
                                        a => {
                                            return Err(RuntimeError::RuntimeError {
                                                message: format!(
                                                    "expected number in index, found {}",
                                                    a.to_str(globals)
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

        Ok((out, inner_returns))
    }

    //more like is_undefinable
    pub fn is_defined(&self, context: &Context, globals: &mut Globals) -> bool {
        // use crate::fmt::SpwnFmt;
        // println!("hello? {}", self.fmt(0));
        let mut current_ptr = match &self.value.body {
            ast::ValueBody::Symbol(a) => {
                if let Some(ptr) = context.variables.get(a) {
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
                    Value::TypeIndicator(t) => match context.implementations.get(t) {
                        Some(imp) => {
                            if let Some(val) = imp.get(m) {
                                current_ptr = *val;
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
                    message: format!("Cannot use operator {:?} in definition", a),
                    info: info.clone(),
                })
            }
        };

        let mut current_ptr = match &self.value.body {
            ast::ValueBody::Symbol(a) => {
                if let Some(ptr) = context.variables.get(a) {
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

            a => {
                return Err(RuntimeError::RuntimeError {
                    message: format!("Expected symbol or type-indicator, found {}", a.fmt(0)),
                    info: info.clone(),
                })
            }
        };

        for p in &self.path {
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
                            if !stored.2 {
                                return Err(RuntimeError::RuntimeError {
                                    message: "Cannot edit members of a constant value".to_string(),
                                    info: info.clone(),
                                });
                            }
                            if let Value::Dict(d) = &mut stored.0 {
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
                ast::Path::Associated(m) => {
                    match &globals.stored_values[current_ptr] {
                        Value::TypeIndicator(t) => match context.implementations.get_mut(t) {
                            Some(imp) => {
                                if let Some(val) = imp.get(m) {
                                    current_ptr = *val;
                                } else {
                                    (*imp).insert(m.clone(), value);
                                    defined = false;
                                    current_ptr = value;
                                }
                            }
                            None => {
                                let mut new_imp = HashMap::new();
                                new_imp.insert(m.clone(), value);
                                context.implementations.insert(*t, new_imp);
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

impl ast::CompoundStatement {
    pub fn to_scope(
        &self,
        context: &Context,
        globals: &mut Globals,
        info: CompilerInfo,
        start_group: Option<Group>,
    ) -> Result<(Function, Returns), RuntimeError> {
        //create the function context
        let mut new_context = context.next_fn_id(globals);

        //pick a start group
        let start_group = if let Some(g) = start_group {
            g
        } else {
            Group::next_free(&mut globals.closed_groups)
        };

        new_context.start_group = start_group;
        let new_info = info;
        let (contexts, inner_returns) =
            compile_scope(&self.statements, vec![new_context], globals, new_info)?;

        for c in contexts {
            if let Some(i) = c.broken {
                return Err(RuntimeError::RuntimeError {
                    message: "break statement is never used because it's inside a function"
                        .to_string(),
                    info: i,
                });
            }
        }

        Ok((Function { start_group }, inner_returns))
    }
}
