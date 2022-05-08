//! Defining all native types (and functions?)
#![allow(unused_assignments)]
use internment::LocalIntern;
use shared::SpwnSource;
use shared::StoredValue;

use crate::compiler_types::*;
use crate::context::*;
use crate::globals::Globals;
use crate::leveldata::*;
use errors::{create_error, RuntimeError};
use fnv::FnvHashMap;
use parser::ast::ObjectMode;

use std::fs;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::value::*;
use crate::value_storage::*;

use std::io::stdout;
use std::io::Write;

use std::collections::hash_map::DefaultHasher;

// BUILT IN STD
use include_dir::{Dir, File};
use std::env;

#[cfg(not(debug_assertions))]
const STANDARD_LIBS: Dir = include_dir!("../libraries");

#[cfg(debug_assertions)]
const STANDARD_LIBS: Dir = Dir {
    path: "",
    files: &[],
    dirs: &[],
};

pub fn get_lib_file<'a, S: AsRef<Path>>(path: S) -> Option<File<'a>> {
    get_file(&STANDARD_LIBS, path.as_ref())
}

fn get_file<'a>(dir: &'a Dir, path: &Path) -> Option<File<'a>> {
    for file in dir.files {
        let replaced = &file.path.replace('\\', "/");
        let file_path = &Path::new(replaced);

        if Path::new(file_path) == path {
            return Some(*file);
        }
    }

    for dir in dir.dirs {
        if let Some(d) = get_file(dir, path) {
            return Some(d);
        }
    }

    None
}

fn div_zero_check(b: f64, op: &str, builtin: &str, info: &CompilerInfo) -> Result<(), RuntimeError> {
    if b.abs() == 0.0 {
        return Err(RuntimeError::BuiltinError {
            builtin: builtin.to_string(),
            message: format!("Cannot {} by 0", op),
            info: info.clone(),
        })
    } else {
        Ok(())
    }
}

//use text_io;
use errors::compiler_info::{CodeArea, CompilerInfo};

macro_rules! arg_length {
    ($info:expr , $count:expr, $args:expr , $message:expr, $builtin:ident) => {
        if $args.len() != $count {
            return Err(RuntimeError::BuiltinError {
                $builtin,
                message: $message,
                info: $info,
            });
        }
    };
}

pub fn context_trigger(context: &Context, uid_counter: &mut usize) -> GdObj {
    let mut params = FnvHashMap::default();
    params.insert(57, ObjParam::Group(context.start_group));
    (*uid_counter) += 1;
    GdObj {
        params: FnvHashMap::default(),
        func_id: context.func_id,
        mode: ObjectMode::Trigger,
        unique_id: *uid_counter,
    }
}

pub type ArbitraryId = u16;
pub type SpecificId = u16;
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Id {
    Specific(SpecificId),
    Arbitrary(ArbitraryId), // will be given specific ids at the end of compilation
}

macro_rules! id_default_methods {
    ($id_struct:ident, $short_name:expr) => {
        impl $id_struct {
            pub fn new(id: SpecificId) -> $id_struct {
                //creates new specific group
                $id_struct {
                    id: Id::Specific(id),
                }
            }

            pub fn next_free(counter: &mut ArbitraryId) -> $id_struct {
                //creates new specific group
                (*counter) += 1;
                $id_struct {
                    id: Id::Arbitrary(*counter),
                }
            }
        }

        impl std::fmt::Debug for $id_struct {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self.id {
                    Id::Specific(n) => f.write_str(&format!("{}{}", n, $short_name)),
                    Id::Arbitrary(n) => f.write_str(&format!("{}?{}", n, $short_name)),
                }
            }
        }
    };
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Group {
    pub id: Id,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Color {
    pub id: Id,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub id: Id,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub id: Id,
}

id_default_methods!(Group, "g");
id_default_methods!(Color, "c");
id_default_methods!(Block, "b");
id_default_methods!(Item, "i");

impl Value {
    #[allow(clippy::single_match)]
    pub fn member(
        &self,
        member: LocalIntern<String>,
        context: &Context,
        globals: &mut Globals,
        info: CompilerInfo,
    ) -> Option<StoredValue> {
        if member == globals.TYPE_MEMBER_NAME {
            return Some(match self {
                Value::Dict(dict) => match dict.get(&globals.TYPE_MEMBER_NAME) {
                    Some(value) => *value,
                    None => store_const_value(
                        Value::TypeIndicator(self.to_num(globals)),
                        globals,
                        context.start_group,
                        info.position,
                    ),
                },

                _ => store_const_value(
                    Value::TypeIndicator(self.to_num(globals)),
                    globals,
                    context.start_group,
                    info.position,
                ),
            });
        } else {
            match self {
                Value::Str(a) => {
                    if member.as_ref() == "length" {
                        return Some(store_const_value(
                            Value::Number(a.chars().count() as f64),
                            globals,
                            context.start_group,
                            info.position,
                        ));
                    }
                }
                Value::Array(a) => {
                    if member.as_ref() == "length" {
                        return Some(store_const_value(
                            Value::Number(a.len() as f64),
                            globals,
                            context.start_group,
                            info.position,
                        ));
                    }
                }
                Value::Range(start, end, step) => match member.as_ref().as_str() {
                    "start" => {
                        return Some(store_const_value(
                            Value::Number(*start as f64),
                            globals,
                            context.start_group,
                            info.position,
                        ))
                    }
                    "end" => {
                        return Some(store_const_value(
                            Value::Number(*end as f64),
                            globals,
                            context.start_group,
                            info.position,
                        ))
                    }
                    "step_size" => {
                        return Some(store_const_value(
                            Value::Number(*step as f64),
                            globals,
                            context.start_group,
                            info.position,
                        ))
                    }
                    _ => (),
                },
                Value::Macro(m) => match member.as_ref().as_str() {
                    "args" => {
                        let mut args = vec![];
                        for MacroArgDef {
                            name,
                            default,
                            pattern,
                            ..
                        } in &m.args
                        {
                            let mut dict_map = FnvHashMap::default();
                            dict_map.insert(
                                LocalIntern::new(String::from("name")),
                                store_const_value(
                                    Value::Str(name.to_string()),
                                    globals,
                                    context.start_group,
                                    info.position,
                                ),
                            );
                            if let Some(v) = default {
                                dict_map.insert(LocalIntern::new(String::from("default")), *v);
                            }
                            if let Some(v) = pattern {
                                dict_map.insert(LocalIntern::new(String::from("pattern")), *v);
                            }
                            args.push(store_const_value(
                                Value::Dict(dict_map),
                                globals,
                                context.start_group,
                                info.position,
                            ))
                        }
                        return Some(store_const_value(
                            Value::Array(args),
                            globals,
                            context.start_group,
                            info.position,
                        ));
                    }
                    _ => (),
                },
                _ => (),
            };

            let get_impl = |t: u16, m: LocalIntern<String>| match globals.implementations.get(&t) {
                Some(imp) => imp.get(&m).map(|mem| mem.0),
                None => None,
            };

            match self {
                Value::Builtins => match Builtin::from_str(member.as_str()) {
                    Err(_) => None,
                    Ok(builtin) => Some(store_const_value(
                        Value::BuiltinFunction(builtin),
                        globals,
                        context.start_group,
                        info.position,
                    )),
                },
                Value::Dict(dict) => match dict.get(&member) {
                    Some(value) => Some(*value),
                    None => get_impl(self.to_num(globals), member),
                },
                Value::TriggerFunc(f) => {
                    if member.as_ref() == "start_group" {
                        Some(store_const_value(
                            Value::Group(f.start_group),
                            globals,
                            context.start_group,
                            info.position,
                        ))
                    } else {
                        get_impl(self.to_num(globals), member)
                    }
                }
                _ => get_impl(self.to_num(globals), member),
            }
        }
    }
}

use std::str::FromStr;

macro_rules! typed_argument_check {

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident, $builtin:ident)  ($($arg_name:ident),*)) => {
        let ( $($arg_name),*) = clone_and_get_value($arguments[$arg_index], $globals, $context.start_group, true);
    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident, $builtin:ident) mut ($($arg_name:ident),*)) => {
        let ( $(mut $arg_name),*) = $globals.stored_values[$arguments[$arg_index]].clone();
    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident, $builtin:ident) ($($arg_name:ident),*): $arg_type:ident) => {

        let  ( $($arg_name),*) = match clone_and_get_value($arguments[$arg_index], $globals, $context.start_group, true) {
            Value::$arg_type($($arg_name),*) => ($($arg_name),*),

            a => {
                return Err(RuntimeError::BuiltinError {
                    $builtin,
                    message: format!(
                        "Expected {} for argument {}, found {}",
                        stringify!($arg_type),
                        $arg_index + 1,
                        a.to_str($globals)
                    ),
                    info: $info,
                })
            }
        };
    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident, $builtin:ident) mut ($($arg_name:ident),*): $arg_type:ident) => {
        let  ( $(mut $arg_name),*) = match $globals.stored_values[$arguments[$arg_index]].clone() {
            Value::$arg_type($($arg_name),*) => ($($arg_name),*),

            a => {
                return Err(RuntimeError::BuiltinError {
                    $builtin,
                    message: format!(
                        "Expected {} for argument {}, found {}",
                        stringify!($arg_type),
                        $arg_index + 1,
                        a.to_str($globals)
                    ),
                    info: $info,
                })
            }
        };
    };


}

macro_rules! reassign_variable {

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident) mut ($($arg_name:ident),*)) => {

        $globals.stored_values[$arguments[$arg_index]] = ($($arg_name)*);
        $globals.stored_values.set_mutability($arguments[$arg_index], true);

    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident) mut ($($arg_name:ident),*): $arg_type:ident) => {
        $globals.stored_values[$arguments[$arg_index]] = Value::$arg_type($($arg_name),*);
        $globals.stored_values.set_mutability($arguments[$arg_index], true);


    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident) ($($arg_name:ident),*)) => {};

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident) ($($arg_name:ident),*): $arg_type:ident) => {};


}

macro_rules! builtin_arg_mut_check {
    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident) mut ($($arg_name:ident),*)$(: $arg_type:ident)?) => {
        if !$globals.can_mutate($arguments[$arg_index]) {
            return Err(RuntimeError::MutabilityError {
                info: $info,
                val_def: $globals.get_area($arguments[$arg_index]),
            });
        }
        let fn_context = $globals.get_val_fn_context($arguments[$arg_index], $info.clone())?;
        if fn_context != $context.start_group {
            return Err(RuntimeError::ContextChangeMutateError {
                info: $info,
                val_def: $globals.get_area($arguments[$arg_index]),
                context_changes: $context.fn_context_change_stack.clone(),
            });
        }
    };
    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident) ($($arg_name:ident),*)$(: $arg_type:ident)?) => {};
}

macro_rules! raw_check {
    ($globals:ident, $context:ident, $info:ident, $out:ident) => {
        (*$context).return_value =
            store_const_value($out, $globals, $context.start_group, $info.position);
    };

    (#RAW $globals:ident, $context:ident, $info:ident, $out:ident) => {};
}

macro_rules! builtins {

    {
        ($arguments:ident, $info:ident, $globals:ident, $context:ident, $full_context:ident, $builtin:ident)
        $(
            [$variant:ident]
            #[
                safe = $safe:expr,
                desc = $desc:expr,
                example = $example:expr$(,)?
            ]

            $([[$raw:ident]])? fn $name:ident(
                $(#[$argdesc:literal])?
                $(
                    $(
                        $($mut:ident)? ($($arg_name:ident),*)$(: $arg_type:ident)?
                    ),+
                )?
            ) $body:block
        )*
    } => {

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

        pub const BUILTIN_NAMES: &[&str] = &[
            $(
                stringify!($name),
            )*
        ];

        pub const BUILTIN_EXAMPLES: &[(&str, &str)] = &[
            $(
                (stringify!($name), $example),
            )*
        ];

        #[derive(Debug, Clone)]
        pub struct BuiltinPermissions (FnvHashMap<Builtin, bool>);

        impl BuiltinPermissions {
            pub fn new() -> Self {
                let mut map = FnvHashMap::default();
                $(
                    map.insert(Builtin::$variant, $safe);
                )*
                Self(map)
            }
            pub fn is_allowed(&self, b: Builtin) -> bool {
                self.0[&b]
            }
            pub fn set(&mut self, b: Builtin, setting: bool) {
                self.0.insert(b, setting);
            }
            pub fn is_safe(&self, b: Builtin) -> bool {
                match b {
                    $(
                        Builtin::$variant => $safe,
                    )*
                }
            }
        }
        impl Default for BuiltinPermissions {
            fn default() -> Self {
                Self::new()
            }
        }
        #[inline]
        pub fn built_in_function(
            func: Builtin,
            $arguments: Vec<StoredValue>,
            $info: CompilerInfo,
            $globals: &mut Globals,
            contexts: &mut FullContext,
        ) -> Result<(), RuntimeError> {
            #![allow(unused_variables)]
            #![allow(unused_mut)]
            #![allow(unused_parens)]
            if !$globals.permissions.is_allowed(func) {
                if !$globals.permissions.is_safe(func) {
                    return Err(RuntimeError::BuiltinError {
                        builtin: String::from(func),
                        message: format!("This built-in function requires an explicit `--allow {}` flag when running the script", String::from(func)),
                        $info,
                    })
                } else {
                    return Err(RuntimeError::BuiltinError {
                        builtin: String::from(func),
                        message: String::from("This built-in function was denied permission to run"),
                        $info,
                    })
                }
            }
            for full_context in contexts.iter() {
                let $full_context: *mut FullContext = full_context;
                let $context = full_context.inner();
                match func {
                    $(
                        Builtin::$variant => {

                            let $builtin = stringify!($name).to_string();

                            $(
                                #[allow(unused_assignments)]
                                let mut arg_index = 0;
                                $(
                                    if arg_index >= $arguments.len() {
                                        return Err(RuntimeError::BuiltinError {
                                            builtin: stringify!($name).to_string(),
                                            message: String::from(
                                                "Too few arguments provided",
                                            ),
                                            $info,
                                        })
                                    }

                                    builtin_arg_mut_check!(
                                        ($globals, arg_index, $arguments, $info, $context) $($mut)?
                                        ($($arg_name),*)$(: $arg_type)?
                                    );
                                    typed_argument_check!(
                                        ($globals, arg_index, $arguments, $info, $context, $builtin) $($mut)?
                                        ($($arg_name),*)$(: $arg_type)?
                                    );

                                    arg_index += 1;
                                )+
                                if arg_index < $arguments.len() - 1 {
                                    return Err(RuntimeError::BuiltinError {
                                        builtin: stringify!($name).to_string(),
                                        message: String::from(
                                            "Too many arguments provided",
                                        ),
                                        $info,
                                    })
                                }
                            )?

                            let _out = $body;

                            $(

                                arg_index = 0;
                                $(


                                    reassign_variable!(
                                        ($globals, arg_index, $arguments, $info) $($mut)? ($($arg_name),*)$(: $arg_type)?
                                    );

                                    arg_index += 1;
                                )+
                            )?

                            raw_check!($(#$raw)? $globals, $context, $info, _out);



                        }
                    )+
                }
            }
            Ok(())
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

        pub fn builtin_docs() -> String {
            let all = [
                $(
                    (stringify!($name),
                    concat!(
                        concat!(
                        "## Description:\n", $desc, "<div>\n",
                        ),
                        concat!("## Example:\n```spwn\n", $example, "\n```\n"),

                        concat!("**Allowed by default:** ", $safe, "\n"),
                        concat!(
                            "## Arguments: \n",
                            $(
                                "**", $argdesc, "**\n"
                            )?
                        ),
                        $(
                            concat!("| **Name** | **Type** |\n|-|-|\n",
                                $(
                                    concat!("| ", stringify!($($arg_name),*), " | ", concat!($(concat!("_", stringify!($mut), "able_ "),)? ""), concat!($(concat!("_", stringify!($arg_type), "_"),)? "") , " |\n"),
                                )+
                            ),
                        )?
                    )),
                )*
            ];
            let mut out = String::new();

            let mut operators = Vec::new();
            let mut normal_ones = Vec::new();

            for (name, doc) in all {
                let header = format!("## $.{}\n", name);
                let mut new_doc = String::new();
                for line in doc.lines() {
                    new_doc += &format!("> {}\n", line);
                }
                new_doc = header + &new_doc;
                if name.starts_with("_") && name.ends_with("_") {
                    operators.push((name, new_doc));
                } else {
                    normal_ones.push((name, new_doc));
                }
            }

            normal_ones.sort_by(|a, b| a.0.cmp(&b.0));
            operators.sort_by(|a, b| a.0.cmp(&b.0));

            out += "# List of Built-in functions\n";

            for (_, doc) in normal_ones.iter() {
                out += doc;
            }

            out += "# Default Implementations for Operators\n";

            for (_, doc) in operators.iter() {
                out += doc;
            }


            out
        }


    };
}

builtins! {
    (arguments, info, globals, context, full_context, builtin)

    [Assert] #[safe = true, desc = "Throws an error if the argument is not `true`", example = "$.assert(true)"]
    fn assert((b): Bool) {
        if !b {
            return Err(RuntimeError::BuiltinError {
                builtin,
                message: String::from("Assertion failed"),
                info,
            });
        } else {
            Value::Null
        }
    }

    [Print] #[safe = true, desc = "Prints value(s) to the console", example = "$.print(\"Hello world!\")"]
    fn print(#["any"]) {
        let mut out = String::new();
        for val in arguments.iter() {
            match &globals.stored_values[*val] {
                Value::Str(s) => out += s,
                _ => out += &{
                    let ctx = unsafe { FullContext::from_ptr(full_context) };
                    handle_unary_operator(*val, Builtin::DisplayOp, ctx, globals, &info)?;
                    let out = ctx.inner().return_value;
                    let val = &globals.stored_values[out];
                    if let Value::Str(s) = val {
                        s.clone()
                    } else {
                        val.clone().to_str(globals)
                    }
                }
            };

        }
        writeln!(globals.std_out, "{}", out).expect("Error writing to output");
        Value::Null
    }

    [Time] #[safe = true, desc = "Gets the current system time in seconds", example = "now = $.time()"]
    fn time(#["none"]) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            arg_length!(info, 0, arguments, "Expected no arguments".to_string(), builtin);
            use std::time::SystemTime;
            let now = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(time) => time,
                Err(e) => {
                    return Err(RuntimeError::BuiltinError {
                        builtin,
                        message: format!("System time error: {}", e),
                        info,
                    })
                }
            }
            .as_secs_f64();
            Value::Number(now)
        }

        #[cfg(target_arch = "wasm32")]
        Value::Number(0.0)
    }

    [SpwnVersion] #[safe = true, desc = "Gets the current version of spwn", example = "$.spwn_version()"]
    fn spwn_version(#["none"]) {
        arg_length!(info, 0, arguments, "Expected no arguments".to_string(), builtin);

        Value::Str(env!("CARGO_PKG_VERSION").to_string())
    }

    [GetInput] #[safe = true, desc = "Gets some input from the user", example = "// inp = $.get_input('What is your name?')"]
    fn get_input((prompt): Str) {
        print!("{}", prompt);
        stdout()
            .flush()
            .expect("Unexpected error occurred when trying to get user input");
        Value::Str(text_io::read!("{}\n"))
    }



    [B64Encode] #[safe = true, desc = "Returns the input string encoded with base64 encoding (useful for text objects)", example = "$.b64encode(\"hello there\")"]
    fn b64encode((s): Str) {
        let encrypted = base64::encode(s.as_bytes());
        Value::Str(encrypted)
    }

    [B64Decode] #[safe = true, desc = "Returns the input string decoded from base64 encoding (useful for text objects)", example = "$.b64decode(\"aGVsbG8gdGhlcmU=\")"]
    fn b64decode((s): Str) {
        let decrypted = match base64::decode(&s) {
            Ok(s) => s,
            Err(e) => {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: format!("Base 64 error: {}", e),
                    info,
                })
            }
        };
        Value::Str(String::from_utf8_lossy(&decrypted).to_string())
    }



    [HTTPRequest] #[safe = false, desc = "Sends an HTTP request", example = ""] fn http_request((method): Str, (url): Str, (headers): Dict, (body): Str) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut headermap = reqwest::header::HeaderMap::new();
            for (name, value) in &headers {
                let header_name = match reqwest::header::HeaderName::from_bytes(name.as_bytes()) {
                    Ok(hname) => hname,
                    Err(_) => {
                        return Err(RuntimeError::BuiltinError {
                            builtin,
                            message: format!("Could not convert header name: '{}'", name),
                            info
                        })
                    }
                };
                let header_value = globals.stored_values[*value].clone().to_str(globals);
                headermap.insert(header_name, header_value.trim_matches('\'').parse().unwrap());
            }

            let client = reqwest::blocking::Client::new();
            let request_maker = match &method[..] {
                "get" => client.get(&url),
                "post" => client.post(&url),
                "put" => {
                    client.put(&url)
                },
                "patch" => client.patch(&url),
                "delete" => client.delete(&url),
                "head" => client.head(&url),
                _ => {
                    return Err(RuntimeError::BuiltinError {
                        builtin,
                        message: format!("Request type not supported: '{}'", method),
                        info
                    })
                }
            };

            let response = match request_maker
                .headers(headermap)
                .body(body)
                .send() {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Err(RuntimeError::BuiltinError {
                            builtin,
                            message: format!("Could not make request to: '{}'", url),
                            info
                        })
                    }
            };

            let mut output_map = FnvHashMap::default();

            let response_status = store_const_value(
                Value::Number(
                    response.status().as_u16() as f64
                ),
                globals,
                context.start_group,
                CodeArea::new(),
            );

            let response_headermap = response.headers();
            let mut response_headers_value = FnvHashMap::default();
            for (name, value) in response_headermap.iter() {
                let header_value = store_const_value(
                    Value::Str(String::from(value.to_str().expect("Couldn't parse return header value"))),
                    globals,
                    context.start_group,
                    CodeArea::new()
                );
                response_headers_value.insert(LocalIntern::new(String::from(name.as_str())), header_value);
            }

            let response_headers = store_const_value(
                Value::Dict(response_headers_value),
                globals,
                context.start_group,
                CodeArea::new()
            );

            let response_text = store_const_value(
                Value::Str(
                    response.text().expect("Failed to parse response text")
                ),
                globals,
                context.start_group,
                CodeArea::new(),
            );

            output_map.insert(LocalIntern::new(String::from("status")), response_status);
            output_map.insert(LocalIntern::new(String::from("headers")), response_headers);
            output_map.insert(LocalIntern::new(String::from("text")), response_text);
            Value::Dict(output_map)
        }
        #[cfg(target_arch = "wasm32")]
        {
            return Err(RuntimeError::BuiltinError {
                message: "http is not supported on web versions".to_string(),
                builtin: "http_request".to_string(),
                info
            })
        }
    }

    [Hash] #[safe = true, desc = "Calculates the numerical hash of a value", example = "$.hash(\"hello\")"] fn hash((n)) { Value::Number( {
        let mut s = DefaultHasher::new();
        n.hash(&mut s, globals);
        s.finish() / 1000
    } as f64 ) }

    [Sin] #[safe = true, desc = "Calculates the sin of an angle in radians", example = "$.sin(3.1415)"] fn sin((n): Number) { Value::Number(n.sin()) }
    [Cos] #[safe = true, desc = "Calculates the cos of an angle in radians", example = "$.cos(3.1415)"] fn cos((n): Number) { Value::Number(n.cos()) }
    [Tan] #[safe = true, desc = "Calculates the tan of an angle in radians", example = "$.tan(3.1415)"] fn tan((n): Number) { Value::Number(n.tan()) }

    [Asin] #[safe = true, desc = "Calculates the arcsin of a number", example = "$.asin(1)"] fn asin((n): Number) { Value::Number(n.asin()) }
    [Acos] #[safe = true, desc = "Calculates the arccos of a number", example = "$.acos(-1)"] fn acos((n): Number) { Value::Number(n.acos()) }
    [Atan] #[safe = true, desc = "Calculates the arctan of a number", example = "$.atan(1)"] fn atan((n): Number) { Value::Number(n.atan()) }

    [Floor] #[safe = true, desc = "Calculates the floor of a number, AKA the number rounded down to the nearest integer", example = "$.assert($.floor(1.5) == 1)"] fn floor((n): Number) { Value::Number(n.floor()) }
    [Ceil] #[safe = true, desc = "Calculates the ceil of a number, AKA the number rounded up to the nearest integer", example = "$.assert($.ceil(1.5) == 2)"] fn ceil((n): Number) { Value::Number(n.ceil()) }

    [Abs] #[safe = true, desc = "Calculates the absolute value of a number", example = "$.assert($.abs(-100) == 100)"] fn abs((n): Number) {Value::Number(n.abs())}
    [Acosh] #[safe = true, desc = "Calculates the arccosh of a number", example = "$.acosh(1)"] fn acosh((n): Number) {Value::Number(n.acosh())}
    [Asinh] #[safe = true, desc = "Calculates the arcsinh of a number", example = "$.asinh(0)"] fn asinh((n): Number) {Value::Number(n.asinh())}
    [Atan2] #[safe = true, desc = "Calculates the arctan^2 of a number", example = "$.atan2(0, -1)"] fn atan2((x): Number, (y): Number) {Value::Number(x.atan2(y))}
    [Atanh] #[safe = true, desc = "Calculates the arctanh of a number", example = "$.atanh(0.996)"] fn atanh((n): Number) {Value::Number(n.atanh())}
    [Cbrt] #[safe = true, desc = "Calculates the cube root of a number", example = "$.cbrt(27)"] fn cbrt((n): Number) {Value::Number(n.cbrt())}
    [Cosh] #[safe = true, desc = "Calculates the cosh of a number", example = "$.cosh(0)"] fn cosh((n): Number) {Value::Number(n.cosh())}
    [Exp] #[safe = true, desc = "Calculates the e^x of a number", example = "$.exp(5) // e^5"] fn exp((n): Number) {Value::Number(n.exp())}
    [Exp2] #[safe = true, desc = "Calculates the 2^x of a number", example = "$.assert($.exp2(10) == 1024)"] fn exp2((n): Number) {Value::Number(n.exp2())}
    [Expm1] #[safe = true, desc = "Calculates e^x - 1 in a way that is accurate even if the number is close to zero", example = "$.exp_m1(0.002)"] fn exp_m1((n): Number) {Value::Number(n.exp_m1())}
    [Fract] #[safe = true, desc = "Gets the fractional part of a number", example = "$.fract(1.23)"] fn fract((n): Number) {Value::Number(n.fract())}

    [Sqrt] #[safe = true, desc = "Calculates the square root of a number", example = "$.sqrt(2)"] fn sqrt((n): Number) {Value::Number(n.sqrt())}
    [Sinh] #[safe = true, desc = "Calculates the hyperbolic sin of a number", example = "$.sinh(0)"] fn sinh((n): Number) {Value::Number(n.sinh())}
    [Tanh] #[safe = true, desc = "Calculates the hyperbolic tan of a number", example = "$.tanh(0.549)"] fn tanh((n): Number) {Value::Number(n.tanh())}
    [NaturalLog] #[safe = true, desc = "Calculates the ln (natural log) of a number", example = "$.ln(2.71828)"] fn ln((n): Number) {Value::Number(n.ln())}
    [Log] #[safe = true, desc = "Calculates the log base x of a number", example = "$.assert($.log(1024, 2) == 10)"] fn log((n): Number, (base): Number) {Value::Number(n.log(base))}
    [Min] #[safe = true, desc = "Calculates the min of two numbers", example = "$.assert($.min(1, 2) == 1)"] fn min((a): Number, (b): Number) {Value::Number(a.min(b))}
    [Max] #[safe = true, desc = "Calculates the max of two numbers", example = "$.assert($.max(1, 2) == 2)"] fn max((a): Number, (b): Number) {Value::Number(a.max(b))}
    [Round] #[safe = true, desc = "Rounds a number", example = "$.assert($.round(1.2) == 1)"] fn round((n): Number) {Value::Number(n.round())}
    [Hypot] #[safe = true, desc = "Calculates the hypothenuse in a right triangle with sides a and b", example = "$.assert($.hypot(3, 4) == 5) // because 3^2 + 4^2 = 5^2"] fn hypot((a): Number, (b): Number) {Value::Number(a.hypot(b))}

    [Add] #[safe = true, desc = "Adds a Geometry Dash object or trigger to the target level", example = "
extract obj_props
$.add(obj {
    OBJ_ID: 1,
    X: 45,
    Y: 45,
})
    "]
    fn add(#["The object or trigger to add"]) {
        if arguments.is_empty() || arguments.len() > 2 {
            return Err(RuntimeError::BuiltinError {
                builtin,
                message: "Expected 1 argument".to_string(),
                info,
            });
        }
        let (obj, mode) = match globals.stored_values[arguments[0]].clone() {
            Value::Obj(obj, mode) => (obj, mode),
            _ => return Err(RuntimeError::TypeError {
                expected: "@object or @trigger".to_string(),
                found: globals.get_type_str(arguments[0]),
                val_def: globals.get_area(arguments[0]),
                info,
            })
        };

        if obj.is_empty() {
            return Ok(());
        }

        let mut ignore_context = false;
        if arguments.len() == 2 {
            match globals.stored_values[arguments[1]].clone() {
                Value::Bool(b) => ignore_context = b,
                _ => return Err(RuntimeError::TypeError {
                    expected: "boolean".to_string(),
                    found: globals.get_type_str(arguments[1]),
                    val_def: globals.get_area(arguments[1]),
                    info,
                })
            };
        }

        let mut obj_map = FnvHashMap::<u16, ObjParam>::default();

        for p in obj {
            obj_map.insert(p.0, p.1.clone());
            // add params into map
        }

        match mode {
            ObjectMode::Object => {
                if !ignore_context && context.start_group.id != Id::Specific(0) {
                    return Err(RuntimeError::BuiltinError {
                        builtin, // objects cant be added dynamically, of course
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
                (*globals).trigger_order += 1.0;
                (*globals).func_ids[context.func_id]
                    .obj_list
                    .push((obj, crate::compiler_types::TriggerOrder(globals.trigger_order)))
            }
        };
        Value::Null
    }

    [Append] #[safe = true, desc = "Appends a value to the end of an array. You can also use `array.push(value)`", example = "
let arr = []
$.append(arr, 1)
$.assert(arr == [1])
    "]
    fn append(mut (arr): Array, (val)) {
        //set lifetime to the lifetime of the array

        let cloned = clone_value(
            arguments[1],
            globals,
            context.start_group,
            !globals.is_mutable(arguments[1]),
            globals.get_area(arguments[1])
        );

        (arr).push(cloned);

        Value::Null
    }

    [SplitStr] #[safe = true, desc = "Returns an array from the split string. You can also use `string.split(delimiter)`", example = "$.assert($.split_str(\"1,2,3\", \",\") == [\"1\", \"2\", \"3\"])"]
    fn split_str((s): Str, (substr): Str) {

        let mut output = Vec::<StoredValue>::new();

        for split in s.split(&*substr) {
            let entry =
                store_const_value(Value::Str(split.to_string()), globals, context.start_group, CodeArea::new());
            output.push(entry);
        }

        Value::Array(output)
    }

    [EditObj] #[safe = true, desc = "Changes the value of an object key. You can also use `object.set(key, value)`", example = "
extract obj_props
let object = color_trigger(BG, 0, 0, 0, 0.5)
$.edit_obj(object, X, 600)
    "]
    fn edit_obj(mut (o, m): Obj, (key), (value)) {

        let (okey, oval) = {
            let (key, pattern) = match key {
                Value::Number(n) => (n as u16, None),

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
                            found: globals.get_type_str(arguments[1]),
                            val_def: globals.get_area(arguments[1]),
                            info,
                        })
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
                            Value::Number(n) => *n as u16,
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    expected: "number".to_string(),
                                    found: globals.get_type_str(*id.unwrap()),
                                    val_def: globals.get_area(*id.unwrap()),
                                    info,
                                })
                            }
                        },
                        Some(globals.stored_values[*pattern.unwrap()].clone()),
                    )
                }
                a => {
                    return Err(RuntimeError::TypeError {
                        expected: "number or @object_key".to_string(),
                        found: a.get_type_str(globals),
                        val_def: globals.get_area(arguments[1]),
                        info,
                    })
                }
            };

            if m == ObjectMode::Trigger && (key == 57 || key == 62) {
                // group ids and stuff on triggers
                return Err(RuntimeError::CustomError(create_error(
                    info,
                    "You are not allowed to set the group ID(s) or the spawn triggered state of a @trigger. Use obj instead",
                    &[],
                    None,
                )))
            }

            if let Some(ref pat) = pattern {
                if !value.pure_matches_pat(pat, &info, globals, context.clone())? {
                    return Err(RuntimeError::TypeError {
                        expected: pat.to_str(globals),
                        found: value.get_type_str(globals),
                        val_def: globals.get_area(arguments[2]),
                        info,
                    });
                }
            }
            let err = Err(RuntimeError::CustomError(create_error(
                info.clone(),
                &format!(
                    "{} is not a valid object value",
                    value.to_str(globals)
                ),
                &[],
                None,
            )));

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
                            _ => return Err(RuntimeError::CustomError(create_error(
                                info,
                                "Arrays in object parameters can only contain groups",
                                &[],
                                None,
                            )))
                        })
                        }

                        out
                    })
                }
                obj @ Value::Dict(_) => {
                    let typ = obj.member(globals.TYPE_MEMBER_NAME, context, globals, info.clone()).unwrap();
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

        let mut has_key = false;
        for (i, (k, _)) in o.iter().enumerate() {
            if *k == okey {
                has_key = true;
                o[i].1 = oval.clone();
                break;
            }
        }
        if !has_key {
            o.push((okey, oval));
        }

        Value::Null
    }

    [Mutability] #[safe = true, desc = "Checks if a value reference is mutable", example = "
const = 1
$.assert(!$.mutability(const))
let mut = 1
$.assert($.mutability(mut))
    "]
    fn mutability((var)) {
        Value::Bool(globals.can_mutate(arguments[0]))
    }


    [ExtendTriggerFunc] #[safe = true, desc = "Executes a macro in a specific trigger function context", example = "
$.extend_trigger_func(10g, () {
    11g.move(10, 0, 0.5) // will add a move trigger in group 10
})
    "]
    fn extend_trigger_func((group),(mac): Macro) {
        let group = match group {
            Value::Group(g) => g,
            Value::TriggerFunc(f) => f.start_group,
            a => {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: format!(
                        "Expected group or trigger function, found {}",
                        a.to_str(globals)
                    ),
                    info,
                })
            }
        };

        // if context.start_group.id != crate::builtins::Id::Specific(0) {
        //     return Err(RuntimeError::BuiltinError {
        //         builtin, // objects cant be added dynamically, of course
        //         message: String::from(
        //             "you cannot extend trigger funcs at runtime"),
        //         info
        //     });
        // }
        use parser::ast::*;

        let cmp_statement = CompoundStatement { statements: vec![
            Statement {
                body: StatementBody::Expr(Variable {
                    operator: None,
                    path: vec![Path::Call(Vec::new())],
                    value: ValueLiteral { body: ValueBody::Resolved(arguments[1]) },
                    pos: info.position.pos,
                    tag: Attribute { tags: Vec::new() }
                }.to_expression()),
                arrow: false,
                pos: info.position.pos
            }
        ]};

        cmp_statement.to_trigger_func(unsafe { FullContext::from_ptr(full_context) }, globals, info.clone(), Some(group))?;



        Value::Null
    }

    [TriggerFnContext] #[safe = true, desc = "Returns the start group of the current trigger function context", example = "$.trigger_fn_context()"]
    fn trigger_fn_context(#["none"]) {
        Value::Group(context.start_group)
    }

    [Random] #[safe = true, desc = "Generates random numbers, or picks a random element of an array", example = "
$.random() // a completely random number
$.random([1, 2, 3, 6]) // returns either 1, 2, 3, or 6
$.random(1..11) // returns a random integer between 1 and 10
    "]
    fn random(#["see example"]) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use rand::seq::SliceRandom;
            use rand::Rng;
            if arguments.len() > 2 {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: "Expected up to 2 arguments".to_string(),
                    info,
                });
            }

            if arguments.is_empty() {
                Value::Number(rand::thread_rng().gen())
            } else {
                let val = match convert_type(&globals.stored_values[arguments[0]].clone(), type_id!(array), &info, globals, context) {
                    Ok(Value::Array(v)) => v,
                    _ => {
                        return Err(RuntimeError::BuiltinError {
                            builtin,
                            message: format!("Expected type that can be converted to @array for argument 1, found type {}", globals.get_type_str(arguments[0])),
                            info,
                        });
                    }
                };

                if arguments.len() == 1 {
                    let rand_elem = val.choose(&mut rand::thread_rng());

                    if rand_elem.is_some() {
                        clone_and_get_value(
                            *rand_elem.unwrap(),
                            globals,
                            context.start_group,
                            !globals.is_mutable(*rand_elem.unwrap())
                        )
                    } else {
                        Value::Null
                    }
                } else {
                    let times = match &globals.stored_values[arguments[1]] {
                        Value::Number(n) => {
                            convert_to_int(*n, &info)?
                        },
                        _ => {
                            return Err(RuntimeError::BuiltinError {
                                builtin,
                                message: format!("Expected number, found {}", globals.get_type_str(arguments[1])),
                                info,
                            });
                        }
                    };

                    let mut out_arr = Vec::<StoredValue>::new();

                    for _ in 0..times {
                        let rand_elem = val.choose(&mut rand::thread_rng());

                        if rand_elem.is_some() {
                            out_arr.push(clone_value(
                                *rand_elem.unwrap(),
                                globals,
                                context.start_group,
                                !globals.is_mutable(*rand_elem.unwrap()),
                                CodeArea::new()
                            ));
                        } else {
                            break;
                        }
                    }

                    Value::Array(out_arr)
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            return Err(RuntimeError::BuiltinError {
                message: "rng is not supported on web versions".to_string(),
                builtin: "random".to_string(),
                info
            })
        }
    }

    [ReadLevel] #[safe = true, desc = "Returns the level string of the level being written to, or nothing if there is no output level", example = "level_string = $.level_string()"]
    fn level_string() {
        if !arguments.is_empty() {
            return Err(RuntimeError::BuiltinError {
                builtin,
                message: String::from("Expected 0 arguments"),
                info,
            });
        }
        Value::Str(globals.initial_string.clone())
    }
    [ParseLevel] #[safe = true, desc = "Returns a array of the objects in the level being written to, or an empty array if there is no output level", example = "level = $.level_objects()"]
    [[RAW]] fn level_objects() {
        if !arguments.is_empty() {
            return Err(RuntimeError::BuiltinError {
                builtin,
                message: String::from("Expected 0 arguments"),
                info,
            });
        }
        (*context).return_value = match globals.initial_objects {
            Some(obj) => obj,
            None => {
                let stored = store_const_value(
                    Value::Array(
                        crate::parse_levelstring::parse_levelstring(&globals.initial_string)?.into_iter()
                        .map(|v|
                            store_const_value(v, globals, context.start_group, CodeArea::new())
                        ).collect()
                    ),
                    globals,
                    context.start_group,
                    CodeArea::new(),
                );
                globals.initial_objects = Some(stored);
                stored
            }
        };
    }

    [CWD] #[safe = true, desc = "Returns the current working directory", example = "$.cwd() // \"C:/spwn/\""] fn cwd() {
        Value::Str(env::current_dir().unwrap().to_str().unwrap().to_string())
    }

    [DirName] #[safe = true, desc = "Returns the directory of the current spwn file", example = "$.dirname() // \"C:/spwn/\""] fn dirname() {
        let mut path = match info.position.file.as_ref().clone() {
            SpwnSource::File(f) => f,
            SpwnSource::BuiltIn(b) => b,
            SpwnSource::String(s) => PathBuf::from(s.as_ref().clone()),
        };
        path.pop(); // remove the basename
        Value::Str(path.to_str().unwrap().to_string())
    }

    [ReadFile] #[safe = false, desc = "Returns the contents of a file in the local file system (uses the current directory as base for relative paths)", example = "data = $.readfile(\"file.txt\")"]
    fn readfile(#["Path of file to read, and the format it's in (\"text\", \"bin\", \"json\", \"toml\" or \"yaml\")"]) {
        if arguments.is_empty() || arguments.len() > 2 {
            return Err(RuntimeError::BuiltinError {
                builtin,
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
                                builtin,
                                message:
                                    "Data format needs to be a string (\"text\", \"bin\", \"json\", \"toml\" or \"yaml\")"
                                        .to_string(),
                                info,
                            });
                        }
                    }
                    _ => "text",
                };
                let path = std::path::PathBuf::from(p);

                if !path.exists() {
                    return Err(RuntimeError::BuiltinError {
                        builtin,
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
                                    builtin,
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
                                    builtin,
                                    message: format!("Problem opening the file: {}", e),
                                    info,
                                });
                            }
                        };
                        Value::Array(
                            rval.iter()
                                .map(|b| {
                                    store_const_value(Value::Number(*b as f64), globals, context.start_group, CodeArea::new())
                                })
                                .collect(),
                        )
                    }
                    "json" => {
                        let ret = fs::read_to_string(path);
                        let rval = match ret {
                            Ok(file) => file,
                            Err(e) => {
                                return Err(RuntimeError::BuiltinError {
                                    builtin,
                                    message: format!("Problem opening the file: {}", e),
                                    info,
                                });
                            }
                        };
                        let parsed = match serde_json::from_str(&rval) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(RuntimeError::BuiltinError {
                                    builtin,
                                    message: format!("Problem parsing JSON: {}", e),
                                    info,
                                });
                            }
                        };
                        fn parse_json_value(val: serde_json::Value, globals: &mut Globals, context: &Context, info: &CompilerInfo) -> Value {
                            // please sput forgive me for this shitcode ._.
                            match val {
                                serde_json::Value::Null => Value::Null,
                                serde_json::Value::Bool(x) => Value::Bool(x),
                                serde_json::Value::Number(x) => Value::Number(x.as_f64().unwrap()),
                                serde_json::Value::String(x) => Value::Str(x),
                                serde_json::Value::Array(x) => {
                                    let mut arr: Vec<StoredValue> = vec![];
                                    for v in x {
                                        arr.push(store_const_value(parse_json_value(v, globals, context, info), globals, context.start_group, info.position));
                                    }
                                    Value::Array(arr)
                                },
                                serde_json::Value::Object(x) => {
                                    let mut dict: FnvHashMap<LocalIntern<String>, StoredValue> = FnvHashMap::default();
                                    for (key, value) in x {
                                        dict.insert(LocalIntern::new(key), store_const_value(parse_json_value(value, globals, context, info), globals, context.start_group, info.position));
                                    }
                                    Value::Dict(dict)
                                },
                            }
                        }
                        parse_json_value(parsed, globals, context, &info)
                    }
                    "toml" => {
                        let ret = fs::read_to_string(path);
                        let rval = match ret {
                            Ok(file) => file,
                            Err(e) => {
                                return Err(RuntimeError::BuiltinError {
                                    builtin,
                                    message: format!("Problem opening the file: {}", e),
                                    info,
                                });
                            }
                        };
                        let parsed = match toml::from_str(&rval) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(RuntimeError::BuiltinError {
                                    builtin,
                                    message: format!("Problem parsing toml: {}", e),
                                    info,
                                });
                            }
                        };
                        fn parse_toml_value(val: toml::Value, globals: &mut Globals, context: &Context, info: &CompilerInfo) -> Value {
                            // please sput forgive me for this shitcode ._.
                            match val {
                                toml::Value::Boolean(x) => Value::Bool(x),
                                toml::Value::Integer(x) => Value::Number(x as f64),
                                toml::Value::Float(x) => Value::Number(x),
                                toml::Value::String(x) => Value::Str(x),
                                toml::Value::Datetime(x) => Value::Str(x.to_string()),
                                toml::Value::Array(x) => {
                                    let mut arr: Vec<StoredValue> = vec![];
                                    for v in x {
                                        arr.push(store_const_value(parse_toml_value(v, globals, context, info), globals, context.start_group, info.position));
                                    }
                                    Value::Array(arr)
                                },
                                toml::Value::Table(x) => {
                                    let mut dict: FnvHashMap<LocalIntern<String>, StoredValue> = FnvHashMap::default();
                                    for (key, value) in x {
                                        dict.insert(LocalIntern::new(key), store_const_value(parse_toml_value(value, globals, context, info), globals, context.start_group, info.position));
                                    }
                                    Value::Dict(dict)
                                },
                            }
                        }
                        parse_toml_value(parsed, globals, context, &info)
                    }
                    "yaml" => {
                        let ret = fs::read_to_string(path);
                        let rval = match ret {
                            Ok(file) => file,
                            Err(e) => {
                                return Err(RuntimeError::BuiltinError {
                                    builtin,
                                    message: format!("Problem opening the file: {}", e),
                                    info,
                                });
                            }
                        };
                        let parsed: serde_yaml::Value = match serde_yaml::from_str(&rval) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(RuntimeError::BuiltinError {
                                    builtin,
                                    message: format!("Problem parsing toml: {}", e),
                                    info,
                                });
                            }
                        };
                        fn parse_yaml_value(val: &serde_yaml::Value, globals: &mut Globals, context: &Context, info: &CompilerInfo) -> Value {
                            // please sput forgive me for this shitcode ._.
                            match val {
                                serde_yaml::Value::Null => Value::Null,
                                serde_yaml::Value::Bool(x) => Value::Bool(*x),
                                serde_yaml::Value::Number(x) => Value::Number(x.as_f64().unwrap()),
                                serde_yaml::Value::String(x) => Value::Str(x.to_string()),
                                serde_yaml::Value::Sequence(x) => {
                                    let mut arr: Vec<StoredValue> = vec![];
                                    for v in x {
                                        arr.push(store_const_value(parse_yaml_value(v, globals, context, info), globals, context.start_group, info.position));
                                    }
                                    Value::Array(arr)
                                },
                                serde_yaml::Value::Mapping(x) => {
                                    let mut dict: FnvHashMap<LocalIntern<String>, StoredValue> = FnvHashMap::default();
                                    for (key, value) in x.iter() {
                                        dict.insert(LocalIntern::new(key.as_str().unwrap().to_string()), store_const_value(parse_yaml_value(value, globals, context, info), globals, context.start_group, info.position));
                                    }
                                    Value::Dict(dict)
                                },
                            }
                        }
                        parse_yaml_value(&parsed, globals, context, &info)
                    }
                    _ => {
                        return Err(RuntimeError::BuiltinError {
                            builtin,
                            message: "Invalid data format ( use \"text\", \"bin\", \"json\", \"toml\" or \"yaml\" )"
                                .to_string(),
                            info,
                        })
                    }
                }
            }
            _ => {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: "Path needs to be a string".to_string(),
                    info,
                });
            }
        }
    }


    [WriteFile] #[safe = false, desc = "Writes a string to a file in the local file system (any previous content will be overwritten, and a new file will be created if it does not already exist)", example = "$.write_file(\"file.txt\", \"Hello\")"]
    fn writefile((path): Str, (data): Str) {


        match fs::write(path, data) {
            Ok(_) => (),
            Err(e) => {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: format!("Error when writing to file: {}", e),
                    info,
                });
            }
        };
        Value::Null
    }

    [DeleteFile] #[safe = false, desc = "Deletes a file in the local file system", example = "$.deletefile(\"file.txt\")"] fn deletefile((path): Str) {
        match fs::remove_file(path) {
            Ok(_) => (),
            Err(e) => {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: format!("Error when deleting file: {}", e),
                    info,
                });
            }
        };
        Value::Null
    }

    [FileExists] #[safe = false, desc = "Checks if a member exists in the local file system", example = "$.fileexists(\"file.txt\")"] fn fileexists((path): Str) {
        Value::Bool(fs::metadata(path).is_ok())
    }

    [FileKind] #[safe = false, desc = "Returns the kind of a member of the local file system", example = "$.filekind(\"file.txt\")"] fn filekind((path): Str) {
        match fs::metadata(path) {
            Ok(meta) => {
                let kind = match meta.file_type() {
                    _ if meta.is_file() => "file",
                    _ if meta.is_dir() => "dir",
                    _ => "unknown",
                };
                Value::Str(kind.to_string())
            }
            Err(e) => {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: format!("Error when checking file type: {}", e),
                    info,
                });
            },
        }
    }

    [MetaData] #[safe = false, desc = "Returns the metadata of a file or directory in the local file system", example = "$.metadata(\"file.txt\")"] fn metadata((path): Str) {
        match fs::metadata(path) {
            Ok(meta) => {
                let mut dict: FnvHashMap<LocalIntern<String>, StoredValue> = FnvHashMap::default();
                let mut store = |value| store_const_value(value, globals, context.start_group, info.position);
                dict.insert(LocalIntern::new(String::from("size")), store(Value::Number(meta.len() as f64)));
                dict.insert(LocalIntern::new(String::from("modified")), store(Value::Number(meta.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64())));
                dict.insert(LocalIntern::new(String::from("accessed")), store(Value::Number(meta.accessed().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64())));
                dict.insert(LocalIntern::new(String::from("created")), store(Value::Number(meta.created().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64())));
                dict.insert(LocalIntern::new(String::from("readonly")), store(Value::Bool(meta.permissions().readonly())));
                Value::Dict(dict)
            }
            Err(e) => {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: format!("Error when checking for file metadata: {}", e),
                    info,
                });
            },
        }
    }

    [ReadDir] #[safe = false, desc = "Reads the contents of a directory in the local file system", example = "$.readdir(\"/\")"] fn readdir((path): Str) {
        let mut arr: Vec<StoredValue> = vec![];
        for entry in fs::read_dir(path).unwrap() {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    return Err(RuntimeError::BuiltinError {
                        builtin,
                        message: format!("Error when reading directory: {}", e),
                        info,
                    });
                },
            };
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            arr.push(store_const_value(Value::Str(name), globals, context.start_group, info.position));
        }
        Value::Array(arr)
    }

    [MkDir] #[safe = false, desc = "Creates a directory in the local file system", example = "$.mkdir(\"/\")"] fn mkdir((path): Str) {
        match fs::create_dir(path) {
            Ok(_) => (),
            Err(e) => {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: format!("Error when creating directory: {}", e),
                    info,
                });
            },
        };
        Value::Null
    }

    [RmDir] #[safe = false, desc = "Removes an empty directory in the local file system", example = "$.rmdir(\"folder\")"] fn rmdir((path): Str) {
        match fs::remove_dir(path) {
            Ok(_) => (),
            Err(e) => {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: format!("Error when removing directory: {}", e),
                    info,
                });
            },
        };
        Value::Null
    }

    [RmDirAll] #[safe = false, desc = "Removes a directory in the local file system", example = "$.rmdirall(\"folder\")"] fn rmdir_all((path): Str) {
        match fs::remove_dir_all(path) {
            Ok(_) => (),
            Err(e) => {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: format!("Error when removing directory: {}", e),
                    info,
                });
            },
        };
        Value::Null
    }

    [Pop] #[safe = true, desc = "Removes a value from the end of an array, and returns it. You can also use `array.pop()`", example = "
let arr = [1, 2, 3]
$.assert($.pop(arr) == 3)
$.assert(arr == [1, 2])
    "]
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
                    builtin,
                    message: format!("Expected array or string, found @{}", typ),
                    info,
                })
            }
        }
    }

    [Substr] #[safe = true, desc = "Returns a specified part of the input string", example = "$.substr(\"hello there\", 1, 5)"]
    fn substr((val): Str, (start_index): Number, (end_index): Number) {
        let start_index = start_index as usize;
        let end_index = end_index as usize;
        if start_index >= end_index {
            return Err(RuntimeError::BuiltinError {
                builtin,
                message: "Start index is larger than end index".to_string(),
                info,
            });
        }
        if end_index > val.len() {
            return Err(RuntimeError::BuiltinError {
                builtin,
                message: "End index is larger than string".to_string(),
                info,
            });
        }
        Value::Str(val.as_str()[start_index..end_index].to_string())
    }

    [RemoveIndex] #[safe = true, desc = "Removes a specific value from an array, string or dictionary. You can also use `array.remove(index)` or `dict.remove(key)`", example = "
let names = ['James', 'Sophia', 'Romulus', 'Remus', 'Tiberius']
$.remove_index(names, 2)
$.assert(names == ['James', 'Sophia', 'Remus', 'Tiberius'])

let name_age = {
    'Sophia': 34, 
    'Romulus': 14, 
    'Remus': 15, 
}
$.remove_index(name_age, 'Romulus')
$.assert(name_age == {
    'Sophia': 34, 
    'Remus': 15, 
})
    "]
    fn remove_index(mut (arr), (index)) {

        let typ = globals.get_type_str(arguments[0]);

        match &mut arr {
            Value::Array(arr) => {
                let out = (arr).remove(match index {
                    Value::Number(n) => n as usize,
                    _ => {
                        return Err(RuntimeError::BuiltinError {
                            builtin,
                            message: format!("Expected number, found @{}", globals.get_type_str(arguments[0])),
                            info,
                        })
                    }
                });
                globals.stored_values[out].clone()
            }

            Value::Str(s) => Value::Str(s.remove(match index {
                Value::Number(n) => n as usize,
                _ => {
                    return Err(RuntimeError::BuiltinError {
                        builtin,
                        message: format!("Expected number, found @{}", globals.get_type_str(arguments[0])),
                        info,
                    })
                }
            }).to_string()),

            Value::Dict(dict) => {
                let out = match dict.remove(&LocalIntern::new(match index {
                    Value::Str(s) => s,
                    _ => {
                        return Err(RuntimeError::BuiltinError {
                            builtin,
                            message: format!("Expected string, found @{}", globals.get_type_str(arguments[0])),
                            info,
                        })
                    }
                })) {
                    Some(val) => val,
                    None => return Err(RuntimeError::BuiltinError {
                        builtin,
                        message: "Index not found in dictionary".to_string(),
                        info,
                    }),
                };
                globals.stored_values[out].clone()
            }
            _ => {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: format!("Expected array or string, found @{}", typ),
                    info,
                })
            }
        }
    }

    [Regex] #[safe = true, desc = "Performs a regex operation on a string", example = ""]
    fn regex(#["`mode` can be either \"match\", \"replace\", \"find_all\" or \"find_groups\""](regex): Str, (s): Str, (mode): Str, (replace)) {
        use fancy_regex::Regex;


            if let Ok(r) = Regex::new(&regex) {
                match &*mode {
                    "match" => Value::Bool(r.is_match(&s).unwrap()),
                    "replace" => {
                        match &globals.stored_values[arguments[3]] {
                            Value::Str(replacer) => {
                                Value::Str(r.replace_all(&s, replacer).to_string())
                            }
                            _ => {
                                return Err(
                                    RuntimeError::BuiltinError {
                                        builtin,
                                        message: format!("Invalid or missing replacer. Expected @string, found @{}", &globals.get_type_str(arguments[3])),
                                        info
                                    }
                                )
                            }
                        }
                    },
                    "find_all" => {
                        let mut output = Vec::new();

                        for i in r.find_iter(&s){

                            let isafe = i.unwrap();

                            let mut pair = Vec::new();
                            let p1 = store_const_value(Value::Number(isafe.start() as f64), globals, context.start_group, info.position);
                            let p2 = store_const_value(Value::Number(isafe.end() as f64), globals, context.start_group, info.position);

                            pair.push(p1);
                            pair.push(p2);

                            let pair_arr = store_const_value(Value::Array(pair), globals, context.start_group, info.position);
                            output.push(pair_arr);
                        }

                        Value::Array(output)
                    },
                    "find_groups" => {
                        let mut output = Vec::new();

                        for i in r.captures_iter(&s){

                            let capture = i.unwrap();

                            let mut found = false;

                            let mut range = Vec::new();
                            let mut text = String::new();
                            let mut group_name = None;
                            for name in r.capture_names().flatten() {

                                if let Some(m) = capture.name(name) {
                                    found = true;
                                    range.push(
                                        store_const_value(Value::Number(m.start() as f64), globals, context.start_group, info.position)
                                    );
                                    range.push(
                                        store_const_value(Value::Number(m.end() as f64), globals, context.start_group, info.position)
                                    );
                                    text = m.as_str().to_string();
                                    group_name = Some(name.to_string());
                                }

                            }
                            if !found {
                                for g in 1..r.captures_len() {
                                    if let Some(m) = capture.get(g) {
                                        found = true;
                                        range.push(
                                            store_const_value(Value::Number(m.start() as f64), globals, context.start_group, info.position)
                                        );
                                        range.push(
                                            store_const_value(Value::Number(m.end() as f64), globals, context.start_group, info.position)
                                        );
                                        text = m.as_str().to_string();
                                    }
                                }
                            }
                            if !found { continue }

                            let mut match_map = FnvHashMap::default();
                            match_map.insert(
                                LocalIntern::new("range".to_string()),
                                store_const_value(Value::Array(range), globals, context.start_group, info.position),
                            );
                            match_map.insert(
                                LocalIntern::new("text".to_string()),
                                store_const_value(Value::Str(text), globals, context.start_group, info.position),
                            );
                            match_map.insert(
                                LocalIntern::new("name".to_string()),
                                if let Some(n) = group_name {
                                    store_const_value(Value::Str(n), globals, context.start_group, info.position)
                                } else { store_const_value(Value::Null, globals, context.start_group, info.position) },
                            );
                            output.push(store_const_value(Value::Dict(match_map), globals, context.start_group, info.position));
                        }

                        Value::Array(output)
                    },
                    _ => {
                        return Err(RuntimeError::BuiltinError {
                            builtin,
                            message: format!(
                                "Invalid regex mode \"{}\" in regex {}. Expected \"match\", \"replace\", \"find_all\" or \"find_groups\"",
                                mode, r
                            ),
                            info,
                        })
                    }
                }
            } else {
                return Err(RuntimeError::BuiltinError {
                    builtin,
                    message: "Failed to build regex (invalid syntax)".to_string(),
                    info,
                });
            }

    }

    [RangeOp] #[safe = true, desc = "Default implementation of the `..` operator", example = "$._range_(0, 10)"]
    fn _range_((val_a), (b): Number) {
        let end = convert_to_int(b, &info)?;
        match val_a {
            Value::Number(start) => {
                Value::Range(convert_to_int(start, &info)?, end, 1)
            }
            Value::Range(start, step, old_step) => {
                if old_step != 1 {

                    return Err(RuntimeError::CustomError(create_error(
                        info,
                        "Range operator cannot be used on a range that already has a non-default stepsize",
                        &[],
                        None,
                    )));


                }
                Value::Range(
                    start,
                    end,
                    if step <= 0 {

                        return Err(RuntimeError::CustomError(create_error(
                            info,
                            "range cannot have a stepsize less than or 0",
                            &[],
                            None,
                        )));
                    } else {
                        step as usize
                    },
                )
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    found: globals.get_type_str(arguments[0]),
                    val_def: globals.get_area(arguments[0]),
                    info,
                });

            }
        }
    }
    [InclRangeOp] #[safe = true, desc = "Default implementation of the `..=` operator", example = "$._incl_range_(0, 10)"]
    fn _incl_range_((val_a): Number, (b): Number) {
        Value::Range(val_a as i32, (b + 1.0) as i32, 1)
    }

    // unary operators
    [IncrOp] #[safe = true, desc = "Default implementation of the `n++` operator", example = "let n = 0\n$._increment_(n)\n$.assert(n == 1)"]
    fn _increment_(mut (a): Number) { a += 1.0; Value::Number(a - 1.0)}
    [DecrOp] #[safe = true, desc = "Default implementation of the `n--` operator", example = "let n = 1\n$._decrement_(n)\n$.assert(n == 0)"]
    fn _decrement_(mut (a): Number) { a -= 1.0; Value::Number(a + 1.0)}

    [PreIncrOp] #[safe = true, desc = "Default implementation of the `++n` operator", example = "let n = 0\n$.assert($._pre_increment_(n) == 1)"]
    fn _pre_increment_(mut (a): Number) { a += 1.0; Value::Number(a)}
    [PreDecrOp] #[safe = true, desc = "Default implementation of the `--n` operator", example = "let n = 1\n$.assert($._pre_decrement_(n) == 0)"]
    fn _pre_decrement_(mut (a): Number) { a -= 1.0; Value::Number(a)}

    [NegOp] #[safe = true, desc = "Default implementation of the `-n` operator", example = "$.assert($._negate_(1) == -1)"]
    fn _negate_((a): Number) { Value::Number(-a)}
    [NotOp] #[safe = true, desc = "Default implementation of the `!b` operator", example = "$.assert($._not_(false))"]
    fn _not_((a)) {
        match a {
            Value::Bool(b) => Value::Bool(!b),
            Value::Pattern(p) => Value::Pattern(Pattern::Not(Box::new(p))),
            Value::TypeIndicator(t) => Value::Pattern(Pattern::Not(Box::new(Pattern::Type(t)))),
            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "boolean or pattern".to_string(),
                    found: globals.get_type_str(arguments[0]),
                    val_def: globals.get_area(arguments[0]),
                    info,
                });
            }
        }
    }
    // [UnaryRangeOp] #[safe = true, desc = "Default implementation of the `..n` operator", example = "$._unary_range_(n)"]
    // fn _unary_range_((a): Number) { Value::Range(0, convert_to_int(a, &info)?, 1)}
    [EqPatternOp] #[safe = true, desc = "Default implementation of the `==a` operator", example = "$.assert(10 is $._eq_pattern_(10))"]
    fn _eq_pattern_((a)) { Value::Pattern(Pattern::Eq(store_const_value(a, globals, context.start_group, info.position))) }

    [NotEqPatternOp] #[safe = true, desc = "Default implementation of the `!=a` operator", example = "$.assert(10 is $._not_eq_pattern_(5))"]
    fn _not_eq_pattern_((a)) { Value::Pattern(Pattern::NotEq(store_const_value(a, globals, context.start_group, info.position))) }

    [MorePatternOp] #[safe = true, desc = "Default implementation of the `>a` operator", example = "$.assert(10 is $._more_pattern_(9))"]
    fn _more_pattern_((a)) { Value::Pattern(Pattern::MoreThan(store_const_value(a, globals, context.start_group, info.position))) }

    [LessPatternOp] #[safe = true, desc = "Default implementation of the `<a` operator", example = "$.assert(10 is $._less_pattern_(11))"]
    fn _less_pattern_((a)) { Value::Pattern(Pattern::LessThan(store_const_value(a, globals, context.start_group, info.position))) }

    [MoreOrEqPatternOp] #[safe = true, desc = "Default implementation of the `>=a` operator", example = "$.assert(10 is $._more_or_eq_pattern_(10))"]
    fn _more_or_eq_pattern_((a)) { Value::Pattern(Pattern::MoreOrEq(store_const_value(a, globals, context.start_group, info.position))) }

    [LessOrEqPatternOp] #[safe = true, desc = "Default implementation of the `<=a` operator", example = "$.assert(10 is $._less_or_eq_pattern_(10))"]
    fn _less_or_eq_pattern_((a)) { Value::Pattern(Pattern::LessOrEq(store_const_value(a, globals, context.start_group, info.position))) }

    [InPatternOp] #[safe = true, desc = "Default implementation of the `in a` operator", example = "$.assert(10 is $._in_pattern_([8, 10, 12]))"]
    fn _in_pattern_((a)) { Value::Pattern(Pattern::In(store_const_value(a, globals, context.start_group, info.position))) }
    // operators
    [OrOp] #[safe = true, desc = "Default implementation of the `||` operator", example = "$._or_(true, false)"]
    fn _or_((a): Bool, (b): Bool) { Value::Bool(a || b) }
    [AndOp] #[safe = true, desc = "Default implementation of the `&&` operator", example = "$._and_(true, true)"]
    fn _and_((a): Bool, (b): Bool) { Value::Bool(a && b) }


    [MoreThanOp] #[safe = true, desc = "Default implementation of the `>` operator", example = "$._more_than_(100, 50)"]
    fn _more_than_((a): Number, (b): Number) { Value::Bool(a > b) }
    [LessThanOp] #[safe = true, desc = "Default implementation of the `<` operator", example = "$._less_than_(50, 100)"]
    fn _less_than_((a): Number, (b): Number) { Value::Bool(a < b) }

    [MoreOrEqOp] #[safe = true, desc = "Default implementation of the `>=` operator", example = "$._more_or_equal_(100, 100)"]
    fn _more_or_equal_((a): Number, (b): Number) { Value::Bool(a >= b) }
    [LessOrEqOp] #[safe = true, desc = "Default implementation of the `<=` operator", example = "$._less_or_equal_(100, 100)"]
    fn _less_or_equal_((a): Number, (b): Number) { Value::Bool(a <= b) }

    [EqOp] #[safe = true, desc = "Default implementation of the `==` operator", example = "$._equal_(\"hello\", \"hello\")"]
    [[RAW]] fn _equal_((a), (b)) {
        default_value_equality(arguments[0], arguments[1], globals, unsafe { FullContext::from_ptr(full_context) }, &info)?;
    }

    [IsOp] #[safe = true, desc = "Default implementation of the `is` operator", example = "$._is_([1, 2, 3], [@number])"]
    [[RAW]] fn _is_((val), (pattern)) {
        val.matches_pat(&pattern, &info, globals, unsafe { FullContext::from_ptr(full_context) }, true)?;
    }

    [NotEqOp] #[safe = true, desc = "Default implementation of the `!=` operator", example = "$._not_equal_(\"hello\", \"bye\")"]
    [[RAW]] fn _not_equal_((a), (b)) {
        let contexts = unsafe { FullContext::from_ptr(full_context) };
        default_value_equality(arguments[0], arguments[1], globals, contexts, &info)?;
        // negate
        for c in contexts.iter() {
            if let Value::Bool(b) = &mut globals.stored_values[c.inner().return_value] {
                *b = !(*b);
            } else {
                unreachable!() // unchecked?
            }
        }
    }

    [DividedByOp] #[safe = true, desc = "Default implementation of the `/` operator", example = "$._divided_by_(64, 8)"]
    fn _divided_by_((a): Number, (b): Number) {
        div_zero_check(b, "divide", &builtin, &info)?;
        Value::Number(a / b)
    }
    [IntdividedByOp] #[safe = true, desc = "Default implementation of the `/%` operator", example = "$._intdivided_by_(64, 8)"]
    fn _intdivided_by_((a): Number, (b): Number) {
        div_zero_check(b, "divide", &builtin, &info)?;
        Value::Number((a / b).floor())
    }
    [TimesOp] #[safe = true, desc = "Default implementation of the `*` operator", example = "$._times_(8, 8)"]
    fn _times_((a), (b): Number) {
        match a {
            Value::Number(a) => Value::Number(a * b),
            Value::Str(a) => {
                let number = convert_to_int(b, &info)?;
                if number >= 0 {
                    Value::Str(a.repeat(number as usize))
                } else {
                    return Err(RuntimeError::BuiltinError {
                        builtin,
                        message: format!(
                            "Expected {}, found {}",
                            "a positive number",
                            b,
                        ),
                        info,
                    })
                }
            },
            Value::Array(ar) => {
                let mut new_out = Vec::<StoredValue>::new();
                for _ in 0..convert_to_int(b, &info)? {
                    for value in &ar {
                        new_out.push(clone_value(
                            *value,
                            globals,
                            context.start_group,
                            !globals.is_mutable(*value),
                            info.position)
                        );
                    }
                }

                Value::Array(new_out)
            }
            _ => {
                return Err(RuntimeError::CustomError(create_error(
                    info.clone(),
                    "Type mismatch",
                    &[
                        (globals.get_area(arguments[0]), &format!("Value defined as {} here", globals.get_type_str(arguments[0]))),
                        (globals.get_area(arguments[1]), &format!("Value defined as {} here", globals.get_type_str(arguments[1]))),
                        (
                            info.position,
                            &format!("Expected @number and @number, @string and @number or @array and @number, found @{} and @{}", globals.get_type_str(arguments[0]), globals.get_type_str(arguments[1])),
                        ),
                    ],
                    None,
                )))

            }
        }
    }
    [ModOp] #[safe = true, desc = "Default implementation of the `%` operator", example = "$._mod_(70, 8)"]
    fn _mod_((a): Number, (b): Number) {
        div_zero_check(b, "modulo", &builtin, &info)?;
        Value::Number(a.rem_euclid(b))
    }
    [PowOp] #[safe = true, desc = "Default implementation of the `^` operator", example = "$._pow_(8, 2)"]
    fn _pow_((a): Number, (b): Number) { Value::Number(a.powf(b)) }
    [PlusOp] #[safe = true, desc = "Default implementation of the `+` operator", example = "$._plus_(32, 32)"]
    fn _plus_((a), (b)) {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::Str(a), Value::Str(b)) => Value::Str(a + &b),
            (Value::Array(a), Value::Array(b)) => Value::Array({
                let mut new_arr = Vec::new();
                for el in a.iter().chain(b.iter()) {
                    new_arr.push(clone_value(*el, globals, context.start_group, !globals.is_mutable(*el), info.position));
                }
                new_arr

            }),
            _ => {



                return Err(RuntimeError::CustomError(create_error(
                    info.clone(),
                    "Type mismatch",
                    &[
                        (globals.get_area(arguments[0]), &format!("Value defined as {} here", globals.get_type_str(arguments[0]))),
                        (globals.get_area(arguments[1]), &format!("Value defined as {} here", globals.get_type_str(arguments[1]))),
                        (
                            info.position,
                            &format!("Expected @number and @number, @string and @string or @array and @array, found @{} and @{}", globals.get_type_str(arguments[0]), globals.get_type_str(arguments[1])),
                        ),
                    ],
                    None,
                )));
            }
        }
    }
    [MinusOp] #[safe = true, desc = "Default implementation of the `-` operator", example = "$._minus_(128, 64)"]
    fn _minus_((a): Number, (b): Number) { Value::Number(a - b) }
    [AssignOp] #[safe = true, desc = "Default implementation of the `=` operator", example = "let val = 0\n$._assign_(val, 64)\n$.assert(val == 64)"]
    fn _assign_(mut (a), (b)) {
        a = b;
        (*globals.stored_values.map.get_mut(arguments[0]).unwrap()).def_area = info.position;
        Value::Null
    }
    [SwapOp] #[safe = true, desc = "Default implementation of the `<=>` operator", example = "let a = 10\nlet b = 5\n$._swap_(a, b)\n$.assert(a == 5)\n$.assert(b == 10)"]
    fn _swap_(mut (a), mut (b)) {

        std::mem::swap(&mut a, &mut b);
        (*globals.stored_values.map.get_mut(arguments[0]).unwrap()).def_area = info.position;
        (*globals.stored_values.map.get_mut(arguments[1]).unwrap()).def_area = info.position;
        Value::Null
    }

    [InOp] #[safe = true, desc = "Default implementation of the `in` operator", example = "$._in_(2, [1,2,3])"]
    fn _in_((a), (b)) {
        match (a.clone(), b.clone()) {
            (_, Value::Array(ar)) => {
                let mut out = false;
                for v in ar.clone() {
                    // use custom == impl?
                    if strict_value_equality(v, arguments[0], globals) {
                        out = true;
                        break;
                    }
                }
                Value::Bool(out)
            }

            (Value::Str(b), Value::Dict(d)) => {
                // use custom == impl?
                Value::Bool(d.get(&LocalIntern::new(b)).is_some())
            }

            (Value::Str(s), Value::Str(s2)) => Value::Bool(s2.contains(&*s)),

            (Value::Number(n), Value::Obj(o, _m)) => {
                let obj_has: bool = o.iter().any(|k| k.0 == n as u16);
                Value::Bool(obj_has)
            }

            (Value::Dict(d), Value::Obj(o, _m)) => {
                let gotten_type = d.get(&globals.TYPE_MEMBER_NAME);

                if gotten_type == None
                    || globals.stored_values[*gotten_type.unwrap()]
                        != Value::TypeIndicator(19)
                {
                    // 19 = object_key??
                    return Err(RuntimeError::TypeError {
                        expected: "either @number or @object_key".to_string(),
                        found: globals.get_type_str(arguments[0]),
                        val_def: globals.get_area(arguments[0]),
                        info,
                    });
                }

                let id = d.get(&globals.OBJ_KEY_ID);
                if id == None {
                    return Err(RuntimeError::BuiltinError {
                        builtin,
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
                            expected: "number".to_string(),
                            val_def: globals.get_area(*id.unwrap()),
                            found: globals.get_type_str(*id.unwrap()),
                            info,
                        })
                    }
                };
                let obj_has: bool = o.iter().any(|k| k.0 == ob_key);
                Value::Bool(obj_has)
            }

            (_, Value::Obj(_, _)) => {
                return Err(RuntimeError::TypeError {
                    expected: "@number or @object_key".to_string(),
                    found: globals.get_type_str(arguments[0]),
                    val_def: globals.get_area(arguments[0]),
                    info,
                })
            }

            (_, Value::Str(_)) => {
                return Err(RuntimeError::TypeError {
                    expected: "string to compare".to_string(),
                    found: globals.get_type_str(arguments[0]),
                    val_def: globals.get_area(arguments[0]),
                    info,
                })
            }

            (_, Value::Dict(_)) => {
                return Err(RuntimeError::TypeError {
                    expected: "string as key".to_string(),
                    found: globals.get_type_str(arguments[0]),
                    val_def: globals.get_area(arguments[0]),
                    info,
                })
            }

            (Value::Pattern(p1), _) => Value::Bool(
                if let Value::Pattern(p) = convert_type(&b, type_id!(pattern), &info, globals, context)? {
                    p1.in_pat(&p, globals)?
                } else {
                    unreachable!()
                }
            ),

            (_, Value::Pattern(p2)) => Value::Bool(
                if let Value::Pattern(p) = convert_type(&a, type_id!(pattern), &info, globals, context)? {
                    p.in_pat(&p2, globals)?
                } else {
                    unreachable!()
                }
            ),

            (Value::TypeIndicator(_), Value::TypeIndicator(_)) => Value::Bool(
                if let (Value::Pattern(p1), Value::Pattern(p2)) = (
                    convert_type(&a, type_id!(pattern), &info, globals, context)?,
                    convert_type(&b, type_id!(pattern), &info, globals, context)?,
                ) {
                    p1.in_pat(&p2, globals)?
                } else {
                    unreachable!()
                }
            ),


            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "array, dictionary, object, or string".to_string(),
                    found: globals.get_type_str(arguments[1]),
                    val_def: globals.get_area(arguments[0]),
                    info,
                })
            }
        }
    }

    [AsOp] #[safe = true, desc = "Default implementation of the `as` operator", example = "$._as_(1000, @string)"]
    fn _as_((a), (t): TypeIndicator) { convert_type(&a,t,&info,globals,context)? }

    [SubtractOp] #[safe = true, desc = "Default implementation of the `-=` operator", example = "let val = 25\n$._subtract_(val, 10)\n$.assert(val == 15)"]
    fn _subtract_(mut (a): Number, (b): Number) { a -= b; Value::Null }
    [AddOp] #[safe = true, desc = "Default implementation of the `+=` operator", example = "let val = 25\n$._add_(val, 10)\n$.assert(val == 35)"]
    fn _add_(mut (a), (b)) {
        match (&mut a, b) {
            (Value::Number(a), Value::Number(b)) => *a += b,
            (Value::Str(a), Value::Str(b)) => *a += &b,
            (Value::Array(a), Value::Array(b)) => {
                for el in b.iter() {
                    a.push(clone_value(*el, globals, context.start_group, !globals.is_mutable(*el), info.position));
                }
            },
            _ => return Err(RuntimeError::CustomError(create_error(
                info.clone(),
                "Type mismatch",
                &[
                    (globals.get_area(arguments[0]), &format!("Value defined as {} here", globals.get_type_str(arguments[0]))),
                    (globals.get_area(arguments[1]), &format!("Value defined as {} here", globals.get_type_str(arguments[1]))),
                    (
                        info.position,
                        &format!("Expected @number and @number, @string and @string or @array and @array, found @{} and @{}", globals.get_type_str(arguments[0]), globals.get_type_str(arguments[1])),
                    ),
                ],
                None,
            )))
        }
        Value::Null
    }
    [MultiplyOp] #[safe = true, desc = "Default implementation of the `*=` operator", example = "let val = 5\n$._multiply_(val, 10)\n$.assert(val == 50)"]
    fn _multiply_(mut (a), (b): Number)         {
        match &mut a {
            Value::Number(a) => *a *= b,
            Value::Str(a) => {
                let number = convert_to_int(b, &info)?;
                if number >= 0 {
                    *a = a.repeat(number as usize)
                } else {
                    return Err(RuntimeError::BuiltinError {
                        builtin,
                        message: format!(
                            "Expected {}, found {}",
                            "a positive number",
                            b,
                        ),
                        info,
                    })
                }
            },
            _ => {
                return Err(RuntimeError::CustomError(create_error(
                    info.clone(),
                    "Type mismatch",
                    &[
                        (globals.get_area(arguments[0]), &format!("Value defined as {} here", globals.get_type_str(arguments[0]))),
                        (globals.get_area(arguments[1]), &format!("Value defined as {} here", globals.get_type_str(arguments[1]))),
                        (
                            info.position,
                            &format!("Expected @number and @number or @string and @number, found @{} and @{}", globals.get_type_str(arguments[0]), globals.get_type_str(arguments[1])),
                        ),
                    ],
                    None,
                )))

            }
        };
        Value::Null
    }
    [DivideOp] #[safe = true, desc = "Default implementation of the `/=` operator", example = "let val = 9\n$._divide_(val, 3)\n$.assert(val == 3)"]
    fn _divide_(mut (a): Number, (b): Number) {
        div_zero_check(b, "divide", &builtin, &info)?;
        a /= b;
        Value::Null
    }
    [IntdivideOp] #[safe = true, desc = "Default implementation of the `/%=` operator", example = "let val = 10\n$._intdivide_(val, 3)\n$.assert(val == 3)"]
    fn _intdivide_(mut (a): Number, (b): Number) {
        div_zero_check(b, "divide", &builtin, &info)?;
        a /= b; a = a.floor();
        Value::Null
    }
    [ExponateOp] #[safe = true, desc = "Default implementation of the `^=` operator", example = "let val = 3\n$._exponate_(val, 3)\n$.assert(val == 27)"]
    fn _exponate_(mut (a): Number, (b): Number) { a = a.powf(b); Value::Null }
    [ModulateOp] #[safe = true, desc = "Default implementation of the `%=` operator", example = "let val = 10\n$._modulate_(val, 3)\n$.assert(val == 1)"]
    fn _modulate_(mut (a): Number, (b): Number) {
        div_zero_check(b, "modulo", &builtin, &info)?;
        a = a.rem_euclid(b);
        Value::Null
    }

    [EitherOp] #[safe = true, desc = "Default implementation of the `|` operator", example = "$._either_(@number, @counter)"]
    fn _either_((a), (b)) {
        Value::Pattern(Pattern::Either(
            if let Value::Pattern(p) = convert_type(&a, type_id!(pattern), &info, globals, context)? {
                Box::new(p)
            } else {
                unreachable!()
            },
            if let Value::Pattern(p) = convert_type(&b, type_id!(pattern), &info, globals, context)? {
                Box::new(p)
            } else {
                unreachable!()
            },
        ))
    }

    [BothOp] #[safe = true, desc = "Default implementation of the `&` operator", example = "$._both_(@number, @counter)"]
    fn _both_((a), (b)) {
        Value::Pattern(Pattern::Both(
            if let Value::Pattern(p) = convert_type(&a, type_id!(pattern), &info, globals, context)? {
                Box::new(p)
            } else {
                unreachable!()
            },
            if let Value::Pattern(p) = convert_type(&b, type_id!(pattern), &info, globals, context)? {
                Box::new(p)
            } else {
                unreachable!()
            },
        ))
    }

    [DisplayOp] #[safe = true, desc = "returns the default value display string for the given value", example = "$._display_(counter()) // \"@counter::{ item: ?i, bits: 16 }\""] fn _display_((a)) {
        Value::Str(a.to_str_full(globals, |val, globals| display_val(val.clone(), unsafe { FullContext::from_ptr(full_context) }, globals, &info) )?)
    }
    [Display] #[safe = true, desc = "returns the value display string for the given value", example = "$.display(counter()) // \"counter(?i, bits = 16)\""] fn display((a)) {
        let ctx = unsafe { FullContext::from_ptr(full_context) };
        handle_unary_operator(arguments[0], Builtin::DisplayOp, ctx, globals, &info)?;
        // add error on context split?
        globals.stored_values[ctx.inner().return_value].clone()
    }

}
