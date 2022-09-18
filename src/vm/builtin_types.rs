use super::{
    interpreter::{Globals, ValueKey},
    types::TypeBuilder,
    value::{SpwnIterator, Value, ValueType},
};

use crate::sources::CodeArea;
use crate::vm::to_value::ToValueResult;
use crate::vm::types::MethodFunction;
use crate::{attr, method};

use crate::leveldata::object_data::GdObj;

impl Globals {
    pub fn init_types(&mut self) {
        TypeBuilder::new(ValueType::Array)
            .add_member(
                self,
                "length",
                attr!(g, _c, |this: Vec<ValueKey>| { this.len() }),
            )
            .add_method(
                self,
                "push",
                method!(g, c, |mut this: Vec<ValueKey>, ref el| {
                    this.push(el);
                }),
            )
            .add_method(
                self,
                "reverse",
                method!(g, c, |mut this: Vec<ValueKey>| { this.reverse() }),
            )
            .finish_type(self);

        TypeBuilder::new(ValueType::Iterator)
            .add_method(
                self,
                "next",
                method!(g, c, |mut this: SpwnIterator| {
                    let mut output = None;

                    match this {
                        SpwnIterator::Array { data, pos } => {
                            let current = data.get(*pos);
                            if let Some(v) = current {
                                *pos += 1;
                                output = Some(*v);
                            }
                        }
                        SpwnIterator::String { data, pos } => {
                            let current = data.get(*pos);
                            if let Some(v) = current.cloned() {
                                *pos += 1;
                                output = Some(g.memory.insert(
                                    Value::String(v.to_string()).into_stored(CodeArea::internal()),
                                ));
                            }
                        }
                        SpwnIterator::Dict { data, pos } => {
                            let current = data.get(*pos);
                            if let Some(v) = current.cloned() {
                                *pos += 1;
                                let key = v.0;
                                let string_val = g
                                    .memory
                                    .insert(Value::String(key).into_stored(CodeArea::internal()));

                                let arr = g.memory.insert(
                                    Value::Array(vec![string_val, v.1])
                                        .into_stored(CodeArea::internal()),
                                );

                                output = Some(arr)
                            }
                        }
                        _ => todo!(),
                    };

                    output
                }),
            )
            .finish_type(self);

        TypeBuilder::new(ValueType::Builtins)
            .add_method(
                self,
                "print",
                method!(g, c, |val| { println!("{}", val.to_str(g)) }),
            )
            .add_method(self, "add", method!(g, c, |obj: GdObj| {}))
            .add_method(
                self,
                "iter",
                method!(g, c, |val| {
                    match val {
                        Value::Array(arr) => SpwnIterator::Array {
                            data: arr.clone(),
                            pos: 0,
                        },

                        Value::Dict(dict) => SpwnIterator::Dict {
                            data: dict.iter().map(|(a, b)| (a.clone(), b.clone())).collect(),
                            pos: 0,
                        },

                        Value::String(str) => SpwnIterator::String {
                            data: str.chars().collect(),
                            pos: 0,
                        },
                        _ => panic!("No iter implementation: {:?}", val),
                    }
                }),
            )
            .finish_type(self);
    }
}

/*

import "thing.spwn"

Import("thing.spwn")
Code

*/

// fn merge(mut arr: Vec<i32>, left: usize, mid: usize, right: usize) -> Vec<i32> {
//     let n1 = mid - left;
//     let n2 = right - mid;
//     let mut L1 = arr.clone();
//     let mut R1 = arr.clone();
//     let L = &L1[left..mid];
//     let R = &R1[mid..right];
//     /* Merge the temp arrays back into arr[l..r]*/
//     let mut i = 0; // Initial index of first subarray
//     let mut j = 0; // Initial index of second subarray
//     let mut k = left; // Initial index of merged subarray
//     while i < n1 && j < n2 {
//         if L[i] < R[j] {
//             arr[k] = L[i];
//             i = i + 1;
//         } else {
//             arr[k] = R[j];
//             j = j + 1;
//         }
//         k = k + 1;
//     }
//     while i < n1 {
//         arr[k] = L[i];
//         i = i + 1;
//         k = k + 1;
//     }
//     /* Copy the remaining elements of R[], if there
//     are any */
//     while j < n2 {
//         arr[k] = R[j];
//         j = j + 1;
//         k = k + 1;
//     }
//     arr
// }
