use crate::vm::builtins::builtin_utils::impl_type;
use crate::vm::error::RuntimeError;
use crate::vm::value::{StoredValue, Value};

impl_type! {
    impl String {
        Constants:

        Functions(vm, call_area):


        // todo: better time complexiyt lol
        /// Returns `true` if the string contains the given substring (`O(nm)` time complexity)
        fn contains(String(s) as self, String(substr) as substr) {
            for i in 0..s.len() {
                let mut j = i;
                let mut k = 0;
                while substr[k] == s[j] {
                    j += 1;
                    k += 1;
                    if k == substr.len() {
                        return Ok(Value::Bool(true));
                    }
                }
            }
            Value::Bool(false)
        }

        /// Returns `true`  if the string ends with the given suffix
        fn ends_with(String(s) as self, String(suffix) as suffix) {
            Value::Bool(s.ends_with(&suffix))
        }

        /// Returns `true`  if the string starts with the given prefix
        fn starts_with(String(s) as self, String(prefix) as prefix) {
            Value::Bool(s.starts_with(&prefix))
        }

        // todo: better time complexiyt lol
        /// Returns the index of the first occurrence of the given substring (`O(nm)` time complexity)
        fn index(String(s) as self, String(substr) as substr) {
            for i in 0..s.len() {
                let mut j = i;
                let mut k = 0;
                while substr[k] == s[j] {
                    j += 1;
                    k += 1;
                    if k == substr.len() {
                        return Ok(Value::Maybe(Some(vm.memory.insert(StoredValue {
                            value: Value::Int(i as i64),
                            area: call_area
                        }))));
                    }
                }
            }
            Value::Maybe(None)
        }

        /// Returns `true` if the string is numeric
        fn is_digit(String(s) as self) {
            Value::Bool(s.iter().all(|c| c.is_ascii_digit()))
        }

        /// Returns `true` if the string is empty
        fn is_empty(String(s) as self) {
            Value::Bool(s.is_empty())
        }

        /// Returns `true` if the string is lowercase
        fn is_lower(String(s) as self) {
            Value::Bool(s.iter().all(|c| c.is_lowercase()))
        }

        /// Returns `true` if the string is uppercase
        fn is_upper(String(s) as self) {
            Value::Bool(s.iter().all(|c| c.is_uppercase()))
        }

        /// Returns the whole string in lowercase
        fn lowercase(String(s) as self) {
            Value::String(s.iter().flat_map(|c| c.to_lowercase()).collect())
        }

        /// returns the whole string in uppercase
        fn uppercase(String(s) as self) {
            Value::String(s.iter().flat_map(|c| c.to_uppercase()).collect())
        }

        /// Returns the string reversed
        fn reversed(String(s) as self) {
            Value::String(s.iter().rev().cloned().collect())
        }

        // TODO: multiple seprator????
        /// Splits the string by the specified seperator.
        fn split(String(s) as self, String(sep) as sep) {
            let s: String = s.into_iter().collect();
            let sep: String = sep.into_iter().collect();

            Value::Array(
                s.split(&sep).map(|s| {
                    vm.memory.insert(
                        StoredValue {
                            value: Value::String(s.chars().collect()),
                            area: call_area.clone(),
                        }
                    )
                }).collect()
            )
        }


        /// Gets a substring beginning at the specified start and ending at the specified end.
        fn substr(String(s) as self, Int(start) as start if (>0), Int(end) as end if (>0)) { // crazy
            if start > end {
                return Err(todo!());
            }
            if start as usize > s.len() {
                return Err(RuntimeError::IndexOutOfBounds {
                    len: s.len(),
                    index: start,
                    area: call_area,
                    typ: crate::vm::value::ValueType::String,
                    call_stack: vm.get_call_stack(),
                });
            }
            if end as usize > s.len() {
                return Err(RuntimeError::IndexOutOfBounds {
                    len: s.len(),
                    index: end,
                    area: call_area,
                    typ: crate::vm::value::ValueType::String,
                    call_stack: vm.get_call_stack(),
                });
            }
            Value::String(s[start as usize..end as usize].to_vec())
        }

        // todo: add custom arg from trimming (maybe new builtin??)
        /// Returns a string slice with leading and trailing whitespace removed.
        fn trim(String(s) as self) {
            let s: String = s.into_iter().collect();
            Value::String(s.trim().chars().collect())
        }

    }
}
