use super::{
    interpreter::{Globals, ValueKey},
    types::TypeBuilder,
    value::{value_ops, Value, ValueType},
};
use crate::sources::CodeArea;
use crate::vm::to_value::ToValueResult;
use crate::{attr, method};

use crate::leveldata::object_data::GdObj;

impl Globals {
    pub fn init_types(&mut self) {
        TypeBuilder::new(ValueType::Array)
            .add_member(self, "length", attr!(g, Value::Array(this) => this.len()))
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
            // .add_method(self, "sort", method!(g, #mut Value::Array(arr) => {
            //     // arr.sort_by(compare)
            //     // merge sort pls copilot
            //     if arr.len() == 0 {
            //         return Ok(Value::Array(arr));
            //     }
            //     let mut arr = arr.clone();
            //     let mut stack = Vec::new();
            //     stack.push((0, arr.len()));
            //     while let Some((start, end)) = stack.pop() {
            //         if end - start <= 1 {
            //             continue;
            //         }
            //         let mid = (start + end) / 2;
            //         stack.push((start, mid));
            //         stack.push((mid, end));
            //     }
            //     while let Some((start, end)) = stack.pop() {
            //         if end - start <= 1 {
            //             continue;
            //         }
            //         let mid = (start + end) / 2;
            //         let mut i = start;
            //         let mut j = mid;
            //         let mut new_arr = Vec::new();
            //         while i < mid && j < end {
            //             let cmp = value_ops::compare(&arr[i], &arr[j]);
            //             if cmp == Ordering::Less {
            //                 new_arr.push(arr[i]);
            //                 i += 1;
            //             } else {
            //                 new_arr.push(arr[j]);
            //                 j += 1;
            //             }
            //         }
            //         while i < mid {
            //             new_arr.push(arr[i]);
            //             i += 1;
            //         }
            //         while j < end {
            //             new_arr.push(arr[j]);
            //             j += 1;
            //         }
            //         arr.splice(start..end, new_arr);
            //     }
            //     Ok(arr)
            // })
            .finish_type(self);

        TypeBuilder::new(ValueType::Builtins)
            .add_method(
                self,
                "print",
                method!(g, c, |val| { println!("{}", val.to_str(g)) }),
            )
            .add_method(
                self,
                "add",
                method!(g, c, |obj: GdObj| {
                    dbg!(obj);
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
