#![allow(unused_variables)]

use std::collections::HashMap;

use crate::compilation::code::{Code, ConstID, InstrNum, KeysID, MacroBuildID, VarID};
use crate::sources::CodeSpan;
use crate::vm::context::SkipMode;
use crate::vm::value::{Argument, Macro, Pattern};

use super::context::{FullContext, ReturnType};
use super::error::RuntimeError;
use super::interpreter::{run_func, Globals};
use super::value::{value_ops, Value};

macro_rules! run_helper {
    ($context:ident, $globals:ident, $data:ident) => {
        #[allow(unused_macros)]
        #[allow(unused_macros)]
        macro_rules! pop {
            (Key) => {
                $context.inner().stack.pop().unwrap()
            };
            (Ref) => {
                &$globals.memory[$context.inner().stack.pop().unwrap()]
            };
            (Shallow) => {
                $globals.memory[$context.inner().stack.pop().unwrap()].clone()
            };
            (Shallow Store) => {
                $globals
                    .memory
                    .insert($globals.memory[$context.inner().stack.pop().unwrap()].clone())
            };
            (Deep) => {{
                let val = $globals.memory[$context.inner().stack.pop().unwrap()].clone();
                val.deep_clone($globals)
            }};
            (Deep Store) => {{
                $globals.key_deep_clone($context.inner().stack.pop().unwrap())
            }};
        }

        #[allow(unused_macros)]
        macro_rules! push {
            ($v:expr) => {{
                let key = $globals.memory.insert($v);
                $context.inner().stack.push(key);
            }};
            (Key $v:expr) => {{
                $context.inner().stack.push($v);
            }};
        }

        #[allow(unused_macros)]
        macro_rules! store {
            ($v:expr) => {
                $globals.memory.insert($v)
            };
        }

        #[allow(unused_macros)]
        macro_rules! area {
            () => {
                $data.code.source.area($data.span)
            };
        }
    };
}

// data passedd into an instruction function
pub struct InstrData<'a> {
    pub code: &'a Code,
    pub span: CodeSpan,
}

pub fn run_load_const(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
    id: ConstID,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    push!(data.code.const_register[id].to_value().into_stored(area!()));
    Ok(())
}
use paste::paste;
macro_rules! op_helper {
    ($($op_fn:ident),*) => {
        $(
            paste! {
                pub fn [<run_ $op_fn>](
                    globals: &mut Globals,
                    data: &InstrData,
                    context: &mut FullContext,
                ) -> Result<(), RuntimeError> {
                    run_helper!(context, globals, data);
                    let b = pop!(Ref);
                    let a = pop!(Ref);
                    let result = value_ops::$op_fn(a, b, area!(), globals)?;
                    push!(result);
                    Ok(())
                }
            }
        )*
    };
}

op_helper! { plus, minus, mult, div, modulo, pow, eq, neq, gt, gte, lt, lte }

pub fn run_negate(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    let v = pop!(Ref);
    let result = value_ops::unary_negate(v, area!())?;
    push!(result);
    Ok(())
}

pub fn run_not(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    let v = pop!(Ref);
    let result = value_ops::unary_not(v, area!())?;
    push!(result);
    Ok(())
}

pub fn run_load_var(
    _globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
    id: VarID,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    match context.inner().vars[id.0 as usize] {
        Some(k) => push!(Key k),
        None => return Err(RuntimeError::UndefinedVariable { area: area!() }),
    }
    Ok(())
}

pub fn run_set_var(
    _globals: &mut Globals,
    _data: &InstrData,
    context: &mut FullContext,
    id: VarID,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    let k = pop!(Key);
    context.inner().vars[id.0 as usize] = Some(k);
    Ok(())
}

pub fn run_build_array(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
    len: InstrNum,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    let mut items = vec![];
    for _ in 0..len.0 {
        items.push(pop!(Key));
    }
    items.reverse();
    push!(Value::Array(items).into_stored(data.code.source.area(data.span)));
    Ok(())
}

pub fn run_build_dict(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
    keys_id: KeysID,
) -> Result<(), RuntimeError> {
    todo!()
}

pub fn run_jump(
    _globals: &mut Globals,
    _data: &InstrData,
    context: &mut FullContext,
    pos: InstrNum,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    context.inner().pos = pos.0 as isize - 1;
    Ok(())
}

pub fn run_jump_if_false(
    globals: &mut Globals,
    _data: &InstrData,
    context: &mut FullContext,
    pos: InstrNum,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    let v = &globals.memory[pop!(Key)];
    if !value_ops::to_bool(v)? {
        context.inner().pos = pos.0 as isize - 1;
    }
    Ok(())
}

pub fn run_pop_top(
    _globals: &mut Globals,
    _data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    pop!(Key);
    Ok(())
}

pub fn run_push_empty(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    push!(Value::Empty.into_stored(area!()));
    Ok(())
}

pub fn run_wrap_maybe(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    let top = pop!(Deep Store);
    push!(Value::Maybe(Some(top)).into_stored(area!()));
    Ok(())
}

pub fn run_push_none(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    push!(Value::Maybe(None).into_stored(area!()));
    Ok(())
}

pub fn run_trigger_func_call(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    todo!()
}

pub fn run_push_trigger_fn(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    todo!()
}

pub fn run_print(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    println!(
        "{}",
        ansi_term::Color::Green
            .bold()
            .paint(format!("{:?}", pop!(Ref).value))
    );
    Ok(())
}

pub fn run_to_iter(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    todo!()
}

pub fn run_iter_next(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
    pos: InstrNum,
) -> Result<(), RuntimeError> {
    todo!()
}

pub fn run_build_macro(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
    build: MacroBuildID,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);

    let (func_id, arg_info) = data.code.macro_build_register[build].clone();
    let mut args = vec![];
    let ret_pattern = pop!(Deep Store);

    for (name, has_type, has_default) in arg_info.iter().rev() {
        let default = if *has_default {
            Some(pop!(Deep Store))
        } else {
            None
        };
        let pattern = if *has_type {
            Some(pop!(Deep Store))
        } else {
            None
        };
        args.push(Argument { default, pattern })
    }
    args.reverse();

    let mut captured = HashMap::new();
    for i in &data.code.funcs[func_id].capture_ids {
        match &mut context.inner().vars[i.0 as usize] {
            Some(k) => {
                captured.insert(*i, *k);
            }
            var @ None => {
                let k = store!(Value::Empty.into_stored(area!()));
                *var = Some(k);
                captured.insert(*i, k);
            }
        }
    }

    push!(Value::Macro(Macro {
        func_id,
        captured,
        args,
        ret_pattern
    })
    .into_stored(area!()));
    Ok(())
}

pub fn run_push_any_pattern(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    push!(Value::Pattern(Pattern::Any).into_stored(area!()));
    Ok(())
}

pub fn run_impl(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
    keys_id: KeysID,
) -> Result<(), RuntimeError> {
    todo!()
}

pub fn run_call(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
    passed_args: InstrNum,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);

    let v = pop!(Shallow);
    match &v.value {
        Value::Macro(m) => {
            let idx = m.func_id;
            let passed_args = passed_args.0 as usize;
            if passed_args > m.args.len() {
                return Err(RuntimeError::TooManyArguments {
                    expected: m.args.len(),
                    provided: passed_args,
                    call_area: area!(),
                    func_area: v.def_area.clone(),
                });
            }
            let mut arg_values = vec![None; m.args.len()];

            // set defaults
            for (i, arg) in m.args.iter().enumerate() {
                if let Some(default) = arg.default {
                    arg_values[i] = Some(default)
                }
            }

            // set positional
            for i in 0..passed_args {
                let val = pop!(Deep Store);
                arg_values[m.args.len() - 1 - i] = Some(val);
            }

            // apply
            for (i, var_id) in data.code.funcs[idx].arg_ids.iter().enumerate() {
                // set variable
                let val = match arg_values[i] {
                    Some(v) => v,
                    None => {
                        return Err(RuntimeError::ArgumentNotSatisfied {
                            arg_name: todo!(),
                            call_area: area!(),
                            arg_area: todo!(),
                        })
                    }
                };
                context.inner().vars[var_id.0 as usize] = Some(val);
            }

            let stored_pos = context.inner().pos;

            run_func(globals, data.code, idx, context)?;

            for context in context.iter(SkipMode::IncludeReturns) {
                match context.inner().returned {
                    Some(ReturnType::Explicit(v)) => {
                        context.inner().stack.push(v);
                        context.inner().returned = None;
                        context.inner().pos = stored_pos;
                    }
                    _ => unreachable!(),
                }
            }
        }
        _ => {
            return Err(RuntimeError::CannotCall {
                base: v.clone(),
                area: v.def_area.clone(),
            })
        }
    }
    Ok(())
}

pub fn run_return(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);
    let val = pop!(Deep Store); // Key?
    context.inner().returned = Some(ReturnType::Explicit(val));
    Ok(())
}

pub fn run_index(
    globals: &mut Globals,
    data: &InstrData,
    context: &mut FullContext,
) -> Result<(), RuntimeError> {
    run_helper!(context, globals, data);

    let index = pop!(Shallow);
    let base = pop!(Shallow);
    match base.value {
        Value::Array(arr) => match index.value {
            Value::Int(n) => push!(Key arr[n as usize]),
            _ => panic!("fuck uu"),
        },
        _ => panic!("fuck u"),
    }
    Ok(())
}
