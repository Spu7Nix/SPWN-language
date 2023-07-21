use std::cell::RefCell;
use std::rc::Rc;

use crate::compiling::bytecode::{CallExpr, Register};
use crate::interpreting::builtins::core::runtime_display;
// use crate::interpreting::builtins::core::RUNTIME_DISPLAY_VALUEREF;
use crate::interpreting::builtins::{impl_type, raw_macro, Instrs, RustFnInstr};
use crate::interpreting::context::CallInfo;
use crate::interpreting::value::{BuiltinClosure, MacroData, MacroTarget, Value};
use crate::interpreting::vm::{FuncCoord, LoopFlow, ValueRef};

impl_type! {
    impl Array {
        Constants:
        /// tEst
        const A = Int(2);

        Functions(vm, program, area):
        /// dsf "fuck"
        fn push(&mut Array(v) as "self", bink) {
            v.borrow_mut().push(bink.clone());
            Value::Empty
        }

        /// dsf "fuck"
        fn map(Array(v) as "self", f: Macro) {
            let len = v.borrow().len();
            Instrs(&[
                &|vm| {
                    vm.context_stack.current_mut().extra_stack.push(
                        Value::Array(vec![]).into_stored(area.clone())
                    );
                    vm.context_stack.current_mut().extra_stack.push(
                        Value::Int(0).into_stored(area.clone())
                    );
                    Ok(LoopFlow::Normal)
                },
                &|vm| {
                    let Value::Int(idx) = vm.context_stack.current_mut().extra_stack[1].value else {
                        unreachable!()
                    };
                    let idx = idx as usize;
                    if idx >= len {
                        vm.context_stack.last_mut().jump_current(3);
                    } else {
                        vm.call_macro(
                            f.get_ref().clone(),
                            &CallExpr { dest: None, positional: Box::new([v.borrow()[idx].clone()]), named: Box::new([]) },
                            program,
                            area.clone()
                        )?;
                    };
                    Ok(LoopFlow::ContinueLoop)
                },
                &|vm| {
                    {
                        let elem = vm.context_stack.current_mut().extra_stack.pop().unwrap();
                        let Value::Array(arr) = &mut vm.context_stack.current_mut().extra_stack[0].value else {
                            unreachable!()
                        };
                        arr.push(ValueRef::new(elem));
                    }
                    {
                        let Value::Int(n) = &mut vm.context_stack.current_mut().extra_stack[1].value else {
                            unreachable!()
                        };
                        *n += 1;
                    }

                    vm.context_stack.last_mut().jump_current(1);
                    Ok(LoopFlow::ContinueLoop)
                },
                &|vm| {
                    vm.context_stack.current_mut().extra_stack.pop().unwrap();
                    Ok(LoopFlow::Normal)
                },
            ])
        }

        /// display
        fn _display_(Array(slf) as "self") -> String {
            let len = slf.borrow().len();
            // println!("clock");
            Instrs(&[
                &|vm| {
                    // println!("bitch");
                    vm.context_stack.current_mut().push_extra_stack(
                        Value::String(Rc::new(['['])).into_stored(area.clone())
                    );

                    vm.context_stack.current_mut().push_extra_stack(
                        Value::Int(0).into_stored(area.clone())
                    );
                    Ok(LoopFlow::Normal)
                },
                &|vm| {
                    let Value::Int(idx) = vm.context_stack.current_mut().extra_stack_top(0).value else {
                        unreachable!()
                    };
                    let idx = idx as usize;
                    if idx >= len {
                        vm.context_stack.last_mut().jump_current(3);
                    } else {
                        // println!("clack");
                        runtime_display(
                            vec![slf.borrow()[idx].clone()],
                            vm,
                            program,
                            area.clone(),
                        )?;
                        // println!("pinois");
                    };
                    Ok(LoopFlow::ContinueLoop)
                },
                &|vm| {
                    let mut curr = vm.context_stack.current_mut();
                    {
                        let Value::String(elem) = curr.pop_extra_stack().unwrap().value else {
                            unreachable!()
                        };
                        let Value::String(out) = &mut curr.extra_stack_top_mut(1).value else {
                            unreachable!()
                        };
                        *out = out.iter().chain(elem.iter()).copied().chain(", ".chars()).collect();
                    }
                    {
                        let Value::Int(n) = &mut curr.extra_stack_top_mut(0).value else {
                            unreachable!()
                        };
                        *n += 1;
                    }
                    std::mem::drop(curr);

                    vm.context_stack.last_mut().jump_current(1);
                    Ok(LoopFlow::ContinueLoop)
                },
                &|vm| {
                    vm.context_stack.current_mut().pop_extra_stack().unwrap();
                    let mut curr = vm.context_stack.current_mut();

                    let Value::String(out) = &mut curr.extra_stack_top_mut(0).value else {
                        unreachable!()
                    };

                    *out = out[0..(out.len() - 2)].into();
                    *out = out.iter().copied().chain("]".chars()).collect();

                    Ok(LoopFlow::Normal)
                },
            ])
        }

        /// panic
        fn boinky(Int(n) as "boinke") {
            let x = *n.borrow();
            let mut n = 0i64;

            raw_macro! { let testicle = (String(s) as "bick") {
                n += x;
                println!("{}: {}", s.borrow().iter().collect::<String>(), n);

                Value::Int(n * 2)
            } vm program area }

            Value::Macro(MacroData {
                target: MacroTarget::FullyRust {
                    fn_ptr: BuiltinClosure(Rc::new(RefCell::new(testicle))),
                    args: Box::new(["bick".into()]),
                    spread_arg: None
                },

                defaults: Box::new([]),
                self_arg: None,

                is_method: false,
            })
        }

        /// dghdfgjfhgdfhgjhfd
        fn _iter_(Array(slf) as "self") {
            let mut n = 0usize;

            let arr = slf.borrow().clone();

            raw_macro! { let next = () {
                let ret = if n >= arr.len() {
                    Value::Maybe(None)
                } else {
                    let n = arr[n].clone();
                    Value::Maybe(Some(n))
                };
                n += 1;

                ret
            } vm program area }

            Value::Iterator(MacroData {
                target: MacroTarget::FullyRust {
                    fn_ptr: BuiltinClosure::new(next),
                    args: Box::new([]),
                    spread_arg: None
                },

                defaults: Box::new([]),
                self_arg: None,

                is_method: false,
            })
        }

    }
}
