///types and functions used by the compiler
use crate::ast;
use crate::builtin::*;
use crate::compiler::create_error;
use crate::compiler_info::CodeArea;
use crate::context::*;
use crate::globals::Globals;
use crate::levelstring::*;
use crate::value::*;

//use std::boxed::Box;
use crate::compiler_info::CompilerInfo;
use crate::value_storage::*;
use std::collections::HashMap;
use std::path::PathBuf;

use smallvec::{smallvec, SmallVec};

use crate::compiler::{compile_scope, RuntimeError, CONTEXT_MAX};

pub type TypeId = u16;
//                                                               This bool is for if this value
//                                                               was implemented in the current module
pub type Implementations = HashMap<TypeId, HashMap<String, (StoredValue, bool)>>;

pub type FnIdPtr = usize;

pub type Returns = SmallVec<[(StoredValue, Context); CONTEXT_MAX]>;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum ImportType {
    Script(PathBuf),
    Lib(String),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BreakType {
    Macro,
    Loop,
    ContinueLoop,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionId {
    pub parent: Option<usize>, //index of parent id, if none it is a top-level id
    pub width: Option<u32>,    //width of this id, is none when its not calculated yet
    //pub name: String,          //name of this id, used for the label
    pub obj_list: Vec<(GdObj, usize)>, //list of objects in this function id, + their order id
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
    context: &Context,
    globals: &mut Globals,
    info: &CompilerInfo,
) -> Result<Returns, RuntimeError> {
    Ok(
        if let Some(val) = globals.stored_values[value1].clone().member(
            String::from(macro_name),
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
                let val2 = globals.stored_values[value2].clone();

                if let Some(target_typ) = m.args[0].3 {
                    let pat = &globals.stored_values[target_typ].clone();

                    if !val2.matches_pat(pat, info, globals, context)? {
                        //if types dont match, act as if there is no macro at all

                        return Ok(smallvec![(
                            store_value(
                                built_in_function(
                                    macro_name,
                                    vec![value1, value2],
                                    info.clone(),
                                    globals,
                                    context,
                                )?,
                                1,
                                globals,
                                context,
                                info.position
                            ),
                            context.clone(),
                        )]);
                    }
                }

                let (values, _) = execute_macro(
                    (
                        *m,
                        //copies argument so the original value can't be mutated
                        //prevents side effects and shit
                        vec![ast::Argument::from(
                            clone_value(
                                value2,
                                1,
                                globals,
                                context.start_group,
                                false,
                                info.position,
                            ),
                            info.position.pos,
                        )],
                    ),
                    context,
                    globals,
                    value1,
                    info.clone(),
                )?;
                values
            } else {
                smallvec![(
                    store_value(
                        built_in_function(
                            macro_name,
                            vec![value1, value2],
                            info.clone(),
                            globals,
                            context
                        )?,
                        1,
                        globals,
                        context,
                        info.position
                    ),
                    context.clone(),
                )]
            }
        } else {
            smallvec![(
                store_value(
                    built_in_function(
                        macro_name,
                        vec![value1, value2],
                        info.clone(),
                        globals,
                        context
                    )?,
                    1,
                    globals,
                    context,
                    info.position
                ),
                context.clone(),
            )]
        },
    )
}

pub fn handle_unary_operator(
    value: StoredValue,
    macro_name: Builtin,
    context: &Context,
    globals: &mut Globals,
    info: &CompilerInfo,
) -> Result<Returns, RuntimeError> {
    Ok(
        if let Some(val) = globals.stored_values[value].clone().member(
            String::from(macro_name),
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

                let (values, _) =
                    execute_macro((*m, Vec::new()), context, globals, value, info.clone())?;
                values
            } else {
                smallvec![(
                    store_value(
                        built_in_function(macro_name, vec![value], info.clone(), globals, context)?,
                        1,
                        globals,
                        context,
                        info.position
                    ),
                    context.clone(),
                )]
            }
        } else {
            smallvec![(
                store_value(
                    built_in_function(macro_name, vec![value], info.clone(), globals, context)?,
                    1,
                    globals,
                    context,
                    info.position
                ),
                context.clone(),
            )]
        },
    )
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

impl ast::Expression {
    pub fn eval(
        &self,
        context: &Context,
        globals: &mut Globals,
        mut info: CompilerInfo,
        constant: bool,
    ) -> Result<(Returns, Returns), RuntimeError> {
        //second returns is in case there are any values in the expression that includes a return statement
        let mut vals = self.values.iter();
        let first = vals.next().unwrap();
        let first_value = first.to_value(context.clone(), globals, info.clone(), constant)?;
        let mut acum = first_value.0;
        let mut inner_returns = first_value.1;

        let mut start_pos = first.pos.0;

        if self.operators.is_empty() {
            //if only variable
            return Ok((acum, inner_returns));
        }

        for (i, var) in vals.enumerate() {
            let mut new_acum: Returns = SmallVec::new();
            let end_pos = var.pos.1;
            info.position.pos = (start_pos, end_pos);
            //every value in acum will be operated with the value of var in the corresponding context
            for (acum_val, c) in acum {
                use ast::Operator::*;

                //only eval the first one on Or and And
                let (or_overwritten, and_overwritten) =
                    if let Some(imp) = globals.implementations.get(&5) {
                        (imp.get("_or_") != None, imp.get("_and_") != None)
                    } else {
                        (false, false)
                    };
                if self.operators[i] == Or
                    && !or_overwritten
                    && globals.stored_values[acum_val] == Value::Bool(true)
                {
                    let stored =
                        store_const_value(Value::Bool(true), 1, globals, &c, CodeArea::new());
                    new_acum.push((stored, c));
                    continue;
                } else if self.operators[i] == And
                    && !and_overwritten
                    && globals.stored_values[acum_val] == Value::Bool(false)
                {
                    let stored =
                        store_const_value(Value::Bool(false), 1, globals, &c, CodeArea::new());
                    new_acum.push((stored, c));
                    continue;
                }

                //what the value in acum becomes
                let evaled = var.to_value(c, globals, info.clone(), constant)?;
                inner_returns.extend(evaled.1);

                for (val, c2) in &evaled.0 {
                    //let val_fn_context = globals.get_val_fn_context(val, info.clone());
                    use Builtin::*;
                    let vals: Returns = match self.operators[i] {
                        Or => handle_operator(acum_val, *val, OrOp, c2, globals, &info)?,
                        And => handle_operator(acum_val, *val, AndOp, c2, globals, &info)?,
                        More => handle_operator(acum_val, *val, MoreThanOp, c2, globals, &info)?,
                        Less => handle_operator(acum_val, *val, LessThanOp, c2, globals, &info)?,
                        MoreOrEqual => {
                            handle_operator(acum_val, *val, MoreOrEqOp, c2, globals, &info)?
                        }
                        LessOrEqual => {
                            handle_operator(acum_val, *val, LessOrEqOp, c2, globals, &info)?
                        }
                        Slash => handle_operator(acum_val, *val, DividedByOp, c2, globals, &info)?,

                        IntDividedBy => {
                            handle_operator(acum_val, *val, IntdividedByOp, c2, globals, &info)?
                        }

                        Star => handle_operator(acum_val, *val, TimesOp, c2, globals, &info)?,

                        Modulo => handle_operator(acum_val, *val, ModOp, c2, globals, &info)?,

                        Power => handle_operator(acum_val, *val, PowOp, c2, globals, &info)?,
                        Plus => handle_operator(acum_val, *val, PlusOp, c2, globals, &info)?,
                        Minus => handle_operator(acum_val, *val, MinusOp, c2, globals, &info)?,
                        Equal => handle_operator(acum_val, *val, EqOp, c2, globals, &info)?,
                        NotEqual => handle_operator(acum_val, *val, NotEqOp, c2, globals, &info)?,

                        Either => handle_operator(acum_val, *val, EitherOp, c2, globals, &info)?,
                        Range => handle_operator(acum_val, *val, RangeOp, c2, globals, &info)?,
                        //MUTABLE ONLY
                        //ADD CHECk
                        Assign => handle_operator(acum_val, *val, AssignOp, c2, globals, &info)?,

                        Swap => handle_operator(acum_val, *val, SwapOp, c2, globals, &info)?,

                        As => handle_operator(acum_val, *val, AsOp, c2, globals, &info)?,

                        Has => handle_operator(acum_val, *val, HasOp, c2, globals, &info)?,

                        ast::Operator::Add => {
                            handle_operator(acum_val, *val, AddOp, c2, globals, &info)?
                        }

                        Subtract => {
                            handle_operator(acum_val, *val, SubtractOp, c2, globals, &info)?
                        }

                        Multiply => {
                            handle_operator(acum_val, *val, MultiplyOp, c2, globals, &info)?
                        }

                        Exponate => {
                            handle_operator(acum_val, *val, ExponateOp, c2, globals, &info)?
                        }

                        Modulate => {
                            handle_operator(acum_val, *val, ModulateOp, c2, globals, &info)?
                        }

                        Divide => handle_operator(acum_val, *val, DivideOp, c2, globals, &info)?,

                        IntDivide => {
                            handle_operator(acum_val, *val, IntdivideOp, c2, globals, &info)?
                        }
                    };
                    new_acum.extend(vals);
                }
            }
            acum = new_acum;
            start_pos = var.pos.0;
        }
        Ok((acum, inner_returns))
    }
}

pub fn execute_macro(
    (m, args): (Macro, Vec<ast::Argument>),
    context: &Context,
    globals: &mut Globals,
    parent: StoredValue,
    info: CompilerInfo,
) -> Result<(Returns, Returns), RuntimeError> {
    let mut inner_inner_returns = SmallVec::new();
    let mut new_contexts: SmallVec<[Context; CONTEXT_MAX]> = SmallVec::new();
    let fn_context = context.start_group;
    if !m.args.is_empty() {
        // second returns is for any compound statements in the args
        let (evaled_args, inner_returns) = all_combinations(
            args.iter().map(|x| x.value.clone()).collect(),
            context,
            globals,
            info.clone(),
            true,
        )?;
        inner_inner_returns.extend(inner_returns);

        for (arg_values, mut new_context) in evaled_args {
            new_context.variables = m.def_context.variables.clone();
            let mut new_variables: HashMap<String, StoredValue> = HashMap::new();

            //parse each argument given into a local macro variable
            //index of arg if no arg is specified
            let mut def_index = if m.args[0].0 == "self" { 1 } else { 0 };
            for (i, arg) in args.iter().enumerate() {
                match &arg.symbol {
                    Some(name) => {
                        let arg_def = m.args.iter().enumerate().find(|e| e.1 .0 == *name);
                        if let Some((_arg_i, arg_def)) = arg_def {
                            //type check!!
                            //maybe make type check function
                            if let Some(t) = arg_def.3 {
                                let val = globals.stored_values[arg_values[i]].clone();
                                let pat = globals.stored_values[t].clone();

                                if !val.matches_pat(&pat, &info, globals, context)? {
                                    return Err(RuntimeError::PatternMismatchError {
                                        pattern: pat.to_str(globals),
                                        val: val.get_type_str(globals),
                                        val_def: globals.get_area(arg_values[i]),
                                        pat_def: globals.get_area(t),
                                        info: info.clone().with_area(CodeArea {
                                            pos: arg.pos,
                                            ..info.position
                                        }),
                                    });
                                }
                            };

                            new_variables.insert(name.clone(), arg_values[i]);
                        } else {
                            return Err(RuntimeError::UndefinedErr {
                                undefined: name.clone(),
                                info: info.clone().with_area(CodeArea {
                                    pos: arg.pos,
                                    ..info.position
                                }),
                                desc: "macro argument".to_string(),
                            });
                        }
                    }
                    None => {
                        if (def_index) > m.args.len() - 1 {
                            return Err(RuntimeError::CustomError(create_error(
                                info.clone(),
                                "Too many arguments!",
                                &[
                                    (
                                        m.get_arg_area(),
                                        &format!(
                                            "Macro was defined to take {} argument{} here",
                                            m.args.len(),
                                            if m.args.len() == 1 { "" } else { "s" }
                                        ),
                                    ),
                                    (info.position, "Recieved too many arguments here"),
                                ],
                                None,
                            )));
                        }

                        //type check!!
                        if let Some(t) = m.args[def_index].3 {
                            let val = globals.stored_values[arg_values[i]].clone();
                            let pat = globals.stored_values[t].clone();

                            if !val.matches_pat(&pat, &info, globals, context)? {
                                return Err(RuntimeError::PatternMismatchError {
                                    pattern: pat.to_str(globals),
                                    val: val.get_type_str(globals),
                                    val_def: globals.get_area(arg_values[i]),
                                    pat_def: globals.get_area(t),
                                    info: info.clone().with_area(CodeArea {
                                        pos: arg.pos,
                                        ..info.position
                                    }),
                                });
                            }
                        };

                        new_variables.insert(
                            m.args[def_index].0.clone(),
                            clone_value(
                                arg_values[i],
                                1,
                                globals,
                                context.start_group,
                                true,
                                m.args[def_index].4,
                            ),
                        );
                        def_index += 1;
                    }
                }
            }
            //insert defaults and check non-optional arguments
            let mut m_args_iter = m.args.iter();
            if m.args[0].0 == "self" {
                if globals.stored_values[parent] == Value::Null {
                    return Err(RuntimeError::CustomError(create_error(
                        info.clone(),
                        "
This macro requires a parent (a \"self\" value), but it seems to have been called alone (or on a null value).
Should be used like this: value.macro(arguments)",
                        &[(m.args[0].4, "Macro defined as taking a 'self' argument here"), (info.position, "Called alone here")],
                        None,
                    )));
                }
                //self doesn't need to be cloned, as it is a reference (kinda)
                new_context.variables.insert("self".to_string(), parent);
                m_args_iter.next();
            }
            for arg in m_args_iter {
                if !new_variables.contains_key(&arg.0) {
                    match &arg.1 {
                        Some(default) => {
                            new_variables.insert(
                                arg.0.clone(),
                                clone_value(*default, 1, globals, context.start_group, true, arg.4),
                            );
                        }

                        None => {
                            return Err(RuntimeError::CustomError(create_error(
                                info.clone(),
                                &format!("Non-optional argument '{}' not satisfied!", arg.0),
                                &[
                                    (arg.4, "Value defined as mandatory here (because no default was given)"),
                                    (info.position, "Argument not provided here")
                                ],
                                None,
                            )));
                        }
                    }
                }
            }

            new_context.variables.extend(new_variables);

            new_contexts.push(new_context);
        }
    } else {
        if !args.is_empty() {
            return Err(RuntimeError::CustomError(create_error(
                info.clone(),
                "This macro takes no arguments!",
                &[
                    (
                        m.get_arg_area(),
                        "Macro was defined as taking no arguments here",
                    ),
                    (info.position, "Recieved too many arguments here"),
                ],
                None,
            )));
        }
        let mut new_context = context.clone();
        new_context.variables = m.def_context.variables.clone();
        /*let mut new_variables: HashMap<String, StoredValue> = HashMap::new();

        if m.args[0].0 == "self" {
            new_variables.insert("self".to_string(), store_value(parent.clone(), globals));
        }

        new_context.variables.extend(new_variables);*/

        new_contexts.push(new_context);
    }
    let mut new_info = info.clone();

    new_info.add_to_call_stack(CodeArea {
        file: m.def_file,
        pos: (0, 0),
    });
    let mut compiled = compile_scope(&m.body, new_contexts, globals, new_info)?;

    // stop break chain
    for c in &mut compiled.0 {
        if let Some((i, r)) = &(*c).broken {
            if *r != BreakType::Macro {
                return Err(RuntimeError::BreakNeverUsedError {
                    breaktype: *r,
                    info: i.clone(),
                    broke: i.position,
                    dropped: info.position,
                    reason: "the macro ended".to_string(),
                });
            }
        }
        (*c).broken = None;
    }

    let mut returns = if compiled.1.is_empty() {
        compiled.0.iter().map(|x| (1, x.clone())).collect()
    } else {
        compiled.1
    };

    for (_, c) in &mut returns {
        if c.start_group != fn_context {
            c.fn_context_change_stack.push(info.position);
        }
    }

    Ok((
        returns
            .iter()
            .map(|x| {
                //set mutable to false
                (*globals.stored_values.map.get_mut(&x.0).unwrap()).mutable = false;
                (
                    x.0,
                    Context {
                        variables: context.variables.clone(),
                        ..x.1.clone()
                    },
                )
            })
            .collect(),
        inner_inner_returns,
    ))
}
type ReturnsList = Vec<(Vec<StoredValue>, Context)>;
pub fn all_combinations(
    a: Vec<ast::Expression>,
    context: &Context,
    globals: &mut Globals,
    info: CompilerInfo,
    constant: bool,
) -> Result<(ReturnsList, Returns), RuntimeError> {
    let mut out = ReturnsList::new();
    let mut inner_returns = Returns::new();
    if a.is_empty() {
        //if there are so value, there is only one combination
        out.push((Vec::new(), context.clone()));
    } else {
        let mut a_iter = a.iter();
        //starts with all the combinations of the first expr
        let (start_values, start_returns) =
            a_iter
                .next()
                .unwrap()
                .eval(context, globals, info.clone(), constant)?;
        out.extend(start_values.iter().map(|x| (vec![x.0], x.1.clone())));
        inner_returns.extend(start_returns);
        //for the rest of the expressions
        for expr in a_iter {
            //all the new combinations end up in this
            let mut new_out: Vec<(Vec<StoredValue>, Context)> = Vec::new();
            //run through all the lists in out
            for (inner_arr, c) in out.iter() {
                //for each one, run through all the returns in that context
                let (values, returns) = expr.eval(c, globals, info.clone(), constant)?;
                inner_returns.extend(returns);
                for (v, c2) in values.iter() {
                    //push a new list with each return pushed to it
                    new_out.push((
                        {
                            let mut new_arr = inner_arr.clone();
                            new_arr.push(*v);
                            new_arr
                        },
                        c2.clone(),
                    ));
                }
            }
            //set out to this new one and repeat
            out = new_out;
        }
    }
    Ok((out, inner_returns))
}
pub fn eval_dict(
    dict: Vec<ast::DictDef>,
    context: &Context,
    globals: &mut Globals,
    info: CompilerInfo,
    constant: bool,
) -> Result<(Returns, Returns), RuntimeError> {
    let mut inner_returns = Returns::new();
    let (evaled, returns) = all_combinations(
        dict.iter()
            .map(|def| match def {
                ast::DictDef::Def(d) => d.1.clone(),
                ast::DictDef::Extract(e) => e.clone(),
            })
            .collect(),
        context,
        globals,
        info.clone(),
        constant,
    )?;
    inner_returns.extend(returns);
    let mut out = Returns::new();
    for expressions in evaled {
        let mut dict_out: HashMap<String, StoredValue> = HashMap::new();
        for (expr_index, def) in dict.iter().enumerate() {
            match def {
                ast::DictDef::Def(d) => {
                    dict_out.insert(
                        d.0.clone(),
                        clone_value(
                            expressions.0[expr_index],
                            1,
                            globals,
                            expressions.1.start_group,
                            !globals.is_mutable(expressions.0[expr_index]),
                            info.position,
                        ),
                    );
                }
                ast::DictDef::Extract(_) => {
                    let val = clone_and_get_value(
                        expressions.0[expr_index],
                        1,
                        globals,
                        expressions.1.start_group,
                        !globals.is_mutable(expressions.0[expr_index]),
                    );
                    dict_out.extend(match val.clone() {
                        Value::Dict(d) => d.clone(),
                        a => {
                            return Err(RuntimeError::TypeError {
                                expected: "dictionary or $".to_string(),
                                found: a.get_type_str(globals),
                                val_def: globals.get_area(expressions.0[expr_index]),
                                info,
                            })
                        }
                    });
                }
            };
        }
        out.push((
            store_value(Value::Dict(dict_out), 1, globals, context, info.position),
            expressions.1,
        ));
    }
    Ok((out, inner_returns))
}

impl ast::CompoundStatement {
    pub fn to_scope(
        &self,
        context: &Context,
        globals: &mut Globals,
        info: CompilerInfo,
        start_group: Option<Group>,
    ) -> Result<(TriggerFunction, Returns), RuntimeError> {
        //create the function context
        let mut new_context = context.next_fn_id(globals);

        //pick a start group
        let start_group = if let Some(g) = start_group {
            g
        } else {
            Group::next_free(&mut globals.closed_groups)
        };

        new_context.start_group = start_group;

        let new_info = info.clone();
        new_context.fn_context_change_stack = vec![info.position];
        let (contexts, inner_returns) =
            compile_scope(&self.statements, smallvec![new_context], globals, new_info)?;

        for c in contexts {
            if let Some((i, r)) = c.broken {
                return Err(RuntimeError::BreakNeverUsedError {
                    breaktype: r,
                    info: i.clone(),
                    broke: i.position,
                    dropped: info.position,
                    reason: "it's inside a trigger function".to_string(),
                });
            }
        }

        Ok((TriggerFunction { start_group }, inner_returns))
    }
}
