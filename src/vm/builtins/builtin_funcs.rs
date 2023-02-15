use paste::paste;

use super::builtin_utils::{Invoke, Spread};
use crate::gd::gd_object::{GdObject, Trigger};
use crate::gd::ids::Id;
use crate::parsing::ast::ObjectType;
use crate::sources::CodeArea;
use crate::vm::builtins::builtin_utils::Mut;
use crate::vm::error::RuntimeError;
use crate::vm::interpreter::{RuntimeResult, ValueKey, Vm};
use crate::vm::value::arg_aliases::{ABuiltins, AInt, AString, *};
use crate::vm::value::{BuiltinFn, Value, ValueType};

macro_rules! or {
    ( $($t:ty)|* ) => {
        Or<( $( Option<$t> ),* )>
    };
}

macro_rules! override_names {
    ($($name:ident : [$($func:ident,)*],)*) => {
        paste! {
            impl ValueType {
                pub fn get_override(self, name: &str) -> Option<BuiltinFn> {
                    match self {
                        $(
                            Self::$name => {
                                match name {
                                    $(
                                        stringify!($func) => Some(BuiltinFn(&|args: Vec<ValueKey>, vm, area| {
                                            [<$name:snake>]::$func.invoke(args, vm, area)
                                        })),
                                    )*
                                    _ => None
                                }
                            },
                        )*
                        _ => None,
                    }
                }
            }
        }
    };
}

////////////////////////

pub mod builtins {
    use super::*;

    pub fn print(
        _self: ABuiltins,
        values: Spread<Value>,
        AString(end): AString,
        AString(sep): AString,
        vm: &mut Vm,
    ) -> RuntimeResult<Value> {
        print!(
            "{}{}",
            values
                .iter()
                .map(|v| match v {
                    Value::String(s) => s.iter().collect(),
                    _ => v.runtime_display(vm),
                })
                .collect::<Vec<_>>()
                .join(&sep.iter().collect::<String>()),
            end.iter().collect::<String>(),
        );

        Ok(Value::Empty)
    }

    pub fn add(
        _self: ABuiltins,
        AObject(params, mode): AObject,
        ignore_context: ABool,
        vm: &mut Vm,
        area: CodeArea,
    ) -> RuntimeResult<Value> {
        let obj = GdObject { params, mode };

        match mode {
            ObjectType::Object => {
                if !ignore_context.0 && vm.contexts.group() != Id::Specific(0) {
                    return Err(RuntimeError::AddObjectInTriggerContext {
                        area,
                        call_stack: vm.get_call_stack(),
                    });
                }
                vm.objects.push(obj)
            }
            ObjectType::Trigger => vm.triggers.push(
                Trigger {
                    obj,
                    order: vm.trigger_order_count.next(),
                }
                .apply_context(vm.contexts.group()),
            ),
        }

        Ok(Value::Empty)
    }

    pub fn epsilon(_self: ABuiltins) -> RuntimeResult<Value> {
        Ok(Value::Epsilon)
    }

    pub fn trigger_fn_context(_self: ABuiltins, vm: &mut Vm) -> RuntimeResult<Value> {
        Ok(Value::Group(vm.contexts.group()))
    }
}

pub mod float {
    use super::*;

    pub fn sin(AFloat(slf): AFloat) -> RuntimeResult<Value> {
        Ok(Value::Float(slf.sin()))
    }
}

use crate::vm::value::IteratorData;

pub mod array {
    use super::*;

    pub fn push(slf: Mut<AArray>, elem: ValueKey, vm: &mut Vm) -> RuntimeResult<Value> {
        let val = slf.get_mut(vm);
        val.0.push(elem);

        Ok(Value::Empty)
    }

    pub fn reversed(AArray(mut slf): AArray, vm: &mut Vm) -> RuntimeResult<Value> {
        slf.reverse();
        Ok(Value::Array(slf.iter().map(|k| vm.deep_clone_key_insert(*k)).collect()))
    }

    pub fn iter(array: ValueKey) -> RuntimeResult<Value> {
        Ok(Value::Iterator(IteratorData::Array { array, index: 0 }))
    }
}

pub mod iterator {
    use super::*;
    use crate::vm::value::StoredValue;

    pub fn next((slf, slf_area): (Mut<AIterator>, CodeArea), vm: &mut Vm) -> RuntimeResult<Value> {
        let val = match &vm.memory[slf.key].value {
            Value::Iterator(IteratorData::Array { array, index }) => {
                match &vm.memory[*array].value {
                    Value::Array(values) => values.get(*index).cloned(),
                    _ => todo!(), // maybe add error here incase its mutated???
                }
            } // 🙂

            Value::Iterator(IteratorData::Range { range, index }) => {
                match (range.0..range.1).step_by(range.2).nth(*index) {
                    Some(v) => Some(vm.memory.insert(StoredValue {
                        value: Value::Int(v),
                        area: slf_area,
                    })),
                    None => None,
                }
            }
            // dict string TODO
            _ => unreachable!(),
        };

        let cloned_val = val.map(|v| vm.deep_clone_key_insert(v));

        // increment index
        match slf.get_mut(vm) {
            MutAIterator(IteratorData::Array { index, .. }) => *index += 1,
            MutAIterator(IteratorData::Range { index, .. }) => *index += 1,
            _ => unreachable!(),
        };

        Ok(Value::Maybe(cloned_val))
    }
}

pub mod dict {
    use super::*;
    use crate::vm::interpreter::Visibility;

    pub fn insert(
        slf: Mut<ADict>,
        AString(key): AString,
        elem: ValueKey,
        vm: &mut Vm,
    ) -> RuntimeResult<Value> {
        let key = vm.intern_vec(&key);
        let val = slf.get_mut(vm);
        val.0.insert(key, (elem, Visibility::Public));

        Ok(Value::Empty)
    }
}
/*

1..2..11 -> 1 3 5 7 9 -> 9..2..0
1..3..11 -> 1 4 7 10 -> 10..3..0

11 - 1 - 1 = 9; 9 // 2 = 4; 4 * 2 + 1 = 9
10 - 1 - 1 = 8; 8 // 2 = 4; 4 * 2 + 1 = 9

11 - 1 - 1 = 9; 9 // 3 = 3; 3 * 3 + 1 = 10
12 - 1 - 1 = 10; 10 // 3 = 3; 3 * 3 + 1 = 10
13 - 1 - 1 = 10; 10 // 3 = 3; 3 * 3 + 1 = 10


*/

pub mod range {
    use super::*;

    pub fn reversed(ARange(start, end, step): ARange) -> RuntimeResult<Value> {
        let new_end = (end - 1 - start) / (step as i64) * (step as i64) + start;

        Ok(Value::Range(new_end, start - 1, step))
    }

    pub fn contains(ARange(start, end, step): ARange, AInt(n): AInt) -> RuntimeResult<Value> {
        let contains = (start..end).step_by(step).any(|e| e == n);
        Ok(Value::Bool(contains))
    }
}

pub mod maybe {
    use super::*;

    pub fn unwrap(AMaybe(v): AMaybe) -> RuntimeResult<Value> {
        todo!()
        // Ok(Value::Range(end, start, step))
    }
}

//////////////////////

override_names! {
    Builtins: [
        print,
        add,
        epsilon,
        trigger_fn_context,
    ],
    Float: [
        sin,
    ],
    Array: [
        push,
        iter,
        reversed,
    ],
    Dict: [
        insert,
    ],
    Range: [
        reversed, contains,
    ],
    Iterator: [
        next,
    ],
}
