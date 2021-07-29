//! Defining all native types (and functions?)
use crate::ast::ObjectMode;
use crate::compiler::{RuntimeError, NULL_STORAGE};
use crate::compiler_types::*;
use crate::context::*;
use crate::globals::Globals;
use crate::levelstring::*;
use std::collections::HashMap;
use std::fs;

use crate::value::*;
use crate::value_storage::*;
use std::io::stdout;
use std::io::Write;
//use text_io;
use crate::compiler_info::CompilerInfo;

macro_rules! arg_length {
    ($info:expr , $count:expr, $args:expr , $message:expr) => {
        if $args.len() != $count {
            return Err(RuntimeError::BuiltinError {
                message: $message,
                info: $info,
            });
        }
    };
}

pub type ArbitraryId = u16;
pub type SpecificId = u16;
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Id {
    Specific(SpecificId),
    Arbitrary(ArbitraryId), // will be given specific ids at the end of compilation
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Group {
    pub id: Id,
}

impl std::fmt::Debug for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.id {
            Id::Specific(n) => f.write_str(&format!("{}g", n)),
            Id::Arbitrary(n) => f.write_str(&format!("{}?g", n)),
        }
    }
}

impl Group {
    pub fn new(id: SpecificId) -> Self {
        //creates new specific group
        Group {
            id: Id::Specific(id),
        }
    }

    pub fn next_free(counter: &mut ArbitraryId) -> Self {
        //creates new specific group
        (*counter) += 1;
        Group {
            id: Id::Arbitrary(*counter),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Color {
    pub id: Id,
}

impl Color {
    pub fn new(id: SpecificId) -> Self {
        //creates new specific color
        Self {
            id: Id::Specific(id),
        }
    }

    pub fn next_free(counter: &mut ArbitraryId) -> Self {
        //creates new specific color
        (*counter) += 1;
        Self {
            id: Id::Arbitrary(*counter),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub id: Id,
}

impl Block {
    pub fn new(id: SpecificId) -> Self {
        //creates new specific block
        Self {
            id: Id::Specific(id),
        }
    }

    pub fn next_free(counter: &mut ArbitraryId) -> Self {
        //creates new specific block
        (*counter) += 1;
        Self {
            id: Id::Arbitrary(*counter),
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub id: Id,
}

impl Item {
    pub fn new(id: SpecificId) -> Self {
        //creates new specific item id
        Self {
            id: Id::Specific(id),
        }
    }

    pub fn next_free(counter: &mut ArbitraryId) -> Self {
        //creates new specific item id
        (*counter) += 1;
        Self {
            id: Id::Arbitrary(*counter),
        }
    }
}

pub fn context_trigger(context: &Context, uid_counter: &mut usize) -> GdObj {
    let mut params = HashMap::new();
    params.insert(57, ObjParam::Group(context.start_group));
    (*uid_counter) += 1;
    GdObj {
        params: HashMap::new(),
        func_id: context.func_id,
        mode: ObjectMode::Trigger,
        unique_id: *uid_counter,
        sync_group: context.sync_group,
        sync_part: context.sync_part,
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
                // Value::Func(f) => {
                //     if member == "group" {
                //         return Some(store_value(
                //             Value::Group(f.start_group),
                //             1,
                //             globals,
                //             context,
                //         ));
                //     }
                // }
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
                Value::Builtins => match Builtin::from_str(member.as_str()) {
                    Err(_) => None,
                    Ok(builtin) => Some(store_value(
                        Value::BuiltinFunction(builtin),
                        1,
                        globals,
                        context,
                    )),
                },
                Value::Dict(dict) => match dict.get(&member) {
                    Some(value) => Some(*value),
                    None => get_impl(my_type, member),
                },
                Value::TriggerFunc(f) => {
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

use std::str::FromStr;

macro_rules! typed_argument_check {

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident)  ($($arg_name:ident),*)) => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        #[allow(unused_parens)]
        let ( $($arg_name),*) = $globals.stored_values[$arguments[$arg_index]].clone();
    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident) mut ($($arg_name:ident),*)) => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        #[allow(unused_parens)]
        let ( $(mut $arg_name),*) = $globals.stored_values[$arguments[$arg_index]].clone();
    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident) ($($arg_name:ident),*): $arg_type:ident) => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        #[allow(unused_parens)]
        let  ( $($arg_name),*) = match $globals.stored_values[$arguments[$arg_index]].clone() {
            Value::$arg_type($($arg_name),*) => ($($arg_name),*),

            a => {
                return Err(RuntimeError::BuiltinError {
                    message: format!(
                        "Expected {} for argument {}, found {}",
                        stringify!($arg_type),
                        $arg_index + 1,
                        a.to_str($globals)
                    ),
                    $info,
                })
            }
        };
    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident) mut ($($arg_name:ident),*): $arg_type:ident) => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        #[allow(unused_parens)]
        let  ( $(mut $arg_name),*) = match $globals.stored_values[$arguments[$arg_index]].clone() {
            Value::$arg_type($($arg_name),*) => ($($arg_name),*),

            a => {
                return Err(RuntimeError::BuiltinError {
                    message: format!(
                        "Expected {} for argument {}, found {}",
                        stringify!($arg_type),
                        $arg_index + 1,
                        a.to_str($globals)
                    ),
                    $info,
                })
            }
        };
    };


}

macro_rules! reassign_variable {

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident) mut ($($arg_name:ident),*)) => {
        $globals.stored_values[$arguments[$arg_index]] = ($($arg_name)*);
    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident) mut ($($arg_name:ident),*): $arg_type:ident) => {
        $globals.stored_values[$arguments[$arg_index]] = Value::$arg_type($($arg_name),*);
    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident) ($($arg_name:ident),*)) => {};

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident) ($($arg_name:ident),*): $arg_type:ident) => {};


}

macro_rules! builtin_arg_mut_check {
    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident) mut ($($arg_name:ident),*)$(: $arg_type:ident)?) => {
        if !$globals.can_mutate($arguments[$arg_index]) {
            return Err(RuntimeError::BuiltinError {
                message: String::from("This value is not mutable"),
                $info,
            });
        }
        let fn_context = $globals.get_val_fn_context($arguments[$arg_index], $info.clone())?;
        if fn_context != $context.start_group {
            return Err(RuntimeError::RuntimeError {
                message: CANNOT_CHANGE_ERROR.to_string(),
                $info,
            });
        }
    };
    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident) ($($arg_name:ident),*)$(: $arg_type:ident)?) => {};
}

macro_rules! builtins {

    {
        ($arguments:ident, $info:ident, $globals:ident, $context:ident)
        $(
            [$variant:ident] fn $name:ident(
                $(
                    $(
                        $($mut:ident)? ($($arg_name:ident),*)$(: $arg_type:ident)?
                    ),+
                )?
            ) $body:block
        )*
    } => {

        #[derive(Debug,Clone, Copy, PartialEq, Eq)]
        pub enum Builtin {
            $(
                $variant,
            )*
        }
        pub const BUILTIN_LIST: &[Builtin] = &[
            $(
                Builtin::$variant,
            )*
        ];
        pub fn built_in_function(
            func: Builtin,
            $arguments: Vec<StoredValue>,
            $info: CompilerInfo,
            $globals: &mut Globals,
            $context: &Context,
        ) -> Result<Value, RuntimeError> {

            match func {
                $(
                    Builtin::$variant => {

                        $(
                            #[allow(unused_assignments)]
                            let mut arg_index = 0;
                            $(
                                if arg_index >= $arguments.len() {
                                    return Err(RuntimeError::BuiltinError {
                                        message: format!(
                                            "Too many arguments provided (expected {})",
                                            $arguments.len()
                                        ),
                                        $info,
                                    })
                                }

                                builtin_arg_mut_check!(
                                    ($globals, arg_index, $arguments, $info, $context) $($mut)?
                                    ($($arg_name),*)$(: $arg_type)?
                                );
                                typed_argument_check!(
                                    ($globals, arg_index, $arguments, $info) $($mut)?
                                    ($($arg_name),*)$(: $arg_type)?
                                );

                                arg_index += 1;
                            )+
                            if arg_index < $arguments.len() - 1 {
                                return Err(RuntimeError::BuiltinError {
                                    message: format!(
                                        "Too few arguments provided (expected {})",
                                        $arguments.len()
                                    ),
                                    $info,
                                })
                            }
                        )?

                        let out = $body;

                        $(

                            arg_index = 0;
                            $(


                                reassign_variable!(
                                    ($globals, arg_index, $arguments, $info) $($mut)? ($($arg_name),*)$(: $arg_type)?
                                );


                                arg_index += 1;
                            )+
                        )?
                        Ok(out)

                    }
                )+
            }
        }

        impl std::str::FromStr for Builtin {
            type Err = ();

            fn from_str(s: &str) -> std::result::Result<Builtin, Self::Err> {
                match s {
                    $(stringify!($name) => Ok(Self::$variant),)*
                    _ => Err(())
                }
            }
        }
        impl From<Builtin> for String {
            fn from(b: Builtin) -> Self {
                match b {
                    $(
                        Builtin::$variant => stringify!($name).to_string(),
                    )*
                }
            }
        }


    };
}

builtins! {
    (arguments, info, globals, context)

    [Assert]
    fn assert((b): Bool) {
        if !b {
            return Err(RuntimeError::BuiltinError {
                message: String::from("Assertion failed"),
                info,
            });
        } else {
            Value::Null
        }
    }

    [Print]
    fn print() {
        let mut out = String::new();
        for val in arguments {
            out += &globals.stored_values[val].to_str(globals);
        }
        println!("{}", out);
        Value::Null
    }

    [Time]
    fn time() {
        arg_length!(info, 0, arguments, "Expected no arguments".to_string());
        use std::time::SystemTime;
        let now = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(time) => time,
            Err(e) => {
                return Err(RuntimeError::BuiltinError {
                    message: format!("System time error: {}", e),
                    info,
                })
            }
        }
        .as_secs();
        Value::Number(now as f64)
    }

    [SpwnVersion]
    fn spwn_version() {
        arg_length!(info, 0, arguments, "Expected no arguments".to_string());

        Value::Str(env!("CARGO_PKG_VERSION").to_string())
    }

    [GetInput]
    fn get_input((prompt): Str) {
        print!("{}", prompt);
        stdout()
            .flush()
            .expect("Unexpected error occurred when trying to get user input");
        Value::Str(text_io::read!("{}\n"))
    }

    [Matches]
    fn matches((val), (pattern)) {
        Value::Bool(val.matches_pat(&pattern, &info, globals, context)?)
    }

    [B64Encode]
    fn b64encode((s): Str) {
        let encrypted = base64::encode(&s.as_bytes());
        Value::Str(encrypted)
    }

    [B64Decode]
    fn b64decode((s): Str) {
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

    [Sin] fn sin((n): Number) { Value::Number(n.sin()) }
    [Cos] fn cos((n): Number) { Value::Number(n.cos()) }
    [Tan] fn tan((n): Number) { Value::Number(n.tan()) }

    [Asin] fn asin((n): Number) { Value::Number(n.asin()) }
    [Acos] fn acos((n): Number) { Value::Number(n.acos()) }
    [Atan] fn atan((n): Number) { Value::Number(n.atan()) }

    [Floor] fn floor((n): Number) { Value::Number(n.floor()) }
    [Ceil] fn ceil((n): Number) { Value::Number(n.ceil()) }

    [Abs] fn abs((n): Number) {Value::Number(n.abs())}
    [Acosh] fn acosh((n): Number) {Value::Number(n.acosh())}
    [Asinh] fn asinh((n): Number) {Value::Number(n.asinh())}
    [Atan2] fn atan2((x): Number, (y): Number) {Value::Number(x.atan2(y))}
    [Atanh] fn atanh((n): Number) {Value::Number(n.atanh())}
    [Cbrt] fn cbrt((n): Number) {Value::Number(n.cbrt())}
    [Cosh] fn cosh((n): Number) {Value::Number(n.cosh())}
    [Exp] fn exp((n): Number) {Value::Number(n.exp())}
    [Exp2] fn exp2((n): Number) {Value::Number(n.exp2())}
    [Expm1] fn exp_m1((n): Number) {Value::Number(n.exp_m1())}
    [Fract] fn fract((n): Number) {Value::Number(n.fract())}


    [Add]
    fn add((obj, mode): Obj) {

        let mut obj_map = HashMap::<u16, ObjParam>::new();

        for p in obj {
            obj_map.insert(p.0, p.1.clone());
            // add params into map
        }

        match mode {
            ObjectMode::Object => {
                if context.start_group.id != Id::Specific(0) {
                    return Err(RuntimeError::BuiltinError { // objects cant be added dynamically, of course
                        message: String::from(
                            "you cannot add an obj type object at runtime"),
                        info
                    });
                }
                (*globals).uid_counter += 1;
                let obj = GdObj {
                    params: obj_map,
                    func_id: context.func_id,
                    mode: ObjectMode::Object,
                    unique_id: globals.uid_counter,
                    sync_group: context.sync_group,
                    sync_part: context.sync_part,
                };
                (*globals).objects.push(obj)
            }
            ObjectMode::Trigger => {

                let obj = GdObj {
                    params: obj_map,
                    mode: ObjectMode::Trigger,
                    ..context_trigger(context, &mut globals.uid_counter)
                }
                .context_parameters(context);
                (*globals).trigger_order += 1;
                (*globals).func_ids[context.func_id]
                    .obj_list
                    .push((obj, globals.trigger_order))
            }
        };
        Value::Null
    }

    [Append]
    fn append(mut (arr): Array, (val)) {
        //set lifetime to the lifetime of the array

        let cloned = clone_value(
            arguments[1],
            globals.get_lifetime(arguments[0]),
            globals,
            context.start_group,
            !globals.is_mutable(arguments[1]),
        );

        (arr).push(cloned);

        Value::Null
    }

    [SplitStr]
    fn split_str((s): Str, (substr): Str) {

        let mut output = Vec::<StoredValue>::new();

        for split in s.split(&*substr) {
            let entry =
                store_const_value(Value::Str(split.to_string()), 1, globals, context);
            output.push(entry);
        }

        Value::Array(output)
    }

    [EditObj]
    fn edit_obj(mut (o, m): Obj, (key), (value)) {

        let (okey, oval) = {
            let (key, pattern) = match key {
                Value::Number(n) => (n as u16, None),

                Value::Dict(d) => {
                    // this is specifically for object_key dicts
                    let gotten_type = d.get(TYPE_MEMBER_NAME);
                    if gotten_type == None
                        || globals.stored_values[*gotten_type.unwrap()]
                            != Value::TypeIndicator(19)
                    {
                        // 19 = object_key??
                        return Err(RuntimeError::RuntimeError {
                            message: "expected either @number or @object_key as object key"
                                .to_string(),
                            info,
                        });
                    }

                    let id = d.get("id");
                    if id == None {
                        return Err(RuntimeError::RuntimeError {
                            // object_key has an ID member for the key basically
                            message: "object key has no 'id' member".to_string(),
                            info,
                        });
                    }
                    let pattern = d.get("pattern");
                    if pattern == None {
                        return Err(RuntimeError::RuntimeError {
                            // same with pattern, for the expected type
                            message: "object key has no 'pattern' member".to_string(),
                            info,
                        });
                    }

                    (
                        match &globals.stored_values[*id.unwrap()] {
                            // check if the ID is actually an int. it should be
                            Value::Number(n) => *n as u16,
                            _ => {
                                return Err(RuntimeError::RuntimeError {
                                    message: format!(
                                        "object key's id has to be @number, found {}",
                                        globals.get_type_str(*id.unwrap())
                                    ),
                                    info,
                                })
                            }
                        },
                        Some(globals.stored_values[*pattern.unwrap()].clone()),
                    )
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

            if m == ObjectMode::Trigger && (key == 57 || key == 62) {
                // group ids and stuff on triggers
                return Err(RuntimeError::RuntimeError {
                    message: "You are not allowed to set the group ID(s) or the spawn triggered state of a @trigger. Use obj instead".to_string(),
                    info,
                });
            }

            if let Some(ref pat) = pattern {
                if !value.matches_pat(&pat, &info, globals, &context)? {
                    return Err(RuntimeError::RuntimeError {
                        message: format!(
                            "key required value to match {}, found {}",
                            pat.to_str(globals),
                            value.to_str(globals)
                        ),
                        info,
                    });
                }
            }
            let err = Err(RuntimeError::RuntimeError {
                message: format!("{} is not a valid object value", value.to_str(globals)),
                info: info.clone(),
            });

            let out_val = match &value {
                // its just converting value to objparam basic level stuff
                Value::Number(n) => ObjParam::Number(*n),
                Value::Str(s) => ObjParam::Text(s.clone()),
                Value::TriggerFunc(g) => ObjParam::Group(g.start_group),

                Value::Group(g) => ObjParam::Group(*g),
                Value::Color(c) => ObjParam::Color(*c),
                Value::Block(b) => ObjParam::Block(*b),
                Value::Item(i) => ObjParam::Item(*i),

                Value::Bool(b) => ObjParam::Bool(*b),

                Value::Array(a) => {
                    ObjParam::GroupList({
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
                    })
                }
                obj @ Value::Dict(_) => {
                    let typ = obj.member(TYPE_MEMBER_NAME.to_string(), context, globals).unwrap();
                    if globals.stored_values[typ] == Value::TypeIndicator(20) {
                        ObjParam::Epsilon
                    } else {
                        return err;
                    }
                }
                _ => {
                    return err;
                }
            };

            (key, out_val)
        };

        if !o.contains(&(okey, oval.clone())) {
            o.push((okey, oval))
        }


        Value::Null
    }

    [Mutability]
    fn mutability((var)) {
        Value::Bool(globals.can_mutate(arguments[0]))
    }

    [ExtendTriggerFunc]
    fn extend_trigger_func((group),(mac): Macro) {
        let group = match group {
            Value::Group(g) => g,
            Value::TriggerFunc(f) => f.start_group,
            a => {
                return Err(RuntimeError::BuiltinError {
                    message: format!(
                        "Expected group or trigger function, found {}",
                        a.to_str(globals)
                    ),
                    info,
                })
            }
        };


        let mut new_context = context.clone();
        new_context.start_group = group;

        execute_macro((*mac, Vec::new()), &new_context, globals, NULL_STORAGE, info)?;

        Value::Null
    }

    [ReadFile]
    fn readfile() {
        if arguments.is_empty() || arguments.len() > 2 {
            return Err(RuntimeError::BuiltinError {
                message: String::from("Expected 1 or 2 arguments, the path to the file and the data format (default: utf-8)"),
                info,
            });
        }

        let val = globals.stored_values[arguments[0]].clone();
        match val {
            Value::Str(p) => {
                let format = match arguments.get(1) {
                    Some(val) => {
                        if let Value::Str(s) = &globals.stored_values[*val] {
                            s
                        } else {
                            return Err(RuntimeError::BuiltinError {
                                message:
                                    "Data format needs to be a string (\"text\" or \"bin\")"
                                        .to_string(),
                                info,
                            });
                        }
                    }
                    _ => "text",
                };
                let path = globals
                    .path
                    .clone()
                    .parent()
                    .expect("Your file must be in a folder!")
                    .join(&p);

                if !path.exists() {
                    return Err(RuntimeError::BuiltinError {
                        message: "Path doesn't exist".to_string(),
                        info,
                    });
                }
                match format {
                    "text" => {
                        let ret = fs::read_to_string(path);
                        let rval = match ret {
                            Ok(file) => file,
                            Err(e) => {
                                return Err(RuntimeError::BuiltinError {
                                    message: format!("Problem opening the file: {}", e),
                                    info,
                                });
                            }
                        };
                        Value::Str(rval)
                    }
                    "bin" => {
                        let ret = fs::read(path);
                        let rval = match ret {
                            Ok(file) => file,
                            Err(e) => {
                                return Err(RuntimeError::BuiltinError {
                                    message: format!("Problem opening the file: {}", e),
                                    info,
                                });
                            }
                        };
                        Value::Array(
                            rval.iter()
                                .map(|b| {
                                    store_value(Value::Number(*b as f64), 1, globals, context)
                                })
                                .collect(),
                        )
                    }
                    _ => {
                        return Err(RuntimeError::BuiltinError {
                            message: "Invalid data format ( use \"text\" or \"bin\")"
                                .to_string(),
                            info,
                        })
                    }
                }
            }
            _ => {
                return Err(RuntimeError::BuiltinError {
                    message: "Path needs to be a string".to_string(),
                    info,
                });
            }
        }
    }

    [Pop]
    fn pop(mut (arr)) {

        let typ = globals.get_type_str(arguments[0]);

        match &mut arr {
            Value::Array(arr) => match arr.pop() {
                Some(val) => globals.stored_values[val].clone(),
                None => Value::Null,
            },
            Value::Str(s) => match s.pop() {
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

    [Substr]
    fn substr((val): Str, (start_index): Number, (end_index): Number) {
        let start_index = start_index as usize;
        let end_index = end_index as usize;
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

    [RemoveIndex]
    fn remove_index(mut (arr), (index): Number) {

        let typ = globals.get_type_str(arguments[0]);

        match &mut arr {
            Value::Array(arr) => {
                let out = (arr).remove(index as usize);
                globals.stored_values[out].clone()
            }

            Value::Str(s) => Value::Str(s.remove(index as usize).to_string()),
            _ => {
                return Err(RuntimeError::BuiltinError {
                    message: format!("Expected array or string, found @{}", typ),
                    info,
                })
            }
        }
    }

    [Regex] fn regex((regex): Str, (s): Str, (mode): Str, (replace): Str) {
        use regex::Regex;


            if let Ok(r) = Regex::new(&regex) {
                match &*mode {
                    "match" => Value::Bool(r.is_match(&s)),
                    "replace" => {
                        match &globals.stored_values[arguments[3]] {
                            Value::Str(replacer) => {
                                Value::Str(r.replace_all(&s, replacer).to_string())
                            }
                            _ => {
                                return Err(
                                    RuntimeError::BuiltinError {
                                        message: format!("Invalid or missing replacer. Expected @string, found @{}", &globals.get_type_str(arguments[3])),
                                        info
                                    }
                                )
                            }
                        }
                    }
                    _ => {
                        return Err(RuntimeError::BuiltinError {
                            message: format!(
                                "Invalid regex mode \"{}\" in regex {}. Expected \"match\" or \"replace\"",
                                mode, r
                            ),
                            info,
                        })
                    }
                }
            } else {
                return Err(RuntimeError::BuiltinError {
                    message: "Failed to build regex (invalid syntax)".to_string(),
                    info,
                });
            }

    }



    [RangeOp]
    fn _range_((val_a), (b): Number) {
        let end = convert_to_int(b, &info)?;
        match val_a {
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
                    if step <= 0 {
                        return Err(RuntimeError::RuntimeError {
                            message: "range cannot have a stepsize less than or 0"
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
                    message: format!(
                        "range start: expected @number, found @{}",
                        globals.get_type_str(arguments[0])
                    ),
                    info,
                });
            }
        }
    }

    [OrOp]              fn _or_((a): Bool, (b): Bool)                   { Value::Bool(a || b) }
    [AndOp]             fn _and_((a): Bool, (b): Bool)                  { Value::Bool(a && b) }

    [MoreThanOp]        fn _more_than_((a): Number, (b): Number)        { Value::Bool(a > b) }
    [LessThanOp]        fn _less_than_((a): Number, (b): Number)        { Value::Bool(a < b) }

    [MoreOrEqOp]        fn _more_or_equal_((a): Number, (b): Number)    { Value::Bool(a >= b) }
    [LessOrEqOp]        fn _less_or_equal_((a): Number, (b): Number)    { Value::Bool(a <= b) }

    [EqOp]              fn _equal_((a): Number, (b): Number)            { Value::Bool((a - b).abs() < f64::EPSILON) }
    [NotEqOp]           fn _not_equal_((a): Number, (b): Number)        { Value::Bool((a - b).abs() > f64::EPSILON) }

    [DividedByOp]       fn _divided_by_((a): Number, (b): Number)       { Value::Number(a / b) }
    [IntdividedByOp]    fn _intdivided_by_((a): Number, (b): Number)    { Value::Number((a / b).floor()) }
    [TimesOp]           fn _times_((a): Number, (b): Number)            { Value::Number(a * b) }
    [ModOp]             fn _mod_((a): Number, (b): Number)              { Value::Number(a % b) }
    [PowOp]             fn _pow_((a): Number, (b): Number)              { Value::Number(a.powf(b)) }
    [PlusOp]            fn _plus_((a): Number, (b): Number)             { Value::Number(a + b) }
    [MinusOp]           fn _minus_((a): Number, (b): Number)            { Value::Number(a - b) }

    [AssignOp]           fn _assign_(mut (a), (b))                      { a = b; Value::Null }
    [SwapOp]           fn _swap_(mut (a), mut (b))                      { std::mem::swap(&mut a, &mut b); Value::Null }

    [HasOp]
    fn _has_((a), (b)) {
        match (a, b) {
            (Value::Array(ar), _) => {
                let mut out = false;
                for v in ar.clone() {
                    if value_equality(v, arguments[1], globals) {
                        out = true;
                        break;
                    }
                }
                Value::Bool(out)
            }

            (Value::Dict(d), Value::Str(b)) => {
                let mut out = false;
                for k in d.keys() {
                    if k == &b {
                        out = true;
                        break;
                    }
                }
                Value::Bool(out)
            }

            (Value::Str(s), Value::Str(s2)) => Value::Bool(s.contains(&*s2)),

            (Value::Obj(o, _m), Value::Number(n)) => {
                let obj_has: bool = o.iter().any(|k| k.0 == n as u16);
                Value::Bool(obj_has)
            }

            (Value::Obj(o, _m), Value::Dict(d)) => {
                let gotten_type = d.get(TYPE_MEMBER_NAME);

                if gotten_type == None
                    || globals.stored_values[*gotten_type.unwrap()]
                        != Value::TypeIndicator(19)
                {
                    // 19 = object_key??
                    return Err(RuntimeError::TypeError {
                        expected: "either @number or @object_key".to_string(),
                        found: globals.get_type_str(arguments[1]),
                        info,
                    });
                }

                let id = d.get("id");
                if id == None {
                    return Err(RuntimeError::BuiltinError {
                        // object_key has an ID member for the key basically
                        message: "object key has no 'id' member".to_string(),
                        info,
                    });
                }
                let ob_key = match &globals.stored_values[*id.unwrap()] {
                    // check if the ID is actually an int. it should be
                    Value::Number(n) => *n as u16,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            expected: "@number as object key".to_string(),
                            found: globals.get_type_str(*id.unwrap()),
                            info,
                        })
                    }
                };
                let obj_has: bool = o.iter().any(|k| k.0 == ob_key);
                Value::Bool(obj_has)
            }

            (Value::Obj(_, _), _) => {
                return Err(RuntimeError::TypeError {
                    expected: "@number or @object_key".to_string(),
                    found: globals.get_type_str(arguments[1]),
                    info,
                })
            }

            (Value::Str(_), _) => {
                return Err(RuntimeError::TypeError {
                    expected: "string to compare".to_string(),
                    found: globals.get_type_str(arguments[1]),
                    info,
                })
            }

            (Value::Dict(_), _) => {
                return Err(RuntimeError::TypeError {
                    expected: "string as key".to_string(),
                    found: globals.get_type_str(arguments[1]),
                    info,
                })
            }

            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "array, dictionary, object, or string".to_string(),
                    found: globals.get_type_str(arguments[0]),
                    info,
                })
            }
        }
    }

    [AsOp]              fn _as_((a), (t): TypeIndicator)                    { convert_type(&a,t,&info,globals,&context)? }

    [SubtractOp]        fn _subtract_(mut (a): Number, (b): Number)         { a -= b; Value::Null }
    [AddOp]             fn _add_(mut (a): Number, (b): Number)              { a += b; Value::Null }
    [MultiplyOp]        fn _multiply_(mut (a): Number, (b): Number)         { a *= b; Value::Null }
    [DivideOp]          fn _divide_(mut (a): Number, (b): Number)           { a /= b; Value::Null }
    [IntdivideOp]       fn _intdivide_(mut (a): Number, (b): Number)        { a /= b; a = a.floor(); Value::Null }
    [ExponateOp]        fn _exponate_(mut (a): Number, (b): Number)         { a = a.powf(b); Value::Null }
    [ModulateOp]        fn _modulate_(mut (a): Number, (b): Number)         { a %= b; Value::Null }

    [EitherOp]
    fn _either_((a), (b)) {
        Value::Pattern(Pattern::Either(
            if let Value::Pattern(p) = convert_type(&a, 18, &info, globals, &context)? {
                Box::new(p)
            } else {
                unreachable!()
            },
            if let Value::Pattern(p) = convert_type(&b, 18, &info, globals, &context)? {
                Box::new(p)
            } else {
                unreachable!()
            },
        ))
    }

}

const CANNOT_CHANGE_ERROR: &str = "
Cannot change a variable that was defined in another trigger function context
(consider using a counter)
";
