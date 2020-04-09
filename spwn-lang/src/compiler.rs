//! Tools for compiling SPWN into GD object strings
use crate::ast;
use crate::levelstring::*;
use crate::native::*;
use std::collections::HashMap;

//use std::collections::HashMap;
use crate::parser::ParseNotes;
use std::fs;
use std::path::PathBuf;

use crate::compiler_types::*;
//use ValSuccess::{Evaluatable, Literal};

pub fn compile_spwn(
    statements: Vec<ast::Statement>,
    path: PathBuf,
    gd_path: PathBuf,
    notes: ParseNotes,
) -> (Globals, String) {
    //variables that get changed throughout the compiling
    let mut globals = Globals {
        closed_groups: notes.closed_groups,
        closed_colors: notes.closed_colors,
        closed_blocks: notes.closed_blocks,
        closed_items: notes.closed_items,
        path: path,
        obj_list: Vec::new(),
        lowest_y: HashMap::new(),
        stored_values: Vec::new(),
    };

    println!("Loading level data...");

    let file_content =
        fs::read_to_string(gd_path).expect("Your local geometry dash files were not found");
    let level_string = get_level_string(file_content)
        //remove previous spwn objects
        .split(";")
        .map(|obj| if obj.contains("108,7777") { "" } else { obj })
        .collect::<Vec<&str>>()
        .join(";");
    get_used_ids(&level_string, &mut globals);

    let start_info = CompilerInfo {
        depth: 0,
        path: vec!["main scope".to_string()],
        line: statements[0].line,
    };
    use std::time::Instant;

    println!("Compiling script...");
    let start_time = Instant::now();

    compile_scope(&statements, vec![Context::new()], &mut globals, start_info);

    println!(
        "Compiled in {} milliseconds!",
        start_time.elapsed().as_millis()
    );

    (globals, level_string)
}

pub fn compile_scope(
    statements: &Vec<ast::Statement>,
    mut contexts: Vec<Context>,
    globals: &mut Globals,
    mut info: CompilerInfo,
) -> (Vec<Context>, Returns) {
    let mut statements_iter = statements.iter();

    let mut returns: Returns = Vec::new();

    /*let indent = {
        let mut new_string = String::new();
        for _ in 0..info.depth {
            new_string += "|-->";
        }
        new_string
    };*/

    while let Some(statement) = statements_iter.next() {
        //find out what kind of statement this is
        //let start_time = Instant::now();

        println!(
            "{} -> Compiling a statement in {} contexts",
            info.path.join(">"),
            contexts.len()
        );
        use ast::StatementBody::*;

        let stored_context = if statement.arrow {
            Some(contexts.clone())
        } else {
            None
        };

        info.line = statement.line;

        match &statement.body {
            Expr(expr) => {
                let mut new_contexts: Vec<Context> = Vec::new();
                for context in contexts {
                    //we dont care about the return value in this case
                    let (evaled, inner_returns) = expr.eval(context, globals, info.clone());
                    returns.extend(inner_returns);
                    new_contexts.extend(evaled.iter().map(|x| x.1.clone()));
                }
                contexts = new_contexts;
            }

            Definition(def) => {
                let mut all_values: Returns = Vec::new();

                for context in contexts {
                    if let ast::ValueLiteral::CmpStmt(f) = &def.value.values[0].value {
                        if def.value.values.len() == 1 {
                            //create the function context
                            let mut new_context = context.clone();
                            new_context.spawn_triggered = true;
                            //pick a start group
                            let start_group = Group {
                                id: next_free(&mut globals.closed_groups),
                            };
                            new_context.variables.insert(
                                def.symbol.clone(),
                                store_value(Value::Func(Function { start_group }), globals),
                            );
                            all_values.push((Value::Func(Function { start_group }), context));
                            new_context.start_group = start_group;
                            let (_, inner_returns) = compile_scope(
                                &f.statements,
                                vec![new_context],
                                globals,
                                info.next("function body"),
                            );
                            returns.extend(inner_returns);
                        } else {
                            let (evaled, inner_returns) =
                                def.value.eval(context, globals, info.clone());
                            returns.extend(inner_returns);
                            all_values.extend(evaled);
                        }
                    } else {
                        let (evaled, inner_returns) =
                            def.value.eval(context, globals, info.clone());
                        returns.extend(inner_returns);
                        all_values.extend(evaled);
                    }
                    //copied because im lazy
                }
                contexts = Vec::new();
                for (val, mut context) in all_values {
                    if def.symbol == "*" {
                        match val {
                            Value::Dict(d) => {
                                context.variables.extend(d.clone());
                            }
                            _ => panic!(compile_error(
                                "Only dict can have their values extracted",
                                info
                            )),
                        }
                    } else {
                        context
                            .variables
                            .insert(String::from(&def.symbol), store_value(val, globals));
                    }
                    contexts.push(context);
                }
            }

            If(if_stmt) => {
                let mut all_values: Returns = Vec::new();
                for context in contexts {
                    let (evaled, inner_returns) =
                        if_stmt
                            .condition
                            .eval(context, globals, info.next("if condition"));
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }
                contexts = Vec::new();

                for (val, context) in all_values {
                    match val {
                        Value::Bool(b) => {
                            //internal if statement
                            if b {
                                let compiled = compile_scope(
                                    &if_stmt.if_body,
                                    vec![context],
                                    globals,
                                    info.next("if body"),
                                );
                                returns.extend(compiled.1);
                            // TODO: add the returns from these scopes
                            } else {
                                match &if_stmt.else_body {
                                    Some(body) => {
                                        let compiled = compile_scope(
                                            body,
                                            vec![context],
                                            globals,
                                            info.next("else body"),
                                        );
                                        returns.extend(compiled.1);
                                    }
                                    None => {}
                                };
                            }
                        }
                        _ => panic!(compile_error(
                            "Expected boolean condition in if statement",
                            info
                        )),
                    }
                }
            }

            Impl(imp) => {
                let mut new_contexts: Vec<Context> = Vec::new();
                for context in contexts.clone() {
                    let (evaled, inner_returns) = imp.symbol.to_value(
                        context.clone(),
                        globals,
                        info.next("implementation symbol"),
                    );
                    returns.extend(inner_returns);
                    for (typ, c) in evaled {
                        if let Value::Str(s) = typ {
                            let (evaled, inner_returns) = eval_dict(
                                imp.members.clone(),
                                c,
                                globals,
                                info.next("implementation"),
                            );
                            returns.extend(inner_returns);
                            for (val, c2) in evaled {
                                let mut new_context = c2.clone();
                                if let Value::Dict(d) = val {
                                    match new_context.implementations.get_mut(&s) {
                                        Some(implementation) => {
                                            for (key, val) in d.into_iter() {
                                                (*implementation).insert(key, val);
                                            }
                                        }
                                        None => {
                                            new_context.implementations.insert(s.clone(), d);
                                        }
                                    }
                                } else {
                                    unreachable!();
                                }
                                new_contexts.push(new_context);
                            }
                        /**/
                        } else {
                            panic!(compile_error("Must implement on a type (a string)", info));
                        }
                    }
                }
                //println!("{:?}", new_contexts[0].implementations);
                contexts = new_contexts;

                /**/
            }
            Call(call) => {
                for context in &mut contexts {
                    context.x += 1;
                }
                let mut all_values: Returns = Vec::new();
                for context in contexts {
                    let (evaled, inner_returns) =
                        call.function.to_value(context, globals, info.clone());
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }
                contexts = Vec::new();
                let mut obj_list = Vec::<GDObj>::new();
                for (func, context) in all_values {
                    contexts.push(context.clone());
                    obj_list.push(
                        GDObj {
                            obj_id: 1268,
                            groups: vec![context.start_group],
                            target: match func {
                                Value::Func(g) => g.start_group,
                                Value::Group(g) => g,
                                _ => panic!(compile_error("Not callable", info)),
                            },

                            ..context_trigger(context.clone(), globals)
                        }
                        .context_parameters(context.clone()),
                    );
                }
                (*globals).obj_list.extend(obj_list);
            }

            Add(v) => {
                for context in &mut contexts {
                    context.x += 1;
                }
                let mut all_values: Returns = Vec::new();
                for context in contexts {
                    let (evaled, inner_returns) = v.eval(context, globals, info.clone());
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }
                contexts = Vec::new();
                let mut obj_list = Vec::<GDObj>::new();
                for (val, context) in all_values {
                    contexts.push(context.clone());
                    match val {
                        Value::Obj(obj) => {
                            obj_list.push(
                                GDObj {
                                    params: obj,
                                    groups: vec![context.start_group],
                                    ..context_trigger(context.clone(), globals)
                                }
                                .context_parameters(context.clone()),
                            );
                        }

                        _ => panic!(compile_error("Expected Object", info)),
                    }
                }
                (*globals).obj_list.extend(obj_list);
            }
            For(f) => {
                let mut all_arrays: Returns = Vec::new();
                for context in contexts {
                    let (evaled, inner_returns) = f.array.eval(context, globals, info.clone());
                    returns.extend(inner_returns);
                    all_arrays.extend(evaled);
                }
                contexts = Vec::new();
                for (val, context) in all_arrays {
                    match val {
                        Value::Array(arr) => {
                            let mut new_contexts = vec![context];

                            for element in arr {
                                for mut c in new_contexts.clone() {
                                    c.variables.insert(
                                        f.symbol.clone(),
                                        store_value(element.clone(), globals),
                                    ); //this will store a lot of values, maybe fix this sometime idk

                                    let (end_contexts, inner_returns) = compile_scope(
                                        &f.body,
                                        vec![c],
                                        globals,
                                        info.next("for loop"),
                                    );
                                    returns.extend(inner_returns);
                                    new_contexts = end_contexts;
                                }
                            }
                            contexts.extend(new_contexts);
                        }

                        _ => panic!(compile_error(
                            &format!(
                                "Expected array, got {:?}",
                                val.member("TYPE".to_string(), &context, globals, info.clone())
                            ),
                            info
                        )),
                    }
                }
            }
            Return(val) => {
                let mut all_values: Returns = Vec::new();
                for context in contexts.clone() {
                    let (evaled, inner_returns) =
                        val.eval(context, globals, info.next("return value"));
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }

                returns.extend(all_values);
            }

            Error(e) => {
                for context in contexts.clone() {
                    let (evaled, _) = e.message.eval(context, globals, info.next("return value"));
                    for (msg, _) in evaled {
                        println!(
                            "ERROR: {:?}",
                            match msg {
                                Value::Str(s) => s,
                                _ => "no message".to_string(),
                            }
                        );
                    }
                }
                panic!(compile_error(
                    "Error statement, see message(s) above.",
                    info
                ))
            }

            EOI => {}
        }
        if let Some(c) = stored_context {
            //resetting the context if async
            contexts = c;
        }

        /*println!(
            "{} -> Compiled '{}' in {} milliseconds!",
            path,
            statement_type,
            start_time.elapsed().as_millis(),
        );*/
    }

    //(*globals).highest_x = context.x;
    (contexts, returns)
}

pub fn import_module(
    path: &PathBuf,
    globals: &mut Globals,
    info: CompilerInfo,
) -> (Value, Implementations) {
    let module_path = globals
        .path
        .clone()
        .parent()
        .expect("Your file must be in a folder to import modules!")
        .join(&path);
    let (parsed, notes) = crate::parse_spwn(&module_path);
    (*globals).closed_groups.extend(notes.closed_groups);
    (*globals).closed_colors.extend(notes.closed_colors);
    (*globals).closed_blocks.extend(notes.closed_blocks);
    (*globals).closed_items.extend(notes.closed_items);
    let (contexts, _) = compile_scope(&parsed, vec![Context::new()], globals, info.next("module"));
    if contexts.len() > 1 {
        panic!(
            "Imported files does not (currently) support context splitting in the main scope.
            Please remove any context splitting statements outside of function or macro definitions."
        )
    }
    (
        match contexts[0].variables.get("exports") {
            Some(val) => (*globals).stored_values[*val as usize].clone(),
            None => Value::Null,
        },
        contexts[0].implementations.clone(),
    )
}

const ID_MAX: u16 = 999;

pub fn next_free(ids: &mut Vec<u16>) -> u16 {
    for i in 1..ID_MAX {
        if !ids.contains(&i) {
            (*ids).push(i);
            return i;
        }
    }
    panic!("All ids of this type are used up!");
}
