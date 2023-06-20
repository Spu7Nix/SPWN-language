use ahash::AHashMap;
use delve::{EnumFromStr, EnumToStr, EnumVariantNames};
use serde::{Deserialize, Serialize};

use crate::new_id_wrapper;
use crate::sources::CodeArea;

new_id_wrapper! {
    CustomTypeID: u16;
}

// #[derive(Debug, Clone, PartialEq)]
// pub struct StoredValue {
//     pub value: Value,
//     pub area: CodeArea,
// }

#[rustfmt::skip]
macro_rules! value {
    (
        $(
            $(#[$($meta:meta)*] )?
            $name:ident
                $( ( $( $t0:ty ),* ) )?
                $( { $( $n:ident: $t1:ty ,)* } )?
            ,
        )*

        => $i_name:ident
            $( ( $( $it0:ty ),* ) )?
            $( { $( $in:ident: $it1:ty ,)* } )?
        ,
    ) => {
        #[derive(Debug, Clone, PartialEq, Default)]
        pub enum Value {
            $(
                $(#[$($meta)*])?
                $name $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ,)* } )?,
            )*
            $i_name $( ( $( $it0 ),* ) )? $( { $( $in: $it1 ,)* } )?,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, Default, EnumFromStr, EnumVariantNames, EnumToStr)]
        #[delve(rename_all = "snake_case")]
        pub enum ValueType {
            $(
                $( #[$($meta)* ])?
                $name,
            )*

            #[delve(skip)]
            Custom(CustomTypeID),
        }

        impl Value {
            pub fn get_type(&self) -> ValueType {
                match self {
                    Self::Instance { typ, .. } => ValueType::Custom(*typ),

                    $(
                        Self::$name {..} => ValueType::$name,
                    )*
                }
            }
        }

    };
}

// impl ValueType {
//     pub fn runtime_display(self, vm: &Vm) -> String {
//         format!(
//             "@{}",
//             match self {
//                 Self::Custom(t) => vm.resolve(&vm.types[t].value.name),
//                 _ => <ValueType as Into<&str>>::into(self).into(),
//             }
//         )
//     }
// }

// value! {
//     Int(i64),
//     Float(f64),
//     Bool(bool),
//     String(Vec<char>),

//     Array(Vec<ValueRef>),
//     Dict(AHashMap<Spur, (ValueRef, Visibility)>),

//     Group(Id),
//     Channel(Id),
//     Block(Id),
//     Item(Id),

//     Builtins,

//     Range(i64, i64, usize), //start, end, step

//     Maybe(Option<ValueRef>),

//     #[default]
//     Empty,
//     Macro(MacroData),

//     Type(ValueType),

//     Module {
//         exports: AHashMap<Spur, ValueRef>,
//         types: Vec<(CustomTypeID, bool)>,
//     },

//     TriggerFunction {
//         group: Id,
//         prev_context: Id,
//     },

//     Error(usize),

//     //Object(AHashMap<u8, ObjParam>, ObjectType),
//     //ObjectKey(ObjectKey),

//     Epsilon,

//     //Iterator(IteratorData),

//     Chroma {
//         r: u8, g: u8, b: u8, a: u8,
//     },

//     => Instance {
//         typ: CustomTypeID,
//         items: AHashMap<Spur, (ValueRef, Visibility)>,
//     },
// }
