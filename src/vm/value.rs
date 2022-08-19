use std::collections::HashMap;

use ahash::AHashMap;

use super::{
    error::RuntimeError,
    interpreter::{BuiltinKey, Globals, TypeKey, ValueKey},
    types::Instance,
};
use crate::{
    compilation::code::VarID,
    leveldata::{gd_types::Id, object_data::GdObj},
    sources::CodeArea,
};

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

    // pub fn expect_value_type<T>(&self, typ: ValueTypeUnion) -> Result<Value, RuntimeError>
    // where
    //     ValueTypeUnion: From<ValueType>,
    // {
    //     if !typ.0.contains(&self.value.typ()) {
    //         Err(RuntimeError::TypeMismatch {
    //             v: *self,
    //             expected: typ.into(),
    //             area: self.def_area,
    //         })
    //     } else {
    //         Ok(self.value)
    //     }
    // }
}

macro_rules! spwn_types {
    (
        $(
            $name:ident $( $value:tt )?,
        )+
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum Value {
            $(
                $name $( $value )?,
            )+
            Instance(Instance),
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum ValueType {
            $(
                $name,
            )+
            Custom(TypeKey),
        }
        impl Value {
            pub fn typ(&self) -> ValueType {
                match self {
                    $(
                        Value::$name(..) => ValueType::$name,
                    )+
                    Value::Instance(Instance { typ, .. }) => ValueType::Custom(*typ),
                }
            }
        }
        use convert_case::{Case, Casing};

        impl ValueType {
            pub fn to_str(self, globals: &Globals) -> String {
                format!(
                    "@{}",
                    match self {
                        $(
                            ValueType::$name => stringify!($name).to_case(Case::Snake),
                        )+
                        ValueType::Custom(k) => globals.types[k].name.clone(),
                    }
                )
            }
        }
        // use super::types::Type;
        // impl Globals {
        //     fn populate_type_slotmap(&mut self) {
        //         $(
        //             let n = stringify!($name).to_case(Case::Snake);
        //             self.types.insert(Type {
        //                 name: n,
        //                 members: AHashMap::default(),
        //             });
        //         )+
        //     }
        // }

    };
}

spwn_types! {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),

    Array(Vec<ValueKey>),
    Dict(AHashMap<String, ValueKey>),

    Empty(),

    Maybe(Option<ValueKey>),

    Macro(Macro),
    Pattern(Pattern),

    TriggerFunction(Id),

    Group(Id),
    Channel(Id),
    Block(Id),
    Item(Id),

    Object(GdObj),

    Type(ValueType),

    Builtins(),

    // Instance(Instance),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: String,
    pub default: Option<ValueKey>,
    pub pattern: Option<ValueKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Macro {
    Custom {
        func_id: usize,
        captured: HashMap<VarID, ValueKey>,
        args: Vec<Argument>,
        ret_pattern: ValueKey,
    },

    Builtin {
        self_arg: ValueKey,
        func_ptr: BuiltinKey,
    },
}

impl Value {
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
            | Value::Empty()
            | Value::Builtins()
            | Value::Channel(_)
            | Value::Group(_)
            | Value::Item(_)
            | Value::Block(_)
            | Value::Type(_)
            | Value::TriggerFunction { .. }
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
            Value::Instance(Instance { typ: ty, fields }) => Value::Instance(Instance {
                typ: *ty,
                fields: fields
                    .iter()
                    .map(|(k, v)| (k.clone(), globals.key_deep_clone(*v)))
                    .collect(),
            }),
            Value::Maybe(v) => Value::Maybe(v.map(|v| globals.key_deep_clone(v))),
            Value::Macro(_) => {
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
            Value::Type(v) => v.to_str(globals),
            Value::Empty() => "()".into(),
            Value::Builtins() => "$".into(),
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
            Value::Pattern(p) => p.to_str(globals),
            Value::Group(id) => format!("{}g", id.to_str()),
            Value::Channel(id) => format!("{}c", id.to_str()),
            Value::Item(id) => format!("{}i", id.to_str()),
            Value::Block(id) => format!("{}b", id.to_str()),
            Value::TriggerFunction(start_group) => format!("!{{...}}:{}", start_group.to_str()),
            Value::Macro(Macro::Custom {
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

            Value::Macro(Macro::Builtin { self_arg, .. }) => {
                format!("{}.thing", globals.memory[*self_arg].value.to_str(globals))
            }
            Value::Object(a) => format!("{:?}", a),
            Value::Instance(s) => format!(
                "@{}::{{{}}}",
                globals.types[s.typ].name,
                s.fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, globals.memory[*v].value.to_str(globals)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Debug)]
pub struct ValueTypeUnion(pub Vec<ValueType>);

impl From<ValueType> for ValueTypeUnion {
    fn from(t: ValueType) -> ValueTypeUnion {
        ValueTypeUnion(vec![t])
    }
}

// ValueType::String | ValueType::Int
impl std::ops::BitOr for ValueType {
    type Output = ValueTypeUnion;

    fn bitor(self, rhs: Self) -> Self::Output {
        ValueTypeUnion(vec![self, rhs])
    }
}

// ValueType::String | ValueType::Int | ValueType::Bool
impl std::ops::BitOr<ValueType> for ValueTypeUnion {
    type Output = ValueTypeUnion;

    fn bitor(self, rhs: ValueType) -> Self::Output {
        let mut out = self.0;
        out.push(rhs);
        ValueTypeUnion(out)
    }
}

impl ValueTypeUnion {
    pub fn to_string(&self, globals: &Globals) -> String {
        if self.0.is_empty() {
            "idfk nothing ig lol😂😂😂".into()
        } else if self.0.len() == 1 {
            self.0[0].to_str(globals)
        } else {
            let (last, comma) = self.0.split_last().unwrap();
            format!(
                "{} or {}",
                comma
                    .iter()
                    .map(|v| v.to_str(globals))
                    .collect::<Vec<_>>()
                    .join(", "),
                last.to_str(globals)
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Any,
    Type(ValueType),
}

impl Pattern {
    pub fn to_str(&self, globals: &Globals) -> String {
        match self {
            Pattern::Any => "_".into(),
            Pattern::Type(t) => t.to_str(globals),
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
    use super::{Pattern, StoredValue, Value, ValueType};
    use crate::{
        sources::CodeArea,
        vm::{error::RuntimeError, interpreter::Globals},
    };

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

    pub fn to_pat(a: &StoredValue) -> Result<Pattern, RuntimeError> {
        match &a.value {
            Value::Type(typ) => Ok(Pattern::Type(*typ)),
            Value::Pattern(p) => Ok(p.clone()),
            _ => Err(RuntimeError::CannotConvert {
                a: a.clone(),
                to: ValueType::Pattern,
            }),
        }
    }

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
