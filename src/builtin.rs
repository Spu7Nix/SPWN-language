//! Defining all native types (and functions?)
use internment::Intern;

use crate::ast::ObjectMode;
use crate::compiler::{create_error, RuntimeError};
use crate::compiler_types::*;
use crate::context::*;
use crate::globals::Globals;
use crate::levelstring::*;
use std::collections::HashMap;
use std::fs;

use crate::value::*;
use crate::value_storage::*;
use rand::seq::SliceRandom;
use rand::Rng;
use std::io::stdout;
use std::io::Write;

use reqwest;

//use text_io;
use crate::compiler_info::{CodeArea, CompilerInfo};

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
    }
}

#[test]
fn test_intern() {
    let str2 = Intern::new(String::from("hello"));

    dbg!(str2.as_ref() == "hello");
}

fn headermap_into_str(map: &reqwest::header::HeaderMap) -> String {
    let mut output = String::from("{");
    for (key, value) in map.iter() {
        output.push_str(&base64::encode(key.as_str()));
        output.push_str(": ");
        output.push_str(&base64::encode(value.to_str().expect("Failed to parse header value"))); // Guaranteed to work- function inputs are responses from a request
        output.push_str(",");
    }
    output.push_str("}");
    output
}
fn str_into_headermap(as_string: &String) -> Result<reqwest::header::HeaderMap, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    let pairs = as_string.split(',');
    for pair in pairs {
        let parts: Vec<&str> = pair.split(':').collect();
        if parts[0].len() > 0 {
            let decoded_header_name = match base64::decode(parts[0]) {
                Ok(name) => name,
                Err(_) => {return Err(format!("{} is not a valid b64 string", parts[0]))}
            };
            let header_name = match reqwest::header::HeaderName::from_bytes(&decoded_header_name) {
                Ok(name) => name,
                Err(_) => {return Err(format!("{} is not a valid header name", String::from_utf8_lossy(&decoded_header_name)))}
            };
            let decoded_header_value = match base64::decode(parts[1]) {
                Ok(value) => value,
                Err(_) => {return Err(format!("{} is not a valid b64 string", parts[0]))}
            };
            let header_value = match reqwest::header::HeaderValue::from_bytes(&decoded_header_value) {
                Ok(value) => value,
                Err(_) => {return Err(format!("{} is not a valid header value", String::from_utf8_lossy(&decoded_header_name)))}
            };
            headers.insert(header_name, header_value);
        }
    }
    Ok(headers)
}
fn encode_http_response(response: reqwest::blocking::Response) -> String {
    let mut response_builder = String::new();
    response_builder.push_str(&base64::encode(&(response.status()).as_str()));
    response_builder.push_str("||");
    response_builder.push_str(&base64::encode(&headermap_into_str(response.headers())));
    response_builder.push_str("||");
    response_builder.push_str(&base64::encode(response.text().expect("Couldn't load response text"))); // will always work (if it doesn't and someone sends in a bug report i can properly error handle this)
    return response_builder
}

impl Value {
    pub fn member(
        &self,
        member: Intern<String>,
        context: &Context,
        globals: &mut Globals,
        info: CompilerInfo,
    ) -> Option<StoredValue> {
        let get_impl = |t: u16, m: Intern<String>| match globals.implementations.get(&t) {
            Some(imp) => imp.get(&m).map(|mem| mem.0),
            None => None,
        };
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
                            Value::Number(a.len() as f64),
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
                _ => (),
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

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident)  ($($arg_name:ident),*)) => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        #[allow(unused_parens)]
        let ( $($arg_name),*) = clone_and_get_value($arguments[$arg_index], $globals, $context.start_group, true);
    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident) mut ($($arg_name:ident),*)) => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        #[allow(unused_parens)]
        let ( $(mut $arg_name),*) = $globals.stored_values[$arguments[$arg_index]].clone();
    };

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident) ($($arg_name:ident),*): $arg_type:ident) => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        #[allow(unused_parens)]

        let  ( $($arg_name),*) = match clone_and_get_value($arguments[$arg_index], $globals, $context.start_group, true) {
            Value::$arg_type($($arg_name),*) => ($($arg_name),*),

            a => {
                return Err(RuntimeError::BuiltinError {
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

    (($globals:ident, $arg_index:ident, $arguments:ident, $info:ident, $context:ident) mut ($($arg_name:ident),*): $arg_type:ident) => {
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

macro_rules! builtins {

    {
        ($arguments:ident, $info:ident, $globals:ident, $context:ident, $full_context:ident)
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
            contexts: &mut FullContext,
        ) -> Result<(), RuntimeError> {
            for full_context in contexts.iter() {
                let $full_context: *mut FullContext = full_context;
                let $context = full_context.inner();
                match func {
                    $(
                        Builtin::$variant => {

                            $(
                                #[allow(unused_assignments)]
                                let mut arg_index = 0;
                                $(
                                    if arg_index >= $arguments.len() {
                                        return Err(RuntimeError::BuiltinError {
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
                                        ($globals, arg_index, $arguments, $info, $context) $($mut)?
                                        ($($arg_name),*)$(: $arg_type)?
                                    );

                                    arg_index += 1;
                                )+
                                if arg_index < $arguments.len() - 1 {
                                    return Err(RuntimeError::BuiltinError {
                                        message: String::from(
                                            "Too many arguments provided",
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
                            (*$context).return_value = store_const_value(out, $globals, $context.start_group, $info.position);

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


    };
}

builtins! {
    (arguments, info, globals, context, full_context)

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
        for val in arguments.iter() {
            match &globals.stored_values[*val] {
                Value::Str(s) => out += s,
                a => out += &a.to_str(globals)
            };

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
        let encrypted = base64::encode(s.as_bytes());
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

    [HTTPPost] fn http_post((url): Str, (headers): Str, (body): Str) {
        let client = reqwest::blocking::Client::new();
        let request_headers = match str_into_headermap(&headers) {
            Ok(headers) => headers,
            Err(error) => {
                return Err(RuntimeError::BuiltinError {
                    message: error,
                    info,
                })
            }
        };
        let response = match client.post(&url).headers(request_headers).body(body).send() {
            Ok(data) => data,
            Err(_) => {
                return Err(RuntimeError::BuiltinError {
                    message: format!("Could not make request to '{}'. Check the URL is valid and your internet connection is working.", url),
                    info,
                })
            }
        };
        Value::Str(encode_http_response(response)) 
    }

    [HTTPPut] fn http_put((url): Str, (headers): Str, (body): Str) {
        let client = reqwest::blocking::Client::new();
        let request_headers = match str_into_headermap(&headers) {
            Ok(headers) => headers,
            Err(error) => {
                return Err(RuntimeError::BuiltinError {
                    message: error,
                    info,
                })
            }
        };
        let response = match client.put(&url).headers(request_headers).body(body).send() {
            Ok(data) => data,
            Err(_) => {
                return Err(RuntimeError::BuiltinError {
                    message: format!("Could not make request to '{}'. Check the URL is valid and your internet connection is working.", url),
                    info,
                })
            }
        };
        Value::Str(encode_http_response(response)) 
    }

    [HTTPDelete] fn http_delete((url): Str, (headers): Str, (body): Str) {
        let client = reqwest::blocking::Client::new();
        let request_headers = match str_into_headermap(&headers) {
            Ok(headers) => headers,
            Err(error) => {
                return Err(RuntimeError::BuiltinError {
                    message: error,
                    info,
                })
            }
        };
        let response = match client.delete(&url).headers(request_headers).body(body).send() {
            Ok(data) => data,
            Err(_) => {
                return Err(RuntimeError::BuiltinError {
                    message: format!("Could not make request to '{}'. Check the URL is valid and your internet connection is working.", url),
                    info,
                })
            }
        };
        Value::Str(encode_http_response(response)) 
    }

    [HTTPHead] fn http_head((url): Str, (headers): Str, (body): Str) {
        let client = reqwest::blocking::Client::new();
        let request_headers = match str_into_headermap(&headers) {
            Ok(headers) => headers,
            Err(error) => {
                return Err(RuntimeError::BuiltinError {
                    message: error,
                    info,
                })
            }
        };
        let response = match client.head(&url).headers(request_headers).body(body).send() {
            Ok(data) => data,
            Err(_) => {
                return Err(RuntimeError::BuiltinError {
                    message: format!("Could not make request to '{}'. Check the URL is valid and your internet connection is working.", url),
                    info,
                })
            }
        };
        Value::Str(encode_http_response(response)) 
    }


    [HTTPPatch] fn http_patch((url): Str, (headers): Str, (body): Str) {
        let client = reqwest::blocking::Client::new();
        let request_headers = match str_into_headermap(&headers) {
            Ok(headers) => headers,
            Err(error) => {
                return Err(RuntimeError::BuiltinError {
                    message: error,
                    info,
                })
            }
        };
        let response = match client.patch(&url).headers(request_headers).body(body).send() {
            Ok(data) => data,
            Err(_) => {
                return Err(RuntimeError::BuiltinError {
                    message: format!("Could not make request to '{}'. Check the URL is valid and your internet connection is working.", url),
                    info,
                })
            }
        };
        Value::Str(encode_http_response(response)) 
    }

    [HTTPGet] fn http_get((url): Str, (headers): Str, (body): Str) {
        let client = reqwest::blocking::Client::new();
        let request_headers = match str_into_headermap(&headers) {
            Ok(headers) => headers,
            Err(error) => {
                return Err(RuntimeError::BuiltinError {
                    message: error,
                    info,
                })
            }
        };
        let response = match client.get(&url).headers(request_headers).body(body).send() {
            Ok(data) => data,
            Err(_) => {
                return Err(RuntimeError::BuiltinError {
                    message: format!("Could not make request to '{}'. Check the URL is valid and your internet connection is working.", url),
                    info,
                })
            }
        };
        Value::Str(encode_http_response(response)) 
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

    [Sqrt] fn sqrt((n): Number) {Value::Number(n.sqrt())}
    [Sinh] fn sinh((n): Number) {Value::Number(n.sinh())}
    [Tanh] fn tanh((n): Number) {Value::Number(n.tanh())}
    [NaturalLog] fn ln((n): Number) {Value::Number(n.ln())}
    [Log] fn log((n): Number, (base): Number) {Value::Number(n.log(base))}
    [Min] fn min((a): Number, (b): Number) {Value::Number(a.min(b))}
    [Max] fn max((a): Number, (b): Number) {Value::Number(a.max(b))}
    [Round] fn round((n): Number) {Value::Number(n.round())}
    [Hypot] fn hypot((a): Number, (b): Number) {Value::Number(a.hypot(b))}

    [Add]
    fn add() {
        if arguments.is_empty() || arguments.len() > 2 {
            return Err(RuntimeError::BuiltinError {
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

        let mut obj_map = HashMap::<u16, ObjParam>::new();

        for p in obj {
            obj_map.insert(p.0, p.1.clone());
            // add params into map
        }

        match mode {
            ObjectMode::Object => {
                if !ignore_context && context.start_group.id != Id::Specific(0) {
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
                    .push((obj, crate::levelstring::TriggerOrder(globals.trigger_order)))
            }
        };
        Value::Null
    }

    [Append]
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

    [SplitStr]
    fn split_str((s): Str, (substr): Str) {

        let mut output = Vec::<StoredValue>::new();

        for split in s.split(&*substr) {
            let entry =
                store_const_value(Value::Str(split.to_string()), globals, context.start_group, CodeArea::new());
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
                if !value.matches_pat(pat, &info, globals, context)? {
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
        use crate::ast::*;

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

        unsafe {
            cmp_statement.to_trigger_func(full_context.as_mut().unwrap(), globals, info.clone(), Some(group))?;
        }



        Value::Null
    }

    [TriggerFnContext]
    fn trigger_fn_context() {
        Value::Group(context.start_group)
    }

    [Random]
    fn random() {
        if arguments.len() > 2 {
            return Err(RuntimeError::BuiltinError {
                message: "Expected up to 2 arguments, found none".to_string(),
                info,
            });
        }

        if arguments.is_empty() {
            Value::Number(rand::thread_rng().gen())
        } else {
            let val = match convert_type(&globals.stored_values[arguments[0]].clone(), 10, &info, globals, context) {
                Ok(Value::Array(v)) => v,
                _ => {
                    return Err(RuntimeError::BuiltinError {
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
                                    message: format!("Problem opening the file: {}", e),
                                    info,
                                });
                            }
                        };
                        let parsed = match serde_json::from_str(&rval) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(RuntimeError::BuiltinError {
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
                                    let mut dict: HashMap<Intern<String>, StoredValue> = HashMap::new();
                                    for (key, value) in x {
                                        dict.insert(Intern::new(key), store_const_value(parse_json_value(value, globals, context, info), globals, context.start_group, info.position));
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
                                    message: format!("Problem opening the file: {}", e),
                                    info,
                                });
                            }
                        };
                        let parsed = match toml::from_str(&rval) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(RuntimeError::BuiltinError {
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
                                    let mut dict: HashMap<Intern<String>, StoredValue> = HashMap::new();
                                    for (key, value) in x {
                                        dict.insert(Intern::new(key), store_const_value(parse_toml_value(value, globals, context, info), globals, context.start_group, info.position));
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
                                    message: format!("Problem opening the file: {}", e),
                                    info,
                                });
                            }
                        };
                        let parsed: serde_yaml::Value = match serde_yaml::from_str(&rval) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(RuntimeError::BuiltinError {
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
                                    let mut dict: HashMap<Intern<String>, StoredValue> = HashMap::new();
                                    for (key, value) in x.iter() {
                                        dict.insert(Intern::new(key.as_str().unwrap().to_string()), store_const_value(parse_yaml_value(value, globals, context, info), globals, context.start_group, info.position));
                                    }
                                    Value::Dict(dict)
                                },
                            }
                        }
                        parse_yaml_value(&parsed, globals, context, &info)
                    }
                    _ => {
                        return Err(RuntimeError::BuiltinError {
                            message: "Invalid data format ( use \"text\", \"bin\", \"json\", \"toml\" or \"yaml\" )"
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

    [Regex] fn regex((regex): Str, (s): Str, (mode): Str, (replace)) {
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
                    },
                    "findall" => {
                        let mut output = Vec::new();

                        for i in r.find_iter(&s){
                            let mut pair = Vec::new();
                            let p1 = store_const_value(Value::Number(i.start() as f64), globals, context.start_group, info.position);
                            let p2 = store_const_value(Value::Number(i.end() as f64), globals, context.start_group, info.position);

                            pair.push(p1);
                            pair.push(p2);

                            let pair_arr = store_const_value(Value::Array(pair), globals, context.start_group, info.position);
                            output.push(pair_arr);
                        }

                        Value::Array(output)
                    },
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
    // unary operators
    [IncrOp]            fn _increment_(mut (a): Number)                 { a += 1.0; Value::Number(a - 1.0)}
    [DecrOp]            fn _decrement_(mut (a): Number)                 { a -= 1.0; Value::Number(a + 1.0)}

    [PreIncrOp]         fn _pre_increment_(mut (a): Number)             { a += 1.0; Value::Number(a)}
    [PreDecrOp]         fn _pre_decrement_(mut (a): Number)             { a -= 1.0; Value::Number(a)}

    [NegOp]             fn _negate_((a): Number)                        { Value::Number(-a)}
    [NotOp]             fn _not_((a): Bool)                             { Value::Bool(!a)}
    [UnaryRangeOp]      fn _unary_range_((a): Number)                   { Value::Range(0, convert_to_int(a, &info)?, 1)}

    // operators
    [OrOp]              fn _or_((a): Bool, (b): Bool)                   { Value::Bool(a || b) }
    [AndOp]             fn _and_((a): Bool, (b): Bool)                  { Value::Bool(a && b) }

    [MoreThanOp]        fn _more_than_((a): Number, (b): Number)        { Value::Bool(a > b) }
    [LessThanOp]        fn _less_than_((a): Number, (b): Number)        { Value::Bool(a < b) }

    [MoreOrEqOp]        fn _more_or_equal_((a): Number, (b): Number)    { Value::Bool(a >= b) }
    [LessOrEqOp]        fn _less_or_equal_((a): Number, (b): Number)    { Value::Bool(a <= b) }

    [EqOp]              fn _equal_((a), (b))                            { Value::Bool(value_equality(arguments[0], arguments[1], globals)) }
    [NotEqOp]           fn _not_equal_((a), (b))                        { Value::Bool(!value_equality(arguments[0], arguments[1], globals)) }

    [DividedByOp]       fn _divided_by_((a): Number, (b): Number)       { Value::Number(a / b) }
    [IntdividedByOp]    fn _intdivided_by_((a): Number, (b): Number)    { Value::Number((a / b).floor()) }
    [TimesOp]
    fn _times_((a), (b): Number) {
        match a {
            Value::Number(a) => Value::Number(a * b),
            Value::Str(a) => Value::Str(a.repeat(convert_to_int(b, &info)? as usize)),
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
                            &format!("Expected @number and @number or @string and @number, found @{} and @{}", globals.get_type_str(arguments[0]), globals.get_type_str(arguments[1])),
                        ),
                    ],
                    None,
                )))

            }
        }
    }
    [ModOp]             fn _mod_((a): Number, (b): Number)              { Value::Number(a.rem_euclid(b)) }
    [PowOp]             fn _pow_((a): Number, (b): Number)              { Value::Number(a.powf(b)) }
    [PlusOp] fn _plus_((a), (b)) {
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
    [MinusOp]           fn _minus_((a): Number, (b): Number)            { Value::Number(a - b) }
    [AssignOp]           fn _assign_(mut (a), (b))                      {
        a = b;
        (*globals.stored_values.map.get_mut(&arguments[0]).unwrap()).def_area = info.position;
        Value::Null
    }
    [SwapOp]           fn _swap_(mut (a), mut (b))                      {

        std::mem::swap(&mut a, &mut b);
        (*globals.stored_values.map.get_mut(&arguments[0]).unwrap()).def_area = info.position;
        (*globals.stored_values.map.get_mut(&arguments[1]).unwrap()).def_area = info.position;
        Value::Null
    }

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


                Value::Bool(d.get(&Intern::new(b)).is_some())
            }

            (Value::Str(s), Value::Str(s2)) => Value::Bool(s.contains(&*s2)),

            (Value::Obj(o, _m), Value::Number(n)) => {
                let obj_has: bool = o.iter().any(|k| k.0 == n as u16);
                Value::Bool(obj_has)
            }

            (Value::Obj(o, _m), Value::Dict(d)) => {
                let gotten_type = d.get(&globals.TYPE_MEMBER_NAME);

                if gotten_type == None
                    || globals.stored_values[*gotten_type.unwrap()]
                        != Value::TypeIndicator(19)
                {
                    // 19 = object_key??
                    return Err(RuntimeError::TypeError {
                        expected: "either @number or @object_key".to_string(),
                        found: globals.get_type_str(arguments[1]),
                        val_def: globals.get_area(arguments[1]),
                        info,
                    });
                }

                let id = d.get(&globals.OBJ_KEY_ID);
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

            (Value::Obj(_, _), _) => {
                return Err(RuntimeError::TypeError {
                    expected: "@number or @object_key".to_string(),
                    found: globals.get_type_str(arguments[1]),
                    val_def: globals.get_area(arguments[1]),
                    info,
                })
            }

            (Value::Str(_), _) => {
                return Err(RuntimeError::TypeError {
                    expected: "string to compare".to_string(),
                    found: globals.get_type_str(arguments[1]),
                    val_def: globals.get_area(arguments[1]),
                    info,
                })
            }

            (Value::Dict(_), _) => {
                return Err(RuntimeError::TypeError {
                    expected: "string as key".to_string(),
                    found: globals.get_type_str(arguments[1]),
                    val_def: globals.get_area(arguments[1]),
                    info,
                })
            }

            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "array, dictionary, object, or string".to_string(),
                    found: globals.get_type_str(arguments[0]),
                    val_def: globals.get_area(arguments[1]),
                    info,
                })
            }
        }
    }

    [AsOp]              fn _as_((a), (t): TypeIndicator)                    { convert_type(&a,t,&info,globals,context)? }

    [SubtractOp]        fn _subtract_(mut (a): Number, (b): Number)         { a -= b; Value::Null }
    [AddOp]
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
    [MultiplyOp]        fn _multiply_(mut (a), (b): Number)         {
        match &mut a {
            Value::Number(a) => *a *= b,
            Value::Str(a) => *a = a.repeat(convert_to_int(b, &info)? as usize),
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
    [DivideOp]          fn _divide_(mut (a): Number, (b): Number)           { a /= b; Value::Null }
    [IntdivideOp]       fn _intdivide_(mut (a): Number, (b): Number)        { a /= b; a = a.floor(); Value::Null }
    [ExponateOp]        fn _exponate_(mut (a): Number, (b): Number)         { a = a.powf(b); Value::Null }
    [ModulateOp]        fn _modulate_(mut (a): Number, (b): Number)         { a = a.rem_euclid(b); Value::Null }

    [EitherOp]
    fn _either_((a), (b)) {
        Value::Pattern(Pattern::Either(
            if let Value::Pattern(p) = convert_type(&a, 18, &info, globals, context)? {
                Box::new(p)
            } else {
                unreachable!()
            },
            if let Value::Pattern(p) = convert_type(&b, 18, &info, globals, context)? {
                Box::new(p)
            } else {
                unreachable!()
            },
        ))
    }

}
