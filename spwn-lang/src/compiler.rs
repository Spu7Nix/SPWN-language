//! Tools for compiling SPWN into GD object strings

use crate::ast;
use crate::builtin::*;
use crate::compiler_info::CodeArea;
use crate::compiler_info::CompilerInfo;
use crate::context::*;
use crate::fmt::SpwnFmt;
use crate::globals::Globals;
use crate::levelstring::*;
use crate::value::*;
use crate::value_storage::*;
use crate::STD_PATH;
use std::collections::{HashMap, HashSet};
use std::mem;

use crate::parser::{ParseNotes, SyntaxError};
use std::fs;
use std::path::PathBuf;

use crate::compiler_types::*;
use crate::print_with_color;

use ariadne::Fmt;
use internment::Intern;
use termcolor::Color as TColor;

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

    PackageError {
        err: Box<RuntimeError>,
        info: CompilerInfo,
    },

    TypeError {
        expected: String,
        found: String,
        val_def: CodeArea,
        info: CompilerInfo,
    },

    PatternMismatchError {
        pattern: String,
        val: String,
        pat_def: CodeArea,
        val_def: CodeArea,
        info: CompilerInfo,
    },

    CustomError(ErrorReport),

    BuiltinError {
        message: String,
        info: CompilerInfo,
    },

    MutabilityError {
        val_def: CodeArea,
        info: CompilerInfo,
    },
    ContextChangeMutateError {
        val_def: CodeArea,
        info: CompilerInfo,
        context_changes: Vec<CodeArea>,
    },
    ContextChangeError {
        message: String,
        info: CompilerInfo,
        context_changes: Vec<CodeArea>,
    },

    BreakNeverUsedError {
        breaktype: BreakType,
        info: CompilerInfo,
        broke: CodeArea,
        dropped: CodeArea,
        reason: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct RainbowColorGenerator {
    h: f64,
    s: f64,
    b: f64,
}

impl RainbowColorGenerator {
    pub fn new(h: f64, s: f64, b: f64) -> Self {
        Self { h, s, b }
    }

    pub fn next(&mut self) -> ariadne::Color {
        self.h += 20.0;
        self.h %= 360.0;

        let hsl = *self;

        let c = (1.0 - (hsl.b * 2.0 - 1.0).abs()) * hsl.s;
        let h = hsl.h / 60.0;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let m = hsl.b - c * 0.5;

        let (red, green, blue) = if h >= 0.0 && h < 0.0 {
            (c, x, 0.0)
        } else if (1.0..2.0).contains(&h) {
            (x, c, 0.0)
        } else if (2.0..3.0).contains(&h) {
            (0.0, c, x)
        } else if (3.0..4.0).contains(&h) {
            (0.0, x, c)
        } else if (4.0..5.0).contains(&h) {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        ariadne::Color::RGB(
            ((red + m) * 255.0) as u8,
            ((green + m) * 255.0) as u8,
            ((blue + m) * 255.0) as u8,
        )
    }
}
pub fn create_report(rep: ErrorReport) -> ariadne::Report<CodeArea> {
    use ariadne::{Config, Label, Report, ReportKind};

    let info = rep.info;
    let message = rep.message;
    let labels = rep.labels;
    let note = rep.note;
    let position = info.position;

    let mut colors = RainbowColorGenerator::new(0.0, 1.5, 0.8);

    let mut report = Report::build(ReportKind::Error, position.file.as_ref(), position.pos.0)
        .with_config(Config::default().with_cross_gap(true))
        .with_message(message.clone());

    let mut i = 1;
    for area in info.call_stack {
        let color = colors.next();
        report = report.with_label(
            Label::new(area)
                .with_order(i)
                .with_message(&format!(
                    "{}: Error comes from this macro call",
                    i.to_string().fg(color)
                ))
                .with_color(color)
                .with_priority(1),
        );
        i += 1;
    }

    if labels.is_empty() || !labels.iter().any(|(a, _)| a == &position) {
        let color = colors.next();
        report = report.with_label(
            Label::new(position)
                .with_order(i)
                .with_color(color)
                .with_message(message)
                .with_priority(2),
        );
    }
    if i == 1 && labels.len() == 1 {
        let color = colors.next();
        report = report.with_label(
            Label::new(labels[0].0)
                .with_message(labels[0].1.clone())
                .with_order(i)
                .with_color(color)
                .with_priority(2),
        );
    } else if !labels.is_empty() {
        for (area, label) in labels {
            let color = colors.next();
            report = report.with_label(
                Label::new(area)
                    .with_message(&format!("{}: {}", i.to_string().fg(color), label))
                    .with_order(i)
                    .with_color(color)
                    .with_priority(2),
            );
            i += 1;
        }
    }
    if let Some(note) = note {
        report = report.with_note(note);
    }
    report.finish()
}

pub fn create_error(
    info: CompilerInfo,
    message: &str,
    labels: &[(CodeArea, &str)],
    note: Option<&str>,
) -> ErrorReport {
    ErrorReport {
        info: info.clone(),
        message: message.to_string(),
        labels: labels
            .iter()
            .map(|(a, s)| {
                if !a.file.as_ref().exists() {
                    (
                        CodeArea {
                            pos: (0, 0),
                            ..info.position
                        },
                        s.to_string(),
                    )
                } else {
                    (*a, s.to_string())
                }
            })
            .collect(),
        note: note.map(|s| s.to_string()),
    }
}

pub struct ErrorReport {
    pub info: CompilerInfo,
    pub message: String,
    pub labels: Vec<(CodeArea, String)>,
    pub note: Option<String>,
}

impl From<RuntimeError> for ErrorReport {
    fn from(err: RuntimeError) -> ErrorReport {
        let mut colors = RainbowColorGenerator::new(120.0, 1.5, 0.8);
        let a = colors.next();
        let b = colors.next();

        match err {
            RuntimeError::UndefinedErr {
                undefined,
                desc,
                info,
            } => create_error(
                info.clone(),
                &format!("Use of undefined {}", desc),
                &[(
                    info.position,
                    &format!("'{}' is undefined", undefined.fg(b)),
                )],
                None,
            ),
            RuntimeError::PackageSyntaxError { err, info } => {
                let syntax_error = ErrorReport::from(err);
                let mut labels = vec![(info.position, "Error when parsing this library/module")];
                labels.extend(syntax_error.labels.iter().map(|(a, b)| (*a, b.as_str())));
                create_error(info, &syntax_error.message, &labels, None)
            }

            RuntimeError::PackageError { err, info } => {
                let syntax_error = ErrorReport::from(*err);
                let mut labels = vec![(info.position, "Error when running this library/module")];
                labels.extend(syntax_error.labels.iter().map(|(a, b)| (*a, b.as_str())));
                create_error(info, &syntax_error.message, &labels, None)
            }

            RuntimeError::TypeError {
                expected,
                found,
                info,
                val_def,
            } => create_error(
                info.clone(),
                "Type mismatch",
                &[
                    (
                        val_def,
                        &format!("Value defined as {} here", found.clone().fg(b)),
                    ),
                    (
                        info.position,
                        &format!("Expected {}, found {}", expected.fg(a), found.fg(b)),
                    ),
                ],
                None,
            ),

            RuntimeError::PatternMismatchError {
                pattern,
                val,
                info,
                val_def,
                pat_def,
            } => create_error(
                info.clone(),
                "Pattern mismatch",
                &[
                    (
                        val_def,
                        &format!("Value defined as {} here", val.clone().fg(b)),
                    ),
                    (
                        pat_def,
                        &format!("Pattern defined as {} here", pattern.clone().fg(b)),
                    ),
                    (
                        info.position,
                        &format!("Expected {}, found {}", pattern.fg(a), val.fg(b)),
                    ),
                ],
                None,
            ),

            RuntimeError::CustomError(report) => report,

            RuntimeError::BuiltinError { message, info } => create_error(
                info.clone(),
                "Error when using built-in function",
                &[(info.position, &message)],
                None,
            ),

            RuntimeError::MutabilityError { val_def, info } => create_error(
                info.clone(),
                "Attempted to change immutable variable",
                &[
                    (val_def, "Value was defined as immutable here"),
                    (info.position, "This tries to change the value"),
                ],
                None,
            ),

            RuntimeError::ContextChangeMutateError {
                val_def,
                info,
                context_changes,
            } => {
                let mut labels = vec![(val_def, "Value was defined here")];

                #[allow(clippy::comparison_chain)]
                if context_changes.len() == 1 {
                    labels.push((
                        context_changes[0],
                        "New trigger function context was defined here",
                    ));
                } else if context_changes.len() > 1 {
                    labels.push((*context_changes.last().unwrap(), "Context was changed here"));

                    for change in context_changes[1..(context_changes.len() - 1)].iter().rev() {
                        labels.push((*change, "This changes the context inside the macro"));
                    }

                    labels.push((
                        context_changes[0],
                        "New trigger function context was defined here",
                    ));
                }

                labels.push((info.position, "Attempted to change value here"));

                create_error(
                    info,
                    "Attempted to change a variable defined in a different trigger function context",
                    &labels,
                    Some("Consider using a counter"),
                )
            }

            RuntimeError::ContextChangeError {
                message,
                info,
                context_changes,
            } => {
                let mut labels = Vec::new();

                #[allow(clippy::comparison_chain)]
                if context_changes.len() == 1 {
                    labels.push((
                        context_changes[0],
                        "New trigger function context was defined here",
                    ));
                } else if context_changes.len() > 1 {
                    labels.push((*context_changes.last().unwrap(), "Context was changed here"));

                    for change in context_changes[1..(context_changes.len() - 1)].iter().rev() {
                        labels.push((*change, "This changes the context inside the macro"));
                    }

                    labels.push((
                        context_changes[0],
                        "New trigger function context was defined here",
                    ));
                }

                create_error(info, &message, &labels, None)
            }

            RuntimeError::BreakNeverUsedError {
                info,
                breaktype,
                broke,
                dropped,
                reason,
            } => create_error(
                info,
                &format!(
                    "{} statement never used",
                    match breaktype {
                        BreakType::ContinueLoop => "Continue",
                        BreakType::Loop => "Break",
                        BreakType::Macro(_, _) => "Return",
                        BreakType::Switch(_) => unreachable!("Switch break in the wild"),
                    }
                ),
                &[
                    (broke, "Decleared here"),
                    (
                        dropped,
                        &format!("Can't reach past here because {}", reason),
                    ),
                ],
                None,
            ),
        }
    }
}

pub const NULL_STORAGE: StoredValue = 1;
pub const BUILTIN_STORAGE: StoredValue = 0;

pub fn compile_spwn(
    statements: Vec<ast::Statement>,
    path: PathBuf,
    included_paths: Vec<PathBuf>,
    notes: ParseNotes,
) -> Result<Globals, RuntimeError> {
    //variables that get changed throughout the compiling
    let mut globals = Globals::new(path.clone());
    globals.includes = included_paths;
    if statements.is_empty() {
        return Err(RuntimeError::CustomError(create_error(
            CompilerInfo::from_area(crate::compiler_info::CodeArea {
                file: Intern::new(path),
                pos: (0, 0),
            }),
            "this script is empty",
            &[],
            None,
        )));
    }
    let mut start_context = FullContext::new();
    //store at pos 0
    // store_value(Value::Builtins, 1, &mut globals, &start_context);
    // store_value(Value::Null, 1, &mut globals, &start_context);

    let start_info = CompilerInfo {
        ..CompilerInfo::from_area(crate::compiler_info::CodeArea {
            file: Intern::new(path.clone()),
            pos: statements[0].pos,
        })
    };
    use std::time::Instant;

    //println!("Importing standard library...");
    print_with_color("Building script ...", TColor::Cyan);
    print_with_color("———————————————————————————\n", TColor::White);
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
                start_context.inner().new_variable(a, b, c)
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
                    file: Intern::new(path),
                },
                reason: "the program ended".to_string(),
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

    //----------------------------------------------------------------------- **

    Ok(globals)
}

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

    for statement in statements.iter() {
        contexts.reset_return_vals();
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
            Some(contexts.iter().map(|a| a.clone()).collect::<Vec<_>>())
        } else {
            None
        };

        //println!("{}:{}:{}", info.current_file.to_string_lossy(), info.pos.0.0, info.pos.0.1);
        //use crate::fmt::SpwnFmt;
        match &statement.body {
            Expr(expr) => {
                let is_assign = !expr.operators.is_empty()
                    && expr.operators[0] == ast::Operator::Assign
                    && !expr.values[0]
                        .is_undefinable(contexts.iter().next().unwrap().inner(), globals);

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
                            for full_context in contexts.iter() {
                                let storage =
                                    symbol.define(full_context.inner(), globals, &info)?;

                                //pick a start group
                                let start_group = Group::next_free(&mut globals.closed_groups);
                                //store value
                                globals.stored_values[storage] =
                                    Value::TriggerFunc(TriggerFunction { start_group });

                                full_context.inner().fn_context_change_stack = vec![info.position];
                                //new_info.last_context_change_stack = vec![info.position.clone()];

                                f.to_trigger_func(
                                    full_context,
                                    globals,
                                    info.clone(),
                                    Some(start_group),
                                )?;
                            }
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
                            new_expr.eval(contexts, globals, info.clone(), !mutable)?;

                            for full_context in contexts.iter() {
                                let (context, val) = full_context.inner_value();
                                let storage = symbol.define(context, globals, &info)?;
                                //clone the value so as to not share the reference

                                let cloned = clone_and_get_value(
                                    val,
                                    globals,
                                    context.start_group,
                                    !mutable,
                                );

                                globals.stored_values[storage] = cloned;
                            }
                        }
                    }
                } else {
                    expr.eval(contexts, globals, info.clone(), true)?;
                }
            }

            Extract(val) => {
                val.eval(contexts, globals, info.clone(), true)?;
                for full_context in contexts.iter() {
                    let (context, val) = full_context.inner_value();
                    let fn_context = context.start_group;
                    match globals.stored_values[val].clone() {
                        Value::Dict(d) => {
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
                                context.new_variable(a, b, c);
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

                                context.new_variable(Intern::new(String::from(*name)), p, 0);
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
                    let mut params = HashMap::new();
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
                let i_name = f.symbol;
                // skips all broken contexts, so as to not interfere with potential breaks in the following loops

                for full_context in contexts.iter() {
                    let (_, val) = full_context.inner_value();
                    globals.push_new_preserved();
                    globals.push_preserved_val(val);

                    match globals.stored_values[val].clone() {
                        // what are we iterating
                        Value::Array(arr) => {
                            // its an array!for

                            for element in arr {
                                // going through the array items
                                full_context.disable_breaks(BreakType::ContinueLoop);

                                full_context.set_variable_and_clone(
                                    i_name,
                                    element,
                                    -1, // so that it gets removed at the end of the scope
                                    true,
                                    globals,
                                    globals.get_area(element),
                                );

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
                                    (*c.inner()).new_variable(
                                        i_name,
                                        store_const_value(
                                            Value::Array(vec![key, val]),
                                            globals,
                                            fn_context,
                                            globals.get_area(v),
                                        ),
                                        -1,
                                    );
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

                                full_context.set_variable_and_store(
                                    i_name,
                                    Value::Str(ch.to_string()),
                                    -1, // so that it gets removed at the end of the scope
                                    true,
                                    globals,
                                    info.position,
                                );

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

                                full_context.set_variable_and_store(
                                    i_name,
                                    Value::Number(num as f64),
                                    -1, // so that it gets removed at the end of the scope
                                    true,
                                    globals,
                                    info.position,
                                );

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
                    let err = globals.stored_values[c.inner().return_value].to_str(globals);
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

        if let Some(c) = stored_context {
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
                    if !merge_contexts(&mut not_broken, globals) {
                        break;
                    }
                }

                broken.extend(not_broken);

                *contexts =
                    FullContext::stack(&mut broken.into_iter().map(FullContext::Single)).unwrap();
            } else if not_broken.is_empty() {
                break;
            }
        }

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
            globals.stored_values.map.len() as u32 - globals.stored_values.prev_value_count;

        if increase > 5000 {
            globals.collect_garbage(contexts);
        }
    }

    // TODO: get rid of lifetimes

    contexts.exit_scope();

    Ok(())
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
        ImportType::Script(p) => globals
            .path
            .clone()
            .parent()
            .expect("Your file must be in a folder to import modules!")
            .join(&p),

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
    //println!("importing: {:?}", path);
    if !forced {
        if let Some(ret) = globals.prev_imports.get(path).cloned() {
            merge_impl(&mut globals.implementations, &ret.1);
            for c in contexts.iter() {
                c.inner().return_value = ret.0;
            }
            return Ok(());
        }
    }

    let mut module_path = get_import_path(path, globals, info.clone())?;

    if module_path.is_dir() {
        module_path = module_path.join("lib.spwn");
    } else if module_path.is_file() && module_path.extension().is_none() {
        module_path.set_extension("spwn");
    } else if !module_path.is_file() {
        return Err(RuntimeError::CustomError(create_error(
            info,
            &format!(
                "Couldn't find library file ({})",
                module_path.to_string_lossy()
            ),
            &[],
            None,
        )));
    }

    let module_path = Intern::new(module_path);

    let unparsed = match fs::read_to_string(module_path.as_ref()) {
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
    };
    let (parsed, notes) = match crate::parse_spwn(unparsed, module_path.as_ref().clone()) {
        Ok(p) => p,
        Err(err) => return Err(RuntimeError::PackageSyntaxError { err, info }),
    };

    let mut start_context = FullContext::new();

    globals.push_new_preserved();
    for c in contexts.with_breaks() {
        for stack in c.inner().get_variables().values() {
            for (v, _) in stack.iter() {
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
        let mut stored = HashMap::new();

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
                start_context.inner().new_variable(a, b, c)
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
    (*globals).path = module_path;

    let mut new_info = info.clone();

    new_info.position.file = module_path;
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

    for fc in start_context.with_breaks() {
        let c = fc.inner();
        if let Some((r, i)) = c.broken {
            if let BreakType::Macro(v, _) = r {
                for full_context in contexts.iter() {
                    let fn_context = full_context.inner().start_group;
                    (*full_context).inner().return_value = match v {
                        Some(v) => clone_value(v, globals, fn_context, true, info.position),
                        None => NULL_STORAGE,
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
                //     .increment_single_lifetime(*val, 1, &mut HashSet::new());
            }
        }
        for (k1, k2) in to_be_deleted {
            (*globals).implementations.get_mut(&k1).unwrap().remove(&k2);
        }

        //merge impls
        merge_impl(&mut globals.implementations, &stored_impl);
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
