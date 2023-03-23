use crate::vm::builtins::builtin_utils::impl_type;
use crate::vm::error::RuntimeError;
use crate::vm::value::{StoredValue, Value};
use crate::vm::value_ops;

impl_type! {
    impl Array {
        Constants:

        Functions(vm, call_area):

        /// returns the index of the first occurence of an element in an array
        // fn index(Array(a) as self, el: _) {
        //     Value::Maybe(a.iter().position(|x| value_ops::equality(&vm.memory[*x].value, &el, vm)).map(|x|
        //         vm.memory.insert(StoredValue {
        //             value: Value::Int(x as i64),
        //             area: call_area
        //         })))
        // }

        fn is_empty(Array(a) as self) {
            Value::Bool(a.is_empty())
        }

        fn join(Array(a) as self, String(sep) as sep) {
            let mut s = Vec::new();
            for (i, el) in a.iter().enumerate() {
                match &vm.memory[*el].value {
                    Value::String(s2) => s.extend(s2),
                    _ => return Err(RuntimeError::TypeMismatch {
                        v: (vm.memory[*el].value.get_type(), vm.memory[*el].area.clone()),
                        area: call_area,
                        expected: crate::vm::value::ValueType::String,
                        call_stack: vm.get_call_stack(),
                    })
                }
                if i != a.len() - 1 {
                    s.extend(sep.clone());
                }
            }
            Value::String(s)
        }

        fn pop(slf: &Array) { // yea what flow said in rust we uses slf instead of self
            let k = slf.get_mut_ref(vm).pop();
            Value::Maybe(k.map(|x| vm.deep_clone_key_insert(x)))
        }

        fn push(slf: &Array, el: ValueKey) {
            let cloned = vm.deep_clone_key_insert(el);
            slf.get_mut_ref(vm).push(cloned);
            Value::Empty
        }

        fn insert(slf: &Array, Int(index) as index, el: ValueKey) {
            let len = (*slf.get_ref(vm)).len();
            if index < 0 || index as usize > len {
                return Err(RuntimeError::IndexOutOfBounds {
                    index,
                    len,
                    area: call_area,
                    typ: crate::vm::value::ValueType::Array,
                    call_stack: vm.get_call_stack(),
                });
            }
            let cloned = vm.deep_clone_key_insert(el);
            slf.get_mut_ref(vm).insert(index as usize, cloned);
            Value::Empty
        }

        fn remove(slf: &Array, Int(index) as index) {
            let len = (*slf.get_ref(vm)).len();
            if index < 0 || index as usize >= len {
                return Err(RuntimeError::IndexOutOfBounds {
                    index,
                    len,
                    area: call_area,
                    typ: crate::vm::value::ValueType::Array,
                    call_stack: vm.get_call_stack(),
                });
            }
            let k = slf.get_mut_ref(vm).remove(index as usize);
            vm.deep_clone_key(k).value
        }

        /// is array EVEN deep clones?????D?D                   j
        fn reversed(Array(a) as self) {
            let mut a = a;
            a.reverse();
            Value::Array(a)
        }

        fn shift(slf: &Array) {
            let k = slf.get_mut_ref(vm).remove(0);
            vm.deep_clone_key(k).value
        }

        fn unshift(slf: &Array, el: ValueKey) {
            let cloned = vm.deep_clone_key_insert(el);
            slf.get_mut_ref(vm).insert(0, cloned);
            Value::Empty
        }

        fn pick(Array(array) as self, quantity: Empty | Int if (>0) = {()}, Bool(duplicates) as allow_duplicates = {false}) {
            use rand::prelude::*;

            let mut rng = rand::thread_rng();

            match quantity {
                QuantityValue::Empty(_) => {
                    array.choose(&mut rng).map_or(Value::Maybe(None), |v| {
                        let value_key =  vm.deep_clone_key_insert(*v);
                        vm.memory[value_key].value.clone()
                    })
                },
                QuantityValue::Int(q) => {
                    if array.is_empty() { return Ok(Value::Array(vec![])) }

                    if duplicates {
                        let mut output = vec![];
                        for i in 0..q.0 {
                            output.push(vm.deep_clone_key_insert(*array.choose(&mut rng).unwrap()))
                        }
                        Value::Array(output)
                    } else {
                        let mut cloned_array: Vec<_> = array.iter().map(|v| vm.deep_clone_key_insert(*v)).collect();
                        let mut output = vec![];
                        for i in 0..q.0.min(cloned_array.len() as i64) {
                            output.push(cloned_array.remove(rng.gen_range(0..cloned_array.len())))
                        }
                        Value::Array(output)
                    }
                },
            }
        }

        fn shuffle(Array(array) as self) {
            use rand::prelude::*;

            let mut rng = rand::thread_rng();

            let mut cloned_array = array.iter().map(|v| vm.deep_clone_key_insert(*v)).collect::<Vec<_>>();
            cloned_array.shuffle(&mut rng);

            Value::Array(cloned_array)
        }
    }
}
