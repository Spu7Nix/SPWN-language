use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

use super::contexts::{Context, FullContext};
use super::error::RuntimeError;
// use super::types::{Instance, Type};
use super::value::{value_ops, Value, ValueType};

use crate::compiler::compiler::{Code, InstrNum, Instruction};
use crate::interpreter::value::{Macro, MacroArg, Pattern};
use crate::sources::CodeArea;

new_key_type! {
    pub struct ValueKey;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StoredValue {
    pub value: Value,
    pub def_area: CodeArea,
}

pub struct Globals {
    pub memory: SlotMap<ValueKey, StoredValue>,

    pub types: AHashMap<String, ValueType>,
    // pub types: AHashMap<String, String>,
    //pub instances: AHashMap<Instance, Type>,
}
impl Globals {
    pub fn new() -> Self {
        Self {
            memory: SlotMap::default(),
            types: AHashMap::new(),
            //instances: AHashMap::new(),
        }
    }
    pub fn init(&mut self) {
        // self.types.insert("int".into(), ValueType::Int);
        // self.types.insert("float".into(), ValueType::Float);
        // self.types.insert("string".into(), ValueType::String);
        // self.types.insert("bool".into(), ValueType::Bool);
        // self.types.insert("empty".into(), ValueType::Empty);
        // self.types.insert("array".into(), ValueType::Array);
        // self.types.insert("dictionary".into(), ValueType::Dict);
        // self.types.insert("maybe".into(), ValueType::Maybe);
        // self.types
        //     .insert("type_indicator".into(), ValueType::TypeIndicator);
        // self.types.insert("pattern".into(), ValueType::Pattern);
        // self.types.insert("group".into(), ValueType::Group);
        // self.types
        //     .insert("trigger_function".into(), ValueType::TriggerFunc);
        // self.types.insert("macro".into(), ValueType::Macro);
    }
    pub fn key_deep_clone(&mut self, k: ValueKey) -> ValueKey {
        let val = self.memory[k].clone();
        let val = val.deep_clone(self);
        self.memory.insert(val)
    }
    pub fn deep_clone(&mut self, k: ValueKey) -> StoredValue {
        let val = self.memory[k].clone();
        val.deep_clone(self)
    }
}

pub fn execute_code(globals: &mut Globals, code: &Code) -> Result<(), RuntimeError> {
    let mut contexts = FullContext::single(code.var_count);

    loop {
        let mut finished = true;
        'out_for: for context in contexts.iter() {
            if !context.inner().pos.is_empty() {
                finished = false;
            } else {
                continue;
            }

            let (func, mut i) = *context.inner().pos();

            macro_rules! pop_deep_clone {
                () => {{
                    let val = globals.memory[context.inner().stack.pop().unwrap()].clone();
                    val.deep_clone(globals)
                }};
                (Store) => {{
                    globals.key_deep_clone(context.inner().stack.pop().unwrap())
                }};
            }
            macro_rules! pop_ref {
                () => {
                    &globals.memory[context.inner().stack.pop().unwrap()]
                };
            }
            macro_rules! pop_shallow {
                () => {
                    globals.memory[context.inner().stack.pop().unwrap()].clone()
                };
            }

            macro_rules! push {
                ($v:expr) => {{
                    let key = globals.memory.insert($v);
                    context.inner().stack.push(key);
                }};
            }

            macro_rules! push_store {
                ($v:expr) => {{
                    #[allow(unused_unsafe)]
                    let key = globals.memory.insert($v);
                    context.inner().stack.push(key);
                }};
            }
            macro_rules! store {
                ($v:expr) => {
                    globals.memory.insert($v)
                };
            }

            macro_rules! op_helper {
                (
                    $($instr:ident: $func:ident,)*
                ) => {
                    match &code.bytecode_funcs[func].instructions[i] {
                        $(
                            Instruction::$instr => {
                                let span = code.get_bytecode_span(func, i);
                                let b = pop_ref!();
                                let a = pop_ref!();
                                let key = globals.memory.insert(value_ops::$func(a, b, code.make_area(span), globals)?);
                                context.inner().stack.push(key);
                            }
                        )*
                        _ => (),
                    }
                };
            }

            op_helper! {
                Plus: plus,
                Minus: minus,
                Mult: mult,
                Div: div,
                Mod: modulo,
                Pow: pow,
                Eq: eq,
                NotEq: not_eq,
                Greater: greater,
                GreaterEq: greater_eq,
                Lesser: lesser,
                LesserEq: lesser_eq,
                Is: is_op,
            };

            match &code.bytecode_funcs[func].instructions[i] {
                Instruction::LoadConst(id) => {
                    let span = code.get_bytecode_span(func, i);
                    let key = globals.memory.insert(
                        code.constants
                            .get(*id)
                            .clone()
                            .to_value()
                            .into_stored(code.make_area(span)),
                    );
                    context.inner().stack.push(key);
                }
                Instruction::Negate => {
                    let span = code.get_bytecode_span(func, i);
                    let a = pop_ref!();
                    push_store!(value_ops::unary_negate(a, code.make_area(span))?);
                }
                Instruction::Not => {
                    let span = code.get_bytecode_span(func, i);
                    let a = pop_ref!();
                    push_store!(value_ops::unary_not(a, code.make_area(span))?);
                }
                Instruction::LoadVar(id) => {
                    let a = context.inner().get_var(*id);
                    context.inner().stack.push(a)
                }
                Instruction::SetVar(id) => {
                    let top = pop_deep_clone!();
                    context.inner().set_var(*id, top, globals);
                }
                Instruction::Print => {
                    let top = pop_ref!();
                    println!(
                        "{}",
                        ansi_term::Color::Green
                            .bold()
                            .paint(top.value.to_str(globals))
                    )
                }
                Instruction::LoadType(id) => {
                    let span = code.get_bytecode_span(func, i);
                    let name = code.names.get(*id);
                    match globals.types.get(name) {
                        Some(typ) => {
                            push!(Value::TypeIndicator(*typ).into_stored(code.make_area(span)))
                        }
                        None => {
                            return Err(RuntimeError::UndefinedType {
                                name: name.clone(),
                                area: code.make_area(span),
                            })
                        }
                    }
                }
                Instruction::BuildArray(len) => {
                    let span = code.get_bytecode_span(func, i);
                    let mut elems = vec![];
                    for _ in 0..*len {
                        elems.push(pop_deep_clone!(Store));
                    }
                    elems.reverse();
                    push!(Value::Array(elems).into_stored(code.make_area(span)));
                }
                Instruction::PushEmpty => {
                    let span = code.get_bytecode_span(func, i);
                    push!(Value::Empty.into_stored(code.make_area(span)));
                }
                Instruction::PopTop => {
                    context.inner().stack.pop();
                }
                Instruction::Jump(id) => {
                    i = *code.destinations.get(*id) - 1;
                }
                Instruction::JumpIfFalse(id) => {
                    if !value_ops::to_bool(pop_ref!())? {
                        i = *code.destinations.get(*id) - 1;
                    }
                }
                Instruction::ToIter => todo!(),
                Instruction::IterNext(_) => todo!(),
                Instruction::BuildDict(id) => {
                    let span = code.get_bytecode_span(func, i);
                    let keys = code.name_sets.get(*id);
                    let map = keys
                        .iter()
                        .cloned()
                        .zip((0..keys.len()).map(|_| pop_deep_clone!(Store)))
                        .collect();
                    push!(Value::Dict(map).into_stored(code.make_area(span)));
                }
                Instruction::Return => todo!(),
                Instruction::Continue => todo!(),
                Instruction::Break => todo!(),
                Instruction::MakeMacro(id) => {
                    let span = code.get_bytecode_span(func, i);
                    let arg_spans = code.macro_arg_spans.get(&(func, i)).unwrap();
                    let (func_id, arg_info) = code.macro_build_info.get(*id);
                    let ret_type = pop_deep_clone!(Store);
                    let mut args = vec![];
                    for ((name, typ, def), span) in arg_info.iter().zip(arg_spans) {
                        let def = if *def {
                            Some(pop_deep_clone!(Store))
                        } else {
                            None
                        };
                        let typ = if *typ {
                            Some(pop_deep_clone!(Store))
                        } else {
                            None
                        };
                        args.push(MacroArg {
                            name: name.clone(),
                            area: code.make_area(*span),
                            pattern: typ,
                            default: def,
                        });
                    }
                    args.reverse();
                    // println!("balls {:?}", )
                    let capture = code.bytecode_funcs[*func_id]
                        .capture_ids
                        .iter()
                        .map(|id| context.inner().get_var(*id))
                        .collect::<Vec<_>>();
                    push!(Value::Macro(Macro {
                        func_id: *func_id,
                        args,
                        ret_type,
                        capture,
                    })
                    .into_stored(code.make_area(span)));
                }
                Instruction::PushAnyPattern => {
                    let span = code.get_bytecode_span(func, i);
                    push!(Value::Pattern(Pattern::Any).into_stored(code.make_area(span)));
                }
                Instruction::MakeMacroPattern(_) => todo!(),
                Instruction::Index => {
                    let idx = pop_shallow!();
                    let base = pop_shallow!();
                    match &base.value {
                        Value::Array(arr) => match idx.value {
                            Value::Int(n) => {
                                let k = globals.deep_clone(arr[n as usize]);
                                push!(k);
                            }
                            _ => todo!(),
                        },
                        _ => todo!(),
                    }
                }
                Instruction::Call(id) => {
                    let span = code.get_bytecode_span(func, i);
                    let base = pop_shallow!();
                    match &base.value {
                        Value::Macro(m) => {
                            let param_spans = code.macro_arg_spans.get(&(func, i)).unwrap();
                            let param_list = code.name_sets.get(*id);

                            let mut param_map = AHashMap::new();

                            let mut params = vec![];
                            let mut named_params = vec![];

                            for (name, param_span) in param_list.iter().zip(param_spans) {
                                if name.is_empty() {
                                    params.push((pop_deep_clone!(), param_span));
                                } else {
                                    if let Some(p) =
                                        m.args.iter().position(|MacroArg { name: s, .. }| s == name)
                                    {
                                        param_map.insert(name.clone(), p);
                                    } else {
                                        return Err(RuntimeError::UndefinedArgument {
                                            name: name.into(),
                                            macr: base.clone(),
                                            area: code.make_area(*param_span),
                                        });
                                    }
                                    named_params.push((
                                        name.clone(),
                                        pop_deep_clone!(),
                                        param_span,
                                    ));
                                }
                            }

                            if params.len() > m.args.len() {
                                let call_span = code.get_bytecode_span(func, i);
                                return Err(RuntimeError::TooManyArguments {
                                    expected: m.args.len(),
                                    provided: params.len(),
                                    call_area: code.make_area(call_span),
                                    func: base.clone(),
                                });
                            }

                            let mut arg_fill = m
                                .args
                                .iter()
                                .map(
                                    |MacroArg {
                                         pattern, default, ..
                                     }| {
                                        (
                                            pattern.map(|id| globals.deep_clone(id)),
                                            default.map(|id| globals.deep_clone(id)),
                                        )
                                    },
                                )
                                .collect::<Vec<_>>();
                            params.reverse();
                            named_params.reverse();

                            for (i, (val, param_span)) in params.into_iter().enumerate() {
                                if let Some(pat) = &arg_fill[i].0 {
                                    if !value_ops::matches_pat(&val.value, &value_ops::to_pat(pat)?)
                                    {
                                        return Err(RuntimeError::PatternMismatch {
                                            v: val,
                                            pat: pat.clone(),
                                            area: code.make_area(*param_span),
                                        });
                                    }
                                }
                                arg_fill[i].1 = Some(val);
                            }

                            for (name, val, param_span) in named_params.into_iter() {
                                let arg_pos = param_map[&name];
                                if let Some(pat) = &arg_fill[arg_pos].0 {
                                    if !value_ops::matches_pat(&val.value, &value_ops::to_pat(pat)?)
                                    {
                                        return Err(RuntimeError::PatternMismatch {
                                            v: val,
                                            pat: pat.clone(),
                                            area: code.make_area(*param_span),
                                        });
                                    }
                                }
                                arg_fill[arg_pos].1 = Some(val);
                            }

                            for ((_, arg), MacroArg { name, area, .. }) in
                                arg_fill.iter().zip(&m.args)
                            {
                                if arg.is_none() {
                                    let call_area = code.get_bytecode_span(func, i);
                                    return Err(RuntimeError::ArgumentNotSatisfied {
                                        arg_name: name.clone(),
                                        call_area: code.make_area(call_area),
                                        arg_area: area.clone(),
                                    });
                                }
                            }

                            /*
                                this whole calling part is long and ugly af dont worry will refactor
                            */

                            // context.inner().push_vars();

                            // println!("capture: {:?}", m.capture);
                            // println!("context vars: {:?}", context.inner().vars);
                            // println!("func data: {:#?}", code.bytecode_funcs[func]);

                            context.inner().push_vars(
                                &code.bytecode_funcs[m.func_id].scoped_var_ids,
                                code,
                                globals,
                            );
                            context.inner().push_vars(
                                &code.bytecode_funcs[m.func_id].capture_ids,
                                code,
                                globals,
                            );
                            // println!("context vars 2: {:?}", context.inner().vars);

                            for ((_, arg), id) in arg_fill
                                .into_iter()
                                .zip(&code.bytecode_funcs[m.func_id].arg_ids)
                            {
                                context.inner().set_var(*id, arg.unwrap(), globals);
                            }

                            for (v, id) in m
                                .capture
                                .iter()
                                .zip(&code.bytecode_funcs[m.func_id].capture_ids)
                            {
                                context.inner().replace_var(*id, *v);
                            }

                            context.inner().pos.push((m.func_id, 0));
                            continue;
                        }
                        _ => {
                            return Err(RuntimeError::CannotCall {
                                base: base.clone(),
                                area: code.make_area(span),
                            })
                        }
                    }
                }
                Instruction::TriggerFuncCall => todo!(),
                Instruction::SaveContexts => todo!(),
                Instruction::ReviseContexts => todo!(),
                Instruction::MergeContexts => {}
                Instruction::PushNone => todo!(),
                Instruction::WrapMaybe => todo!(),
                Instruction::PushContextGroup => todo!(),
                Instruction::PopContextGroup => todo!(),
                Instruction::PushTriggerFnValue => todo!(),
                Instruction::TypeDef(_) => todo!(),
                Instruction::Impl(_) => todo!(),
                Instruction::Instance(_) => todo!(),
                Instruction::Split => {
                    let b = pop_deep_clone!(Store);
                    let a = pop_deep_clone!(Store);
                    context.split_context(globals);
                    match context {
                        FullContext::Single(_) => unreachable!(),
                        FullContext::Split(c_a, c_b) => {
                            c_a.inner().stack.push(a);
                            c_b.inner().stack.push(b);
                            c_a.inner().advance_to(code, i + 1);
                            c_b.inner().advance_to(code, i + 1);
                            finished = false;
                            break 'out_for;
                        }
                    }
                }

                Instruction::PushVars(idx) => {
                    let vars = code.scope_vars.get(*idx);
                    context
                        .inner()
                        .push_vars(code.scope_vars.get(*idx), code, globals);
                }
                Instruction::PopVars(idx) => {
                    let vars = code.scope_vars.get(*idx);
                    context.inner().pop_vars(code.scope_vars.get(*idx));
                }

                Instruction::Plus
                | Instruction::Minus
                | Instruction::Mult
                | Instruction::Div
                | Instruction::Mod
                | Instruction::Pow
                | Instruction::Eq
                | Instruction::NotEq
                | Instruction::Greater
                | Instruction::GreaterEq
                | Instruction::Lesser
                | Instruction::LesserEq
                | Instruction::Is => (),
            }

            context.inner().advance_to(code, i + 1);
        }
        if finished {
            break;
        }
    }
    Ok(())
}
