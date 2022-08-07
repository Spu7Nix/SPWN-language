use std::collections::HashMap;

use crate::{
    compilation::code::VarID,
    leveldata::{gd_types::Id, object_data::GdObj},
    sources::CodeArea,
};

use super::interpreter::{Globals, TypeKey, ValueKey};

#[derive(Debug, Clone)]
pub struct StoredValue {
    pub value: Value,
    pub def_area: CodeArea,
}
impl StoredValue {
    pub fn deep_clone(&self, globals: &mut Globals) -> StoredValue {
        StoredValue {
            value: self.value.deep_clone(globals),
            def_area: self.def_area.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),

    Array(Vec<ValueKey>),
    Dict(HashMap<String, ValueKey>),

    Empty,

    Maybe(Option<ValueKey>),

    Macro(Macro),
    Pattern(Pattern),

    TriggerFn { start_group: Id },

    Group(Id),
    Channel(Id),
    Block(Id),
    Item(Id),

    Object(GdObj),

    TypeIndicator(TypeKey),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: String,
    pub default: Option<ValueKey>,
    pub pattern: Option<ValueKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    pub func_id: usize,
    pub captured: HashMap<VarID, ValueKey>,
    pub args: Vec<Argument>,
    pub ret_pattern: ValueKey,
}

impl Value {
    pub fn typ(&self) -> ValueType {
        match self {
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::String(_) => ValueType::String,
            Value::Bool(_) => ValueType::Bool,
            Value::Array(_) => ValueType::Array,
            Value::Dict(_) => ValueType::Dict,
            Value::Empty => ValueType::Empty,
            Value::Maybe(_) => ValueType::Maybe,
            Value::Macro(_) => ValueType::Macro,
            Value::Pattern(_) => ValueType::Pattern,
            Value::TriggerFn { .. } => ValueType::TriggerFn,
            Value::Channel(_) => ValueType::Channel,
            Value::Group(_) => ValueType::Group,
            Value::Item(_) => ValueType::Item,
            Value::Block(_) => ValueType::Block,
            Value::Object(_) => ValueType::Object,
            Value::TypeIndicator(_) => ValueType::TypeIndicator,
        }
    }
    pub fn into_stored(self, area: CodeArea) -> StoredValue {
        StoredValue {
            value: self,
            def_area: area,
        }
    }
    pub fn deep_clone(&self, globals: &mut Globals) -> Value {
        match self {
            Value::Int(_)
            | Value::Float(_)
            | Value::String(_)
            | Value::Bool(_)
            | Value::Empty
            | Value::Channel(_)
            | Value::Group(_)
            | Value::Item(_)
            | Value::Block(_)
            | Value::TypeIndicator(_)
            | Value::TriggerFn { .. }
            | Value::Object(_) => self.clone(),
            Value::Pattern(_) => self.clone(),
            // | Value::TypeIndicator(_)
            // | Value::Pattern(_)
            // | Value::Group(_)
            // | Value::Color(_)
            // | Value::Block(_)
            // | Value::Item(_)
            // | Value::TriggerFunc { .. }
            // | Value::Object(_) => self.clone(),
            Value::Array(arr) => Value::Array(
                arr.iter()
                    .map(|v| globals.key_deep_clone(*v))
                    .collect::<Vec<_>>(),
            ),
            Value::Dict(map) => Value::Dict(
                map.iter()
                    .map(|(k, v)| (k.clone(), globals.key_deep_clone(*v)))
                    .collect(),
            ),
            Value::Maybe(v) => Value::Maybe(v.map(|v| globals.key_deep_clone(v))),
            Value::Macro(Macro { .. }) => {
                self.clone()
                // let args = args
                //     .iter()
                //     .map(|m| MacroArg {
                //         name: m.name.clone(),
                //         area: m.area.clone(),
                //         pattern: m.pattern.clone(),
                //         default: m.default.map(|d| globals.key_deep_clone(d)),
                //     })
                //     .collect();
                // Value::Macro(Macro {
                //     func_id: *func_id,
                //     args,
                //     ret_type: ret_type.clone(),
                //     capture: capture.clone(),
                // })
                // todo!()
            }
        }
    }
    pub fn to_str(&self, globals: &Globals) -> String {
        match self {
            Value::Int(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
            Value::String(v) => v.to_string(),
            Value::Bool(v) => v.to_string(),
            Value::TypeIndicator(v) => format!("@{}", globals.types[*v].name),
            Value::Empty => "()".into(),
            Value::Array(arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|v| globals.memory[*v].value.to_str(globals))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Dict(map) => format!(
                "{{{}}}",
                map.iter()
                    .map(|(k, v)| format!("{}: {}", k, globals.memory[*v].value.to_str(globals)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Maybe(None) => "?".into(),
            Value::Maybe(Some(v)) => format!("{}?", globals.memory[*v].value.to_str(globals)),
            // Value::TypeIndicator(typ) => typ.to_str(),
            Value::Pattern(p) => p.to_str(),
            Value::Group(id) => format!("{}g", id.to_str()),
            Value::Channel(id) => format!("{}c", id.to_str()),
            Value::Item(id) => format!("{}i", id.to_str()),
            Value::Block(id) => format!("{}b", id.to_str()),
            Value::TriggerFn { start_group } => format!("!{{...}}:{}", start_group.to_str()),
            Value::Macro(Macro {
                args, ret_pattern, ..
            }) => {
                format!(
                    "({}) -> {} {{...}}",
                    args.iter()
                        .map(
                            |Argument {
                                 name: n,
                                 pattern: t,
                                 default: d,
                                 ..
                             }| {
                                format!(
                                    "{}{}{}",
                                    n,
                                    if let Some(t) = t {
                                        format!(": {}", globals.memory[*t].value.to_str(globals))
                                    } else {
                                        "".into()
                                    },
                                    if let Some(d) = d {
                                        format!(" = {}", globals.memory[*d].value.to_str(globals))
                                    } else {
                                        "".into()
                                    },
                                )
                            }
                        )
                        .collect::<Vec<_>>()
                        .join(", "),
                    globals.memory[*ret_pattern].value.to_str(globals),
                )
            }
            Value::Object(a) => format!("{:?}", a),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Int,
    Float,
    String,
    Bool,
    Array,
    Dict,
    Empty,
    Maybe,
    Macro,
    Pattern,
    TriggerFn,
    Channel,
    Group,
    Item,
    Block,
    Object,
    TypeIndicator,
}

impl ValueType {
    pub fn to_str(self) -> String {
        format!(
            "@{}",
            match self {
                ValueType::Int => "int",
                ValueType::Float => "float",
                ValueType::String => "string",
                ValueType::Bool => "bool",
                ValueType::Empty => "empty",
                ValueType::Array => "array",
                ValueType::Dict => "dict",
                ValueType::Maybe => "maybe",
                ValueType::Macro => "macro",
                ValueType::Pattern => "pattern",
                ValueType::TriggerFn => "trigger_func",
                ValueType::Channel => "channel",
                ValueType::Group => "group",
                ValueType::Item => "item",
                ValueType::Block => "block",
                ValueType::Object => "object",
                ValueType::TypeIndicator => "type_indicator",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Any,
}

impl Pattern {
    pub fn to_str(&self) -> String {
        match self {
            Pattern::Any => "_".into(),
            // Pattern::Type(t) => t.to_str(),
            // Pattern::Macro { args, ret } => format!(
            //     "({}) -> {}",
            //     args.iter()
            //         .map(|arg| arg.to_str())
            //         .collect::<Vec<_>>()
            //         .join(", "),
            //     ret.to_str(),
            // ),
        }
    }
}

pub mod value_ops {
    use crate::{
        sources::CodeArea,
        vm::{error::RuntimeError, interpreter::Globals},
    };

    use super::{StoredValue, Value, ValueType};

    pub fn equality(a: &Value, b: &Value, globals: &Globals) -> bool {
        match (a, b) {
            (Value::Int(n1), Value::Float(n2)) => *n1 as f64 == *n2,
            (Value::Float(n1), Value::Int(n2)) => *n1 == *n2 as f64,

            (Value::Array(arr1), Value::Array(arr2)) => {
                if arr1.len() != arr2.len() {
                    false
                } else {
                    arr1.iter().zip(arr2).all(|(a, b)| {
                        equality(
                            &globals.memory[*a].value,
                            &globals.memory[*b].value,
                            globals,
                        )
                    })
                }
            }
            (Value::Dict(map1), Value::Dict(map2)) => {
                if map1.len() != map2.len() {
                    false
                } else {
                    for (k, a) in map1 {
                        match map2.get(k) {
                            Some(b) => {
                                if !equality(
                                    &globals.memory[*a].value,
                                    &globals.memory[*b].value,
                                    globals,
                                ) {
                                    return false;
                                }
                            }
                            None => return false,
                        }
                    }
                    true
                }
            }

            (Value::Maybe(None), Value::Maybe(None)) => true,
            (Value::Maybe(Some(a)), Value::Maybe(Some(b))) => equality(
                &globals.memory[*a].value,
                &globals.memory[*b].value,
                globals,
            ),

            _ => a == b,
        }
    }

    // pub fn matches_pat(val: &Value, pat: &Pattern) -> bool {
    //     match (val, pat) {
    //         (_, Pattern::Any) => true,
    //         (_, Pattern::Type(t)) => &val.get_type() == t,
    //         (
    //             Value::Macro(Macro {
    //                 func_id,
    //                 args,
    //                 capture,
    //                 ret_type,
    //             }),
    //             Pattern::Macro {
    //                 args: arg_patterns,
    //                 ret: ret_pattern,
    //             },
    //         ) => {
    //             &ret_type.0 == &**ret_pattern
    //                 && args
    //                     .iter()
    //                     .zip(arg_patterns)
    //                     .all(|(a, p)| &a.get_pattern() == p)
    //         }
    //         (_, _) => false,
    //     }
    // }

    pub fn to_bool(a: &StoredValue) -> Result<bool, RuntimeError> {
        match &a.value {
            Value::Bool(b) => Ok(*b),
            _ => Err(RuntimeError::CannotConvert {
                a: a.clone(),
                to: ValueType::Bool,
            }),
        }
    }

    // pub fn to_pat(a: &StoredValue) -> Result<Pattern, RuntimeError> {
    //     match &a.value {
    //         Value::TypeIndicator(typ) => Ok(Pattern::Type(*typ)),
    //         Value::Pattern(p) => Ok(p.clone()),
    //         _ => Err(RuntimeError::CannotConvert {
    //             a: a.clone(),
    //             to: ValueType::Pattern,
    //         }),
    //     }
    // }

    // pub fn to_iter(a: &StoredValue, for_area: CodeArea) -> Result<ValueIter, RuntimeError> {
    //     match &a.value {
    //         Value::Array(v) => Ok(ValueIter::Array(v.clone(), 0)),
    //         Value::String(s) => Ok(ValueIter::String(s.clone(), a.def_area.clone(), 0)),
    //         Value::Dict(map) => Ok(ValueIter::Dict {
    //             dict_area: a.def_area.clone(),
    //             for_area,
    //             idx: 0,
    //             elems: map.iter().map(|(k, v)| (k.clone(), *v)).collect::<Vec<_>>(),
    //         }),
    //         _ => Err(RuntimeError::CannotIterate { a: a.clone() }),
    //     }
    // }

    pub fn plus(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        _globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 + *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 + *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 + *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 + *n2),
            (Value::String(s1), Value::String(s2)) => Value::String(s1.clone() + s2),

            (Value::Array(arr1), Value::Array(arr2)) => {
                Value::Array(arr1.iter().chain(arr2).cloned().collect::<Vec<_>>())
            }

            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "+".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn minus(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        _globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 - *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 - *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 - *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 - *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "-".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn mult(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        _globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 * *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 * *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 * *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 * *n2),

            (Value::Int(n), Value::String(s)) => {
                Value::String(s.repeat(if *n < 0 { 0 } else { *n as usize }))
            }
            (Value::String(s), Value::Int(n)) => {
                Value::String(s.repeat(if *n < 0 { 0 } else { *n as usize }))
            }
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "*".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn div(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        _globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 / *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 / *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 / *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 / *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "/".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn modulo(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        _globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 % *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 % *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 % *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 % *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "%".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn pow(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        _globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => {
                Value::Int((*n1 as f64).powf(*n2 as f64).floor() as i64)
            }
            (Value::Int(n1), Value::Float(n2)) => Value::Float((*n1 as f64).powf(*n2)),
            (Value::Float(n1), Value::Int(n2)) => Value::Float((*n1).powf(*n2 as f64)),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(n1.powf(*n2)),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "^".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn eq(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        Ok(Value::Bool(equality(&a.value, &b.value, globals)).into_stored(area))
    }
    pub fn neq(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        Ok(Value::Bool(!equality(&a.value, &b.value, globals)).into_stored(area))
    }
    pub fn gt(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        _globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Bool(*n1 > *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Bool(*n1 as f64 > *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Bool(*n1 > *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Bool(*n1 > *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: ">".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn gte(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        _globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Bool(*n1 >= *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Bool(*n1 as f64 >= *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Bool(*n1 >= *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Bool(*n1 >= *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: ">=".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn lt(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        _globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Bool(*n1 < *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Bool((*n1 as f64) < *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Bool(*n1 < *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Bool(*n1 < *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "<".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn lte(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
        _globals: &Globals,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Bool(*n1 <= *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Bool(*n1 as f64 <= *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Bool(*n1 <= *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Bool(*n1 <= *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "<=".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }

    pub fn unary_negate(a: &StoredValue, area: CodeArea) -> Result<StoredValue, RuntimeError> {
        let value = match &a.value {
            Value::Int(n) => Value::Int(-n),
            Value::Float(n) => Value::Float(-n),
            _ => {
                return Err(RuntimeError::InvalidUnaryOperand {
                    a: a.clone(),
                    op: "-".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn unary_not(a: &StoredValue, area: CodeArea) -> Result<StoredValue, RuntimeError> {
        let value = match &a.value {
            Value::Bool(n) => Value::Bool(!n),
            _ => {
                return Err(RuntimeError::InvalidUnaryOperand {
                    a: a.clone(),
                    op: "-".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    // pub fn is_op(
    //     a: &StoredValue,
    //     b: &StoredValue,
    //     area: CodeArea,
    //     globals: &Globals,
    // ) -> Result<StoredValue, RuntimeError> {
    //     let value = match (&a.value, &b.value) {
    //         (a, Value::TypeIndicator(typ)) => Value::Bool(&a.get_type() == typ),
    //         (a, Value::Pattern(pat)) => Value::Bool(matches_pat(a, pat)),
    //         (_, _) => {
    //             return Err(RuntimeError::TypeMismatch {
    //                 v: b.clone(),
    //                 expected: "@type_indicator or @pattern".into(),
    //                 area,
    //             })
    //         }
    //     };
    //     Ok(value.into_stored(area))
    // }
}
