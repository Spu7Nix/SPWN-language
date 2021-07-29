//! Tools for compiling SPWN into GD object strings
use crate::ast;
use crate::builtin::*;
use crate::compiler_info::CompilerInfo;
use crate::context::*;
use crate::globals::Globals;
use crate::levelstring::*;
use crate::value::*;
use crate::value_storage::*;
use crate::STD_PATH;
use std::collections::{HashMap, HashSet};

use crate::parser::{ParseNotes, SyntaxError};
use std::fs;
use std::path::{Path, PathBuf};

use crate::compiler_types::*;
use crate::print_with_color;
pub const CONTEXT_MAX: usize = 2;

use termcolor::Color as TColor;

#[derive(Debug)]
pub enum RuntimeError {
    UndefinedErr {
        undefined: String,
        desc: String,
        info: CompilerInfo,
    },

    PackageSyntaxError {
        err: SyntaxError,
        info: CompilerInfo,
    },

    TypeError {
        expected: String,
        found: String,
        info: CompilerInfo,
    },

    RuntimeError {
        message: String,
        info: CompilerInfo,
    },

    BuiltinError {
        message: String,
        info: CompilerInfo,
    },
}
pub fn print_error_intro(pos: crate::parser::FileRange, file: &Path) {
    use std::io::Write;
    use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

    let mut stdout = StandardStream::stderr(ColorChoice::Always);

    let mut write_with_color = |text: &str, color: Color| {
        stdout
            .set_color(ColorSpec::new().set_fg(Some(color)))
            .unwrap();
        write!(&mut stdout, "{}", text).unwrap();
    };

    let path_str = format!(
        "{}:{}:{}",
        file.to_string_lossy().to_string(),
        pos.0 .0,
        pos.0 .1 + 1
    );

    write_with_color("Error", TColor::Red);
    write_with_color(&format!(" at {}\n", path_str), TColor::White);

    if pos.0 .0 == pos.1 .0 {
        use std::io::BufRead;
        if let Ok(file) = fs::File::open(&file) {
            if let Some(Ok(line)) = std::io::BufReader::new(file).lines().nth(pos.0 .0 - 1) {
                let line_num = pos.1 .0.to_string();

                let mut spacing = String::new();

                for _ in 0..line_num.len() {
                    spacing += " ";
                }

                let squiggly_line = "^";

                write_with_color(
                    (spacing.clone() + " |\n" + &line_num + " |").as_str(),
                    TColor::Cyan,
                );
                write_with_color((line.replace("\t", " ") + "\n").as_str(), TColor::White);
                write_with_color((spacing + " |").as_str(), TColor::Cyan);
                let mut out = String::new();
                for _ in 0..(pos.0 .1) {
                    out += " ";
                }
                for _ in 0..(pos.1 .1 - pos.0 .1) {
                    out += squiggly_line;
                }
                out += "\n";
                write_with_color(&out, TColor::Red);
                stdout
                    .set_color(ColorSpec::new().set_fg(Some(TColor::White)))
                    .unwrap();
            }
        }
    };
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info = match self {
            RuntimeError::UndefinedErr {
                undefined: _,
                desc: _,
                info,
            } => info,
            RuntimeError::PackageSyntaxError { err: _, info } => info,

            RuntimeError::TypeError {
                expected: _,
                found: _,
                info,
            } => info,

            RuntimeError::RuntimeError { message: _, info } => info,

            RuntimeError::BuiltinError { message: _, info } => info,
        };

        print_error_intro(info.pos, &info.current_file);

        match self {
            RuntimeError::UndefinedErr {
                undefined,
                desc,
                info: _,
            } => write!(f, "{} '{}' is not defined", desc, undefined,),
            RuntimeError::PackageSyntaxError { err, info: _ } => {
                write!(f, "Error when parsing library: {}", err)
            }

            RuntimeError::TypeError {
                expected,
                found,
                info: _,
            } => write!(f, "Type mismatch: expected {}, found {}", expected, found,),

            RuntimeError::RuntimeError { message, info: _ } => write!(f, "{}", message,),

            RuntimeError::BuiltinError { message, info: _ } => {
                write!(f, "Error when calling built-in-function: {}", message,)
            }
        }
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub const NULL_STORAGE: usize = 1;
pub const BUILTIN_STORAGE: usize = 0;

pub fn compile_spwn(
    statements: Vec<ast::Statement>,
    path: PathBuf,
    included_paths: Vec<PathBuf>,
    notes: ParseNotes,
) -> Result<Globals, RuntimeError> {
    //variables that get changed throughout the compiling
    let mut globals = Globals::new(path.clone());
    if statements.is_empty() {
        return Err(RuntimeError::RuntimeError {
            message: "this script is empty".to_string(),
            info: CompilerInfo {
                depth: 0,
                path: vec!["main scope".to_string()],
                pos: ((0, 0), (0, 0)),
                current_file: path,
                current_module: String::new(),
                includes: vec![],
            },
        });
    }
    let mut start_context = Context::new();
    //store at pos 0
    // store_value(Value::Builtins, 1, &mut globals, &start_context);
    // store_value(Value::Null, 1, &mut globals, &start_context);

    let start_info = CompilerInfo {
        depth: 0,
        path: vec!["main scope".to_string()],
        pos: statements[0].pos,
        current_file: path,
        current_module: String::new(),
        includes: included_paths,
    };
    use std::time::Instant;

    //println!("Importing standard library...");
    print_with_color("Building script ...", TColor::Cyan);
    print_with_color("———————————————————————————\n", TColor::White);
    let start_time = Instant::now();

    if !notes.tag.tags.iter().any(|x| x.0 == "no_std") {
        let standard_lib = import_module(
            &ImportType::Lib(STD_PATH.to_string()),
            &start_context,
            &mut globals,
            start_info.clone(),
            false,
        )?;

        if standard_lib.len() != 1 {
            return Err(RuntimeError::RuntimeError {
                message: "The standard library can not split the context".to_string(),
                info: start_info,
            });
        }

        start_context = standard_lib[0].1.clone();

        if let Value::Dict(d) = &globals.stored_values[standard_lib[0].0] {
            start_context.variables.extend(d.clone());
        } else {
            return Err(RuntimeError::RuntimeError {
                message: "The standard library must return a dictionary".to_string(),
                info: start_info,
            });
        }
    }

    let (contexts, _) = compile_scope(
        &statements,
        smallvec![start_context],
        &mut globals,
        start_info,
    )?;

    for c in contexts {
        if let Some((i, _)) = c.broken {
            return Err(RuntimeError::RuntimeError {
                message: "break statement is never used".to_string(),
                info: i,
            });
        }
    }

    print_with_color("———————————————————————————\n", TColor::White);

    /*  Build Timing ----------------------------------------------------- **
        New build timing changes the unit form milliseconds, to seconds,
        to minutes depending on the time building took.
    */

    // Define the different units
    let build_time_secs = start_time.elapsed().as_secs();
    let build_time_millis = start_time.elapsed().as_millis();
    let build_time_mins = build_time_secs / 60;

    // Check which unit to unit to use
    if build_time_secs < 1 {
        print_with_color(
            &format!("Built in {} milliseconds!", build_time_millis),
            TColor::Green,
        );
    } else if build_time_secs > 1 && build_time_secs < 60 {
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

    //----------------------------------------------------------------------- **

    Ok(globals)
}

use smallvec::{smallvec, SmallVec};

pub fn compile_scope(
    statements: &[ast::Statement],
    mut contexts: SmallVec<[Context; CONTEXT_MAX]>,
    globals: &mut Globals,
    mut info: CompilerInfo,
) -> Result<(SmallVec<[Context; CONTEXT_MAX]>, Returns), RuntimeError> {
    let mut returns: Returns = SmallVec::new();

    //take out broken contexts

    let mut broken_contexts = SmallVec::new();
    let mut to_be_removed = SmallVec::<[usize; CONTEXT_MAX]>::new();

    for (i, c) in contexts.iter().enumerate() {
        if c.broken != None {
            broken_contexts.push(c.clone());
            to_be_removed.push(i)
        }
    }

    for i in to_be_removed.iter().rev() {
        contexts.swap_remove(*i);
    }
    if contexts.is_empty() {
        return Ok((broken_contexts, returns));
    }

    globals.stored_values.increment_lifetimes();

    for statement in statements.iter() {
        //find out what kind of statement this is
        //let start_time = Instant::now();

        //print_error_intro(info.pos, &info.current_file);

        // println!(
        //     "{} -> Compiling a statement in {} contexts",
        //     info.path.join(">"),
        //     contexts.len()
        // );
        info.pos = statement.pos;
        if contexts.is_empty() {
            return Err(RuntimeError::RuntimeError {
                message: "No context! This is probably a bug, please contact sputnix".to_string(),
                info,
            });
        }
        use ast::StatementBody::*;

        let stored_context = if statement.arrow {
            Some(contexts.clone())
        } else {
            None
        };

        //println!("{}:{}:{}", info.current_file.to_string_lossy(), info.pos.0.0, info.pos.0.1);
        //use crate::fmt::SpwnFmt;
        match &statement.body {
            Expr(expr) => {
                let mut new_contexts: SmallVec<[Context; CONTEXT_MAX]> = SmallVec::new();
                for context in &contexts {
                    let is_assign = !expr.operators.is_empty()
                        && expr.operators[0] == ast::Operator::Assign
                        && !expr.values[0].is_undefinable(&context, globals);

                    //println!("{:?}, {}", expr, is_assign);

                    if is_assign {
                        let mut new_expr = expr.clone();
                        let symbol = new_expr.values.remove(0);
                        //use crate::fmt::SpwnFmt;
                        new_expr.operators.remove(0); //assign operator
                        let mutable = symbol.operator == Some(ast::UnaryOperator::Let);

                        //let mut new_context = context.clone();

                        match (
                            new_expr.values.len() == 1
                                && new_expr.values[0].path.is_empty()
                                && new_expr.values[0].operator.is_none(),
                            &new_expr.values[0].value.body,
                        ) {
                            (true, ast::ValueBody::CmpStmt(f)) => {
                                //to account for recursion

                                //create the function context
                                let mut new_context = context.clone();
                                let storage = symbol.define(&mut new_context, globals, &info)?;

                                //pick a start group
                                let start_group = Group::next_free(&mut globals.closed_groups);
                                //store value
                                globals.stored_values[storage] =
                                    Value::TriggerFunc(TriggerFunction { start_group });

                                new_context.start_group = start_group;

                                let new_info = info.clone();
                                let (_, inner_returns) = compile_scope(
                                    &f.statements,
                                    smallvec![new_context],
                                    globals,
                                    new_info,
                                )?;
                                returns.extend(inner_returns);

                                let mut after_context = context.clone();

                                let var_storage =
                                    symbol.define(&mut after_context, globals, &info)?;

                                globals.stored_values[var_storage] =
                                    Value::TriggerFunc(TriggerFunction { start_group });

                                new_contexts.push(after_context);
                            }
                            // (true, ast::ValueBody::Macro(m)) => {
                            //     let (evaled, inner_returns) =
                            //         new_expr.eval(context, globals, info.clone(), !mutable)?;

                            //     returns.extend(inner_returns);
                            //     for (e, c2) in evaled {
                            //         let mut new_context = c2.clone();
                            //         let storage =
                            //             symbol.define(&mut new_context, globals, &info, None)?;

                            //         if let Value::Macro(m) = &mut globals.stored_values[e] {
                            //             m.def_context
                            //         } else {
                            //             unreachable!()
                            //         }

                            //         globals.stored_values[storage] =
                            //             globals.stored_values[e].clone();
                            //         new_contexts.push(new_context);
                            //     }
                            // }
                            _ => {
                                let (evaled, inner_returns) =
                                    new_expr.eval(context, globals, info.clone(), !mutable)?;

                                returns.extend(inner_returns);
                                for (e, c2) in evaled {
                                    let mut new_context = c2.clone();
                                    let storage =
                                        symbol.define(&mut new_context, globals, &info)?;
                                    //clone the value so as to not share the reference

                                    let cloned = clone_value(
                                        e,
                                        globals.get_lifetime(storage),
                                        globals,
                                        new_context.start_group,
                                        !mutable,
                                    );

                                    globals.stored_values[storage] =
                                        globals.stored_values[cloned].clone();
                                    new_contexts.push(new_context);
                                }
                            }
                        }
                    } else {
                        //we dont care about the return value in this case
                        let (evaled, inner_returns) =
                            expr.eval(context, globals, info.clone(), false)?;
                        returns.extend(inner_returns);
                        new_contexts.extend(evaled.iter().map(|x| {
                            //globals.stored_values.map.remove(&x.0);
                            x.1.clone()
                        }));
                    }
                }
                contexts = new_contexts;
            }

            Extract(val) => {
                let mut all_values: Returns = SmallVec::new();
                for context in &contexts {
                    let (evaled, inner_returns) = val.eval(context, globals, info.clone(), true)?;
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }

                contexts = SmallVec::new();
                for (val, mut context) in all_values {
                    match globals.stored_values[val].clone() {
                        Value::Dict(d) => {
                            context.variables.extend(
                                d.iter()
                                    .map(|(k, v)| {
                                        (
                                            k.clone(),
                                            clone_value(*v, 1, globals, context.start_group, false),
                                        )
                                    })
                                    .collect::<HashMap<String, StoredValue>>(),
                            );
                        }
                        Value::Builtins => {
                            for name in BUILTIN_LIST.iter() {
                                let p = store_value(
                                    Value::BuiltinFunction(*name),
                                    1,
                                    globals,
                                    &context,
                                );

                                context.variables.insert(String::from(*name), p);
                            }
                        }
                        a => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!(
                                    "This type ({}) can not be extracted!",
                                    a.to_str(globals)
                                ),
                                info,
                            })
                        }
                    }

                    contexts.push(context);
                }
            }

            TypeDef(name) => {
                //initialize type
                let already = globals.type_ids.get(name);
                if let Some(t) = already {
                    if !(t.1 == info.current_file && t.2 == info.pos.0) {
                        return Err(RuntimeError::RuntimeError {
                            message: format!("the type '{}' is already defined", name),
                            info,
                        });
                    }
                } else {
                    (*globals).type_id_count += 1;
                    (*globals).type_ids.insert(
                        name.clone(),
                        (globals.type_id_count, info.current_file.clone(), info.pos.0),
                    );
                }
                //Value::TypeIndicator(globals.type_id_count)
            }

            If(if_stmt) => {
                let mut all_values: Returns = SmallVec::new();
                for context in &contexts {
                    let (evaled, inner_returns) =
                        if_stmt
                            .condition
                            .eval(context, globals, info.clone(), true)?;
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }
                contexts = SmallVec::new();
                for (val, context) in all_values {
                    match &globals.stored_values[val] {
                        Value::Bool(b) => {
                            //internal if statement
                            if *b {
                                let new_info = info.clone();
                                let compiled = compile_scope(
                                    &if_stmt.if_body,
                                    smallvec![context.clone()],
                                    globals,
                                    new_info,
                                )?;
                                returns.extend(compiled.1);
                                contexts.extend(compiled.0.iter().map(|c| Context {
                                    variables: context.variables.clone(),

                                    ..c.clone()
                                }));
                            } else {
                                match &if_stmt.else_body {
                                    Some(body) => {
                                        let new_info = info.clone();
                                        let compiled = compile_scope(
                                            body,
                                            smallvec![context.clone()],
                                            globals,
                                            new_info,
                                        )?;
                                        returns.extend(compiled.1);
                                        contexts.extend(compiled.0.iter().map(|c| Context {
                                            variables: context.variables.clone(),

                                            ..c.clone()
                                        }));
                                    }
                                    None => contexts.push(context),
                                };
                            }
                        }
                        a => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!(
                                    "Expected boolean condition in if statement, found {}",
                                    a.to_str(globals)
                                ),
                                info,
                            })
                        }
                    }
                }
            }

            Impl(imp) => {
                let message = "cannot run impl statement in a trigger function context, consider moving it to the start of your script.".to_string();
                if contexts.len() > 1 || contexts[0].start_group.id != Id::Specific(0) {
                    return Err(RuntimeError::RuntimeError { message, info });
                }

                let new_info = info.clone();
                let (evaled, inner_returns) =
                    imp.symbol
                        .to_value(contexts[0].clone(), globals, new_info, true)?;

                if evaled.len() > 1 {
                    return Err(RuntimeError::RuntimeError {
                        message: "impl statements with context-splitting values are not allowed"
                            .to_string(),
                        info,
                    });
                }
                returns.extend(inner_returns);
                let (typ, c) = evaled[0].clone();

                if c.start_group.id != Id::Specific(0) {
                    return Err(RuntimeError::RuntimeError { message, info });
                }
                match globals.stored_values[typ].clone() {
                    Value::TypeIndicator(s) => {
                        let new_info = info.clone();
                        let (evaled, inner_returns) =
                            eval_dict(imp.members.clone(), &c, globals, new_info, true)?;
                        if evaled.len() > 1 {
                            return Err(RuntimeError::RuntimeError {
                                message:
                                    "impl statements with context-splitting values are not allowed"
                                        .to_string(),
                                info,
                            });
                        }
                        //Returns inside impl values dont really make sense do they
                        if !inner_returns.is_empty() {
                            return Err(RuntimeError::RuntimeError {
                                message: "you can't use return from inside an impl statement"
                                    .to_string(),
                                info,
                            });
                        }
                        let (val, _) = evaled[0];
                        // make this not ugly, future me
                        globals.stored_values.increment_single_lifetime(
                            val,
                            1000,
                            &mut HashSet::new(),
                        );

                        if let Value::Dict(d) = &globals.stored_values[val] {
                            match globals.implementations.get_mut(&s) {
                                Some(implementation) => {
                                    for (key, val) in d.iter() {
                                        (*implementation).insert(key.clone(), (*val, true));
                                    }
                                }
                                None => {
                                    globals.implementations.insert(
                                        s,
                                        d.iter()
                                            .map(|(key, value)| (key.clone(), (*value, true)))
                                            .collect(),
                                    );
                                }
                            }
                        } else {
                            unreachable!();
                        }
                    }
                    a => {
                        return Err(RuntimeError::RuntimeError {
                            message: format!(
                                "Expected type-indicator, found {}",
                                a.to_str(globals)
                            ),
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
                let mut all_values: Returns = SmallVec::new();
                for context in contexts {
                    let (evaled, inner_returns) =
                        call.function
                            .to_value(context, globals, info.clone(), true)?;
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }
                contexts = SmallVec::new();
                //let mut obj_list = Vec::<GDObj>::new();
                for (func, context) in all_values {
                    contexts.push(context.clone());
                    let mut params = HashMap::new();
                    params.insert(
                        51,
                        match &globals.stored_values[func] {
                            Value::TriggerFunc(g) => ObjParam::Group(g.start_group),
                            Value::Group(g) => ObjParam::Group(*g),
                            a => {
                                return Err(RuntimeError::RuntimeError {
                                    message: format!(
                                        "Expected trigger function or group, found: {}",
                                        a.to_str(globals)
                                    ),
                                    info,
                                })
                            }
                        },
                    );
                    params.insert(1, ObjParam::Number(1268.0));
                    (*globals).trigger_order += 1;

                    (*globals).func_ids[context.func_id].obj_list.push((
                        GdObj {
                            params,

                            ..context_trigger(&context, &mut globals.uid_counter)
                        }
                        .context_parameters(&context),
                        globals.trigger_order,
                    ))
                }
            }

            For(f) => {
                let mut all_arrays: Returns = SmallVec::new();
                for context in &contexts {
                    let (evaled, inner_returns) =
                        f.array.eval(context, globals, info.clone(), true)?;
                    returns.extend(inner_returns);
                    all_arrays.extend(evaled);
                }
                contexts = SmallVec::new();
                /*
                Before going further you should probably understand what contexts mean.
                A "context", in SPWN, is like a parallel universe. Each context is nearly
                identical to a code block, except it expands a runtime value that is meant to be
                converted into a compile time item. Every time you want to do something like
                convert a counter to a number, SPWN will branch the current code block into a number
                of contexts, one for each possible value from the conversion. All of the branched contexts
                will be evaluated in isolation to each other.
                */
                for (val, context) in all_arrays {
                    match globals.stored_values[val].clone() {
                        // what are we iterating
                        Value::Array(arr) => {
                            // its an array!

                            let mut new_contexts: SmallVec<[Context; CONTEXT_MAX]> =
                                smallvec![context.clone()];
                            // new contexts: any contexts that are created in the loop. currently only has 1

                            let mut out_contexts: SmallVec<[Context; CONTEXT_MAX]> =
                                SmallVec::new(); // out contexts: anything declared outside the loop

                            for element in arr {
                                // going through the array items

                                for c in &mut new_contexts {
                                    // reset all variables per context
                                    (*c).variables = context.variables.clone();
                                    (*c).variables.insert(f.symbol.clone(), element);
                                }

                                let new_info = info.clone(); // file position info

                                let (end_contexts, inner_returns) =
                                    compile_scope(&f.body, new_contexts, globals, new_info)?; // eval the stuff

                                // end_contexts has any new contexts made in the loop

                                new_contexts = SmallVec::new();
                                for mut c in end_contexts {
                                    // add contexts made in the loop to the new_contexts, if they dont have a break
                                    match c.broken {
                                        Some((_, BreakType::Loop)) => {
                                            c.broken = None;
                                            out_contexts.push(c)
                                        }
                                        Some((_, BreakType::Macro)) => out_contexts.push(c),
                                        Some((_, BreakType::ContinueLoop)) => {
                                            c.broken = None;
                                            new_contexts.push(c)
                                        }
                                        _ => new_contexts.push(c),
                                    }
                                }

                                returns.extend(inner_returns); // return stuff
                                if new_contexts.is_empty() {
                                    break;
                                }
                            }

                            out_contexts.extend(new_contexts);
                            contexts.extend(out_contexts.iter().map(|c| Context {
                                variables: context.variables.clone(),
                                ..c.clone()
                            }));

                            // finally append all newly created ones to the global count
                        }
                        Value::Dict(d) => {
                            // its a dict!

                            let mut new_contexts: SmallVec<[Context; CONTEXT_MAX]> =
                                smallvec![context.clone()];
                            // new contexts: any contexts that are created in the loop. currently only has 1

                            let mut out_contexts: SmallVec<[Context; CONTEXT_MAX]> =
                                SmallVec::new(); // out contexts: anything declared outside the loop

                            for (k, v) in d {
                                // going through the array items

                                for c in &mut new_contexts {
                                    // reset all variables per context
                                    (*c).variables = context.variables.clone();
                                    let key_stored = store_const_value(
                                        // store the dict key
                                        Value::Str(k.clone()),
                                        1,
                                        globals,
                                        c,
                                    );
                                    let stored = store_const_value(
                                        // store the val key
                                        Value::Array(vec![key_stored, v]),
                                        1,
                                        globals,
                                        c,
                                    );
                                    (*c).variables.insert(f.symbol.clone(), stored);
                                }

                                let new_info = info.clone(); // file position info

                                let (end_contexts, inner_returns) =
                                    compile_scope(&f.body, new_contexts, globals, new_info)?; // eval the stuff
                                                                                              // end_contexts has any new contexts made in the loop

                                new_contexts = SmallVec::new();
                                for mut c in end_contexts {
                                    // add contexts made in the loop to the new_contexts, if they dont have a break
                                    match c.broken {
                                        Some((_, BreakType::Loop)) => {
                                            c.broken = None;
                                            out_contexts.push(c)
                                        }
                                        Some((_, BreakType::Macro)) => out_contexts.push(c),
                                        Some((_, BreakType::ContinueLoop)) => {
                                            c.broken = None;
                                            new_contexts.push(c)
                                        }
                                        _ => new_contexts.push(c),
                                    }
                                }

                                returns.extend(inner_returns); // return stuff
                                if new_contexts.is_empty() {
                                    break;
                                }
                            }
                            out_contexts.extend(new_contexts);
                            contexts.extend(out_contexts.iter().map(|c| Context {
                                variables: context.variables.clone(),
                                ..c.clone()
                            }));
                            // finally append all newly created ones to the global count
                        }
                        Value::Str(s) => {
                            //let iterator_val = store_value(Value::Null, globals);
                            //let scope_vars = context.variables.clone();

                            let mut new_contexts: SmallVec<[Context; CONTEXT_MAX]> =
                                smallvec![context.clone()];
                            let mut out_contexts: SmallVec<[Context; CONTEXT_MAX]> =
                                SmallVec::new();

                            for ch in s.chars() {
                                //println!("{}", new_contexts.len());
                                for c in &mut new_contexts {
                                    (*c).variables = context.variables.clone();
                                    let stored = store_const_value(
                                        Value::Str(ch.to_string()),
                                        1,
                                        globals,
                                        c,
                                    );
                                    (*c).variables.insert(f.symbol.clone(), stored);
                                }

                                let new_info = info.clone();

                                let (end_contexts, inner_returns) =
                                    compile_scope(&f.body, new_contexts, globals, new_info)?;

                                new_contexts = SmallVec::new();
                                for mut c in end_contexts {
                                    // add contexts made in the loop to the new_contexts, if they dont have a break
                                    match c.broken {
                                        Some((_, BreakType::Loop)) => {
                                            c.broken = None;
                                            out_contexts.push(c)
                                        }
                                        Some((_, BreakType::Macro)) => out_contexts.push(c),
                                        Some((_, BreakType::ContinueLoop)) => {
                                            c.broken = None;
                                            new_contexts.push(c)
                                        }
                                        _ => new_contexts.push(c),
                                    }
                                }

                                returns.extend(inner_returns);

                                if new_contexts.is_empty() {
                                    break;
                                }
                            }
                            out_contexts.extend(new_contexts);
                            contexts.extend(out_contexts.iter().map(|c| Context {
                                variables: context.variables.clone(),
                                ..c.clone()
                            }));
                        }

                        Value::Range(start, end, step) => {
                            let mut normal = (start..end).step_by(step);
                            let mut rev = (end..start).step_by(step).rev();
                            let range: &mut dyn Iterator<Item = i32> =
                                if start < end { &mut normal } else { &mut rev };

                            let mut new_contexts: SmallVec<[Context; CONTEXT_MAX]> =
                                smallvec![context.clone()];
                            let mut out_contexts: SmallVec<[Context; CONTEXT_MAX]> =
                                SmallVec::new();

                            for num in range {
                                //println!("{}", new_contexts.len());
                                let element =
                                    store_value(Value::Number(num as f64), 0, globals, &context);
                                for c in &mut new_contexts {
                                    (*c).variables = context.variables.clone();
                                    (*c).variables.insert(f.symbol.clone(), element);
                                }

                                let new_info = info.clone();

                                //println!("{}", new_contexts.len());

                                let (end_contexts, inner_returns) =
                                    compile_scope(&f.body, new_contexts, globals, new_info)?;

                                new_contexts = SmallVec::new();
                                for mut c in end_contexts {
                                    // add contexts made in the loop to the new_contexts, if they dont have a break
                                    match c.broken {
                                        Some((_, BreakType::Loop)) => {
                                            c.broken = None;
                                            out_contexts.push(c)
                                        }
                                        Some((_, BreakType::Macro)) => out_contexts.push(c),
                                        Some((_, BreakType::ContinueLoop)) => {
                                            c.broken = None;
                                            new_contexts.push(c)
                                        }
                                        _ => new_contexts.push(c),
                                    }
                                }

                                returns.extend(inner_returns);

                                if new_contexts.is_empty() {
                                    break;
                                }
                            }
                            out_contexts.extend(new_contexts);
                            contexts.extend(out_contexts.iter().map(|c| Context {
                                variables: context.variables.clone(),
                                ..c.clone()
                            }));
                        }

                        a => {
                            return Err(RuntimeError::RuntimeError {
                                message: format!("{} is not iteratable!", a.to_str(globals)),
                                info,
                            })
                        }
                    }
                }
            }
            Break => {
                //set all contexts to broken
                for c in &mut contexts {
                    (*c).broken = Some((info.clone(), BreakType::Loop));
                }
                break;
            }

            Continue => {
                //set all contexts to broken
                for c in &mut contexts {
                    (*c).broken = Some((info.clone(), BreakType::ContinueLoop));
                }
                break;
            }

            Return(return_val) => {
                match return_val {
                    Some(val) => {
                        let mut all_values: Returns = SmallVec::new();
                        for context in &contexts {
                            let (evaled, inner_returns) =
                                val.eval(context, globals, info.clone(), true)?;
                            returns.extend(inner_returns);
                            all_values.extend(evaled);
                        }

                        returns.extend(all_values);
                    }

                    None => {
                        let mut all_values: Returns = SmallVec::new();
                        for context in &contexts {
                            all_values.push((
                                store_value(Value::Null, 1, globals, context),
                                context.clone(),
                            ));
                        }
                        returns.extend(all_values);
                    }
                };
                if !statement.arrow {
                    //set all contexts to broken
                    for c in &mut contexts {
                        (*c).broken = Some((info.clone(), BreakType::Macro));
                    }
                    break;
                }
            }

            Error(e) => {
                for context in &contexts {
                    let (evaled, _) = e.message.eval(context, globals, info.clone(), true)?;
                    for (msg, _) in evaled {
                        eprintln!(
                            "{}",
                            match &globals.stored_values[msg] {
                                Value::Str(s) => s,
                                _ => "no message",
                            },
                        );
                    }
                }
                return Err(RuntimeError::RuntimeError {
                    message: "Error statement, see message(s) above.".to_string(),
                    info,
                });
            }
        }

        let mut to_be_removed = Vec::new();

        for (i, c) in contexts.iter().enumerate() {
            if c.broken != None {
                broken_contexts.push(c.clone());
                to_be_removed.push(i)
            }
        }

        for i in to_be_removed.iter().rev() {
            contexts.swap_remove(*i);
        }

        // does this make sense?? why wasn't this here earlier??
        if contexts.is_empty() {
            return Ok((broken_contexts, returns));
        }

        if let Some(c) = stored_context {
            //resetting the context if async
            for c in contexts {
                if let Some((i, _)) = c.broken {
                    return Err(RuntimeError::RuntimeError {
                        message:
                            "break statement is never used because it's inside an arrow statement"
                                .to_string(),
                        info: i,
                    });
                }
            }
            contexts = c;
        }

        //try to merge contexts
        //if statement_index < statements.len() - 1 {
        loop {
            if !merge_contexts(&mut contexts, globals) {
                break;
            }
        }
        //}

        /*println!(
            "{} -> Compiled '{}' in {} milliseconds!",
            path,
            statement_type,
            start_time.elapsed().as_millis(),
        );*/
    }

    //return values need longer lifetimes
    for (val, _) in &returns {
        globals
            .stored_values
            .increment_single_lifetime(*val, 1, &mut HashSet::new());
    }

    globals.stored_values.decrement_lifetimes();
    //collect garbage
    globals.stored_values.clean_up();

    // put broken contexts back
    contexts.extend(broken_contexts);

    //(*globals).highest_x = context.x;
    Ok((contexts, returns))
}

fn merge_impl(target: &mut Implementations, source: &Implementations) {
    for (key, imp) in source.iter() {
        match target.get_mut(key) {
            Some(target_imp) => (*target_imp).extend(imp.iter().map(|x| (x.0.clone(), *x.1))),
            None => {
                (*target).insert(*key, imp.clone());
            }
        }
    }
}

pub fn import_module(
    path: &ImportType,
    context: &Context,
    globals: &mut Globals,
    info: CompilerInfo,
    forced: bool,
) -> Result<Returns, RuntimeError> {
    if !forced {
        if let Some(ret) = globals.prev_imports.get(path) {
            merge_impl(&mut globals.implementations, &ret.1);
            return Ok(smallvec![(
                store_value(ret.0.clone(), 1, globals, context),
                context.clone()
            )]);
        }
    }

    let mut module_path = match path {
        ImportType::Script(p) => globals
            .path
            .clone()
            .parent()
            .expect("Your file must be in a folder to import modules!")
            .join(&p),

        ImportType::Lib(name) => {
            let mut outpath = info.includes[0].clone();
            let mut found = false;
            for path in &info.includes {
                if path.join("libraries").join(name).exists() {
                    outpath = path.to_path_buf();
                    found = true;
                    break;
                }
            }
            if found {
                outpath
            } else {
                return Err(RuntimeError::RuntimeError {
                    message: "Unable to find library folder in given search paths".to_string(),
                    info,
                });
            }
        }
        // .parent()
        // .unwrap()
        .join("libraries")
        .join(name),
    };

    if module_path.is_dir() {
        module_path = module_path.join("lib.spwn");
    } else if module_path.is_file() && module_path.extension().is_none() {
        module_path.set_extension("spwn");
    } else if !module_path.is_file() {
        return Err(RuntimeError::RuntimeError {
            message: format!(
                "Couldn't find library file ({})",
                module_path.to_string_lossy()
            ),
            info,
        });
    }

    let unparsed = match fs::read_to_string(&module_path) {
        Ok(content) => content,
        Err(e) => {
            return Err(RuntimeError::RuntimeError {
                message: format!(
                    "Something went wrong when opening library file ({}): {}",
                    module_path.to_string_lossy(),
                    e
                ),
                info,
            })
        }
    };
    let (parsed, notes) = match crate::parse_spwn(unparsed, module_path.clone()) {
        Ok(p) => p,
        Err(err) => return Err(RuntimeError::PackageSyntaxError { err, info }),
    };

    let mut start_context = Context::new();

    let mut stored_impl = None;
    if let ImportType::Lib(_) = path {
        stored_impl = Some(globals.implementations.clone());
        globals.implementations = HashMap::new();
    }

    if !notes.tag.tags.iter().any(|x| x.0 == "no_std") {
        let standard_lib = import_module(
            &ImportType::Lib(STD_PATH.to_string()),
            &start_context,
            globals,
            info.clone(),
            false,
        )?;

        if standard_lib.len() != 1 {
            return Err(RuntimeError::RuntimeError {
                message: "The standard library can not split the context".to_string(),
                info,
            });
        }

        start_context = standard_lib[0].1.clone();

        if let Value::Dict(d) = &globals.stored_values[standard_lib[0].0] {
            start_context.variables.extend(d.clone());
        } else {
            return Err(RuntimeError::RuntimeError {
                message: "The standard library must return a dictionary".to_string(),
                info,
            });
        }
    }

    let stored_path = globals.path.clone();
    (*globals).path = module_path.clone();

    let mut new_info = info;

    new_info.current_file = module_path;
    new_info.pos = ((0, 0), (0, 0));

    if let ImportType::Lib(l) = path {
        new_info.current_module = l.clone();
    }

    let (contexts, mut returns) =
        compile_scope(&parsed, smallvec![start_context], globals, new_info)?;

    for c in &contexts {
        if let Some((i, BreakType::Loop)) = &c.broken {
            return Err(RuntimeError::RuntimeError {
                message: "break statement is never used".to_string(),
                info: i.clone(),
            });
        }
    }
    (*globals).path = stored_path;

    if let Some(stored_impl) = stored_impl {
        //change and delete from impls
        let mut to_be_deleted = Vec::new();
        for (k1, imp) in &mut globals.implementations {
            for (k2, (_, in_scope)) in imp {
                if *in_scope {
                    (*in_scope) = false;
                } else {
                    to_be_deleted.push((*k1, k2.clone()));
                }
            }
        }
        for (k1, k2) in to_be_deleted {
            (*globals).implementations.get_mut(&k1).unwrap().remove(&k2);
        }

        //merge impls
        merge_impl(&mut globals.implementations, &stored_impl);
    }

    let out = if returns.is_empty() {
        contexts
            .iter()
            .map(|x| {
                let mut new_context = x.clone();
                new_context.variables = context.variables.clone();
                (NULL_STORAGE, new_context)
            })
            .collect()
    } else {
        for (_, c) in &mut returns {
            (*c).variables = context.variables.clone();
        }

        returns
    };

    if out.len() == 1 && &out[0].1 == context {
        let cloned = clone_and_get_value(out[0].0, 9999, globals, context.start_group, true);
        let s_impl = globals.implementations.clone();

        globals.prev_imports.insert(path.clone(), (cloned, s_impl));
    }

    Ok(out)
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
