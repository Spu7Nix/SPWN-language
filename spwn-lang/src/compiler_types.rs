///types and functions used by the compiler
use crate::ast;
use crate::builtin::*;
use crate::levelstring::*;
//use std::boxed::Box;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::compiler::{compile_scope, import_module, next_free, RuntimeError, NULL_STORAGE};

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
        &self.map.get(&i).expect(&format!("index {} not found", i)).0
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
                (0, (Value::Builtins, Group { id: 0 }, false, 1)),
                (1, (Value::Null, Group { id: 0 }, false, 1)),
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
                for (_, e, _, _) in m.args {
                    if let Some(val) = e {
                        self.increment_single_lifetime(val, amount)
                    }
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
    let old_val = globals.stored_values[index].clone();
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

pub type Returns = Vec<(StoredValue, Context)>;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Context {
    pub start_group: Group,
    pub spawn_triggered: bool,
    pub variables: HashMap<String, StoredValue>,
    //pub self_val: Option<StoredValue>,
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
    pub fn new() -> Self {
        CompilerInfo {
            depth: 0,
            path: vec!["main scope".to_string()],
            line: (0, 0),
            func_id: 0,
        }
    }
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
            //self_val: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Macro {
    pub args: Vec<(String, Option<StoredValue>, ast::Tag, Option<TypeID>)>,
    pub def_context: Context,
    pub body: Vec<ast::Statement>,
    pub tag: ast::Tag,
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
    Array(Vec<StoredValue>),
    Obj(Vec<(u16, String)>, ast::ObjectMode),
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
            Value::Obj(_, mode) => match mode {
                ast::ObjectMode::Object => 11,
                ast::ObjectMode::Trigger => 16,
            },
            Value::Builtins => 12,
            Value::BuiltinFunction(_) => 13,
            Value::TypeIndicator(_) => 14,
            Value::Null => 15,
        }
    }
}

//copied from https://stackoverflow.com/questions/59401720/how-do-i-find-the-key-for-a-value-in-a-hashmap
pub fn find_key_for_value<'a>(map: &'a HashMap<String, u16>, value: u16) -> Option<&'a String> {
    map.iter()
        .find_map(|(key, &val)| if val == value { Some(key) } else { None })
}

pub fn convert_type(
    val: Value,
    typ: TypeID,
    info: CompilerInfo,
    globals: &Globals,
) -> Result<Value, RuntimeError> {
    if typ == 9 {
        return Ok(Value::Str(val.to_str(globals)));
    }

    Ok(match &val {
        Value::Number(n) => match typ {
            0 => Value::Group(Group { id: *n as u16 }),
            1 => Value::Color(Color { id: *n as u16 }),
            2 => Value::Block(Block { id: *n as u16 }),
            3 => Value::Item(Item { id: *n as u16 }),
            4 => Value::Number(*n),
            5 => Value::Bool(*n != 0.0),

            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Number can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info,
                })
            }
        },

        Value::Group(g) => match typ {
            0 => val,
            4 => Value::Number(g.id as f64),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Group can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info,
                })
            }
        },

        Value::Color(c) => match typ {
            1 => val,
            4 => Value::Number(c.id as f64),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Color can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info,
                })
            }
        },

        Value::Block(b) => match typ {
            2 => val,
            4 => Value::Number(b.id as f64),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Block ID can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info,
                })
            }
        },

        Value::Item(i) => match typ {
            3 => val,
            4 => Value::Number(i.id as f64),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Item ID can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info,
                })
            }
        },

        Value::Bool(b) => match typ {
            5 => val,
            4 => Value::Number(if *b { 1.0 } else { 0.0 }),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Boolean can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info,
                })
            }
        },

        Value::Func(f) => match typ {
            6 => val,
            0 => Value::Group(f.start_group),
            _ => {
                return Err(RuntimeError::RuntimeError {
                    message: format!(
                        "Function can't be converted to '{}'!",
                        find_key_for_value(&globals.type_ids, typ).unwrap()
                    ),
                    info,
                })
            }
        },

        _ => {
            return Err(RuntimeError::RuntimeError {
                message: format!(
                    "This value can't be converted to '{}'!",
                    find_key_for_value(&globals.type_ids, typ).unwrap()
                ),
                info,
            })
        }
    })
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
            Value::Macro(m) => {
                let mut out = String::from("(");
                if !m.args.is_empty() {
                    for arg in m.args.iter() {
                        out += &arg.0;
                        match arg.3.clone() {
                            Some(val) => {
                                out += &format!(
                                    ": @{}",
                                    match find_key_for_value(&globals.type_ids, val) {
                                        Some(s) => s,
                                        None => "undefined",
                                    }
                                )
                            }
                            None => (),
                        };
                        match arg.1.clone() {
                            Some(val) => {
                                out += &format!(" = {}", globals.stored_values[val].to_str(globals))
                            }
                            None => (),
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

pub struct Globals {
    pub closed_groups: Vec<u16>,
    pub closed_colors: Vec<u16>,
    pub closed_blocks: Vec<u16>,
    pub closed_items: Vec<u16>,
    pub path: PathBuf,

    pub lowest_y: HashMap<u32, u16>,
    pub stored_values: ValStorage,
    pub val_id: usize,

    pub type_ids: HashMap<String, u16>,
    pub type_id_count: u16,

    pub func_ids: Vec<FunctionID>,
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

    pub fn assert_mutable(
        &self,
        p: StoredValue,
        info: CompilerInfo,
        attempted_op_macro: &str,
    ) -> Result<(), RuntimeError> {
        match self.stored_values.map.get(&p) {
            Some(val) => {
                if val.2 {
                    Ok(())
                } else {
                    Err(RuntimeError::RuntimeError {
                        message: format!(
                            "
This value is not mutable! 
Consider defining it with 'let', or implementing a '{}' macro on its type.",
                            attempted_op_macro
                        ),
                        info,
                    })
                }
            }
            None => unreachable!(),
        }
    }
}

impl Globals {
    pub fn new(notes: crate::parser::ParseNotes, path: PathBuf) -> Self {
        let storage = ValStorage::new();
        let mut globals = Globals {
            closed_groups: notes.closed_groups,
            closed_colors: notes.closed_colors,
            closed_blocks: notes.closed_blocks,
            closed_items: notes.closed_items,
            path,

            lowest_y: HashMap::new(),

            type_ids: HashMap::new(),
            type_id_count: 0,

            val_id: storage.map.len(),
            stored_values: storage,
            func_ids: vec![FunctionID {
                name: "main scope".to_string(),
                parent: None,
                width: None,
                obj_list: Vec::new(),
            }],
        };

        globals.type_ids.insert(String::from("group"), 0);
        globals.type_ids.insert(String::from("color"), 1);
        globals.type_ids.insert(String::from("block"), 2);
        globals.type_ids.insert(String::from("item"), 3);
        globals.type_ids.insert(String::from("number"), 4);
        globals.type_ids.insert(String::from("bool"), 5);
        globals.type_ids.insert(String::from("function"), 6);
        globals.type_ids.insert(String::from("dictionary"), 7);
        globals.type_ids.insert(String::from("macro"), 8);
        globals.type_ids.insert(String::from("string"), 9);
        globals.type_ids.insert(String::from("array"), 10);
        globals.type_ids.insert(String::from("object"), 11);
        globals.type_ids.insert(String::from("spwn"), 13);
        globals.type_ids.insert(String::from("builtin"), 13);
        globals.type_ids.insert(String::from("type_indicator"), 14);
        globals.type_ids.insert(String::from("null"), 15);
        globals.type_ids.insert(String::from("trigger"), 16);

        globals.type_id_count = globals.type_ids.len() as u16;

        globals
    }
}

fn handle_operator<F>(
    value1: StoredValue,
    value2: StoredValue,
    op: F,
    macro_name: &str,
    op_name: &str,
    context: Context,
    globals: &mut Globals,
    info: CompilerInfo,
) -> Result<Returns, RuntimeError>
where
    F: Fn(StoredValue, StoredValue, &mut Globals, &Context) -> Result<StoredValue, RuntimeError>,
{
    Ok(
        if let Some(val) =
            globals.stored_values[value1]
                .clone()
                .member(macro_name.to_string(), &context, globals)
        {
            if let Value::Macro(m) = globals.stored_values[val].clone() {
                let new_info = info.next(op_name, globals, false);
                if m.args.is_empty() {
                    return Err(RuntimeError::RuntimeError {
                        message: String::from("Expected at least one argument in operator macro"),
                        info: new_info,
                    });
                }
                let val2 = globals.stored_values[value2].clone();

                if let Some(target_typ) = m.args[0].3 {
                    let val2storedtyp = val2
                        .member(TYPE_MEMBER_NAME.to_string(), &context, globals)
                        .unwrap();
                    let val2typ = match &globals.stored_values[val2storedtyp] {
                        Value::TypeIndicator(t) => t,
                        _ => unreachable!(),
                    };

                    if *val2typ != target_typ {
                        //if types dont match, act as if there is no macro at all
                        return Ok(vec![(op(value1, value2, globals, &context)?, context)]);
                    }
                }

                let (values, _) = execute_macro(
                    (
                        m,
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
                vec![(op(value1, value2, globals, &context)?, context)]
            }
        } else {
            vec![(op(value1, value2, globals, &context)?, context)]
        },
    )
}

const CANNOT_CHANGE_ERROR: &str = "
Cannot change a variable that was defined in another group/function context
(consider using a counter)
";

impl ast::Expression {
    pub fn eval(
        &self,
        context: Context,
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
                let evaled = var.to_value(c, globals, info.clone(), constant)?;
                inner_returns.extend(evaled.1);

                use ast::Operator::*;

                let acum_val_fn_context = globals.get_val_fn_context(acum_val, info.clone())?;

                for (val, c2) in evaled.0 {
                    //let val_fn_context = globals.get_val_fn_context(val, info.clone());
                    let vals: Returns = match self.operators[i] {
                        Or => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val,
                             val,
                             globals: &mut Globals,
                             c2: &Context|
                             -> Result<StoredValue, RuntimeError> {
                                if let Value::Bool(b) = globals.stored_values[acum_val] {
                                    if let Value::Bool(b2) = globals.stored_values[val] {
                                        Ok(store_value(Value::Bool(b || b2), 1, globals, c2))
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
                            |acum_val, val, globals, c2| {
                                if let Value::Bool(b) = globals.stored_values[acum_val] {
                                    if let Value::Bool(b2) = globals.stored_values[val] {
                                        Ok(store_value(Value::Bool(b && b2), 1, globals, c2))
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
                            |acum_val, val, globals, c2| {
                                if let Value::Number(n) = &globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = &globals.stored_values[val] {
                                        Ok(store_value(Value::Bool(n > n2), 1, globals, c2))
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
                            |acum_val, val, globals, c2| {
                                if let Value::Number(n) = &globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = &globals.stored_values[val] {
                                        Ok(store_value(Value::Bool(n < &n2), 1, globals, c2))
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
                            |acum_val, val, globals, c2| {
                                if let Value::Number(n) = &globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = &globals.stored_values[val] {
                                        Ok(store_value(Value::Bool(n >= &n2), 1, globals, c2))
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
                            |acum_val, val, globals, c2| {
                                if let Value::Number(n) = &globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = &globals.stored_values[val] {
                                        Ok(store_value(Value::Bool(n <= &n2), 1, globals, c2))
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
                        Slash => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val, globals, c2: &Context| {
                                if let Value::Number(n) = &globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = &globals.stored_values[val] {
                                        Ok(store_value(Value::Number(n / n2), 1, globals, c2))
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
                        Star => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val, globals, c2| {
                                if let Value::Number(n) = &globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = &globals.stored_values[val] {
                                        Ok(store_value(Value::Number(n * n2), 1, globals, c2))
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
                            |acum_val, val, globals, c2| {
                                if let Value::Number(n) = &globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = &globals.stored_values[val] {
                                        Ok(store_value(Value::Number(n % n2), 1, globals, c2))
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
                            |acum_val, val, globals, c2| {
                                if let Value::Number(n) = &globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = &globals.stored_values[val] {
                                        Ok(store_value(Value::Number(n.powf(*n2)), 1, globals, c2))
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
                            |acum_val, val, globals, c2| {
                                if let Value::Number(n) = &globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = &globals.stored_values[val] {
                                        Ok(store_value(Value::Number(n + n2), 1, globals, c2))
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
                            "plus",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Minus => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val, globals, c2| {
                                if let Value::Number(n) = &globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = &globals.stored_values[val] {
                                        Ok(store_value(Value::Number(n - n2), 1, globals, c2))
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
                            "minus",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Equal => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val, globals, c2| {
                                Ok(store_value(
                                    Value::Bool(
                                        globals.stored_values[acum_val]
                                            == globals.stored_values[val],
                                    ),
                                    1,
                                    globals,
                                    c2,
                                ))
                            },
                            "_equal_",
                            "logical equal",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        NotEqual => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val, globals, c2| {
                                Ok(store_value(Value::Bool(acum_val != val), 1, globals, c2))
                            },
                            "_not_equal_",
                            "logical inequality",
                            c2,
                            globals,
                            info.clone(),
                        )?,
                        Range => vec![(
                            store_value(
                                Value::Array({
                                    let start = match globals.stored_values[acum_val] {
                                        Value::Number(n) => n as i32,
                                        _ => {
                                            return Err(RuntimeError::RuntimeError {
                                                message: "Both sides of range must be Numbers"
                                                    .to_string(),
                                                info: info.clone(),
                                            })
                                        }
                                    };
                                    let end = match globals.stored_values[val] {
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
                                    .map(|x| store_value(Value::Number(x as f64), 1, globals, &c2))
                                    .collect()
                                }),
                                1,
                                globals,
                                &c2,
                            ),
                            c2,
                        )],
                        //MUTABLE ONLY
                        //ADD CHECk
                        Assign => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val, globals, c2| {
                                globals.assert_mutable(acum_val, info.clone(), "_assign_")?;
                                if acum_val_fn_context != c2.start_group {
                                    return Err(RuntimeError::RuntimeError {
                                        message: CANNOT_CHANGE_ERROR.to_string(),
                                        info: info.clone(),
                                    });
                                }

                                globals.stored_values[acum_val] =
                                    globals.stored_values[val].clone();
                                Ok(acum_val)
                            },
                            "_assign_",
                            "assign",
                            c2,
                            globals,
                            info.clone(),
                        )?,

                        As => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val, globals, c2| {
                                if let Value::TypeIndicator(t) = &globals.stored_values[val] {
                                    Ok(store_value(
                                        convert_type(
                                            globals.stored_values[acum_val].clone(),
                                            *t,
                                            info.clone(),
                                            globals,
                                        )?,
                                        1,
                                        globals,
                                        &c2,
                                    ))
                                } else {
                                    return Err(RuntimeError::RuntimeError {
                                        message: "Expected a type-indicator to convert to!"
                                            .to_string(),
                                        info: info.clone(),
                                    });
                                }
                            },
                            "_as_",
                            "as",
                            c2,
                            globals,
                            info.clone(),
                        )?,

                        Add => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val, globals, c2| {
                                globals.assert_mutable(acum_val, info.clone(), "_add_")?;
                                if acum_val_fn_context != c2.start_group {
                                    return Err(RuntimeError::RuntimeError {
                                        message: CANNOT_CHANGE_ERROR.to_string(),
                                        info: info.clone(),
                                    });
                                }
                                let gotten_val = globals.stored_values[val].clone();
                                if let Value::Number(n) = &mut globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = gotten_val {
                                        (*n) += n2;
                                    } else {
                                        return Err(RuntimeError::RuntimeError {
                                            message: "Cannot add this type to number".to_string(),
                                            info: info.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::RuntimeError {
                                        message: "'_add_' macro is not defined for this type."
                                            .to_string(),
                                        info: info.clone(),
                                    });
                                }
                                Ok(acum_val)
                            },
                            "_add_",
                            "add",
                            c2,
                            globals,
                            info.clone(),
                        )?,

                        Subtract => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val, globals, c2| {
                                globals.assert_mutable(acum_val, info.clone(), "_subtract_")?;
                                if acum_val_fn_context != c2.start_group {
                                    return Err(RuntimeError::RuntimeError {
                                        message: CANNOT_CHANGE_ERROR.to_string(),
                                        info: info.clone(),
                                    });
                                }
                                let gotten_val = globals.stored_values[val].clone();
                                if let Value::Number(n) = &mut globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = gotten_val {
                                        (*n) -= n2;
                                    } else {
                                        return Err(RuntimeError::RuntimeError {
                                            message: "Cannot subtract this type from number"
                                                .to_string(),
                                            info: info.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::RuntimeError {
                                        message: "'_subtract_' macro is not defined for this type."
                                            .to_string(),
                                        info: info.clone(),
                                    });
                                }
                                Ok(acum_val)
                            },
                            "_subtract_",
                            "subtract",
                            c2,
                            globals,
                            info.clone(),
                        )?,

                        Multiply => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val, globals, c2| {
                                globals.assert_mutable(acum_val, info.clone(), "_multiply_")?;
                                if acum_val_fn_context != c2.start_group {
                                    return Err(RuntimeError::RuntimeError {
                                        message: CANNOT_CHANGE_ERROR.to_string(),
                                        info: info.clone(),
                                    });
                                }
                                let gotten_val = globals.stored_values[val].clone();
                                if let Value::Number(n) = &mut globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = gotten_val {
                                        (*n) *= n2;
                                    } else {
                                        return Err(RuntimeError::RuntimeError {
                                            message: "Cannot multiply this type by number"
                                                .to_string(),
                                            info: info.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::RuntimeError {
                                        message: "'_multiply_' macro is not defined for this type."
                                            .to_string(),
                                        info: info.clone(),
                                    });
                                }
                                Ok(acum_val)
                            },
                            "_multiply_",
                            "multiply",
                            c2,
                            globals,
                            info.clone(),
                        )?,

                        Divide => handle_operator(
                            acum_val.clone(),
                            val,
                            |acum_val, val, globals, c2| {
                                globals.assert_mutable(acum_val, info.clone(), "_divide_")?;
                                if acum_val_fn_context != c2.start_group {
                                    return Err(RuntimeError::RuntimeError {
                                        message: CANNOT_CHANGE_ERROR.to_string(),
                                        info: info.clone(),
                                    });
                                }
                                let gotten_val = globals.stored_values[val].clone();
                                if let Value::Number(n) = &mut globals.stored_values[acum_val] {
                                    if let Value::Number(n2) = gotten_val {
                                        (*n) /= n2;
                                    } else {
                                        return Err(RuntimeError::RuntimeError {
                                            message: "Cannot divide this type by number"
                                                .to_string(),
                                            info: info.clone(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::RuntimeError {
                                        message: "'_divide_' macro is not defined for this type."
                                            .to_string(),
                                        info: info.clone(),
                                    });
                                }
                                Ok(acum_val)
                            },
                            "_divide_",
                            "divide",
                            c2,
                            globals,
                            info.clone(),
                        )?,
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
    parent: StoredValue,
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
            true,
        )?;
        inner_inner_returns.extend(inner_returns);

        for (arg_values, mut new_context) in evaled_args {
            new_context.variables = m.def_context.variables.clone();
            let mut new_variables: HashMap<String, StoredValue> = HashMap::new();

            //parse each argument given into a local macro variable
            for (i, arg) in args.iter().enumerate() {
                let abs_index = if m.args[0].0 == "self" { i + 1 } else { i };

                match &arg.symbol {
                    Some(name) => {
                        let arg_def = m.args.iter().find(|e| e.0 == *name);
                        if let Some(arg_def) = arg_def {
                            //type check!!
                            //maybe make type check function
                            match arg_def.3 {
                                Some(t) => {
                                    let val = globals.stored_values[arg_values[i]].clone();
                                    let type_of_val_index = val
                                        .member(TYPE_MEMBER_NAME.to_string(), &context, globals)
                                        .unwrap();

                                    let type_of_val = match globals.stored_values[type_of_val_index]
                                    {
                                        Value::TypeIndicator(t) => t,
                                        _ => unreachable!(),
                                    };

                                    if type_of_val != t {
                                        return Err(RuntimeError::TypeError {
                                            expected: Value::TypeIndicator(t).to_str(globals),
                                            found: Value::TypeIndicator(type_of_val)
                                                .to_str(globals),
                                            info,
                                        });
                                    }
                                }
                                None => (),
                            };

                            new_variables.insert(name.clone(), arg_values[i]);
                        } else {
                            return Err(RuntimeError::UndefinedErr {
                                undefined: name.clone(),
                                info: info.clone(),
                                desc: "macro argument".to_string(),
                            });
                        }
                    }
                    None => {
                        if (abs_index) > m.args.len() - 1 {
                            return Err(RuntimeError::RuntimeError {
                                message: "Too many arguments!".to_string(),
                                info: info.clone(),
                            });
                        }

                        //type check!!
                        match m.args[abs_index].3 {
                            Some(t) => {
                                let val = globals.stored_values[arg_values[i]].clone();
                                let type_of_val_index = val
                                    .member(TYPE_MEMBER_NAME.to_string(), &context, globals)
                                    .unwrap();

                                let type_of_val = match globals.stored_values[type_of_val_index] {
                                    Value::TypeIndicator(t) => t,
                                    _ => unreachable!(),
                                };

                                if type_of_val != t {
                                    return Err(RuntimeError::TypeError {
                                        expected: Value::TypeIndicator(t).to_str(globals),
                                        found: Value::TypeIndicator(type_of_val).to_str(globals),
                                        info,
                                    });
                                }
                            }
                            None => (),
                        };

                        new_variables.insert(m.args[abs_index].0.clone(), arg_values[i]);
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
                new_context.variables.insert("self".to_string(), parent);
                m_args_iter.next();
            }
            for arg in m_args_iter {
                if !new_variables.contains_key(&arg.0) {
                    match &arg.1 {
                        Some(default) => {
                            new_variables.insert(arg.0.clone(), *default);
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
        compiled.0.iter().map(|x| (1, x.clone())).collect()
    } else {
        if compiled.1.len() > 1 {
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
                        let mut params = HashMap::new();
                        params.insert(1, "1268".to_string());
                        params.insert(51, start_group.id.to_string());
                        let obj = GDObj {
                            params,

                            ..context_trigger(cont.clone(), globals, info.clone())
                        }
                        .context_parameters(cont.clone());
                        (*globals).func_ids[info.func_id].obj_list.push(obj);
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
    constant: bool,
) -> Result<(Vec<(Vec<StoredValue>, Context)>, Returns), RuntimeError> {
    let mut out: Vec<(Vec<StoredValue>, Context)> = Vec::new();
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
                .eval(context, globals, info.clone(), constant)?;
        out.extend(
            start_values
                .iter()
                .map(|x| (vec![x.0.clone()], x.1.clone())),
        );
        inner_returns.extend(start_returns);
        //for the rest of the expressions
        for expr in a_iter {
            //all the new combinations end up in this
            let mut new_out: Vec<(Vec<StoredValue>, Context)> = Vec::new();
            //run through all the lists in out
            for (inner_arr, c) in out.iter() {
                //for each one, run through all the returns in that context
                let (values, returns) = expr.eval(c.clone(), globals, info.clone(), constant)?;
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
        context.clone(),
        globals,
        info.clone(),
        constant,
    )?;
    inner_returns.extend(returns);
    let mut out = Returns::new();
    for expressions in evaled {
        let mut expr_index: usize = 0;
        let mut dict_out: HashMap<String, StoredValue> = HashMap::new();
        for def in dict.clone() {
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
                                    info: info.clone(),
                                })
                            }
                        },
                    );
                }
            };
            expr_index += 1;
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
        context: Context,
        globals: &mut Globals,
        info: CompilerInfo,
        //mut define_new: bool,
        constant: bool,
    ) -> Result<(Returns, Returns), RuntimeError> {
        // TODO: Check if this variable has native functions called on it, and if not set this to false
        let mut start_val = Returns::new();
        let mut inner_returns = Returns::new();

        //let mut defined = true;

        use ast::IDClass;

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
                store_value(
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
                    1,
                    globals,
                    &context,
                ),
                context.clone(),
            )),
            ast::ValueBody::Number(num) => start_val.push((
                store_value(Value::Number(*num), 1, globals, &context),
                context.clone(),
            )),
            ast::ValueBody::Dictionary(dict) => {
                let new_info = info.next("dictionary", globals, false);
                let (new_out, new_inner_returns) =
                    eval_dict(dict.clone(), context.clone(), globals, new_info, constant)?;
                start_val = new_out;
                inner_returns = new_inner_returns;
            }
            ast::ValueBody::CmpStmt(cmp_stmt) => {
                let (evaled, returns) = cmp_stmt.to_scope(&context, globals, info.clone())?;
                inner_returns.extend(returns);
                start_val.push((
                    store_value(Value::Func(evaled), 1, globals, &context),
                    context.clone(),
                ));
            }

            ast::ValueBody::Expression(expr) => {
                let (evaled, returns) =
                    expr.eval(context.clone(), globals, info.clone(), constant)?;
                inner_returns.extend(returns);
                start_val.extend(evaled.iter().cloned());
            }

            ast::ValueBody::Bool(b) => start_val.push((
                store_value(Value::Bool(*b), 1, globals, &context),
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
                                info: info.clone(),
                                desc: "variable".to_string(),
                            });
                        }
                    }
                }
            }
            ast::ValueBody::Str(s) => start_val.push((
                store_value(Value::Str(s.clone()), 1, globals, &context),
                context.clone(),
            )),
            ast::ValueBody::Array(a) => {
                let new_info = info.next("array", globals, false);
                let (evaled, returns) =
                    all_combinations(a.clone(), context.clone(), globals, new_info, constant)?;
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
                        Some(id) => store_value(Value::TypeIndicator(*id), 1, globals, &context),
                        None => {
                            return Err(RuntimeError::UndefinedErr {
                                undefined: name.clone(),
                                info: info.clone(),
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
                let new_info = info.next("object", globals, false);
                let (evaled, returns) =
                    all_combinations(all_expr, context.clone(), globals, new_info, constant)?;
                inner_returns.extend(returns);
                for (expressions, context) in evaled {
                    let mut obj: Vec<(u16, String)> = Vec::new();
                    for i in 0..(o.props.len()) {
                        let v = expressions[i * 2].clone();
                        let v2 = expressions[i * 2 + 1].clone();

                        obj.push((
                            match &globals.stored_values[v] {
                                Value::Number(n) => *n as u16,
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
                            match &globals.stored_values[v2] {
                                Value::Number(n) => n.to_string(),
                                Value::Str(s) => s.clone(),
                                Value::Func(g) => g.start_group.id.to_string(),

                                Value::Group(g) => g.id.to_string(),
                                Value::Color(c) => c.id.to_string(),
                                Value::Block(b) => b.id.to_string(),
                                Value::Item(i) => i.id.to_string(),

                                Value::Bool(b) => {
                                    if *b {
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
                    start_val.push((
                        store_value(Value::Obj(obj, o.mode), 1, globals, &context),
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
                let new_info = info.next("macro argument", globals, false);
                let (argument_possibilities, returns) =
                    all_combinations(all_expr, context.clone(), globals, new_info, constant)?;
                inner_returns.extend(returns);
                for defaults in argument_possibilities {
                    let mut args: Vec<(String, Option<StoredValue>, ast::Tag, Option<TypeID>)> =
                        Vec::new();
                    let mut expr_index = 0;
                    for arg in m.args.iter() {
                        args.push((
                            arg.0.clone(),
                            match &arg.1 {
                                Some(_) => {
                                    expr_index += 1;
                                    Some(defaults.0[expr_index - 1])
                                }
                                None => None,
                            },
                            arg.2.clone(),
                            match &arg.3 {
                                Some(_) => {
                                    expr_index += 1;
                                    Some(match &globals.stored_values[defaults.0[expr_index - 1]] {
                                        Value::TypeIndicator(t) => *t,
                                        a => return Err(RuntimeError::RuntimeError {
                                            message: format!("Expected TypeIndicator on argument definition, found: {}", a.to_str(globals)),
                                            info,
                                        })
                                    })
                                }
                                None => None,
                            },
                        ));
                    }

                    start_val.push((
                        store_value(
                            Value::Macro(Macro {
                                args,
                                body: m.body.statements.clone(),
                                def_context: defaults.1.clone(),
                                tag: m.properties.clone(),
                            }),
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
        let mut with_parent: Vec<(StoredValue, Context, StoredValue)> = start_val
            .iter()
            .map(|x| (x.0.clone(), x.1.clone(), 1))
            .collect();
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
                                        info: info.clone(),
                                        desc: "member".to_string(),
                                    });
                                }
                            },
                            x.1.clone(),
                            x.0.clone(),
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
                                                        info: info.clone(),
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
                                                info: info.clone(),
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
                                            info: info.clone(),
                                        });
                                    }
                                },
                                a => {
                                    return Err(RuntimeError::RuntimeError {
                                        message: format!(
                                            "Expected type indicator, found: {}",
                                            a.to_str(globals)
                                        ),
                                        info: info.clone(),
                                    })
                                }
                            },
                            x.1.clone(),
                            x.0.clone(),
                        )
                    }
                }

                ast::Path::Index(i) => {
                    let mut new_out: Vec<(StoredValue, Context, StoredValue)> = Vec::new();

                    for (prev_v, prev_c, _) in with_parent.clone() {
                        match globals.stored_values[prev_v].clone() {
                            Value::Array(arr) => {
                                let new_info = info.next("index", globals, false);
                                let (evaled, returns) =
                                    i.eval(prev_c, globals, new_info, constant)?;
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

                                            new_out.push((
                                                arr[*n as usize].clone(),
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
                                info: info.clone(),
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
                                    (m.clone(), args.clone()),
                                    cont.clone(),
                                    globals,
                                    *parent,
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
                                    constant,
                                )?;
                                inner_returns.extend(returns);

                                let mut all_values = Returns::new();

                                for (args, context) in evaled_args {
                                    let evaled = built_in_function(
                                        &name,
                                        args.iter()
                                            .map(|x| globals.stored_values[*x].clone())
                                            .collect(),
                                        info.clone(),
                                        globals,
                                        context.clone(),
                                    )?;
                                    all_values
                                        .push((store_value(evaled, 1, globals, &context), context))
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
                        if let Value::Number(n) = globals.stored_values[final_value.0] {
                            *final_value = (
                                store_value(Value::Number(-n), 1, globals, &context),
                                final_value.1.clone(),
                            );
                        } else {
                            return Err(RuntimeError::RuntimeError {
                                message: "Cannot make non-number type negative".to_string(),
                                info: info.clone(),
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
                                info: info.clone(),
                            });
                        }
                    }

                    UnaryOperator::Let => (),

                    UnaryOperator::Range => {
                        if let Value::Number(n) = globals.stored_values[final_value.0] {
                            *final_value = (
                                store_value(
                                    Value::Array(
                                        (if n > 0.0 {
                                            0..(n as i32)
                                        } else {
                                            (n as i32)..0
                                        })
                                        .map(|x| {
                                            store_value(
                                                Value::Number(x as f64),
                                                1,
                                                globals,
                                                &context,
                                            )
                                        })
                                        .collect(),
                                    ),
                                    1,
                                    globals,
                                    &context,
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
                    store_const_value(Value::TypeIndicator(*typ), 1, globals, context)
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
                    store_const_value(Value::TypeIndicator(*typ), 1, globals, context)
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
                                    message: format!("Cannot edit members of a constant value"),
                                    info: info.clone(),
                                });
                            }
                            if let Value::Dict(d) = &mut stored.0 {
                                (*d).insert(m.clone(), value);
                                defined = false;
                                current_ptr = value;
                            } else {
                                return Err(RuntimeError::RuntimeError {
                                    message: format!(
                                        "Cannot edit members of a non-dictionary value"
                                    ),
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
