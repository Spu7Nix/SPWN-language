//! Tools for compiling SPWN into GD object strings
use crate::ast;
use crate::builtin::*;
use crate::levelstring::*;
use crate::STD_PATH;
use std::collections::HashMap;

use crate::parser::{ParseNotes, SyntaxError};
use std::fs;
use std::path::PathBuf;

use crate::compiler_types::*;

use ansi_term::Colour;

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
pub fn error_intro(pos: crate::parser::FileRange, file: &PathBuf) -> String {
    let path_str = format!(
        "{}:{}:{}",
        file.to_string_lossy().to_string(),
        pos.0 .0,
        pos.0 .1 + 1
    );

    let line = if pos.0 .0 == pos.1 .0 {
        use std::io::BufRead;
        match fs::File::open(&file) {
            Ok(file) => match std::io::BufReader::new(file).lines().nth(pos.0 .0 - 1) {
                Some(line) => match line {
                    Ok(line) => {
                        let line_num = pos.1 .0.to_string();
                        let start = Colour::Blue.bold().paint(line_num.clone() + " | ");

                        let mut spacing = String::new();

                        for _ in 0..line_num.len() {
                            spacing += " ";
                        }

                        let start_empty = Colour::Blue.bold().paint(spacing + " | ");

                        let squiggly_line = Colour::Red.bold().paint("^");

                        let mut out = format!(
                            "{}\n{}{}\n{}",
                            start_empty,
                            start,
                            line.replace("\t", " "),
                            start_empty
                        );

                        for _ in 0..(pos.0 .1) {
                            out += " ";
                        }
                        for _ in 0..(pos.1 .1 - pos.0 .1) {
                            out += &format!("{}", squiggly_line);
                        }
                        out + "\n"
                    }
                    Err(_) => String::new(),
                },
                None => String::new(),
            },
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };

    let err_msg = Colour::Red.bold().paint("Error");

    format!("{} at {}\n{}", err_msg, path_str, line)
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

        let start = error_intro(info.pos, &info.current_file);

        match self {
            RuntimeError::UndefinedErr {
                undefined,
                desc,
                info: _,
            } => write!(f, "{}{} '{}' is not defined", start, desc, undefined,),
            RuntimeError::PackageSyntaxError { err, info: _ } => {
                write!(f, "{}Error when parsing library: {}", start, err)
            }

            RuntimeError::TypeError {
                expected,
                found,
                info: _,
            } => write!(
                f,
                "{}Type mismatch: expected {}, found {}",
                start, expected, found,
            ),

            RuntimeError::RuntimeError { message, info: _ } => write!(f, "{}{}", start, message,),

            RuntimeError::BuiltinError { message, info: _ } => write!(
                f,
                "{}Error when calling built-in-function: {}",
                start, message,
            ),
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
    //gd_path: Option<PathBuf>,
    notes: ParseNotes,
) -> Result<Globals, RuntimeError> {
    //variables that get changed throughout the compiling
    let mut globals = Globals::new(path.clone());
    let mut start_context = Context::new();
    //store at pos 0
    // store_value(Value::Builtins, 1, &mut globals, &start_context);
    // store_value(Value::Null, 1, &mut globals, &start_context);

    let start_info = CompilerInfo {
        depth: 0,
        path: vec!["main scope".to_string()],
        pos: statements[0].pos,
        current_file: path,
    };
    use std::time::Instant;

    //println!("Importing standard library...");

    println!(
        "\n{}...\n{}\n",
        Colour::Cyan.bold().paint("Building script"),
        Colour::White.bold().paint("———————————————————————————")
    );
    let start_time = Instant::now();

    if !notes.tag.tags.iter().any(|x| x.0 == "no_std") {
        let standard_lib = import_module(
            &PathBuf::from(STD_PATH),
            &start_context,
            &mut globals,
            start_info.clone(),
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

    let (contexts, _) = compile_scope(&statements, vec![start_context], &mut globals, start_info)?;

    for c in contexts {
        if let Some(i) = c.broken {
            return Err(RuntimeError::RuntimeError {
                message: "break statement is never used".to_string(),
                info: i,
            });
        }
    }

    println!(
        "\n{}\n{}\n",
        Colour::White.bold().paint("———————————————————————————"),
        Colour::Green.bold().paint(&format!(
            "Built in {} milliseconds!",
            start_time.elapsed().as_millis()
        ))
    );

    Ok(globals)
}

pub fn compile_scope(
    statements: &[ast::Statement],
    mut contexts: Vec<Context>,
    globals: &mut Globals,
    mut info: CompilerInfo,
) -> Result<(Vec<Context>, Returns), RuntimeError> {
    let statements_iter = statements.iter();

    let mut returns: Returns = Vec::new();

    //take out broken contexts
    let mut new_contexts = Vec::new();
    let mut broken_contexts = Vec::new();
    for c in contexts {
        if c.broken == None {
            new_contexts.push(c)
        } else {
            broken_contexts.push(c)
        }
    }
    if new_contexts.is_empty() {
        return Ok((broken_contexts, returns));
    }
    contexts = new_contexts;
    globals.stored_values.increment_lifetimes();

    for statement in statements_iter {
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
                info,
            });
        }
        use ast::StatementBody::*;

        let stored_context = if statement.arrow {
            Some(contexts.clone())
        } else {
            None
        };

        info.pos = statement.pos;
        //use crate::fmt::SpwnFmt;
        match &statement.body {
            Break => {
                //set all contexts to broken
                for c in &mut contexts {
                    (*c).broken = Some(info.clone());
                }
                break;
            }

            Expr(expr) => {
                let mut new_contexts: Vec<Context> = Vec::new();
                for context in &contexts {
                    let is_assign = !expr.operators.is_empty()
                        && expr.operators[0] == ast::Operator::Assign
                        && !expr.values[0].is_defined(&context, globals);

                    if is_assign {
                        let mut new_expr = expr.clone();
                        let symbol = new_expr.values.remove(0);
                        new_expr.operators.remove(0); //assign operator
                        let constant = symbol.operator != Some(ast::UnaryOperator::Let);

                        //let mut new_context = context.clone();

                        match (new_expr.values.len() == 1, &new_expr.values[0].value.body) {
                            (true, ast::ValueBody::CmpStmt(f)) => {
                                //to account for recursion

                                //create the function context
                                let mut new_context = context.clone();
                                let storage = symbol.define(&mut new_context, globals, &info)?;

                                //pick a start group
                                let start_group = Group::next_free(&mut globals.closed_groups);
                                //store value
                                globals.stored_values[storage] =
                                    Value::Func(Function { start_group });

                                new_context.start_group = start_group;

                                let new_info = info.clone();
                                let (_, inner_returns) = compile_scope(
                                    &f.statements,
                                    vec![new_context],
                                    globals,
                                    new_info,
                                )?;
                                returns.extend(inner_returns);

                                let mut after_context = context.clone();

                                let var_storage =
                                    symbol.define(&mut after_context, globals, &info)?;

                                globals.stored_values[var_storage] =
                                    Value::Func(Function { start_group });

                                new_contexts.push(after_context);
                            }
                            _ => {
                                let (evaled, inner_returns) =
                                    new_expr.eval(context, globals, info.clone(), constant)?;
                                returns.extend(inner_returns);
                                for (e, c2) in evaled {
                                    let mut new_context = c2.clone();
                                    let storage =
                                        symbol.define(&mut new_context, globals, &info)?;
                                    //clone the value so as to not share the reference
                                    let cloned = clone_value(e, 1, globals, &new_context, true);
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
                let mut all_values: Returns = Vec::new();
                for context in &contexts {
                    let (evaled, inner_returns) = val.eval(context, globals, info.clone(), true)?;
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }

                contexts = Vec::new();
                for (val, mut context) in all_values {
                    match globals.stored_values[val].clone() {
                        Value::Dict(d) => {
                            context.variables.extend(
                                d.iter()
                                    .map(|(k, v)| {
                                        (k.clone(), clone_value(*v, 1, globals, &context, false))
                                    })
                                    .collect::<HashMap<String, StoredValue>>(),
                            );
                        }
                        Value::Builtins => {
                            for name in BUILTIN_LIST.iter() {
                                let p = store_value(
                                    Value::BuiltinFunction(String::from(*name)),
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
                (*globals).type_id_count += 1;
                (*globals)
                    .type_ids
                    .insert(name.clone(), globals.type_id_count);
                //Value::TypeIndicator(globals.type_id_count)
            }

            If(if_stmt) => {
                let mut all_values: Returns = Vec::new();
                for context in &contexts {
                    let new_info = info.clone();
                    let (evaled, inner_returns) =
                        if_stmt.condition.eval(context, globals, new_info, true)?;
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }
                contexts = Vec::new();
                for (val, context) in all_values {
                    match &globals.stored_values[val] {
                        Value::Bool(b) => {
                            //internal if statement
                            if *b {
                                let new_info = info.clone();
                                let compiled = compile_scope(
                                    &if_stmt.if_body,
                                    vec![context.clone()],
                                    globals,
                                    new_info,
                                )?;
                                returns.extend(compiled.1);
                                contexts.extend(compiled.0.iter().map(|c| Context {
                                    variables: context.variables.clone(),
                                    implementations: context.implementations.clone(),
                                    ..c.clone()
                                }));
                            } else {
                                match &if_stmt.else_body {
                                    Some(body) => {
                                        let new_info = info.clone();
                                        let compiled = compile_scope(
                                            body,
                                            vec![context.clone()],
                                            globals,
                                            new_info,
                                        )?;
                                        returns.extend(compiled.1);
                                        contexts.extend(compiled.0.iter().map(|c| Context {
                                            variables: context.variables.clone(),
                                            implementations: context.implementations.clone(),
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
                let mut new_contexts: Vec<Context> = Vec::new();
                for context in &contexts {
                    let new_info = info.clone();
                    let (evaled, inner_returns) =
                        imp.symbol
                            .to_value(context.clone(), globals, new_info, true)?;
                    returns.extend(inner_returns);
                    for (typ, c) in evaled {
                        match globals.stored_values[typ].clone() {
                            Value::TypeIndicator(s) => {
                                let new_info = info.clone();
                                let (evaled, inner_returns) =
                                    eval_dict(imp.members.clone(), &c, globals, new_info, true)?;

                                //Returns inside impl values dont really make sense do they
                                returns.extend(inner_returns);
                                for (val, c2) in evaled {
                                    globals.stored_values.increment_single_lifetime(val, 1000);
                                    let mut new_context = c2.clone();
                                    if let Value::Dict(d) = &globals.stored_values[val] {
                                        match new_context.implementations.get_mut(&s) {
                                            Some(implementation) => {
                                                for (key, val) in d.iter() {
                                                    (*implementation).insert(key.clone(), *val);
                                                }
                                            }
                                            None => {
                                                new_context.implementations.insert(s, d.clone());
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
                                    info,
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
                        call.function
                            .to_value(context, globals, info.clone(), true)?;
                    returns.extend(inner_returns);
                    all_values.extend(evaled);
                }
                contexts = Vec::new();
                //let mut obj_list = Vec::<GDObj>::new();
                for (func, context) in all_values {
                    contexts.push(context.clone());
                    let mut params = HashMap::new();
                    params.insert(
                        51,
                        match &globals.stored_values[func] {
                            Value::Func(g) => ObjParam::Group(g.start_group),
                            Value::Group(g) => ObjParam::Group(*g),
                            a => {
                                return Err(RuntimeError::RuntimeError {
                                    message: format!(
                                        "Expected function or group, found: {}",
                                        a.to_str(globals)
                                    ),
                                    info,
                                })
                            }
                        },
                    );
                    params.insert(1, ObjParam::Number(1268.0));

                    (*globals).func_ids[context.func_id].obj_list.push(
                        GDObj {
                            params,

                            ..context_trigger(&context)
                        }
                        .context_parameters(&context),
                    )
                }
            }

            For(f) => {
                let mut all_arrays: Returns = Vec::new();
                for context in &contexts {
                    let (evaled, inner_returns) =
                        f.array.eval(context, globals, info.clone(), true)?;
                    returns.extend(inner_returns);
                    all_arrays.extend(evaled);
                }
                contexts = Vec::new();
                for (val, context) in all_arrays {
                    match globals.stored_values[val].clone() {
                        Value::Array(arr) => {
                            //let iterator_val = store_value(Value::Null, globals);
                            //let scope_vars = context.variables.clone();

                            let mut new_contexts = vec![context.clone()];
                            let mut out_contexts = Vec::new();

                            for element in arr {
                                //println!("{}", new_contexts.len());
                                for c in &mut new_contexts {
                                    (*c).variables = context.variables.clone();
                                    (*c).variables.insert(f.symbol.clone(), element);
                                }

                                let new_info = info.clone();

                                let (end_contexts, inner_returns) =
                                    compile_scope(&f.body, new_contexts, globals, new_info)?;

                                new_contexts = Vec::new();
                                for mut c in end_contexts {
                                    if c.broken == None {
                                        new_contexts.push(c)
                                    } else {
                                        c.broken = None;
                                        out_contexts.push(c)
                                    }
                                }

                                returns.extend(inner_returns);
                            }
                            out_contexts.extend(new_contexts);
                            contexts.extend(out_contexts.iter().map(|c| Context {
                                variables: context.variables.clone(),
                                implementations: context.implementations.clone(),
                                ..c.clone()
                            }));
                        }

                        Value::Range(start, end, step) => {
                            let mut normal = (start..end).step_by(step);
                            let mut rev = (end..start).step_by(step).rev();
                            let range: &mut dyn Iterator<Item = i32> =
                                if start < end { &mut normal } else { &mut rev };

                            let mut new_contexts = vec![context.clone()];
                            let mut out_contexts = Vec::new();

                            for num in range {
                                //println!("{}", new_contexts.len());
                                let element =
                                    store_value(Value::Number(num as f64), 0, globals, &context);
                                for c in &mut new_contexts {
                                    (*c).variables = context.variables.clone();
                                    (*c).variables.insert(f.symbol.clone(), element);
                                }

                                let new_info = info.clone();

                                let (end_contexts, inner_returns) =
                                    compile_scope(&f.body, new_contexts, globals, new_info)?;

                                new_contexts = Vec::new();
                                for mut c in end_contexts {
                                    if c.broken == None {
                                        new_contexts.push(c)
                                    } else {
                                        c.broken = None;
                                        out_contexts.push(c)
                                    }
                                }

                                returns.extend(inner_returns);
                            }
                            out_contexts.extend(new_contexts);
                            contexts.extend(out_contexts.iter().map(|c| Context {
                                variables: context.variables.clone(),
                                implementations: context.implementations.clone(),
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
            Return(return_val) => match return_val {
                Some(val) => {
                    let mut all_values: Returns = Vec::new();
                    for context in &contexts {
                        let new_info = info.clone();
                        let (evaled, inner_returns) = val.eval(context, globals, new_info, true)?;
                        returns.extend(inner_returns);
                        all_values.extend(evaled);
                    }

                    returns.extend(all_values);
                }

                None => {
                    let mut all_values: Returns = Vec::new();
                    for context in &contexts {
                        all_values.push((
                            store_value(Value::Null, 1, globals, context),
                            context.clone(),
                        ));
                    }
                    returns.extend(all_values);
                }
            },

            Error(e) => {
                for context in &contexts {
                    let new_info = info.clone();
                    let (evaled, _) = e.message.eval(context, globals, new_info, true)?;
                    for (msg, _) in evaled {
                        let exclam = Colour::Red.bold().paint("!");
                        eprintln!(
                            "{} {} {}",
                            exclam,
                            match &globals.stored_values[msg] {
                                Value::Str(s) => s,
                                _ => "no message",
                            },
                            exclam
                        );
                    }
                }
                return Err(RuntimeError::RuntimeError {
                    message: "Error statement, see message(s) above.".to_string(),
                    info,
                });
            }
        }

        let mut new_contexts = Vec::new();

        for c in contexts.clone() {
            if c.broken == None {
                new_contexts.push(c)
            } else {
                broken_contexts.push(c)
            }
        }
        if new_contexts.is_empty() {
            break;
        }
        contexts = new_contexts;
        if let Some(c) = &stored_context {
            //resetting the context if async
            for c in contexts {
                if let Some(i) = c.broken {
                    return Err(RuntimeError::RuntimeError {
                        message:
                            "break statement is never used because it's inside an arrow statement"
                                .to_string(),
                        info: i,
                    });
                }
            }
            contexts = (*c).clone();
        }

        /*println!(
            "{} -> Compiled '{}' in {} milliseconds!",
            path,
            statement_type,
            start_time.elapsed().as_millis(),
        );*/
    }

    //return values need longer lifetimes
    for (val, _) in &returns {
        globals.stored_values.increment_single_lifetime(*val, 1);
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
    path: &PathBuf,
    context: &Context,
    globals: &mut Globals,
    info: CompilerInfo,
) -> Result<Returns, RuntimeError> {
    let mut module_path = globals
        .path
        .clone()
        .parent()
        .expect("Your file must be in a folder to import modules!")
        .join(&path);

    if module_path.is_dir() {
        module_path = module_path.join("lib.spwn");
    } else if module_path.extension().is_none() {
        module_path.set_extension("spwn");
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

    if !notes.tag.tags.iter().any(|x| x.0 == "no_std") {
        let standard_lib = import_module(
            &PathBuf::from(STD_PATH),
            &start_context,
            globals,
            info.clone(),
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

    let (contexts, returns) = compile_scope(&parsed, vec![start_context], globals, new_info)?;
    (*globals).path = stored_path;

    Ok(if returns.is_empty() {
        contexts
            .iter()
            .map(|x| {
                let mut new_context = context.clone();

                new_context.start_group = x.start_group;
                merge_impl(&mut new_context.implementations, &x.implementations);
                (NULL_STORAGE, new_context)
            })
            .collect()
    } else {
        returns
            .iter()
            .map(|(val, x)| {
                let mut new_context = context.clone();

                new_context.start_group = x.start_group;
                merge_impl(&mut new_context.implementations, &x.implementations);
                (*val, new_context)
            })
            .collect()
    })
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
