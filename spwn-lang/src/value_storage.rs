///types and functions used by the compiler
use crate::builtin::*;

use crate::compiler_info::CodeArea;
use crate::context::*;
use crate::globals::Globals;
use crate::value::*;

use std::collections::HashMap;

use crate::compiler::{BUILTIN_STORAGE, NULL_STORAGE};

pub type StoredValue = usize; //index to stored value in globals.stored_values

pub struct ValStorage {
    pub map: HashMap<usize, StoredValData>, //val, fn context, mutable, lifetime
}

#[derive(Debug, Clone)]
pub struct StoredValData {
    pub val: Value,
    pub fn_context: Group,
    pub mutable: bool,
    pub def_area: CodeArea,
}
/*
LIFETIME:

value gets deleted when lifetime reaches 0
deeper scope => lifetime++
shallower scope => lifetime--
*/

impl std::ops::Index<usize> for ValStorage {
    type Output = Value;

    fn index(&self, i: usize) -> &Self::Output {
        &self
            .map
            .get(&i)
            .unwrap_or_else(|| panic!("index {} not found", i))
            .val
    }
}

impl std::ops::IndexMut<usize> for ValStorage {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.map.get_mut(&i).unwrap().val
    }
}

use std::collections::HashSet;
impl ValStorage {
    pub fn new() -> Self {
        ValStorage {
            map: vec![
                (
                    BUILTIN_STORAGE,
                    StoredValData {
                        val: Value::Builtins,
                        fn_context: Group::new(0),
                        mutable: false,
                        def_area: CodeArea::new(),
                    },
                ),
                (
                    NULL_STORAGE,
                    StoredValData {
                        val: Value::Null,
                        fn_context: Group::new(0),
                        mutable: false,
                        def_area: CodeArea::new(),
                    },
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }

    pub fn set_mutability(&mut self, index: usize, mutable: bool) {
        if !mutable || !matches!(self[index], Value::Macro(_)) {
            (*self.map.get_mut(&index).unwrap()).mutable = mutable;
        }

        match self[index].clone() {
            Value::Array(a) => {
                for e in a {
                    self.set_mutability(e, mutable);
                }
            }
            Value::Dict(a) => {
                for (_, e) in a {
                    self.set_mutability(e, mutable);
                }
            }
            Value::Macro(_) => (),
            _ => (),
        };
    }

    // pub fn get_lifetime(&self, index: usize) -> u16 {
    //     self.map.get(&index).unwrap().lifetime
    // }
}
// pub fn store_value(
//     val: Value,
//     lifetime: u16,
//     globals: &mut Globals,
//     context: &Context,
//     area: CodeArea,
// ) -> StoredValue {
//     let index = globals.val_id;
//     let mutable = !matches!(val, Value::Macro(_));
//     //println!("index: {}, value: {}", index, val.to_str(&globals));
//     (*globals).stored_values.map.insert(
//         index,
//         StoredValData {
//             val,
//             fn_context: context.start_group,
//             mutable,
//             lifetime,
//             def_area: area,
//         },
//     );
//     (*globals).val_id += 1;
//     index
// }

pub fn clone_and_get_value(
    index: usize,
    globals: &mut Globals,
    fn_context: Group,
    constant: bool,
) -> Value {
    let mut old_val = globals.stored_values[index].clone();

    match &mut old_val {
        Value::Array(arr) => {
            old_val = Value::Array(
                arr.iter()
                    .map(|x| clone_value_preserve_area(*x, globals, fn_context, constant))
                    .collect(),
            );
        }

        Value::Dict(arr) => {
            old_val = Value::Dict(
                arr.iter()
                    .map(|(k, v)| {
                        (
                            *k,
                            clone_value_preserve_area(*v, globals, fn_context, constant),
                        )
                    })
                    .collect(),
            );
        }

        Value::Macro(m) => {
            for arg in &mut m.args {
                if let Some(def_val) = &mut arg.1 {
                    (*def_val) = clone_value_preserve_area(*def_val, globals, fn_context, constant);
                }

                if let Some(def_val) = &mut arg.3 {
                    (*def_val) = clone_value_preserve_area(*def_val, globals, fn_context, constant);
                }
            }

            // for (_, v) in m.def_context.variables.iter_mut() {
            //     (*v) = clone_value(*v, lifetime, globals, context, constant)
            // }
        }
        _ => (),
    };

    old_val
}

pub fn clone_value(
    index: usize,
    globals: &mut Globals,
    fn_context: Group,
    constant: bool,
    area: CodeArea,
) -> StoredValue {
    let old_val = clone_and_get_value(index, globals, fn_context, constant);

    //clone all inner values
    //do the thing
    //bing bang
    //profit
    let new_index = globals.val_id;
    //println!("1index: {}, value: {}", new_index, old_val.to_str(&globals));

    (*globals).stored_values.map.insert(
        new_index,
        StoredValData {
            val: old_val,
            fn_context,
            mutable: !constant,
            def_area: area,
        },
    );
    (*globals).val_id += 1;
    new_index
}

pub fn clone_value_preserve_area(
    index: usize,
    globals: &mut Globals,
    fn_context: Group,
    constant: bool,
) -> StoredValue {
    let old_val = clone_and_get_value(index, globals, fn_context, constant);

    //clone all inner values
    //do the thing
    //bing bang
    //profit
    let new_index = globals.val_id;
    //println!("1index: {}, value: {}", new_index, old_val.to_str(&globals));

    (*globals).stored_values.map.insert(
        new_index,
        StoredValData {
            val: old_val,
            fn_context,
            mutable: !constant,

            def_area: globals.get_area(index),
        },
    );
    (*globals).val_id += 1;
    new_index
}

// pub fn clone_value_to(
//     index: usize,
//     to: usize,
//     lifetime: u16,
//     globals: &mut Globals,
//     fn_context: Group,
//     constant: bool,
//     area: CodeArea,
// ) {
//     let old_val = clone_and_get_value(index, lifetime, globals, fn_context, constant, area.clone());

//     //clone all inner values
//     //do the thing
//     //bing bang
//     //profit
//     let new_index = to;
//     //println!("1index: {}, value: {}", new_index, old_val.to_str(&globals));

//     (*globals).stored_values.map.insert(
//         new_index,
//         StoredValData {
//             val: old_val,
//             fn_context,
//             mutable: !constant,
//             lifetime,
//             def_area: area,
//         },
//     );
// }

pub fn store_const_value(
    val: Value,
    globals: &mut Globals,
    fn_context: Group,
    area: CodeArea,
) -> StoredValue {
    let index = globals.val_id;
    // println!(
    //     "2index: {}, value: {}, area: {:?}",
    //     index,
    //     val.to_str(&globals),
    //     area
    // );

    (*globals).stored_values.map.insert(
        index,
        StoredValData {
            val,
            fn_context,
            mutable: false,
            def_area: area,
        },
    );
    (*globals).val_id += 1;
    index
}

pub fn store_val_m(
    val: Value,
    globals: &mut Globals,
    fn_context: Group,
    constant: bool,
    area: CodeArea,
) -> StoredValue {
    let index = globals.val_id;

    (*globals).stored_values.map.insert(
        index,
        StoredValData {
            val,
            fn_context,
            mutable: !constant,
            def_area: area,
        },
    );
    (*globals).val_id += 1;
    index
}
