use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

use super::contexts::{Context, FullContext};
use super::error::RuntimeError;
use super::types::{Instance, Type};
use super::value::{value_ops, Value, ValueType};

use crate::compiler::compiler::{Code, Instruction};
use crate::interpreter::value::{Macro, Pattern};
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

    //pub types: AHashMap<String, ValueType>,
    pub types: AHashMap<String, String>,
    pub instances: AHashMap<Instance, Type>,
}

impl Globals {
    pub fn new() -> Self {
        Self {
            memory: SlotMap::default(),
            types: AHashMap::new(),
            instances: AHashMap::new(),
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
// 😎

pub fn execute_code(globals: &mut Globals, code: &Code) -> Result<(), RuntimeError> {
    let mut contexts = FullContext::single(code.var_count);
    // brb
    loop {
        let mut finished = true;
        'out_for: for context in contexts.iter() {
            if !context.inner().finished {
                finished = false;
            } else {
                continue;
            }

            let (func, mut i) = context.inner().pos;

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
                    match &code.instructions[func].0[i] {
                        $(
                            Instruction::$instr => {
                                let area = code.get_bytecode_area(func, i);
                                let b = pop_ref!();
                                let a = pop_ref!();
                                let key = globals.memory.insert(value_ops::$func(a, b, area, globals)?);
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

            match &code.instructions[func].0[i] {
                Instruction::LoadConst(id) => {
                    let area = code.get_bytecode_area(func, i);
                    let key = globals
                        .memory
                        .insert(code.constants.get(*id).clone().into_stored(area));
                    context.inner().stack.push(key);
                }
                Instruction::Negate => {
                    let area = code.get_bytecode_area(func, i);
                    let a = pop_ref!();
                    push_store!(value_ops::unary_negate(a, area)?);
                }
                Instruction::Not => {
                    let area = code.get_bytecode_area(func, i);
                    let a = pop_ref!();
                    push_store!(value_ops::unary_not(a, area)?);
                }
                Instruction::LoadVar(id) => {
                    let a = context.inner().get_var(*id);
                    context.inner().stack.push(a)
                }
                Instruction::SetVar(id) => {
                    let top = pop_deep_clone!();
                    let key = globals.memory.insert(top);
                    context.inner().set_var(*id, key);
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
                    let area = code.get_bytecode_area(func, i);
                    let name = code.names.get(*id);
                    match globals.types.get(name) {
                        Some(typ) => {
                            push!(Value::TypeIndicator(*typ).into_stored(area))
                        }
                        None => {
                            return Err(RuntimeError::UndefinedType {
                                name: name.clone(),
                                area,
                            })
                        }
                    }
                }
                Instruction::BuildArray(len) => {
                    let area = code.get_bytecode_area(func, i);
                    let mut elems = vec![];
                    for _ in 0..*len {
                        elems.push(pop_deep_clone!(Store));
                    }
                    elems.reverse();
                    push!(Value::Array(elems).into_stored(area));
                }
                Instruction::PushEmpty => {
                    let area = code.get_bytecode_area(func, i);
                    push!(Value::Empty.into_stored(area));
                }
                Instruction::PopTop => {
                    context.inner().stack.pop();
                }
                Instruction::Jump(id) => {
                    i = *code.destinations.get(*id) - 1;
                }
                Instruction::JumpIfFalse(id) => unsafe {
                    if !value_ops::to_bool(pop_ref!())? {
                        i = *code.destinations.get(*id) - 1;
                    }
                },
                Instruction::ToIter => todo!(),
                Instruction::IterNext(_) => todo!(),
                Instruction::BuildDict(id) => {
                    let area = code.get_bytecode_area(func, i);
                    let keys = code.name_sets.get(*id);
                    let map = keys
                        .iter()
                        .cloned()
                        .zip((0..keys.len()).map(|_| pop_deep_clone!(Store)))
                        .collect();
                    push!(Value::Dict(map).into_stored(area));
                }
                Instruction::Return => todo!(),
                Instruction::Continue => todo!(),
                Instruction::Break => todo!(),
                Instruction::MakeMacro(id) => {
                    let area = code.get_bytecode_area(func, i);
                    let arg_areas = code.macro_arg_areas.get(&(func, i)).unwrap();
                    let (func_id, arg_info) = code.macro_build_info.get(*id);
                    let ret_type = pop_deep_clone!(Store);
                    let mut args = vec![];
                    for ((name, typ, def), area) in arg_info.iter().zip(arg_areas) {
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
                        args.push(((name.clone(), area.clone()), typ, def));
                    }
                    args.reverse();
                    push!(Value::Macro(Macro {
                        func_id: *func_id,
                        args,
                        ret_type
                    })
                    .into_stored(area));
                }
                Instruction::PushAnyPattern => {
                    let area = code.get_bytecode_area(func, i);
                    push!(Value::Pattern(Pattern::Any).into_stored(area));
                }
                Instruction::MakeMacroPattern(_) => todo!(),
                Instruction::Index => todo!(),
                Instruction::Call(id) => {
                    let area = code.get_bytecode_area(func, i);
                    let base = pop_shallow!();
                    match &base.value {
                        Value::Macro(m) => {
                            let param_areas = code.macro_arg_areas.get(&(func, i)).unwrap();
                            let param_list = code.name_sets.get(*id);

                            let mut param_map = AHashMap::new();

                            let mut params = vec![];
                            let mut named_params = vec![];

                            for (name, param_area) in param_list.iter().zip(param_areas) {
                                if name.is_empty() {
                                    params.push((pop_deep_clone!(), param_area));
                                } else {
                                    if let Some(p) =
                                        m.args.iter().position(|((s, _), ..)| s == name)
                                    {
                                        param_map.insert(name.clone(), p);
                                    } else {
                                        return Err(RuntimeError::UndefinedArgument {
                                            name: name.into(),
                                            macr: base.clone(),
                                            area: param_area.clone(),
                                        });
                                    }
                                    named_params.push((
                                        name.clone(),
                                        pop_deep_clone!(),
                                        param_area,
                                    ));
                                }
                            }

                            if params.len() > m.args.len() {
                                let call_area = code.get_bytecode_area(func, i);
                                return Err(RuntimeError::TooManyArguments {
                                    expected: m.args.len(),
                                    provided: params.len(),
                                    call_area,
                                    func: base.clone(),
                                });
                            }

                            let mut arg_fill = m
                                .args
                                .iter()
                                .map(|((_, _), t, d)| {
                                    (
                                        t.map(|id| globals.deep_clone(id)),
                                        d.map(|id| globals.deep_clone(id)),
                                    )
                                })
                                .collect::<Vec<_>>();
                            params.reverse();
                            named_params.reverse();

                            for (i, (val, param_area)) in params.into_iter().enumerate() {
                                if let Some(pat) = &arg_fill[i].0 {
                                    if !value_ops::matches_pat(&val.value, &value_ops::to_pat(pat)?)
                                    {
                                        return Err(RuntimeError::PatternMismatch {
                                            v: val,
                                            pat: pat.clone(),
                                            area: param_area.clone(),
                                        });
                                    }
                                }
                                arg_fill[i].1 = Some(val);
                            }

                            for (name, val, param_area) in named_params.into_iter() {
                                let arg_pos = param_map[&name];
                                if let Some(pat) = &arg_fill[arg_pos].0 {
                                    if !value_ops::matches_pat(&val.value, &value_ops::to_pat(pat)?)
                                    {
                                        return Err(RuntimeError::PatternMismatch {
                                            v: val,
                                            pat: pat.clone(),
                                            area: param_area.clone(),
                                        });
                                    }
                                }
                                arg_fill[arg_pos].1 = Some(val);
                            }

                            for ((_, arg), ((name, area), ..)) in arg_fill.iter().zip(&m.args) {
                                if let Some(arg) = arg {
                                } else {
                                    let call_area = code.get_bytecode_area(func, i);
                                    return Err(RuntimeError::ArgumentNotSatisfied {
                                        arg_name: name.clone(),
                                        call_area,
                                        arg_area: area.clone(),
                                    });
                                }
                            }

                            println!("------ arg fill 2");
                            for (t, v) in arg_fill {
                                println!(
                                    "{:?}",
                                    if let Some(v) = v {
                                        v.value.to_str(globals)
                                    } else {
                                        "None".into()
                                    }
                                );
                            }

                            todo!()
                        }
                        _ => {
                            return Err(RuntimeError::CannotCall {
                                base: base.clone(),
                                area,
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

                Instruction::EnterScope => {}
                Instruction::ExitScope => {}
            }

            context.inner().advance_to(code, i + 1);
        }
        if finished {
            break;
        }
    }
    Ok(())
}
