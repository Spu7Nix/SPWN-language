mod array;

// Taking this moment to say that DreamingInsanity is Super cool
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use itertools::Itertools;

use super::{raw_macro, RustFnReturn};
use crate::compiling::bytecode::{CallExpr, Register};
use crate::interpreting::builtins::{impl_type, Instrs, RustFnInstr};
use crate::interpreting::context::CallInfo;
use crate::interpreting::value::{BuiltinClosure, MacroData, MacroTarget, Value};
use crate::interpreting::vm::{FuncCoord, LoopFlow, ValueRef};
use crate::parsing::ast::VisTrait;
use crate::sources::{CodeArea, ZEROSPAN};

raw_macro! {
    fn runtime_display(value) {
        Instrs(&[
            &|vm| {
                match &value.borrow().value {
                    Value::Int(n) => {
                        vm.context_stack.current_mut().extra_stack.push(
                            Value::String(n.to_string().chars().collect_vec().into()).into_stored(area.clone())
                        );
                        Ok(LoopFlow::Normal)
                    },
                    Value::String(s) => {
                        vm.context_stack.current_mut().extra_stack.push(
                            Value::String(s.clone()).into_stored(area.clone())
                        );
                        Ok(LoopFlow::Normal)
                    },
                    Value::Empty => {
                        vm.context_stack.current_mut().extra_stack.push(
                            Value::String("()".chars().collect_vec().into()).into_stored(area.clone())
                        );
                        Ok(LoopFlow::Normal)
                    }
                    v => {
                        // println!("gaagaa");
                        if let Some(map) = vm.impls.get(&v.get_type()) {
                            // println!("labia");
                            if let Some(f) = map.get::<Rc<[char]>>(&"_display_".chars().collect_vec().into()) {
                                // println!("cumcock");
                                vm.call_macro(
                                    f.value().clone(),
                                    &CallExpr {
                                        dest: None,
                                        positional: Box::new([value.clone()]),
                                        named: Box::new([])
                                    },
                                    program,
                                    area.clone(),
                                )?;
                                return Ok(LoopFlow::ContinueLoop)
                            }
                        }
                        todo!();
                    }
                }
            },
        ])
    } vm program area
}

// struct Bum(pub ValueRef);

// unsafe impl Send for Bum {}
// unsafe impl Sync for Bum {}

// lazy_static::lazy_static! {
//     pub static ref RUNTIME_DISPLAY_VALUEREF: Bum = Bum(ValueRef::new(Value::Macro(MacroData {
//         target: MacroTarget::FullyRust {
//             fn_ptr: BuiltinClosure(Rc::new(RefCell::new(runtime_display))),
//             args: Box::new(["value".into()]),
//             spread_arg: None
//         },

//         defaults: Box::new([]),
//         self_arg: None,

//         is_method: false,
//     }).into_stored(CodeArea {
//         span: ZEROSPAN,
//         src: Rc::new(unsafe { std::mem::zeroed() })
//     })));
// }
