//! Tools for compiling SPWN into GD object strings
use crate::ast;
use crate::builtin::*;
use crate::levelstring::*;
use std::collections::HashMap;

//use std::collections::HashMap;
use crate::parser::{ParseNotes, SyntaxError};
use std::fs;
use std::path::PathBuf;

use crate::compiler_types::*;
//use ValSuccess::{Evaluatable, Literal};

#[derive(Debug)]
pub enum RuntimeError {
    UndefinedErr {
        undefined: String,
        pos: (usize, usize),
    },

    PackageSyntaxError {
        err: SyntaxError,
        pos: (usize, usize),
    },

    IDError {
        id_class: ast::IDClass,
        pos: (usize, usize),
    },

    RuntimeError {
        message: String,
        pos: (usize, usize),
    },

    BuiltinError {
        message: String,
        pos: (usize, usize),
    },
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //write!(f, "SuperErrorSideKick is here!")
        //let mut message = String::from("Runtime/compile error:");
        match self {
            RuntimeError::UndefinedErr { undefined, pos } => write!(
                f,
                "'{}' is not defined at line {}, pos {}",
                undefined, pos.0, pos.1
            ),
            RuntimeError::PackageSyntaxError { err, pos } => write!(
                f,
                "Error when parsing library at line {}, pos {}: {}",
                pos.0, pos.1, err
            ),
            RuntimeError::IDError { id_class, pos } => write!(
                f,
                "Ran out of {} at line {}, pos {}",
                match id_class {
                    ast::IDClass::Group => "groups",
                    ast::IDClass::Color => "colors",
                    ast::IDClass::Item => "item IDs",
                    ast::IDClass::Block => "collision block IDs",
                },
                pos.0,
                pos.1
            ),
            RuntimeError::RuntimeError { message, pos } => {
                write!(f, "{} (line {}, pos {})", message, pos.0, pos.1)
            }

            RuntimeError::BuiltinError { message, pos } => write!(
                f,
                "Error when calling built-in-function: {} (line {}, pos {})",
                message, pos.0, pos.1
            ),
        }
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub fn compile_spwn(
    statements: Vec<ast::Statement>,
    path: PathBuf,
    gd_path: PathBuf,
    notes: ParseNotes,
) -> Result<(Globals, String), RuntimeError> {
    //variables that get changed throughout the compiling
    let mut globals = Globals {
        closed_groups: notes.closed_groups,
        closed_colors: notes.closed_colors,
        closed_blocks: notes.closed_blocks,
        closed_items: notes.closed_items,
        path,

        lowest_y: HashMap::new(),

        type_ids: HashMap::new(),
        type_id_count: 15,

        stored_values: Vec::new(),
        func_ids: vec![FunctionID {
            name: "main scope".to_string(),
            parent: None,
            width: None,
            obj_list: Vec::new(),
        }],
    };

    globals.type_ids.insert(String::from("group"), 0);
    globals.type_ids.insert(String::from("color"), 1);
    globals.type_ids.insert(String::from("block"), 2);
    globals.type_ids.insert(String::from("item"), 3);
    globals.type_ids.insert(String::from("number"), 4);
    globals.type_ids.insert(String::from("bool"), 5);
    globals.type_ids.insert(String::from("function"), 6);
    globals.type_ids.insert(String::from("dictionary"), 7);
    globals.type_ids.insert(String::from("macro"), 8);
    globals.type_ids.insert(String::from("string"), 9);
    globals.type_ids.insert(String::from("array"), 10);
    globals.type_ids.insert(String::from("object"), 11);
    globals.type_ids.insert(String::from("spwn"), 13);
    globals.type_ids.insert(String::from("builtin"), 13);
    globals.type_ids.insert(String::from("type"), 14);
    globals.type_ids.insert(String::from("null"), 15);

    println!("Loading level data...");

    let file_content =
        fs::read_to_string(gd_path).expect("Your local geometry dash files were not found");
    let level_string = get_level_string(file_content)
        //remove previous spwn objects
        .split(";")
        .map(|obj| {
            let key_val: Vec<&str> = obj.split(",").collect();
            let mut ret = obj;
            for i in (0..key_val.len()).step_by(2) {
                if key_val[i] == "57" {
                    let mut groups = key_val[i + 1].split(".");
                    if groups.any(|x| x == SPWN_SIGNATURE_GROUP) {
                        ret = "";
                    }
                }
            }
            ret
        })
        .collect::<Vec<&str>>()
        .join(";");
    get_used_ids(&level_string, &mut globals);

    let start_info = CompilerInfo {
        depth: 0,
        path: vec!["main scope".to_string()],
        line: statements[0].line,
        func_id: 0,
    };
    use std::time::Instant;

    println!("Compiling script...");
    let start_time = Instant::now();

    compile_scope(&statements, vec![Context::new()], &mut globals, start_info)?;

    //delete all unused func ids

    //let all func_id's parents be with objects
    /*let mut new_func_ids = Vec::<FunctionID>::new();

    println!("Func id len: {}", globals.func_ids.len());

    for id in &globals.func_ids {
        if !id.obj_list.is_empty() {
            let mut new_id = id.clone();

            loop {
                match new_id.parent {
                    Some(p) => {
                        if globals.func_ids[p].obj_list.is_empty() {
                            new_id.parent = globals.func_ids[p].parent;
                        } else {
                            break;
                        }
                    }
                    None => break,
                }
            }

            new_func_ids.push(new_id)
        }
    }

    // PROBLEM: new parent ids point to indexes in the previous list, in which many items were deleted.
    // Update the indexes to point to the corresponding items in the new list

    globals.func_ids = new_func_ids;*/

    println!(
        "Compiled in {} milliseconds!",
        start_time.elapsed().as_millis()
    );

    Ok((globals, level_string))
}

pub fn compile_scope(
    statements: &Vec<ast::Statement>,
    mut contexts: Vec<Context>,
    globals: &mut Globals,
    mut info: CompilerInfo,
) -> Result<(Vec<Context>, Returns), RuntimeError> {
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
            info.path.join(">"),
            contexts.len()
        );*/
        if contexts.is_empty() {
            return Err(RuntimeError::RuntimeError {
                message: "No context! This is probably a bug, please contact sputnix".to_string(),
                pos: (0, 0),
            });
        }
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
                    let (evaled, inner_returns) = expr.eval(context, globals, info.clone())?;
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
                                id: next_free(&mut globals.closed_groups, ast::IDClass::Group)?,
                            };
                            new_context.variables.insert(
                                def.symbol.clone(),
                                store_value(Value::Func(Function { start_group }), globals),
                            );
                            all_values.push((Value::Func(Function { start_group }), context));
                            new_context.start_group = start_group;
                            let new_info = info.next(&def.symbol, globals, true);
                            let (_, inner_returns) =
                                compile_scope(&f.statements, vec![new_context], globals, new_info)?;
                            returns.extend(inner_returns);
                        } else {
                            let (evaled, inner_returns) =
                                def.value.eval(context, globals, info.clone())?;
                            returns.extend(inner_returns);
                            all_values.extend(evaled);
                        }
                    } else {
                        let (evaled, inner_returns) =
                            def.value.eval(context, globals, info.clone())?;
                        returns.extend(inner_returns);
                        all_values.extend(evaled);
                    }
                    //copied because im lazy
                }
                contexts = Vec::new();
                for (val, mut context) in all_values {
                    context
                        .variables
                        .insert(String::from(&def.symbol), store_value(val, globals));

                    contexts.push(context);
                }
            }

            Extract(val) => {
                let mut all_values: Returns = Vec::new();
                for context in contexts {
                    let (evaled, inner_returns) = val.eval(context, globals, info.clone())?;
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }

                contexts = Vec::new();
                for (val, mut context) in all_values {
                    match val {
                        Value::Dict(d) => {
                            context.variables.extend(d.clone());
                        }
                        a => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!(
                                    "This type ({}) can not be extracted!",
                                    a.to_str(globals)
                                ),
                                pos: (0, 0),
                            })
                        }
                    }

                    contexts.push(context);
                }
            }

            If(if_stmt) => {
                let mut all_values: Returns = Vec::new();
                for context in contexts.clone() {
                    let new_info = info.next("if condition", globals, false);
                    let (evaled, inner_returns) =
                        if_stmt.condition.eval(context, globals, new_info)?;
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }

                for (val, context) in all_values {
                    match val {
                        Value::Bool(b) => {
                            //internal if statement
                            if b {
                                contexts = Vec::new();
                                let new_info = info.next("if body", globals, true);
                                let compiled = compile_scope(
                                    &if_stmt.if_body,
                                    vec![context],
                                    globals,
                                    new_info,
                                )?;
                                returns.extend(compiled.1);
                                contexts.extend(compiled.0);
                            } else {
                                match &if_stmt.else_body {
                                    Some(body) => {
                                        contexts = Vec::new();
                                        let new_info = info.next("else body", globals, true);
                                        let compiled =
                                            compile_scope(body, vec![context], globals, new_info)?;
                                        returns.extend(compiled.1);
                                        contexts.extend(compiled.0);
                                    }
                                    None => {}
                                };
                            }
                        }
                        a => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!(
                                    "Expected boolean condition in if statement, found {}",
                                    a.to_str(globals)
                                ),
                                pos: (0, 0),
                            })
                        }
                    }
                }
            }

            Impl(imp) => {
                let mut new_contexts: Vec<Context> = Vec::new();
                for context in contexts.clone() {
                    let new_info = info.next("implementation symbol", globals, false);
                    let (evaled, inner_returns) =
                        imp.symbol.to_value(context.clone(), globals, new_info)?;
                    returns.extend(inner_returns);
                    for (typ, c) in evaled {
                        match typ {
                            Value::TypeIndicator(s) => {
                                let new_info = info.next("implementation", globals, true);
                                let (evaled, inner_returns) =
                                    eval_dict(imp.members.clone(), c, globals, new_info)?;
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
                            }
                            a => {
                                return Err(RuntimeError::RuntimeError {
                                    message: format!(
                                        "Expected type-indicator, found {}",
                                        a.to_str(globals)
                                    ),
                                    pos: (0, 0),
                                })
                            }
                        }
                    }
                }
                //println!("{:?}", new_contexts[0].implementations);
                contexts = new_contexts;
            }
            Call(call) => {
                /*for context in &mut contexts {
                    context.x += 1;
                }*/
                let mut all_values: Returns = Vec::new();
                for context in contexts {
                    let (evaled, inner_returns) =
                        call.function.to_value(context, globals, info.clone())?;
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
                                a => {
                                    return Err(RuntimeError::RuntimeError {
                                        message: format!(
                                            "Expected function of group, found: {}",
                                            a.to_str(globals)
                                        ),
                                        pos: (0, 0),
                                    })
                                }
                            },

                            ..context_trigger(context.clone(), globals, info.clone())
                        }
                        .context_parameters(context.clone()),
                    );
                }
                (*globals).func_ids[info.func_id].obj_list.extend(obj_list);
            }

            For(f) => {
                let mut all_arrays: Returns = Vec::new();
                for context in contexts {
                    let (evaled, inner_returns) = f.array.eval(context, globals, info.clone())?;
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
                                    let new_info = info.next("for loop", globals, false);
                                    let (end_contexts, inner_returns) =
                                        compile_scope(&f.body, vec![c], globals, new_info)?;
                                    returns.extend(inner_returns);
                                    new_contexts = end_contexts;
                                }
                            }
                            contexts.extend(new_contexts);
                        }

                        a => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!("{} is not iteratable!", a.to_str(globals)),
                                pos: (0, 0),
                            })
                        }
                    }
                }
            }
            Return(return_val) => match return_val {
                Some(val) => {
                    let mut all_values: Returns = Vec::new();
                    for context in contexts.clone() {
                        let new_info = info.next("implementation symbol", globals, false);
                        let (evaled, inner_returns) = val.eval(context, globals, new_info)?;
                        returns.extend(inner_returns);
                        all_values.extend(evaled);
                    }

                    returns.extend(all_values);
                }

                None => {
                    let mut all_values: Returns = Vec::new();
                    for context in contexts.clone() {
                        all_values.push((Value::Null, context));
                    }
                    returns.extend(all_values);
                }
            },

            Error(e) => {
                for context in contexts.clone() {
                    let new_info = info.next("return value", globals, false);
                    let (evaled, _) = e.message.eval(context, globals, new_info)?;
                    for (msg, _) in evaled {
                        eprintln!(
                            "ERROR: {:?}",
                            match msg {
                                Value::Str(s) => s,
                                _ => "no message".to_string(),
                            }
                        );
                    }
                }
                return Err(RuntimeError::RuntimeError {
                    message: "Error statement, see message(s) above.".to_string(),
                    pos: (0, 0),
                });
            }
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
    Ok((contexts, returns))
}

pub fn import_module(
    path: &PathBuf,
    globals: &mut Globals,
    info: CompilerInfo,
) -> Result<(Value, Implementations), RuntimeError> {
    let module_path = globals
        .path
        .clone()
        .parent()
        .expect("Your file must be in a folder to import modules!")
        .join(&path);

    let unparsed = fs::read_to_string(module_path).expect("Something went wrong reading the file");
    let (parsed, notes) = match crate::parse_spwn(unparsed) {
        Ok(p) => p,
        Err(err) => return Err(RuntimeError::PackageSyntaxError { err, pos: (0, 0) }),
    };
    (*globals).closed_groups.extend(notes.closed_groups);
    (*globals).closed_colors.extend(notes.closed_colors);
    (*globals).closed_blocks.extend(notes.closed_blocks);
    (*globals).closed_items.extend(notes.closed_items);
    let new_info = info.next("module", globals, false);
    let (contexts, _) = compile_scope(&parsed, vec![Context::new()], globals, new_info)?;
    if contexts.len() > 1 {
        return Err(RuntimeError::RuntimeError {
            message: "Imported files does not (currently) support context splitting in the main scope.
            Please remove any context splitting statements outside of function or macro definitions.".to_string(),
            pos: (0,0)
        });
    }
    Ok((
        match contexts[0].variables.get("exports") {
            Some(val) => (*globals).stored_values[*val as usize].clone(),
            None => Value::Null,
        },
        contexts[0].implementations.clone(),
    ))
}

const ID_MAX: u16 = 999;

pub fn next_free(ids: &mut Vec<u16>, id_class: ast::IDClass) -> Result<u16, RuntimeError> {
    for i in 1..ID_MAX {
        if !ids.contains(&i) {
            (*ids).push(i);
            return Ok(i);
        }
    }

    Err(RuntimeError::IDError {
        id_class,
        pos: (0, 0),
    })
    //panic!("All ids of this type are used up!");
}
