//! Tools for compiling SPWN into GD object strings
use crate::ast;
use crate::levelstring::*;
use crate::native::*;

//use std::collections::HashMap;
use std::path::PathBuf;

use crate::compiler_types::*;
//use ValSuccess::{Evaluatable, Literal};

pub fn compile_spwn(statements: Vec<ast::Statement>, path: PathBuf) -> Globals {
    //context at the start of the program

    //variables that get changed throughout the compiling
    let mut globals = Globals {
        closed_groups: Vec::new(),
        closed_colors: Vec::new(),
        closed_blocks: Vec::new(),
        closed_items: Vec::new(),
        path: path,
        obj_list: Vec::new(),
        highest_x: 0,
        stored_values: Vec::new(),
    };

    /*let file_content =
        fs::read_to_string("C:/Users/spu7n/AppData/Local/GeometryDash/CCLocalLevels.dat")
            .expect("Something went wrong reading the file");
    let level_string = get_level_string(file_content);

    let objects = level_string.split(";");

    for obj in objects {
        let props: Vec<&str> = obj.split(",").collect();
        for i in (0..props.len() - 1).step_by(2) {
            let key = props[i];
            let value = props[i + 1];

            match key {
                "57" => {
                    //GROUPS
                    let groups = value.split(".");
                    for g in groups {
                        let group = Group {
                            id: g.parse().unwrap(),
                        };
                        if !globals.closed_groups.contains(&group.id) {
                            globals.closed_groups.push(group.id);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    println!("{:?}", globals.closed_groups);*/
    let start_info = CompilerInfo {
        depth: 0,
        path: vec!["main scope".to_string()],
        line: statements[0].line,
    };

    compile_scope(&statements, vec![Context::new()], &mut globals, start_info);

    globals
}

pub fn compile_scope(
    statements: &Vec<ast::Statement>,
    mut contexts: Vec<Context>,
    globals: &mut Globals,
    mut info: CompilerInfo,
) -> (Vec<Context>, Returns) {
    for context in &mut contexts {
        context.x = globals.highest_x;
    }
    (*globals).highest_x += 30;

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

        /*println!(
            "{} -> Compiling a statement in {} contexts",
            path,
            contexts.len()
        );*/
        let mut statement_type: &str = "";
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
                                store_value(Value::Func(start_group), globals),
                            );
                            all_values.push((Value::Func(start_group), context));
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
                statement_type = "if";
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
                statement_type = "impl";
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
                statement_type = "call";
                let mut all_values: Returns = Vec::new();
                for context in contexts {
                    let (evaled, inner_returns) =
                        call.function.to_value(context, globals, info.clone());
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }
                contexts = Vec::new();
                for (func, context) in all_values {
                    contexts.push(context.clone());
                    (*globals).obj_list.push(
                        GDObj {
                            obj_id: 1268,
                            groups: vec![context.start_group],
                            target: match func {
                                Value::Func(g) => g,
                                Value::Group(g) => g,
                                _ => panic!(compile_error("Not callable", info)),
                            },

                            ..context_trigger(context.clone())
                        }
                        .context_parameters(context.clone()),
                    );
                }
            }

            Add(v) => {
                statement_type = "add";
                let mut all_values: Returns = Vec::new();
                for context in contexts {
                    let (evaled, inner_returns) = v.eval(context, globals, info.clone());
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }
                contexts = Vec::new();
                for (val, context) in all_values {
                    contexts.push(context.clone());
                    match val {
                        Value::Obj(obj) => {
                            (*globals).obj_list.push(
                                GDObj {
                                    params: obj,
                                    groups: vec![context.start_group],
                                    ..context_trigger(context.clone())
                                }
                                .context_parameters(context.clone()),
                            );
                        }

                        _ => panic!(compile_error("Expected Object", info)),
                    }
                }
            }
            For(f) => {
                statement_type = "for loop";

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
                statement_type = "return";
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
    let parsed = crate::parse_spwn(&module_path);
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
