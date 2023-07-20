use std::cell::RefCell;
use std::rc::Rc;

use crate::compiling::bytecode::{CallExpr, Register};
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


        /// panic
        fn boinky(Int(n) as "boinke") {
            let x = *n.borrow();
            let mut n = 0i64;

            raw_macro! { let test = (String(s) as "bick") {
                n += x;
                println!("{}: {}", s.borrow().iter().collect::<String>(), n);

                Value::Empty
            } vm program area }

            Value::Macro(MacroData {
                target: MacroTarget::FullyRust {
                    fn_ptr: BuiltinClosure(Rc::new(RefCell::new(test))),
                    args: Box::new(["bick".into()]),
                    spread_arg: None
                },

                defaults: Box::new([]),
                self_arg: None,

                is_method: false,
            })
        }

        // /// fgfedggggggggggggggggggggggggggg
        // fn dfgsdfg(&mut Array(a) as "self", f: Macro) {

        // }

        // /// dsf "fuck"
        // fn _d(m: Macro) {
        //     &[
        //         |_| {
        //             fhgfgd
        //             fgfedgggggggggggggggggggggggsdfg
        //             dfsgsdfgdf
        //             dfgdfgdfgdfgdd
        //         },
        //         |_| {

        //         },
        //         |_| {

        //         },
        //         |_| {

        //         },
        //     ]

        //     // vm.cacjjkdfhg(
        //     //     // ...
        //     //     || {

        //     //     }
        //     // )

        //     // {
        //     //     let m = m.clone();
        //     //     vm.call_macro(
        //     //         |vm| vm.get_reg_ref(Register(0u8)),
        //     //         &CallExpr {
        //     //             dest: Register(255u8),
        //     //             positional: Box::new([]),
        //     //             named: Box::new([])
        //     //         },
        //     //         program,
        //     //         area,
        //     //     )?;
        //     // }
        //     // vm.run_macro(&CallExpr {base, dest, positional, named}, program, area);
        //     // vm;

        //     // for v in &sinky {
        //     //     println!("{}", v.0.borrow());
        //     // }
        //     // v.borrow_mut().push(bink.clone());
        // }
    }
}

// impl_type! {
//     impl Maybe {
//         Constants:

//         Functions(vm, program, area):
//         /// dsf "fuck"
//         fn map(Maybe(v) as "self", f: Macro) ->  {

//             // let r = match &*v.borrow() {
//             //     Some(v) => {
//             //         vm.call_macro(
//             //             f.get_ref().clone(),
//             //             &CallExpr {
//             //                 dest: Register(255u8),
//             //                 positional: Box::new([]),
//             //                 named: Box::new([]),
//             //             },
//             //             program,
//             //             area,
//             //         )?;
//             //         //
//             //         todo!()
//             //     }
//             //     None => Value::Maybe(None),
//             // };
//             // r
//         }
//     }
// }
