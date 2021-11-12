use crate::builtins::*;

use crate::context::*;
use crate::globals::Globals;
use crate::leveldata::*;
use crate::value::*;
use errors::compiler_info::CodeArea;
use errors::{create_error, RuntimeError};
///types and functions used by the compiler
use parser::ast;
use shared::BreakType;

//use std::boxed::Box;
use crate::value_storage::*;
use errors::compiler_info::CompilerInfo;
use fnv::FnvHashMap;

use crate::compiler::compile_scope;

use internment::LocalIntern;
use shared::StoredValue;

pub type TypeId = u16;
//                                                               This bool is for if this value
//                                                               was implemented in the current module
pub type Implementations = FnvHashMap<TypeId, FnvHashMap<LocalIntern<String>, (StoredValue, bool)>>;

pub type FnIdPtr = usize;

//pub type Returns = SmallVec<[(StoredValue, Context); CONTEXT_MAX]>;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct TriggerOrder(pub f64);
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionId {
    pub parent: Option<usize>, //index of parent id, if none it is a top-level id
    pub width: Option<u32>,    //width of this id, is none when its not calculated yet
    //pub name: String,          //name of this id, used for the label
    pub obj_list: Vec<(GdObj, TriggerOrder)>, //list of objects in this function id, + their order id
}

pub type SyncPartId = usize;
pub struct SyncGroup {
    pub parts: Vec<SyncPartId>,
    pub groups_used: Vec<ArbitraryId>, // groups that are already used by this sync group, and can be reused in later parts
}

pub fn handle_operator(
    value1: StoredValue,
    value2: StoredValue,
    macro_name: Builtin,
    contexts: &mut FullContext,
    globals: &mut Globals,
    info: &CompilerInfo,
) -> Result<(), RuntimeError> {
    contexts.reset_return_vals(globals);
    for full_context in contexts.iter() {
        let fn_context = full_context.inner().start_group;
        if let Some(val) = globals.stored_values[value1].clone().member(
            LocalIntern::new(String::from(macro_name)),
            full_context.inner(),
            globals,
            info.clone(),
        ) {
            if let Value::Macro(m) = globals.stored_values[val].clone() {
                if m.args.is_empty() {
                    return Err(RuntimeError::CustomError(create_error(
                        info.clone(),
                        "Expected at least one argument in operator macro",
                        &[],
                        None,
                    )));
                }
                let val2 = globals.stored_values[value2].clone();

                if let Some(target_typ) = m.args[0].pattern {
                    let pat = &globals.stored_values[target_typ].clone();

                    if !val2.pure_matches_pat(pat, info, globals, full_context.inner().clone())? {
                        //if types dont match, act as if there is no macro at all
                        built_in_function(
                            macro_name,
                            vec![value1, value2],
                            info.clone(),
                            globals,
                            full_context,
                        )?;
                    }
                }

                execute_macro(
                    (
                        *m,
                        //copies argument so the original value can't be mutated
                        //prevents side effects and shit
                        vec![ast::Argument::from(
                            clone_value(value2, globals, fn_context, false, info.position),
                            info.position.pos,
                        )],
                    ),
                    full_context,
                    globals,
                    value1,
                    info.clone(),
                )?;
            } else {
                built_in_function(
                    macro_name,
                    vec![value1, value2],
                    info.clone(),
                    globals,
                    full_context,
                )?;
            }
        } else {
            built_in_function(
                macro_name,
                vec![value1, value2],
                info.clone(),
                globals,
                full_context,
            )?;
        }
    }
    Ok(())
}

pub fn handle_unary_operator(
    value: StoredValue,
    macro_name: Builtin,
    contexts: &mut FullContext,
    globals: &mut Globals,
    info: &CompilerInfo,
) -> Result<(), RuntimeError> {
    contexts.reset_return_vals(globals);
    for full_context in contexts.iter() {
        let context = full_context.inner();
        if let Some(val) = globals.stored_values[value].clone().member(
            LocalIntern::new(String::from(macro_name)),
            context,
            globals,
            info.clone(),
        ) {
            if let Value::Macro(m) = globals.stored_values[val].clone() {
                if m.args.is_empty() {
                    return Err(RuntimeError::CustomError(create_error(
                        info.clone(),
                        "Expected at least one argument in operator macro",
                        &[],
                        None,
                    )));
                }

                execute_macro((*m, Vec::new()), full_context, globals, value, info.clone())?;
            } else {
                built_in_function(macro_name, vec![value], info.clone(), globals, full_context)?;
            }
        } else {
            built_in_function(macro_name, vec![value], info.clone(), globals, full_context)?;
        }
    }
    Ok(())
}

pub fn convert_to_int(num: f64, info: &CompilerInfo) -> Result<i32, RuntimeError> {
    let rounded = num.round();
    if (num - rounded).abs() > 0.000000001 {
        return Err(RuntimeError::CustomError(create_error(
            info.clone(),
            &format!("expected integer, found {}", num),
            &[],
            None,
        )));
    }
    Ok(rounded as i32)
}

pub fn stored_to_variable(v: StoredValue, globals: &Globals) -> ast::Variable {
    ast::ValueBody::Resolved(v).to_variable(globals.get_area(v).pos)
}

pub trait EvalExpression {
    fn eval(
        &self,
        contexts: &mut FullContext,
        globals: &mut Globals,
        info: CompilerInfo,
        constant: bool,
    ) -> Result<(), RuntimeError>;
}

impl From<ast::Operator> for Builtin {
    fn from(op: ast::Operator) -> Self {
        use ast::Operator::*;
        use Builtin::*;
        match op {
            Or => OrOp,
            And => AndOp,
            More => MoreThanOp,
            Less => LessThanOp,
            MoreOrEqual => MoreOrEqOp,
            LessOrEqual => LessOrEqOp,
            Slash => DividedByOp,
            IntDividedBy => IntdividedByOp,
            Star => TimesOp,
            Modulo => ModOp,
            Power => PowOp,
            Plus => PlusOp,
            Minus => MinusOp,
            Equal => EqOp,
            NotEqual => NotEqOp,
            Either => EitherOp,
            Both => BothOp,
            Range => RangeOp,
            //MUTABLE ONLY
            //ADD CHECk
            Assign => AssignOp,
            Swap => SwapOp,
            As => AsOp,
            In => InOp,
            ast::Operator::Add => AddOp,
            Subtract => SubtractOp,
            Multiply => MultiplyOp,
            Exponate => ExponateOp,
            Modulate => ModulateOp,
            Divide => DivideOp,
            IntDivide => IntdivideOp,
            Is => IsOp,
        }
    }
}

impl EvalExpression for ast::Expression {
    fn eval(
        &self,
        contexts: &mut FullContext,
        globals: &mut Globals,
        mut info: CompilerInfo,
        constant: bool,
    ) -> Result<(), RuntimeError> {
        contexts.reset_return_vals(globals);
        let mut vals = self.values.iter();
        let first = vals.next().unwrap();

        globals.push_new_preserved();

        first.to_value(contexts, globals, info.clone(), constant)?;

        let mut start_pos = first.pos.0;

        for (i, var) in vals.enumerate() {
            let end_pos = var.pos.1;
            info.position.pos = (start_pos, end_pos);
            //every value in acum will be operated with the value of var in the corresponding context
            for full_context in contexts.iter() {
                //only eval the first one on Or and And
                let (or_overwritten, and_overwritten) =
                    if let Some(imp) = globals.implementations.get(&5) {
                        (
                            imp.get(&globals.OR_BUILTIN).is_some(),
                            imp.get(&globals.AND_BUILTIN).is_some(),
                        )
                    } else {
                        (false, false)
                    };
                let acum_val = full_context.inner().return_value;

                globals.push_preserved_val(acum_val);

                if self.operators[i] == ast::Operator::Or
                    && !or_overwritten
                    && globals.stored_values[acum_val] == Value::Bool(true)
                {
                    let stored = store_const_value(
                        Value::Bool(true),
                        globals,
                        full_context.inner().start_group,
                        CodeArea::new(),
                    );
                    full_context.inner().return_value = stored;
                    continue;
                } else if self.operators[i] == ast::Operator::And
                    && !and_overwritten
                    && globals.stored_values[acum_val] == Value::Bool(false)
                {
                    let stored = store_const_value(
                        Value::Bool(false),
                        globals,
                        full_context.inner().start_group,
                        CodeArea::new(),
                    );
                    full_context.inner().return_value = stored;
                    continue;
                }

                //what the value in acum becomes
                var.to_value(full_context, globals, info.clone(), constant)?;

                for c2 in full_context.iter() {
                    //let val_fn_context = globals.get_val_fn_context(val, info.clone());
                    let val = c2.inner().return_value;
                    handle_operator(acum_val, val, self.operators[i].into(), c2, globals, &info)?;
                }
            }
            start_pos = var.pos.0;
        }
        globals.pop_preserved();
        Ok(())
    }
}

pub fn execute_macro(
    (m, args): (Macro, Vec<ast::Argument>),
    contexts: &mut FullContext,
    globals: &mut Globals,
    parent: StoredValue,
    info: CompilerInfo,
) -> Result<(), RuntimeError> {
    contexts.reset_return_vals(globals);
    globals.push_new_preserved();
    for context in contexts.with_breaks() {
        for val in context
            .inner()
            .get_variables()
            .values()
            .map(|stack| stack.iter().map(|VariableData { val: a, .. }| *a))
            .flatten()
        {
            globals.push_preserved_val(val)
        }
    }

    for MacroArgDef {
        default, pattern, ..
    } in m.args.iter()
    {
        if let Some(e) = default {
            globals.push_preserved_val(*e)
        }
        if let Some(e) = pattern {
            globals.push_preserved_val(*e)
        }
    }

    // dbg!(
    //     &m.args,
    //     globals.stored_values.preserved_stack.last().unwrap()
    // );

    let combinations = all_combinations(
        args.iter().map(|x| x.value.clone()).collect(),
        contexts,
        globals,
        info.clone(),
        true,
    )?;

    for (arg_values, _) in &combinations {
        for val in arg_values {
            globals.push_preserved_val(*val)
        }
    }

    //dbg!(&combinations);

    for (arg_values, full_context) in combinations {
        let mut new_variables: FnvHashMap<LocalIntern<String>, Vec<VariableData>> =
            Default::default();
        let context = full_context.inner();

        let fn_context = context.start_group;

        new_variables.extend(m.def_variables.iter().map(|(a, b)| {
            (
                *a,
                vec![VariableData {
                    val: *b,
                    layers: -1,
                    redefinable: false,
                }],
            )
        }));

        //parse each argument given into a local macro variable
        //index of arg if no arg is specified
        let mut def_index = if !m.args.is_empty() && m.args[0].name == globals.SELF_MEMBER_NAME {
            1
        } else {
            0
        };
        for (i, arg) in args.iter().enumerate() {
            match &arg.symbol {
                Some(name) => {
                    let arg_def = m.args.iter().enumerate().find(|e| e.1.name == *name);
                    if let Some((_arg_i, arg_def)) = arg_def {
                        //type check!!
                        //maybe make type check function
                        if let Some(t) = arg_def.pattern {
                            let val = globals.stored_values[arg_values[i]].clone();
                            let pat = globals.stored_values[t].clone();

                            let arg_def_info = info.clone().with_area(CodeArea {
                                pos: arg_def.position,
                                file: m.def_file,
                            });

                            if !val.pure_matches_pat(
                                &pat,
                                &arg_def_info,
                                globals,
                                context.clone(),
                            )? {
                                let arg_info = info.clone().with_area(CodeArea {
                                    pos: arg.pos,
                                    ..info.position
                                });
                                return Err(RuntimeError::PatternMismatchError {
                                    pattern: pat.to_str(globals),
                                    val: val.get_type_str(globals),
                                    val_def: globals.get_area(arg_values[i]),
                                    pat_def: globals.get_area(t),
                                    info: arg_info,
                                });
                            }
                        };
                        if arg_def.as_ref {
                            new_variables.insert(
                                *name,
                                vec![VariableData {
                                    val: arg_values[i],
                                    layers: -1,
                                    redefinable: false,
                                }],
                            );
                        } else {
                            new_variables.insert(
                                arg_def.name,
                                vec![VariableData {
                                    val: clone_value(
                                        arg_values[i],
                                        globals,
                                        context.start_group,
                                        true,
                                        CodeArea {
                                            pos: arg_def.position,
                                            file: m.def_file,
                                        },
                                    ),
                                    layers: -1,
                                    redefinable: false,
                                }],
                            );
                        }
                    } else {
                        return Err(RuntimeError::UndefinedErr {
                            undefined: name.as_ref().clone(),
                            info: info.clone().with_area(CodeArea {
                                pos: arg.pos,
                                ..info.position
                            }),
                            desc: "macro argument".to_string(),
                        });
                    }
                }
                None => {
                    if def_index >= m.args.len() {
                        return Err(RuntimeError::CustomError(create_error(
                            info.clone(),
                            "Too many arguments!",
                            &[
                                (
                                    CodeArea {
                                        pos: m.arg_pos,
                                        file: m.def_file,
                                    },
                                    &format!(
                                        "Macro was defined to take {} argument{} here",
                                        m.args.len(),
                                        if m.args.len() == 1 { "" } else { "s" }
                                    ),
                                ),
                                (info.position, "Received too many arguments here"),
                            ],
                            None,
                        )));
                    }

                    //dbg!(&m.args[def_index]);

                    //type check!!
                    if let Some(t) = m.args[def_index].pattern {
                        let val = globals.stored_values[arg_values[i]].clone();
                        let pat = globals.stored_values[t].clone();
                        let arg_def_info = info.clone().with_area(CodeArea {
                            pos: m.args[def_index].position,
                            file: m.def_file,
                        });

                        if !val.pure_matches_pat(&pat, &arg_def_info, globals, context.clone())? {
                            let arg_info = info.clone().with_area(CodeArea {
                                pos: arg.pos,
                                ..info.position
                            });
                            return Err(RuntimeError::PatternMismatchError {
                                pattern: pat.to_str(globals),
                                val: val.get_type_str(globals),
                                val_def: globals.get_area(arg_values[i]),
                                pat_def: globals.get_area(t),
                                info: arg_info,
                            });
                        }
                    };
                    if m.args[def_index].as_ref {
                        new_variables.insert(
                            m.args[def_index].name,
                            vec![VariableData {
                                val: arg_values[i],
                                layers: -1,
                                redefinable: false,
                            }],
                        );
                    } else {
                        new_variables.insert(
                            m.args[def_index].name,
                            vec![VariableData {
                                val: clone_value(
                                    arg_values[i],
                                    globals,
                                    context.start_group,
                                    true,
                                    CodeArea {
                                        pos: m.args[def_index].position,
                                        file: m.def_file,
                                    },
                                ),
                                layers: -1,
                                redefinable: false,
                            }],
                        );
                    }
                    def_index += 1;
                }
            }
        }
        //insert defaults and check non-optional arguments
        let mut m_args_iter = m.args.iter();
        if !m.args.is_empty() && m.args[0].name == globals.SELF_MEMBER_NAME {
            if globals.stored_values[parent] == Value::Null {
                return Err(RuntimeError::CustomError(create_error(
                        info.clone(),
                        "
This macro requires a parent (a \"self\" value), but it seems to have been called alone (or on a null value).
Should be used like this: value.macro(arguments)",
                        &[(CodeArea {pos: m.args[0].position, file: m.def_file }, "Macro defined as taking a 'self' argument here"), (info.position, "Called alone here")],
                        None,
                    )));
            }
            //self doesn't need to be cloned, as it is a reference (kinda)
            new_variables.insert(
                globals.SELF_MEMBER_NAME,
                vec![VariableData {
                    val: parent,
                    layers: -1,
                    redefinable: false,
                }],
            );
            m_args_iter.next();
        }
        for arg in m_args_iter {
            if let std::collections::hash_map::Entry::Vacant(e) = new_variables.entry(arg.name) {
                match &arg.default {
                    Some(default) => {
                        e.insert(vec![VariableData {
                            val: clone_value(
                                *default,
                                globals,
                                context.start_group,
                                true,
                                CodeArea {
                                    pos: arg.position,
                                    file: m.def_file,
                                },
                            ),
                            layers: -1,
                            redefinable: false,
                        }]);
                    }

                    None => {
                        return Err(RuntimeError::CustomError(create_error(
                                info.clone(),
                                &format!("Non-optional argument '{}' not satisfied!", arg.name),
                                &[
                                    (CodeArea {pos: arg.position, file: m.def_file}, "Value defined as mandatory here (because no default was given)"),
                                    (info.position, "Argument not provided here")
                                ],
                                None,
                            )));
                    }
                }
            }
        }

        let prev_vars = full_context.inner().get_variables().clone();

        (*full_context.inner()).set_all_variables(new_variables);

        let mut new_info = info.clone();

        new_info.add_to_call_stack(CodeArea {
            file: m.def_file,
            pos: (0, 0),
        });

        let stored_path = globals.path;
        (*globals).path = m.def_file;
        compile_scope(&m.body, full_context, globals, new_info)?;

        (*globals).path = stored_path;

        let mut out_contexts = Vec::new();
        for context in full_context.with_breaks() {
            (*context.inner()).set_all_variables(prev_vars.clone());

            if let Some((r, i)) = (*context.inner()).broken {
                match r {
                    BreakType::Macro(v, _) => {
                        let ret = if let Some(val) = v {
                            (*context.inner()).return_value = val;
                            val
                        } else {
                            (*context.inner()).return_value = globals.NULL_STORAGE;
                            globals.NULL_STORAGE
                        };
                        if let Some(pat) = m.ret_pattern {
                            //dbg!(&globals.stored_values[pat], &globals.stored_values[ret]);
                            if !globals.stored_values[ret].clone().pure_matches_pat(
                                &globals.stored_values[pat].clone(),
                                &info,
                                globals,
                                context.inner().clone(),
                            )? {
                                return Err(RuntimeError::PatternMismatchError {
                                    pattern: globals.stored_values[pat].clone().to_str(globals),
                                    val: globals.stored_values[ret].clone().to_str(globals),
                                    pat_def: globals.get_area(pat),
                                    val_def: globals.get_area(ret),
                                    info,
                                });
                            }
                        }
                    }
                    a => {
                        return Err(RuntimeError::BreakNeverUsedError {
                            breaktype: a,
                            info: CompilerInfo::from_area(i),
                            broke: i,
                            dropped: info.position,
                            reason: "the macro ended".to_string(),
                        });
                    }
                }
                (*context.inner()).broken = None;
                out_contexts.push(context.clone());
            }
        }
        //dbg!(out_contexts.len(), info.position);
        if !out_contexts.is_empty() {
            *full_context = FullContext::stack(&mut out_contexts.into_iter()).unwrap();
        }

        for c in full_context.iter() {
            if c.inner().start_group != fn_context {
                c.inner().fn_context_change_stack.push(info.position);
            }
        }
    }

    globals.pop_preserved();
    Ok(())
}

pub fn reduce_combinations<'a, T, F>(
    a: Vec<T>,
    contexts: &'a mut FullContext,
    globals: &mut Globals,
    reduce: F,
) -> Result<Vec<(Vec<StoredValue>, &'a mut FullContext)>, RuntimeError>
where
    F: Fn(
        &T,
        &'a mut FullContext,
        Vec<StoredValue>,
        &mut Globals,
    ) -> Result<Vec<(Vec<StoredValue>, &'a mut FullContext)>, RuntimeError>,
{
    globals.push_new_preserved();

    let mut out = vec![(Vec::new(), contexts)];
    for expr in a {
        let mut new_out = Vec::new();
        for (list, full_context) in out.into_iter() {
            let new_list = reduce(&expr, full_context, list.clone(), globals)?;
            new_out.extend(new_list);
        }
        out = new_out;
    }
    globals.pop_preserved();

    Ok(out)
}

pub fn all_combinations<'a>(
    a: Vec<ast::Expression>,
    contexts: &'a mut FullContext,
    globals: &mut Globals,
    info: CompilerInfo,
    constant: bool,
) -> Result<Vec<(Vec<StoredValue>, &'a mut FullContext)>, RuntimeError> {
    reduce_combinations(
        a,
        contexts,
        globals,
        |e: &ast::Expression, ctx, list: Vec<StoredValue>, globals| {
            e.eval(ctx, globals, info.clone(), constant)?;
            let mut added = Vec::new();

            for full_context in ctx.iter() {
                let result = full_context.inner().return_value;
                let mut updated_list = list.clone();

                updated_list.push(result);
                globals.push_preserved_val(result);
                added.push((updated_list, full_context));
            }
            Ok(added)
        },
    )
}

pub fn eval_dict(
    dict: Vec<ast::DictDef>,
    contexts: &mut FullContext,
    globals: &mut Globals,
    info: CompilerInfo,
    constant: bool,
) -> Result<(), RuntimeError> {
    contexts.reset_return_vals(globals);
    let combinations = all_combinations(
        dict.iter()
            .map(|def| match def {
                ast::DictDef::Def(d) => d.1.clone(),
                ast::DictDef::Extract(e) => e.clone(),
            })
            .collect(),
        contexts,
        globals,
        info.clone(),
        constant,
    )?;
    globals.push_new_preserved();
    for (arg_values, _) in &combinations {
        for val in arg_values {
            globals.push_preserved_val(*val)
        }
    }
    for (results, full_context) in combinations {
        let context = full_context.inner();
        let mut dict_out: FnvHashMap<LocalIntern<String>, StoredValue> = Default::default();
        for (expr_index, def) in dict.iter().enumerate() {
            match def {
                ast::DictDef::Def(d) => {
                    dict_out.insert(
                        d.0,
                        clone_value(
                            results[expr_index],
                            globals,
                            context.start_group,
                            !globals.is_mutable(results[expr_index]),
                            info.position,
                        ),
                    );
                }
                ast::DictDef::Extract(_) => {
                    let val = clone_and_get_value(
                        results[expr_index],
                        globals,
                        context.start_group,
                        !globals.is_mutable(results[expr_index]),
                    );
                    dict_out.extend(match val.clone() {
                        Value::Dict(d) => d.clone(),
                        a => {
                            return Err(RuntimeError::TypeError {
                                expected: "dictionary".to_string(),
                                found: a.get_type_str(globals),
                                val_def: globals.get_area(results[expr_index]),
                                info,
                            })
                        }
                    });
                }
            };
        }
        context.return_value = store_const_value(
            Value::Dict(dict_out),
            globals,
            context.start_group,
            info.position,
        );
    }
    globals.pop_preserved();
    Ok(())
}

pub trait ToTriggerFunc {
    fn to_trigger_func(
        &self,
        contexts: &mut FullContext,
        globals: &mut Globals,
        info: CompilerInfo,
        start_group: Option<Group>,
    ) -> Result<(), RuntimeError>;
}

impl ToTriggerFunc for ast::CompoundStatement {
    fn to_trigger_func(
        &self,
        contexts: &mut FullContext,
        globals: &mut Globals,
        info: CompilerInfo,
        start_group: Option<Group>,
    ) -> Result<(), RuntimeError> {
        contexts.reset_return_vals(globals);
        for full_context in contexts.iter() {
            let mut prev_context = full_context.clone();

            //pick a start group
            let start_group = if let Some(g) = start_group {
                g
            } else {
                Group::next_free(&mut globals.closed_groups)
            };

            full_context.inner().next_fn_id(globals);
            full_context.inner().start_group = start_group;
            full_context.inner().fn_context_change_stack = vec![info.position];

            compile_scope(&self.statements, full_context, globals, info.clone())?;

            let mut carried_breaks = Vec::new();

            for c in full_context.with_breaks() {
                if let Some((r, i)) = c.inner().broken {
                    if let BreakType::Macro(_, true) = r {
                        carried_breaks.push(c.clone());
                    } else {
                        return Err(RuntimeError::BreakNeverUsedError {
                            breaktype: r,
                            info: CompilerInfo::from_area(i),
                            broke: i,
                            dropped: info.position,
                            reason: "it's inside a trigger function".to_string(),
                        });
                    }
                }
            }

            (*prev_context.inner()).return_value = store_const_value(
                Value::TriggerFunc(TriggerFunction { start_group }),
                globals,
                prev_context.inner().start_group,
                info.position,
            );

            if !carried_breaks.is_empty() {
                prev_context = FullContext::Split(
                    prev_context.clone().into(),
                    FullContext::stack(&mut carried_breaks.into_iter())
                        .unwrap()
                        .into(),
                );
            }
            *full_context = prev_context;
        }

        //(TriggerFunction { start_group }, inner_returns)

        Ok(())
    }
}
