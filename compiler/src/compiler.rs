//! Tools for compiling SPWN into GD object strings

use errors::create_error;
use internment::LocalIntern;


use shared::BreakType;
use shared::ImportType;
use shared::SpwnSource;

use crate::builtins::*;
use crate::context::*;
use errors::compiler_info::CodeArea;
use errors::compiler_info::CompilerInfo;
use parser::ast;

use crate::globals::Globals;
use crate::leveldata::*;
use crate::value::*;
use crate::value_storage::*;
use crate::STD_PATH;
use fnv::FnvHashMap;

use std::io::Write;
use std::mem;

use errors::RuntimeError;

use parser::parser::ParseNotes;
use std::fs;
use std::path::PathBuf;

use crate::compiler_types::*;

use ariadne::Color as TColor;
use ariadne::Fmt;

pub fn compile_spwn(
    statements: Vec<ast::Statement>,
    source: SpwnSource,
    included_paths: Vec<PathBuf>,
    notes: ParseNotes,
    permissions: BuiltinPermissions,
    initial_level: String,
    std_out: &mut impl Write,
) -> Result<Globals, RuntimeError> {
    //variables that get changed throughout the compiling

    let mut globals = Globals::new(source.clone(), permissions, initial_level, std_out);
    globals.includes = included_paths;

    let print_with_color = |a: &str, color| println!("{}", a.fg(color));

    // if statements.is_empty() {
    //     return Err(RuntimeError::CustomError(create_error(
    //         CompilerInfo::from_area(crate::compiler_info::CodeArea {
    //             file: LocalIntern::new(path),
    //             pos: (0, 0),
    //         }),
    //         "this script is empty",
    //         &[],
    //         None,
    //     )));
    // }
    let mut start_context = FullContext::new(&globals);
    //store at pos 0
    // store_value(Value::Builtins, 1, &mut globals, &start_context);
    // store_value(Value::Null, 1, &mut globals, &start_context);

    let start_info = CompilerInfo {
        ..CompilerInfo::from_area(errors::compiler_info::CodeArea {
            file: LocalIntern::new(source.clone()),
            pos: (0, 0),
        })
    };
    use std::time::Instant;

    //println!("Importing standard library...");
    print_with_color("Building script ...", TColor::Cyan);
    print_with_color("———————————————————————————\n", TColor::White);
    #[cfg(not(target_arch = "wasm32"))]
    let start_time = Instant::now();

    if !notes.tag.tags.iter().any(|x| x.0 == "no_std") {
        import_module(
            &ImportType::Lib(STD_PATH.to_string()),
            &mut start_context,
            &mut globals,
            start_info.clone(),
            false,
        )?;

        if let FullContext::Split(_, _) = start_context {
            return Err(RuntimeError::CustomError(create_error(
                start_info,
                "The standard library can not split the context",
                &[],
                None,
            )));
        }

        if let Value::Dict(d) = &globals.stored_values[start_context.inner().return_value] {
            for (a, b, c) in d.iter().map(|(k, v)| (*k, *v, -1)) {
                start_context.inner().new_redefinable_variable(a, b, c)
            }
        } else {
            return Err(RuntimeError::CustomError(create_error(
                start_info,
                "The standard library must return a dictionary",
                &[],
                None,
            )));
        }
    }

    compile_scope(&statements, &mut start_context, &mut globals, start_info)?;
    if !statements.is_empty() {
        for fc in start_context.with_breaks() {
            let c = fc.inner();
            let end_pos = statements.last().unwrap().pos.1;
            if let Some((r, i)) = c.broken {
                return Err(RuntimeError::BreakNeverUsedError {
                    breaktype: r,
                    info: CompilerInfo::from_area(i),
                    broke: i,
                    dropped: CodeArea {
                        pos: (end_pos, end_pos),
                        file: LocalIntern::new(source),
                    },
                    reason: "the program ended".to_string(),
                });
            }
        }
    }

    print_with_color("———————————————————————————\n", TColor::White);

    /*  Build Timing ----------------------------------------------------- **
        New build timing changes the unit form milliseconds, to seconds,
        to minutes depending on the time building took.
    */
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Define the different units
        let build_time_secs = start_time.elapsed().as_secs();
        let build_time_millis = start_time.elapsed().as_millis();
        let build_time_mins = build_time_secs / 60;

        // Check which unit to unit to use
        if build_time_millis < 1000 {
            print_with_color(
                &format!("Built in {} milliseconds!", build_time_millis),
                TColor::Green,
            );
        } else if build_time_millis < 60000 {
            print_with_color(
                &format!("Built in {} seconds!", build_time_secs),
                TColor::Green,
            );
        } else {
            print_with_color(
                &format!("Built in {} minutes!", build_time_mins),
                TColor::Green,
            );
        }
    }

    //----------------------------------------------------------------------- **

    Ok(globals)
}

use crate::compiler_types::EvalExpression;

pub fn compile_scope(
    statements: &[ast::Statement],
    contexts: &mut FullContext,
    globals: &mut Globals,
    mut info: CompilerInfo,
) -> Result<(), RuntimeError> {
    if contexts.iter().next().is_none() {
        return Ok(());
    }
    contexts.enter_scope();
    contexts.reset_return_vals(globals);

    for statement in statements.iter() {
        //find out what kind of statement this is
        //let start_time = Instant::now();

        //print_error_intro(info.pos, &info.current_file);

        //println!("{}", statement.fmt(0));

        // println!(
        //     "{} -> Compiling a statement in {} contexts",
        //     info.path.join(">"),
        //     contexts.len()
        // );
        info.position.pos = statement.pos;

        // println!(
        //     "{}:0:{}",
        //     info.position.file.to_string_lossy(),
        //     info.position.pos.0
        // );
        use ast::StatementBody::*;

        let stored_context = if statement.arrow {
            let mut stored = Vec::new();
            globals.push_new_preserved();
            for c in contexts.with_breaks() {
                stored.push(c.clone());
                for v in c.inner().get_variables().values() {
                    for VariableData { val: v, .. } in v {
                        globals.push_preserved_val(*v)
                    }
                }
                match c.inner().broken {
                    Some((BreakType::Macro(Some(v), _), _)) | Some((BreakType::Switch(v), _)) => {
                        globals.push_preserved_val(v)
                    }
                    _ => (),
                }
            }
            // TODO: preserve these
            *contexts = FullContext::stack(&mut contexts.iter().map(|c| c.clone())).unwrap();
            Some(stored)
        } else {
            None
        };

        //println!("{}:{}:{}", info.current_file.to_string_lossy(), info.pos.0.0, info.pos.0.1);
        //use crate::fmt::SpwnFmt;
        match &statement.body {
            Definition(def) => {
                do_assignment(
                    &def.symbol.to_expression(),
                    &def.value,
                    contexts,
                    globals,
                    &info,
                    def.mutable,
                    0,
                    None
                )?;
            }
            Expr(expr) => expr.eval(contexts, globals, info.clone(), true)?,

            Extract(val) => {
                val.eval(contexts, globals, info.clone(), true)?;
                for full_context in contexts.iter() {
                    let (context, val) = full_context.inner_value();
                    let fn_context = context.start_group;

                    match globals.stored_values[val].clone() {
                        Value::Dict(ref d) => {
                            let iter = d.iter().map(|(k, v)| {
                                (
                                    *k,
                                    clone_value(
                                        *v,
                                        globals,
                                        fn_context,
                                        !globals.is_mutable(*v),
                                        globals.get_area(*v),
                                    ),
                                    0,
                                )
                            });
                            for (a, b, c) in iter {
                                context.new_redefinable_variable(a, b, c);
                            }
                        }
                        Value::Builtins => {
                            for name in BUILTIN_LIST.iter() {
                                let p = store_const_value(
                                    Value::BuiltinFunction(*name),
                                    globals,
                                    fn_context,
                                    info.position,
                                );

                                context.new_redefinable_variable(
                                    LocalIntern::new(String::from(*name)),
                                    p,
                                    0,
                                );
                            }
                        }
                        a => {
                            return Err(RuntimeError::TypeError {
                                expected: "dictionary or $".to_string(),
                                found: a.get_type_str(globals),
                                val_def: globals.get_area(val),
                                info,
                            })
                        }
                    }
                }
            }

            TypeDef(name) => {
                //initialize type
                let already = globals.type_ids.get(name);
                if let Some(t) = already {
                    if t.1 != info.position {
                        return Err(RuntimeError::CustomError(create_error(
                            info.clone(),
                            &format!("the type '{}' is already defined", name),
                            &[
                                (t.1, "The type was first defined here"),
                                (info.position, "Attempted to redefine here"),
                            ],
                            None,
                        )));
                    }
                } else {
                    (*globals).type_id_count += 1;
                    (*globals)
                        .type_ids
                        .insert(name.clone(), (globals.type_id_count, info.position));
                }
                //Value::TypeIndicator(globals.type_id_count)
            }

            If(if_stmt) => {
                if_stmt
                    .condition
                    .eval(contexts, globals, info.clone(), true)?;
                for full_context in contexts.iter() {
                    let (_, val) = full_context.inner_value();
                    match &globals.stored_values[val] {
                        Value::Bool(b) => {
                            //internal if statement
                            if *b {
                                compile_scope(
                                    &if_stmt.if_body,
                                    full_context,
                                    globals,
                                    info.clone(),
                                )?;
                            } else {
                                match &if_stmt.else_body {
                                    Some(body) => {
                                        compile_scope(body, full_context, globals, info.clone())?;
                                    }
                                    None => (),
                                };
                            }
                        }
                        a => {
                            return Err(RuntimeError::TypeError {
                                expected: "boolean".to_string(),
                                found: a.get_type_str(globals),
                                val_def: globals.get_area(val),
                                info,
                            })
                        }
                    }
                }
            }

            Impl(imp) => {
                let message = "cannot run impl statement in a trigger function context, consider moving it to the start of your script.".to_string();

                if let FullContext::Single(c) = &contexts {
                    if c.start_group.id != Id::Specific(0) {
                        return Err(RuntimeError::ContextChangeError {
                            message,
                            info,
                            context_changes: c.fn_context_change_stack.clone(),
                        });
                    }
                } else {
                    return Err(RuntimeError::CustomError(create_error(
                        info,
                        "impl cannot run in a split context",
                        &[],
                        None,
                    )));
                }

                imp.symbol.to_value(contexts, globals, info.clone(), true)?;

                if let FullContext::Split(_, _) = contexts {
                    return Err(RuntimeError::CustomError(create_error(
                        info,
                        "impl statements with context-splitting values are not allowed",
                        &[],
                        None,
                    )));
                }

                let (c, typ) = contexts.inner_value();

                if c.start_group.id != Id::Specific(0) {
                    return Err(RuntimeError::ContextChangeError {
                        message: "impl type changes the context".to_string(),
                        info,
                        context_changes: c.fn_context_change_stack.clone(),
                    });
                }
                match globals.stored_values[typ].clone() {
                    Value::TypeIndicator(s) => {
                        eval_dict(imp.members.clone(), contexts, globals, info.clone(), true)?;
                        if let FullContext::Split(_, _) = contexts {
                            return Err(RuntimeError::CustomError(create_error(
                                info,
                                "impl statements with context-splitting values are not allowed",
                                &[],
                                None,
                            )));
                        }
                        //Returns inside impl values dont really make sense do they
                        if contexts.inner().broken.is_some() {
                            return Err(RuntimeError::CustomError(create_error(
                                info,
                                "you can't use return from inside an impl statement value",
                                &[],
                                None,
                            )));
                        }
                        let (_, val) = contexts.inner_value();

                        // make this not ugly, future me

                        if let Value::Dict(d) = &globals.stored_values[val] {
                            match globals.implementations.get_mut(&s) {
                                Some(implementation) => {
                                    for (key, val) in d.iter() {
                                        (*implementation).insert(*key, (*val, true));
                                    }
                                }
                                None => {
                                    globals.implementations.insert(
                                        s,
                                        d.iter()
                                            .map(|(key, value)| (*key, (*value, true)))
                                            .collect(),
                                    );
                                }
                            }
                        } else {
                            unreachable!();
                        }
                    }
                    a => {
                        return Err(RuntimeError::TypeError {
                            expected: "type indicator".to_string(),
                            found: a.get_type_str(globals),
                            val_def: globals.get_area(typ),
                            info,
                        })
                    }
                }

                //println!("{:?}", new_contexts[0].implementations);
            }
            Call(call) => {
                /*for context in &mut contexts {
                    context.x += 1;
                }*/
                call.function
                    .to_value(contexts, globals, info.clone(), true)?;

                //let mut obj_list = Vec::<GDObj>::new();
                for full_context in contexts.iter() {
                    let (context, func) = full_context.inner_value();
                    let mut params = FnvHashMap::default();
                    params.insert(
                        51,
                        match &globals.stored_values[func] {
                            Value::TriggerFunc(g) => ObjParam::Group(g.start_group),
                            Value::Group(g) => ObjParam::Group(*g),
                            a => {
                                return Err(RuntimeError::TypeError {
                                    expected: "trigger function or group".to_string(),
                                    found: a.get_type_str(globals),
                                    val_def: globals.get_area(func),
                                    info,
                                })
                            }
                        },
                    );
                    params.insert(1, ObjParam::Number(1268.0));
                    (*globals).trigger_order += 1.0;

                    (*globals).func_ids[context.func_id].obj_list.push((
                        GdObj {
                            params,

                            ..context_trigger(context, &mut globals.uid_counter)
                        }
                        .context_parameters(context),
                        TriggerOrder(globals.trigger_order),
                    ))
                }
            }

            While(w) => {
                for full_context in contexts.iter() {
                    let fn_context = full_context.inner().start_group;
                    loop {
                        full_context.disable_breaks(BreakType::ContinueLoop);

                        w.condition
                            .eval(full_context, globals, info.clone(), true)?;

                        if let FullContext::Split(_, _) = full_context {
                            return Err(RuntimeError::CustomError(create_error(
                                info,
                                "While loop condition can not split the context",
                                &[],
                                Some("Consider using a runtime while loop"),
                            )));
                        }

                        if full_context.inner().start_group != fn_context {
                            return Err(RuntimeError::ContextChangeError {
                                message: "While loop condition can not change the trigger function context (consider using a runtime while loop)".to_string(),
                                info,
                                context_changes: full_context.inner().fn_context_change_stack.clone()
                            });
                        }

                        if let Value::Bool(b) =
                            globals.stored_values[full_context.inner().return_value]
                        {
                            if b {
                                compile_scope(&w.body, full_context, globals, info.clone())?;
                                if let FullContext::Split(_, _) = full_context {
                                    return Err(RuntimeError::CustomError(create_error(
                                        info,
                                        "While loop body can not split the context (consider using a runtime while loop)",
                                        &[],
                                        Some("Consider using a runtime while loop"),
                                    )));
                                }
                                if full_context.inner().start_group != fn_context {
                                    return Err(RuntimeError::ContextChangeError {
                                        message: "While loop body can not change the trigger function context".to_string(),
                                        info,
                                        context_changes: full_context.inner().fn_context_change_stack.clone()
                                    });
                                }
                            } else {
                                full_context.inner().broken =
                                    Some((BreakType::Loop, CodeArea::new()));
                            }
                        } else {
                            return Err(RuntimeError::TypeError {
                                expected: "boolean".to_string(),
                                found: globals.get_type_str(full_context.inner().return_value),
                                val_def: globals.get_area(full_context.inner().return_value),
                                info,
                            });
                        }

                        let all_breaks = full_context.with_breaks().all(|fc| {
                            !matches!(fc.inner().broken, Some((BreakType::ContinueLoop, _)) | None)
                        });
                        if all_breaks {
                            break;
                        }
                    }
                    full_context.disable_breaks(BreakType::Loop);
                    full_context.disable_breaks(BreakType::ContinueLoop);
                }
            }

            For(f) => {
                f.array.eval(contexts, globals, info.clone(), true)?;
                /*
                Before going further you should probably understand what contexts mean.
                A "context", in SPWN, is like a parallel universe. Each context is nearly
                identical to a code block, except it expands a runtime value that is meant to be
                converted into a compile time item. Every time you want to do something like
                convert a counter to a number, SPWN will branch the current code block into a number
                of contexts, one for each possible value from the conversion. All of the branched contexts
                will be evaluated in isolation to each other.
                */
                let i_dest = &f.symbol;
                let mut maybe_single: Option<LocalIntern<String>> = None;
                if i_dest.operators.is_empty()
                    && i_dest.values.len() == 1
                    && i_dest.values[0].path.is_empty()
                    && i_dest.values[0].operator.is_none()
                    && matches!(i_dest.values[0].value.body, ast::ValueBody::Symbol(_))
                {
                    if let ast::ValueBody::Symbol(single) = i_dest.values[0].value.body {
                        maybe_single = Some(single);
                    } else {
                        unreachable!();
                    }
                }

                // skips all broken contexts, so as to not interfere with potential breaks in the following loops

                for full_context in contexts.iter() {
                    let (_, val) = full_context.inner_value();
                    globals.push_new_preserved();
                    globals.push_preserved_val(val);

                    match globals.stored_values[val].clone() {
                        // what are we iterating
                        Value::Array(arr) => {
                            // its an array!for

                            for element in &arr {
                                // going through the array items
                                full_context.disable_breaks(BreakType::ContinueLoop);

                                do_assignment(
                                    &i_dest,
                                    &Some(
                                        ast::ValueBody::Resolved(*element)
                                            .to_variable(globals.get_area(val).pos)
                                            .to_expression(),
                                    ),
                                    full_context,
                                    globals,
                                    &info,
                                    true,
                                    -1,
                                    None
                                )?;

                                /*if let Some(single) = maybe_single {
                                    full_context.set_variable_and_clone(
                                        single,
                                        *element,
                                        -1, // so that it gets removed at the end of the scope
                                        true,
                                        globals,
                                        globals.get_area(*element),
                                    );
                                } else {
                                    do_destructure(
                                        full_context,
                                        globals,
                                        ast::ValueBody::Resolved(*element),
                                    )?;
                                }*/

                                compile_scope(&f.body, full_context, globals, info.clone())?; // eval the stuff

                                let all_breaks = full_context.with_breaks().all(|fc| {
                                    !matches!(
                                        fc.inner().broken,
                                        Some((BreakType::ContinueLoop, _)) | None
                                    )
                                });

                                if all_breaks {
                                    break;
                                }
                            }

                            // finally append all newly created ones to the global count
                        }
                        Value::Dict(d) => {
                            // its a dict!
                            for (k, v) in d {
                                // going through the dict items
                                full_context.disable_breaks(BreakType::ContinueLoop);

                                for c in full_context.iter() {
                                    let fn_context = c.inner().start_group;
                                    let key = store_val_m(
                                        Value::Str(k.as_ref().clone()),
                                        globals,
                                        fn_context,
                                        true,
                                        globals.get_area(v),
                                    );
                                    let val = clone_value(
                                        v,
                                        globals,
                                        fn_context,
                                        true,
                                        globals.get_area(v),
                                    );
                                    // reset all variables per context

                                    let tmp_arr = store_const_value(
                                        Value::Array(vec![key, val]),
                                        globals,
                                        fn_context,
                                        globals.get_area(v),
                                    );
                                    do_assignment(
                                        &i_dest,
                                        &Some(
                                            ast::ValueBody::Resolved(tmp_arr)
                                                .to_variable((0, 0))
                                                .to_expression(),
                                        ),
                                        c,
                                        globals,
                                        &info,
                                        true,
                                        -1,
                                        None
                                    )?;
                                    /*if let Some(single) = maybe_single {
                                        (*c.inner()).new_variable(
                                            single,
                                            store_const_value(
                                                Value::Array(vec![key, val]),
                                                globals,
                                                fn_context,
                                                globals.get_area(v),
                                            ),
                                            -1,
                                        );
                                    } else {

                                        do_destructure(
                                            c,
                                            globals,
                                            ast::ValueBody::Array(vec![
                                                ast::ValueBody::Resolved(key)
                                                    .to_variable((0, 0))
                                                    .to_expression(),
                                                ast::ValueBody::Resolved(val)
                                                    .to_variable((0, 0))
                                                    .to_expression(),
                                            ]),
                                        )?;
                                    }*/
                                }

                                compile_scope(&f.body, full_context, globals, info.clone())?; // eval the stuff

                                let all_breaks = full_context.with_breaks().all(|fc| {
                                    !matches!(
                                        fc.inner().broken,
                                        Some((BreakType::ContinueLoop, _)) | None
                                    )
                                });

                                if all_breaks {
                                    break;
                                }
                            }
                        }
                        Value::Str(s) => {
                            for ch in s.chars() {
                                // going through the array items
                                full_context.disable_breaks(BreakType::ContinueLoop);
                                if let Some(single) = maybe_single {
                                    full_context.set_variable_and_store(
                                        single,
                                        Value::Str(ch.to_string()),
                                        -1, // so that it gets removed at the end of the scope
                                        true,
                                        globals,
                                        info.position,
                                    );
                                } else {
                                    return Err(RuntimeError::CustomError(create_error(
                                        info.clone(),
                                        "Cannot destructure string",
                                        &[],
                                        None,
                                    )));
                                }

                                compile_scope(&f.body, full_context, globals, info.clone())?; // eval the stuff

                                let all_breaks = full_context.with_breaks().all(|fc| {
                                    !matches!(
                                        fc.inner().broken,
                                        Some((BreakType::ContinueLoop, _)) | None
                                    )
                                });

                                if all_breaks {
                                    break;
                                }
                            }
                        }

                        Value::Range(start, end, step) => {
                            let mut normal = (start..end).step_by(step);
                            let mut rev = (end..start).step_by(step).rev();
                            let range: &mut dyn Iterator<Item = i32> =
                                if start < end { &mut normal } else { &mut rev };

                            for num in range {
                                // going through the array items
                                full_context.disable_breaks(BreakType::ContinueLoop);
                                if let Some(single) = maybe_single {
                                    full_context.set_variable_and_store(
                                        single,
                                        Value::Number(num as f64),
                                        -1, // so that it gets removed at the end of the scope
                                        true,
                                        globals,
                                        info.position,
                                    );
                                } else {
                                    return Err(RuntimeError::CustomError(create_error(
                                        info.clone(),
                                        "Cannot destructure range",
                                        &[],
                                        None,
                                    )));
                                }

                                //dbg!(full_context.with_breaks().count());

                                compile_scope(&f.body, full_context, globals, info.clone())?; // eval the stuff

                                //dbg!(full_context.with_breaks().count());

                                let all_breaks = full_context.with_breaks().all(|fc| {
                                    !matches!(
                                        fc.inner().broken,
                                        Some((BreakType::ContinueLoop, _)) | None
                                    )
                                });

                                if all_breaks {
                                    break;
                                }
                            }
                        }

                        a => {
                            return Err(RuntimeError::TypeError {
                                expected: "array, dictionary, string or range".to_string(),
                                found: a.get_type_str(globals),
                                val_def: globals.get_area(val),
                                info,
                            })
                        }
                    }
                    full_context.disable_breaks(BreakType::Loop);
                    full_context.disable_breaks(BreakType::ContinueLoop);
                    globals.pop_preserved();
                }
            }
            Break => {
                //set all contexts to broken
                for c in contexts.iter() {
                    (*c.inner()).broken = Some((BreakType::Loop, info.position));
                }
                break;
            }

            Continue => {
                //set all contexts to broken
                for c in contexts.iter() {
                    (*c.inner()).broken = Some((BreakType::ContinueLoop, info.position));
                }
                break;
            }

            Return(return_val) => {
                for full_context in contexts.iter() {
                    // let full_context = if statement.arrow {
                    //     *full_context = FullContext::Split(
                    //         full_context.clone().into(),
                    //         full_context.clone().into(),
                    //     );
                    //     if let FullContext::Split(_, c) = full_context {
                    //         &mut **c
                    //     } else {
                    //         unreachable!()
                    //     }
                    // } else {
                    //     full_context
                    // };
                    match return_val {
                        Some(val) => {
                            val.eval(full_context, globals, info.clone(), true)?;
                            for context in full_context.iter() {
                                let return_val = context.inner().return_value;
                                context.inner().broken = Some((
                                    BreakType::Macro(
                                        Some(clone_value(
                                            return_val,
                                            globals,
                                            context.inner().start_group,
                                            true,
                                            globals.get_area(return_val),
                                        )),
                                        statement.arrow,
                                    ),
                                    info.position,
                                ));
                            }
                        }

                        None => {
                            full_context.inner().broken =
                                Some((BreakType::Macro(None, statement.arrow), info.position));
                        }
                    };
                }
                if !statement.arrow {
                    break;
                }
            }

            Error(e) => {
                let mut errors = Vec::new();

                e.message.eval(contexts, globals, info.clone(), true)?;
                for c in contexts.iter() {
                    let err = globals.stored_values[c.inner().return_value]
                        .clone()
                        .to_str(globals);
                    errors.push((info.position, err))
                }

                let mut new_errors = Vec::new();
                for (area, msg) in errors.iter() {
                    new_errors.push((*area, msg.as_str()))
                }

                let err = create_error(info, "Runtime Error", &new_errors, None);

                return Err(RuntimeError::CustomError(err));
            }
        }

        contexts.reset_return_vals(globals);

        if let Some(c) = stored_context {
            globals.pop_preserved();
            //resetting the context if async
            let mut list = c;

            for context in contexts.with_breaks() {
                if let Some((r, i)) = context.inner().broken {
                    if let BreakType::Macro(_, true) = r {
                        list.push(context.clone());
                    } else {
                        return Err(RuntimeError::BreakNeverUsedError {
                            breaktype: r,
                            info: CompilerInfo::from_area(i),
                            broke: i,
                            dropped: info.position,
                            reason: "it's inside an arrow statement".to_string(),
                        });
                    }
                }
            }
            *contexts = FullContext::stack(&mut list.into_iter()).unwrap();
        }

        //try to merge contexts
        merge_all_contexts(contexts, globals, false);

        if contexts.iter().next().is_none() {
            break;
        }

        /*println!(
            "{} -> Compiled '{}' in {} milliseconds!",
            path,
            statement_type,
            start_time.elapsed().as_millis(),
        );*/

        let increase =
            globals.stored_values.map.len() as i32 - globals.stored_values.prev_value_count as i32;

        if increase > 5000 {
            globals.collect_garbage(contexts);
        }
    }

    // TODO: get rid of lifetimes

    contexts.exit_scope();

    Ok(())
}

pub fn do_assignment(
    destex: &ast::Expression,
    src: &Option<ast::Expression>,
    contexts: &mut FullContext,
    globals: &mut Globals,
    info: &CompilerInfo,
    mutable: bool,
    scope: i16,
    concat: Option<bool>
) -> Result<(), RuntimeError> {
    if destex.values.is_empty() {
        return Err(RuntimeError::CustomError(create_error(
            info.clone(),
            "Cannot destructure value into empty expression",
            &[],
            None,
        )));
    }
    if destex.values.len() > 1 || !destex.operators.is_empty() {
        let mut area = info.position;
        area.pos = destex.values[1].pos;
        return Err(RuntimeError::CustomError(create_error(
            info.clone(),
            "Cannot destructure value into this expression",
            &[(area, "Invalid destructure pattern")],
            None,
        )));
    }
    let dest = &destex.values[0];

    let destructure_sanitize = |src: &Option<_>| -> Result<(), RuntimeError> {
        //use parser::fmt::_format2;

        let dst = &destex.values[0];

        if dst.operator.is_some() || !dst.path.is_empty() {
            return Err(RuntimeError::CustomError(create_error(
                info.clone(),
                "Invalid destructure symbol",
                &[],
                Some("Remove any unary operators or extentions"),
            )));
        }
        if let None = src {
            return Err(RuntimeError::CustomError(create_error(
                info.clone(),
                "Destructure expressions require a value to destruct",
                &[],
                None,
            )));
        }
        Ok(())
    };

    if let ast::ValueBody::Array(arr) = &dest.value.body {
        destructure_sanitize(src)?;
        array_destructure_define(
            arr,
            src.as_ref().unwrap(),
            contexts,
            globals,
            info,
            mutable,
            scope,
            concat
        )?;
        Ok(())
    } else if let ast::ValueBody::Dictionary(kvs) = &dest.value.body {
        destructure_sanitize(src)?;
        dict_destructure_define(
            kvs, 
            info, 
            src, 
            contexts, 
            globals, 
            mutable, 
            concat
        )?;
        Ok(())
    } else {
        // no destructure here
        let value_is_normalized = if let Some(new_expr) = &src {
            new_expr.values.len() == 1
                && new_expr.values[0].path.is_empty()
                && new_expr.values[0].operator.is_none()
        } else {
            true
        };
        let pre_evaled = if let Some(new_expr) = &src {
            if value_is_normalized
                && matches!(
                    new_expr.values[0].value.body,
                    ast::ValueBody::CmpStmt(_) | ast::ValueBody::Macro(_)
                )
            {
                false
            } else {
                new_expr.eval(contexts, globals, info.clone(), mutable)?;
                true
            }
        } else {
            false
        };

        let defined = if concat != Some(false) {
            dest.try_define(contexts, globals, &info, mutable, scope)?
        } else {

            for f_c in contexts.iter() {
                let tmp = f_c.inner().return_value;
                dest.to_value(f_c, globals, info.clone(), false)?;
                f_c.inner().return_value2 = f_c.inner().return_value;
                f_c.inner().return_value = tmp;
            }

            DefineResult::Ok
        };

        for full_context in contexts.iter() {

            let (defined, storage) = 
                if let Some(overwrite) = concat {
                    let initial_storage = full_context.inner().return_value2;
                    if overwrite {
                        let stub = store_val_m(
                            Value::Null,
                            globals,
                            full_context.inner().start_group,
                            false,
                            info.position,
                        );
                        globals.stored_values[initial_storage] = Value::Array(vec![stub]);

                        (DefineResult::Ok, stub)
                    } else {
                        let stub = store_val_m(
                            Value::Null,
                            globals,
                            full_context.inner().start_group,
                            false,
                            info.position,
                        );

                        match &mut globals.stored_values[initial_storage] {
                            Value::Array(a) => a.push(stub),
                            _ => {
                                unreachable!()
                            }
                        }
                        (DefineResult::Ok, stub)
                    }
                } else {
                    let storage = full_context.inner().return_value2;
                    (defined, storage)
                };

            let assign_implemented = globals.stored_values[full_context.inner().return_value]
                .clone()
                .member(
                    globals.ASSIGN_BUILTIN,
                    full_context.inner(),
                    globals,
                    info.clone(),
                )
                .is_some();

            match (defined, mutable, assign_implemented) {
                (DefineResult::AlreadyDefined(false), false, false) // something is already defined
                | (DefineResult::AlreadyDefined(_), false, true) => { // something is already defined (may be redefinable), but has implemented assign
                    // convert to assign expr

                    if !globals.is_mutable(storage)
                        && globals.stored_values[storage]
                            .clone()
                            .member(
                                globals.ASSIGN_BUILTIN,
                                full_context.inner(),
                                globals,
                                info.clone(),
                            )
                            .is_none()
                    {
                        use parser::fmt::SpwnFmt;
                        let symbol = SpwnFmt::fmt(dest, 0);
                        return Err(RuntimeError::CustomError(create_error(
                            info.clone(),
                            &format!("This constant `{}` is already defined", symbol),
                            &[
                                (
                                    globals.get_area(storage),
                                    &format!("`{}` was first defined here", symbol),
                                ),
                                (info.position, "Attempted to redefine it here"),
                            ],
                            None,
                        )));
                    }

                    if let Some(value) = &src {
                        if pre_evaled {
                            handle_operator(
                                storage,
                                full_context.inner().return_value,
                                Builtin::AssignOp,
                                full_context,
                                globals,
                                info,
                            )?;
                        } else {
                            let symbol = ast::ValueBody::Resolved(storage)
                                .to_variable(dest.pos);
                            let mut expr = ast::Expression {
                                values: Vec::with_capacity(value.values.len() + 1),
                                operators: Vec::with_capacity(
                                    value.operators.len() + 1,
                                ),
                            };
                            expr.values.push(symbol);
                            expr.operators.push(ast::Operator::Assign);
                            expr.values.extend(value.values.clone());
                            expr.operators.extend(value.operators.iter().copied());
                            expr.eval(full_context, globals, info.clone(), true)?;
                        }
                    } else {
                        unreachable!()
                    }
                }
                _ => {
                    if let Some(new_expr) = &src {
                        match (value_is_normalized, &new_expr.values[0].value.body) {
                            (true, ast::ValueBody::CmpStmt(f)) => {
                                //to account for recursion
                                assert!(!pre_evaled);

                                //pick a start group
                                let start_group =
                                    Group::next_free(&mut globals.closed_groups);
                                //store value
                                globals.stored_values[storage] =
                                    Value::TriggerFunc(TriggerFunction { start_group });

                                full_context.inner().fn_context_change_stack =
                                    vec![info.position];
                                //new_info.last_context_change_stack = vec![info.position.clone()];

                                f.to_trigger_func(
                                    full_context,
                                    globals,
                                    info.clone(),
                                    Some(start_group),
                                )?;
                            }

                            (true, ast::ValueBody::Macro(m)) => {
                                assert!(!pre_evaled);

                                macro_to_value(
                                    m,
                                    full_context,
                                    globals,
                                    info.clone(),
                                    !mutable,
                                )?;

                                let (context, val) = full_context.inner_value();

                                //clone the value so as to not share the reference

                                let cloned = clone_and_get_value(
                                    val,
                                    globals,
                                    context.start_group,
                                    !mutable,
                                );

                                globals.stored_values[storage] = cloned;
                            }

                            _ => {
                                assert!(pre_evaled);
                                // dbg!(
                                //     &defined,
                                //     &mutable,
                                //     &same_module,
                                //     &assign_implemented
                                // );

                                let ctx = full_context.inner();

                                if !mutable {
                                    (*globals
                                        .stored_values
                                        .map
                                        .get_mut(storage)
                                        .unwrap())
                                    .def_area = CodeArea {
                                        file: info.position.file,
                                        pos: src.as_ref().unwrap().get_pos(),
                                    };
                                }
                                //clone the value so as to not share the reference

                                let cloned = clone_and_get_value(
                                    ctx.return_value,
                                    globals,
                                    ctx.start_group,
                                    !mutable,
                                );

                                globals.stored_values[storage] = cloned;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn dict_destructure_define(
    kvs: &Vec<ast::DictDef>,
    info: &CompilerInfo,
    src: &Option<ast::Expression>,
    contexts: &mut FullContext,
    globals: &mut Globals,
    mutable: bool,
    concat: Option<bool>,
) -> Result<(), RuntimeError> {
    let ranges: Vec<&ast::Expression> = kvs
        .iter()
        .filter_map(|x| match x {
            ast::DictDef::Extract(d) => Some(d),
            _ => None,
        })
        .collect();
    if ranges.len() > 1 {
        let mut area1 = info.position;
        let mut area2 = info.position;
        area1.pos = ranges[0].values[0].pos;
        area2.pos = ranges[1].values[0].pos;

        return Err(RuntimeError::CustomError(create_error(
            info.clone(),
            "Cannot spread a destructure multiple times",
            &[
                (area1, "First spread is here"),
                (area2, "Attempted to spread again"),
            ],
            None,
        )));
    }
    src.as_ref()
        .unwrap()
        .eval(contexts, globals, info.clone(), true)?;
    for ctx in contexts.iter() {
        let evaled_store = ctx.inner().return_value;

        let mut evaled_src = match globals.stored_values[evaled_store].clone() {
            Value::Dict(d) => d,
            b => {
                return Err(RuntimeError::TypeError {
                    expected: "dictionary".to_string(),
                    found: b.get_type_str(globals),
                    val_def: globals.get_area(evaled_store),
                    info: info.clone(),
                })
            }
        };

        let mut collected = Vec::<&LocalIntern<String>>::new();
        for kv in kvs {
            match kv {
                ast::DictDef::Extract(_) => (),
                ast::DictDef::Def((key, value)) => {
                    if !evaled_src.contains_key(key) {
                        return Err(RuntimeError::CustomError(create_error(
                            info.clone(),
                            &format!("Key '{}' not found", key),
                            &[],
                            None,
                        )));
                    }

                    collected.push(key);
                    do_assignment(
                        &value,
                        &Some(stored_to_variable(evaled_src[key], globals).to_expression()),
                        ctx,
                        globals,
                        &info,
                        mutable,
                        0,
                        concat
                    )?;
                }
            }
        }

        if ranges.len() > 0 {
            for k in collected {
                evaled_src.remove(k);
            }

            let ast_src = evaled_src
                .keys()
                .map(|x| {
                    ast::DictDef::Def((
                        *x,
                        stored_to_variable(evaled_src[x], globals).to_expression(),
                    ))
                })
                .collect();

            do_assignment(
                ranges[0],
                &Some(
                    ast::ValueBody::Dictionary(ast_src)
                        .to_variable((0, 0))
                        .to_expression(),
                ),
                ctx,
                globals,
                &info,
                mutable,
                0,
                concat
            )?;
        }
    }
    Ok(())
}

fn array_destructure_define(
    arr: &Vec<ast::ArrayDef>,
    value: &ast::Expression,
    contexts: &mut FullContext,
    globals: &mut Globals,
    info: &CompilerInfo,
    mutable: bool,
    scope: i16,
    concat: Option<bool>,
) -> Result<(), RuntimeError> {
    value.eval(contexts, globals, info.clone(), true)?;
    for ctx in contexts.iter() {
        match globals.stored_values[ctx.inner().return_value].clone() {
            Value::Array(val_a) => {
                let ranges = arr
                    .iter()
                    .filter(|x| matches!(x.operator, Some(ast::ArrayPrefix::Collect | ast::ArrayPrefix::Spread)))
                    .map(|x| &x.value)
                    .collect::<Vec<_>>();

                if ranges.len() > 1 {
                    let mut area1 = info.position;
                    let mut area2 = info.position;
                    area1.pos = ranges[0].values[0].pos;
                    area2.pos = ranges[1].values[0].pos;

                    return Err(RuntimeError::CustomError(create_error(
                        info.clone(),
                        "Cannot spread a destructure multiple times",
                        &[
                            (area1, "First spread is here"),
                            (area2, "Attempted to spread again"),
                        ],
                        None,
                    )));
                }

                if (arr.len() < val_a.len() && ranges.is_empty()) || arr.len() > val_a.len() {
                    return Err(RuntimeError::CustomError(create_error(
                        info.clone(),
                        &format!(
                            "Expected {} items to destructure, found {}",
                            arr.len(),
                            val_a.len()
                        ),
                        &[],
                        None,
                    )));
                } else {
                    let mut idx: usize = 0;
                    let mut var_idx: usize = 0;
                    loop {
                        let mut idx_step = 1;
                        for expr_ctx in ctx.iter() {
                            if arr[var_idx].value.values.is_empty() {
                                return Err(RuntimeError::CustomError(create_error(
                                    info.clone(),
                                    "Cannot destructure value into empty expression",
                                    &[],
                                    None,
                                )));
                            }

                            let the_expr = &arr[var_idx].value;
                            match arr[var_idx].operator {
                                Some(ast::ArrayPrefix::Collect | ast::ArrayPrefix::Spread) => {
                                    idx_step = 1 + val_a.len() - arr.len();

                                    /*let astvec = ast::ValueBody::Array(
                                        (idx..(idx + idx_step))
                                            .map(|tmp_idx| ast::ArrayDef {
                                                value: ast::ValueBody::Resolved(val_a[tmp_idx])
                                                    .to_variable(the_expr.values[0].pos)
                                                    .to_expression(),
                                                operator: None,
                                            })
                                            .collect(),
                                    )
                                    .to_variable(the_expr.values[0].pos)
                                    .to_expression();*/

                                    let mut overwrite = true;
                                    for i in idx..(idx + idx_step) {

                                        do_assignment(
                                            the_expr,
                                            &Some(ast::ValueBody::Resolved(val_a[i])
                                                    .to_variable(the_expr.values[0].pos)
                                                    .to_expression()
                                            ),
                                            expr_ctx,
                                            globals,
                                            info,
                                            mutable,
                                            scope,
                                            Some(overwrite)
                                        )?;
                                        overwrite = false;
                                    }
                                }
                                _ => {
                                    do_assignment(
                                        the_expr,
                                        &Some(
                                            stored_to_variable(val_a[idx], globals).to_expression(),
                                        ),
                                        expr_ctx,
                                        globals,
                                        info,
                                        mutable,
                                        scope,
                                        concat
                                    )?;
                                }
                            }
                        }

                        idx += idx_step;
                        var_idx += 1;
                        if idx >= val_a.len() {
                            break;
                        }
                    }
                }
            }
            b => {
                return Err(RuntimeError::TypeError {
                    expected: "array".to_string(),
                    found: b.get_type_str(globals),
                    val_def: globals.get_area(ctx.inner().return_value),
                    info: info.clone(),
                })
            }
        }
    }
    Ok(())
}

pub fn merge_all_contexts(
    contexts: &mut FullContext,
    globals: &mut Globals,
    check_return_vals: bool,
) {
    if let FullContext::Split(_, _) = contexts {
        let mut broken = Vec::new();
        let mut not_broken = Vec::new();
        for c in contexts.with_breaks() {
            if c.inner().broken.is_some() {
                broken.push(c.inner().clone())
            } else {
                not_broken.push(c.inner().clone())
            }
        }

        if not_broken.len() > 1 {
            loop {
                if !merge_contexts(&mut not_broken, globals, check_return_vals) {
                    break;
                }
            }

            broken.extend(not_broken);

            *contexts =
                FullContext::stack(&mut broken.into_iter().map(FullContext::Single)).unwrap();
        }
    }
}

fn merge_impl(target: &mut Implementations, source: &Implementations) {
    for (key, imp) in source.iter() {
        match target.get_mut(key) {
            Some(target_imp) => (*target_imp).extend(imp.iter().map(|x| (*x.0, *x.1))),
            None => {
                (*target).insert(*key, imp.clone());
            }
        }
    }
}

pub fn get_import_path(
    path: &ImportType,
    globals: &mut Globals,
    info: CompilerInfo,
) -> Result<PathBuf, RuntimeError> {
    Ok(match path {
        ImportType::Script(p) => {
            let p = if let SpwnSource::File(f) = globals.path.as_ref() {
                f
            } else {
                // should never actually display
                return Err(RuntimeError::CustomError(create_error(
                    Default::default(),
                    "Path is built in",
                    &[],
                    None,
                )));
            }
            .parent()
            .expect("Your file must be in a folder to import modules!")
            .join(&p);
            if !p.exists() {
                return Err(RuntimeError::CustomError(create_error(
                    info,
                    &format!("Couldn't find module file ({})", p.to_string_lossy()),
                    &[],
                    None,
                )));
            };
            p
        }

        ImportType::Lib(name) => {
            let mut outpath = globals.includes[0].clone();
            let mut found = false;

            for path in &globals.includes {
                if path.join("libraries").join(name).exists() {
                    outpath = path.to_path_buf();
                    found = true;
                    break;
                }
            }

            if found {
                outpath
            } else {
                let labels = globals
                    .includes
                    .iter()
                    .map(|p| {
                        (
                            info.position,
                            format!("Not found in {}", p.to_string_lossy()),
                        )
                    })
                    .collect::<Vec<_>>();
                let mut new_labels = Vec::new();

                for (area, text) in labels.iter() {
                    new_labels.push((*area, text.as_str()));
                }
                return Err(RuntimeError::CustomError(create_error(
                    info,
                    "Unable to find library folder",
                    &new_labels,
                    None,
                )));
            }
        }
        // .parent()
        // .unwrap()
        .join("libraries")
        .join(name),
    })
}

pub fn import_module(
    path: &ImportType,
    contexts: &mut FullContext,
    globals: &mut Globals,
    info: CompilerInfo,
    forced: bool,
) -> Result<(), RuntimeError> {
    if !forced {
        if let Some(ret) = globals.prev_imports.get(path).cloned() {
            merge_impl(&mut globals.implementations, &ret.1);
            for c in contexts.iter() {
                c.inner().return_value = ret.0;
            }
            return Ok(());
        }
    }
    let built_in_path = match path {
        ImportType::Script(a) => {
            if let Some(mut p) = globals.built_in_path.clone() {
                p.push(a);
                p
            } else {
                a.clone()
            }
        }
        ImportType::Lib(a) => {
            let mut p = PathBuf::from(a);
            p.push("lib.spwn");
            p
        }
    };

    let stored_built_in_path = globals.built_in_path.clone();
    #[cfg(not(target_arch = "wasm32"))]
    let (unparsed, module_path) = match get_import_path(path, globals, info.clone()) {
        Err(err) => {
            if let Some(file) = get_lib_file(&built_in_path) {
                (*globals).built_in_path = Some(built_in_path.parent().unwrap().to_path_buf());
                (
                    file.contents_utf8()
                        .expect("Bad built-in library file")
                        .to_string(),
                    SpwnSource::BuiltIn(built_in_path),
                )
            } else {
                return Err(err);
            }
        }
        Ok(mut module_path) => {
            if module_path.is_dir() {
                module_path = module_path.join("lib.spwn");
            } else if module_path.is_file() && module_path.extension().is_none() {
                module_path.set_extension("spwn");
            }
            if let Some(ext) = module_path.extension() {
                if ext != "spwn" {
                    return Err(RuntimeError::CustomError(create_error(
                        info,
                        &format!(
                            "Imported files must have a .spwn extension (found {})",
                            ext.to_string_lossy()
                        ),
                        &[],
                        None,
                    )));
                }
            }

            (
                match fs::read_to_string(&module_path) {
                    Ok(content) => content,
                    Err(e) => {
                        return Err(RuntimeError::CustomError(create_error(
                            info.clone(),
                            &format!(
                                "Something went wrong when opening library file ({})",
                                module_path.to_string_lossy(),
                            ),
                            &[(info.position, &format!("{}", e))],
                            None,
                        )));
                    }
                },
                SpwnSource::File(module_path),
            )
        }
    };

    #[cfg(target_arch = "wasm32")]
    let (unparsed, module_path) = match get_lib_file(&built_in_path) {
        Some(file) => {
            (*globals).built_in_path = Some(built_in_path.parent().unwrap().to_path_buf());
            (
                file.contents_utf8()
                    .expect("Bad built-in library file")
                    .to_string(),
                SpwnSource::BuiltIn(built_in_path),
            )
        }
        None => {
            panic!("file {:?} not found", built_in_path)
        }
    };

    let (parsed, notes) =
        match parser::parser::parse_spwn(unparsed, module_path.clone(), BUILTIN_NAMES) {
            Ok(p) => p,
            Err(err) => return Err(RuntimeError::PackageSyntaxError { err, info }),
        };

    let mut start_context = FullContext::new(globals);

    globals.push_new_preserved();
    for c in contexts.with_breaks() {
        for stack in c.inner().get_variables().values() {
            for VariableData { val: v, .. } in stack.iter() {
                globals.push_preserved_val(*v);
            }
        }
    }

    let mut stored_impl = None;
    if let ImportType::Lib(_) = path {
        let mut impl_vals = Vec::new();
        for imp in globals.implementations.values() {
            for (v, _) in imp.values() {
                impl_vals.push(*v);
            }
        }
        for v in impl_vals {
            globals.push_preserved_val(v);
        }
        let mut stored = FnvHashMap::default();

        mem::swap(&mut stored, &mut globals.implementations);
        stored_impl = Some(stored);
    }

    if !notes.tag.tags.iter().any(|x| x.0 == "no_std") {
        import_module(
            &ImportType::Lib(STD_PATH.to_string()),
            &mut start_context,
            globals,
            info.clone(),
            false,
        )?;

        if let FullContext::Split(_, _) = start_context {
            return Err(RuntimeError::CustomError(create_error(
                info,
                "The standard library can not split the context",
                &[],
                None,
            )));
        }

        if let Value::Dict(d) = &globals.stored_values[start_context.inner().return_value] {
            for (a, b, c) in d.iter().map(|(k, v)| (*k, *v, -1)) {
                start_context.inner().new_redefinable_variable(a, b, c)
            }
        } else {
            return Err(RuntimeError::CustomError(create_error(
                info,
                "The standard library must return a dictionary",
                &[],
                None,
            )));
        }
    }

    let stored_path = globals.path;

    (*globals).path = LocalIntern::new(module_path);

    let mut new_info = info.clone();

    new_info.position.file = (*globals).path;
    new_info.position.pos = (0, 0);

    if let ImportType::Lib(l) = path {
        new_info.current_module = l.clone();
    }

    match compile_scope(&parsed, &mut start_context, globals, new_info) {
        Ok(_) => (),
        Err(err) => {
            return Err(RuntimeError::PackageError {
                err: Box::new(err),
                info,
            })
        }
    };

    globals.pop_preserved();

    let save_value = notes.tag.tags.iter().any(|x| x.0 == "cache_output");
    let mut out_values = 0;
    let mut output_saved = None;
    let mut impl_saved = None;

    for fc in start_context.with_breaks() {
        let c = fc.inner();
        if let Some((r, i)) = c.broken {
            if let BreakType::Macro(v, _) = r {
                for full_context in contexts.iter() {
                    let fn_context = full_context.inner().start_group;
                    (*full_context).inner().return_value = match v {
                        Some(v) => {
                            if save_value {
                                if out_values > 0 {
                                    return Err(RuntimeError::CustomError(create_error(
                                        info,
                                        "Cannot cache a context splitting library",
                                        &[],
                                        None,
                                    )));
                                }
                                output_saved = Some(v);
                            }
                            out_values += 1;
                            clone_value(v, globals, fn_context, true, info.position)
                        }
                        None => globals.NULL_STORAGE,
                    };
                }
            } else {
                return Err(RuntimeError::BreakNeverUsedError {
                    breaktype: r,
                    info: CompilerInfo::from_area(i),
                    broke: i,
                    dropped: info.position,
                    reason: "the file ended".to_string(),
                });
            }
        }
    }
    (*globals).path = stored_path;
    (*globals).built_in_path = stored_built_in_path;

    if let Some(stored_impl) = stored_impl {
        //change and delete from impls
        let mut to_be_deleted = Vec::new();
        for (k1, imp) in &mut globals.implementations {
            for (k2, (_, in_scope)) in imp {
                if *in_scope {
                    (*in_scope) = false;
                } else {
                    to_be_deleted.push((*k1, *k2));
                }
                // globals
                //     .stored_values
                //     .increment_single_lifetime(*val, 1, &mut FnvHashSet::new());
            }
        }
        for (k1, k2) in to_be_deleted {
            (*globals).implementations.get_mut(&k1).unwrap().remove(&k2);
        }
        if save_value {
            impl_saved = Some(globals.implementations.clone());
        }

        //merge impls
        merge_impl(&mut globals.implementations, &stored_impl);
    } else if save_value {
        impl_saved = Some(globals.implementations.clone());
    }

    if save_value {
        globals.prev_imports.insert(
            path.clone(),
            (
                output_saved.unwrap_or(globals.NULL_STORAGE),
                impl_saved.unwrap_or_default(),
            ),
        );
    }

    Ok(())
}

// const ID_MAX: u16 = 999;

// pub fn next_free(
//     ids: &mut Vec<u16>,
//     id_class: ast::IDClass,
//     info: CompilerInfo,
// ) -> Result<ID, RuntimeError> {
//     for i in 1..ID_MAX {
//         if !ids.contains(&i) {
//             (*ids).push(i);
//             return Ok(i);
//         }
//     }

//     Err(RuntimeError::IDError { id_class, info })
//     //panic!("All ids of this type are used up!");
// }
