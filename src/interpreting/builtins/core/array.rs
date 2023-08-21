use crate::interpreting::builtins::impl_type;
use crate::interpreting::error::RuntimeError;
use crate::interpreting::multi::Multi;
use crate::interpreting::value::{Value, ValueType};
use crate::interpreting::vm::{DeepClone, ValueRef};
use crate::util::index_wrap;

impl_type! {
    impl Array {
        Constants:

        Functions(ctx, vm, program, area):


        /// fghfddggfd
        fn push(&Array(slf) as "self", elem) {
            slf.borrow_mut().push(vm.deep_clone_ref(elem, false));

            Multi::new_single(ctx, Ok(Value::Empty.into_value_ref(area)))
        }

        /// fgdfgdfg
        fn is_empty(Array(slf) as "self") {
            let is_empty = slf.borrow().is_empty();
            Multi::new_single(ctx, Ok(Value::Bool(is_empty).into_value_ref(area)))
        }

        /// lolool
        fn reverse(&Array(slf) as "self") {
            slf.borrow_mut().reverse();
            Multi::new_single(ctx, Ok(Value::Empty.into_value_ref(area)))
        }

        /// lolool
        fn clear(&Array(slf) as "self") {
            slf.borrow_mut().clear();
            Multi::new_single(ctx, Ok(Value::Empty.into_value_ref(area)))
        }

        /// lolool
        fn pop(&Array(slf) as "self") {
            let elem = slf.borrow_mut().pop().map(|a| Value::Maybe(Some(a))).unwrap_or(Value::Maybe(None));
            Multi::new_single(ctx, Ok(elem.into_value_ref(area)))
        }

        /// lols
        fn remove(&Array(slf) as "self", Int(index) as "index") {
            let len = slf.borrow().len();

            let idx = match index_wrap(*index.borrow(), len, ValueType::Array, &area, vm) {
                Ok(idx) => idx,
                Err(e) => return Multi::new_single(ctx, Err(e))
            };

            let elem = slf.borrow_mut().remove(idx);
            Multi::new_single(ctx, Ok(elem))
        }

        ///ldlw
        fn shift(&Array(slf) as "self") {
            let len = slf.borrow().len();
            let elem = slf.borrow_mut().remove(match index_wrap(0, len, ValueType::Array, &area, vm) {
                Ok(idx) => idx,
                Err(e) => return Multi::new_single(ctx, Ok(Value::Maybe(None).into_value_ref(area)))
            });
            Multi::new_single(ctx, Ok(Value::Maybe(Some(elem)).into_value_ref(area)))
        }

        /// sds
        fn unshift(&Array(slf) as "self", elem) {
            slf.borrow_mut().insert(0, vm.deep_clone_ref(elem, false));
            Multi::new_single(ctx, Ok(Value::Empty.into_value_ref(area)))
        }

        /// sdhefuh
        fn insert(&Array(slf) as "self", Int(index) as "index", elem) {
            let len = slf.borrow().len();

            let idx = match index_wrap(*index.borrow(), len, ValueType::Array, &area, vm) {
                Ok(idx) => idx,
                Err(e) => return Multi::new_single(ctx, Err(e))
            };

            slf.borrow_mut().insert(idx, vm.deep_clone_ref(elem, false));
            Multi::new_single(ctx, Ok(Value::Empty.into_value_ref(area)))
        }

        /// dwhud
        fn map(Array(slf) as "self", func: Macro) {

            let arr = slf.borrow().clone();
            let mut ret = Multi::new_single(ctx, Ok(vec![]));

            for elem in arr {
                ret = ret.try_flat_map(|ctx, v| {

                    let r = vm.call_value(
                        ctx,
                        func.get_ref().clone(),
                        &[(elem.clone(), false)],
                        &[],
                        area.clone(),
                        program
                    );

                    r.try_map(|ctx, new_elem| {
                        let mut v = v.clone();
                        v.push(new_elem);
                        (ctx, Ok(v))
                    })
                });
            }

            ret.try_map(|ctx, v| (ctx, Ok(Value::Array(v).into_value_ref(area.clone()))))
        }


        /// swuhd
        fn all(Array(slf) as "self", func: Macro) {
            let arr = slf.borrow().clone();
            let mut ret = Multi::new_single(ctx, Ok(true));

            for elem in arr {
                ret = ret.try_flat_map(|ctx, b| {
                    if !b {
                        return Multi::new_single(ctx, Ok(false))
                    }

                    let result = vm.call_value(
                        ctx,
                        func.get_ref().clone(),
                        &[(elem.clone(), false)],
                        &[],
                        area.clone(),
                        program
                    );

                    result.try_map(|ctx, v| {
                        let mut b = b;

                        let r = v.borrow();

                        let b2 = match &r.value {
                            Value::Bool(b) => *b,
                            other => return (ctx, Err(RuntimeError::TypeMismatch {
                                value_type: other.get_type(),
                                value_area: r.area.clone(),
                                area: area.clone(),
                                expected: &[ValueType::Bool],
                                call_stack: vm.get_call_stack(),
                            }))
                        };

                        std::mem::drop(r);

                        (ctx, Ok(b && b2))
                    })
                });
            }

            ret.try_map(|ctx, v| (ctx, Ok(Value::Bool(v).into_value_ref(area.clone()))))
        }

        /// swuhd
        fn any(Array(slf) as "self", func: Macro) {
            let arr = slf.borrow().clone();
            let mut ret = Multi::new_single(ctx, Ok(false));

            for elem in arr {
                ret = ret.try_flat_map(|ctx, b| {
                    if b {
                        return Multi::new_single(ctx, Ok(true))
                    }

                    let result = vm.call_value(
                        ctx,
                        func.get_ref().clone(),
                        &[(elem.clone(), false)],
                        &[],
                        area.clone(),
                        program
                    );

                    result.try_map(|ctx, v| {
                        let mut b = b;

                        let r = v.borrow();

                        let b2 = match &r.value {
                            Value::Bool(b) => *b,
                            other => return (ctx, Err(RuntimeError::TypeMismatch {
                                value_type: other.get_type(),
                                value_area: r.area.clone(),
                                area: area.clone(),
                                expected: &[ValueType::Bool],
                                call_stack: vm.get_call_stack(),
                            }))
                        };

                        std::mem::drop(r);

                        (ctx, Ok(b || b2))
                    })
                });
            }

            ret.try_map(|ctx, v| (ctx, Ok(Value::Bool(v).into_value_ref(area.clone()))))
        }

        /// dds
        ///
        fn filter(Array(slf) as "self", func: Macro) {
            let arr = slf.borrow().clone();
            let mut ret = Multi::new_single(ctx, Ok(vec![]));

            for elem in arr {
                ret = ret.try_flat_map(|ctx, vector| {

                    let result = vm.call_value(
                        ctx,
                        func.get_ref().clone(),
                        &[(elem.clone(), false)],
                        &[],
                        area.clone(),
                        program
                    );

                    result.try_map(|ctx, v| {
                        let mut vector = vector.clone();

                        let r = v.borrow();

                        let b = match &r.value {
                            Value::Bool(b) => *b,
                            other => return (ctx, Err(RuntimeError::TypeMismatch {
                                value_type: other.get_type(),
                                value_area: r.area.clone(),
                                area: area.clone(),
                                expected: &[ValueType::Bool],
                                call_stack: vm.get_call_stack(),
                            }))
                        };

                        std::mem::drop(r);

                        if b {
                            vector.push(elem.clone());
                        }

                        (ctx, Ok(vector))
                    })
                });
            }

            ret.try_map(|ctx, v| (ctx, Ok(Value::Array(v).into_value_ref(area.clone()))))
        }

    }
}
