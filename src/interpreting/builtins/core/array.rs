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
        fn push(&mut Array(slf) as "self", elem) {
            slf.borrow_mut().push(vm.deep_clone_ref(elem));

            Multi::new_single(ctx, Ok(ValueRef::new(Value::Empty.into_stored(area))))
        }

        /// fgdfgdfg
        fn is_empty(&Array(slf) as "self") {
            let is_empty = slf.borrow().is_empty();
            Multi::new_single(ctx, Ok(ValueRef::new(Value::Bool(is_empty).into_stored(area))))
        }

        /// lolool
        fn reverse(&mut Array(slf) as "self") {
            slf.borrow_mut().reverse();
            Multi::new_single(ctx, Ok(ValueRef::new(Value::Empty.into_stored(area))))
        }

        /// lolool
        fn clear(&mut Array(slf) as "self") {
            slf.borrow_mut().clear();
            Multi::new_single(ctx, Ok(ValueRef::new(Value::Empty.into_stored(area))))
        }

        /// lolool
        fn pop(&mut Array(slf) as "self") {
            let elem = slf.borrow_mut().pop().map(|a| Value::Maybe(Some(a))).unwrap_or(Value::Maybe(None));
            Multi::new_single(ctx, Ok(ValueRef::new(elem.into_stored(area))))
        }

        /// lols
        fn remove(&mut Array(slf) as "self", Int(index) as "index") {
            let len = slf.borrow().len();

            let idx = match index_wrap(*index.borrow(), len, ValueType::Array, &area, vm) {
                Ok(idx) => idx,
                Err(e) => return Multi::new_single(ctx, Err(e))
            };

            let elem = slf.borrow_mut().remove(idx);
            Multi::new_single(ctx, Ok(elem))
        }

        ///ldlw
        fn shift(&mut Array(slf) as "self") {
            let len = slf.borrow().len();
            let elem = slf.borrow_mut().remove(match index_wrap(0, len, ValueType::Array, &area, vm) {
                Ok(idx) => idx,
                Err(e) => return Multi::new_single(ctx, Ok(ValueRef::new(Value::Maybe(None).into_stored(area))))
            });
            Multi::new_single(ctx, Ok(ValueRef::new(Value::Maybe(Some(elem)).into_stored(area))))
        }

        /// sds
        fn unshift(&mut Array(slf) as "self", elem) {
            slf.borrow_mut().insert(0, vm.deep_clone_ref(elem));
            Multi::new_single(ctx, Ok(ValueRef::new(Value::Empty.into_stored(area))))
        }

        /// sdhefuh
        fn insert(&mut Array(slf) as "self", Int(index) as "index", elem) {
            let len = slf.borrow().len();

            let idx = match index_wrap(*index.borrow(), len, ValueType::Array, &area, vm) {
                Ok(idx) => idx,
                Err(e) => return Multi::new_single(ctx, Err(e))
            };

            slf.borrow_mut().insert(idx, vm.deep_clone_ref(elem));
            Multi::new_single(ctx, Ok(ValueRef::new(Value::Empty.into_stored(area))))
        }

        /// dwhud
        fn map(&Array(slf) as "self", func: Macro) {

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

            ret.try_map(|ctx, v| (ctx, Ok(ValueRef::new(Value::Array(v).into_stored(area.clone())))))
        }


        /// swuhd
        fn all(&Array(slf) as "self", func: Macro) {
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

            ret.try_map(|ctx, v| (ctx, Ok(ValueRef::new(Value::Bool(v).into_stored(area.clone())))))
        }

        /// swuhd
        fn any(&Array(slf) as "self", func: Macro) {
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

            ret.try_map(|ctx, v| (ctx, Ok(ValueRef::new(Value::Bool(v).into_stored(area.clone())))))
        }

        /// dds
        ///
        fn filter(&Array(slf) as "self", func: Macro) {
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

            ret.try_map(|ctx, v| (ctx, Ok(ValueRef::new(Value::Array(v).into_stored(area.clone())))))
        }

    }
}
