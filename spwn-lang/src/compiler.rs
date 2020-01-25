//! Tools for compiling SPWN into GD object strings
use crate::ast;
use crate::levelstring::*;
use crate::native::*;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::compiler_types::*;
use ValSuccess::{Evaluatable, Literal};

pub fn compile_spwn(statements: Vec<ast::Statement>, path: PathBuf) -> Globals {
    let mut default_variables: HashMap<String, Value> = HashMap::new();
    //add easing types
    for (i, easing) in vec![
        "NONE",
        "EASE_IN_OUT",
        "EASE_IN",
        "EASE_OUT",
        "ELASTIC_IN_OUT",
        "ELASTIC_IN",
        "ELASTIC_OUT",
        "BOUNCE_IN_OUT",
        "BOUNCE_IN",
        "BOUNCE_OUT",
        "EXPONENTIAL_IN_OUT",
        "EXPONENTIAL_IN",
        "EXPONENTIAL_OUT",
        "SINE_IN_OUT",
        "SINE_IN",
        "SINE_OUT",
        "BACK_IN_OUT",
        "BACK_IN",
        "BACK_OUT",
    ]
    .iter()
    .enumerate()
    {
        default_variables.insert(easing.to_string(), Value::Number(i as f64));
    }

    for (i, comp) in vec!["EQUAL_TO", "LARGER_THAN", "SMALLER_THAN"]
        .iter()
        .enumerate()
    {
        default_variables.insert(comp.to_string(), Value::Number(i as f64));
    }

    //context at the start of the program
    let start_context = Context {
        x: 0,
        y: 300,
        added_groups: Vec::new(),
        spawn_triggered: false,
        variables: default_variables,
        ret: None,
    };

    //variables that get changed throughout the compiling
    let mut globals = Globals {
        closed_groups: Vec::new(),
        closed_colors: Vec::new(),
        closed_blocks: Vec::new(),
        closed_items: Vec::new(),
        path: path,
        obj_list: Vec::new(),

        highest_x: 0,
        implementations: HashMap::new(),
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

    compile_scope(
        &statements,
        start_context,
        Group { id: 0 },
        &mut globals,
        &Value::Null,
    );

    globals
}

pub fn compile_scope(
    statements: &Vec<ast::Statement>,
    mut context: Context,
    mut start_group: Group,
    globals: &mut Globals,
    placeholder_value: &Value,
) -> Scope {
    context.x = globals.highest_x;

    (*globals).highest_x += 30;

    context.added_groups.sort_by(|a, b| a.id.cmp(&b.id));
    context.added_groups.dedup();

    let mut exports: HashMap<String, Value> = HashMap::new();

    //check if it only has difinitions
    if !statements.iter().any(|x| match x {
        ast::Statement::Definition(_def) => false,
        _ => true,
    }) {
        for (i, g) in context.added_groups.iter().enumerate() {
            if g.id == start_group.id {
                context.added_groups.remove(i);
                break;
            }
        }
        start_group = Group { id: 0 };
    }
    let mut statements_iter = statements.iter();

    while let Some(statement) = statements_iter.next() {
        //find out what kind of statement this is

        match statement {
            ast::Statement::Expr(expr) => {
                match expr.eval(&context, globals, placeholder_value) {
                    Literal(_l) => {}
                    Evaluatable(e) => {
                        let mut new_statements = vec![ast::Statement::Expr(e.2.to_expression())];
                        new_statements.extend(statements_iter.cloned());
                        return evaluate_and_execute(
                            (e.0, e.1),
                            &context,
                            globals,
                            new_statements,
                            start_group,
                        );
                    }
                };
            }

            ast::Statement::Definition(def) => {
                //insert the variable into the variable list
                let val = match &def.value.values[0].value {
                    ast::ValueLiteral::CmpStmt(cmp_stmt) => {
                        //create the function context
                        let mut new_context = context.clone();

                        new_context.spawn_triggered = true;

                        //pick a start group
                        let start_group = Group {
                            id: next_free(&mut globals.closed_groups),
                        };
                        new_context.variables.insert(
                            String::from(&def.symbol),
                            Value::Scope(Scope {
                                group: start_group,
                                members: HashMap::new(),
                            }),
                        );

                        Value::Scope(compile_scope(
                            &cmp_stmt.statements,
                            new_context,
                            start_group,
                            globals,
                            &Value::Null,
                        ))
                    }

                    _ => match def
                        .value
                        .eval(&context.move_down(), globals, placeholder_value)
                    {
                        Literal(l) => l,
                        Evaluatable(e) => {
                            let mut new_statements =
                                vec![ast::Statement::Definition(ast::Definition {
                                    value: e.2.to_expression(),
                                    ..def.clone()
                                })];
                            new_statements.extend(statements_iter.cloned());
                            return evaluate_and_execute(
                                (e.0, e.1),
                                &context,
                                globals,
                                new_statements,
                                start_group,
                            );
                        }
                    },
                };
                if def.symbol == "*" {
                    match val {
                        Value::Scope(s) => {
                            context.variables.extend(s.members.clone());
                            exports.extend(s.members);
                        }

                        Value::Dict(d) => {
                            context.variables.extend(d.clone());
                            exports.extend(d);
                        }
                        _ => panic!("Only compound statements can have their values extracted"),
                    }
                } else {
                    context
                        .variables
                        .insert(String::from(&def.symbol), val.clone());
                    exports.insert(String::from(&def.symbol), val);
                }
            }

            ast::Statement::If(if_stmt) => {
                let condition = match if_stmt.condition.eval(&context, globals, placeholder_value) {
                    Literal(l) => l,
                    Evaluatable(e) => {
                        let mut new_statements = vec![ast::Statement::If(ast::If {
                            condition: e.2.to_expression(),
                            ..if_stmt.clone()
                        })];
                        new_statements.extend(statements_iter.cloned());
                        return evaluate_and_execute(
                            (e.0, e.1),
                            &context,
                            globals,
                            new_statements,
                            start_group,
                        );
                    }
                };

                match condition {
                    Value::Bool(b) => {
                        //internal if statement
                        if b {
                            compile_scope(
                                &if_stmt.if_body,
                                context.move_down(),
                                start_group,
                                globals,
                                &Value::Null,
                            );
                        } else {
                            match &if_stmt.else_body {
                                Some(body) => {
                                    compile_scope(
                                        body,
                                        context.move_down(),
                                        start_group,
                                        globals,
                                        &Value::Null,
                                    );
                                }
                                None => {}
                            };
                        }
                    }
                    _ => panic!("Expected boolean condition in if statement"),
                }
            }

            ast::Statement::Impl(imp) => {
                let evaled =
                    match imp
                        .symbol
                        .to_value(&(context.move_down()), globals, placeholder_value)
                    {
                        Literal(l) => l,
                        Evaluatable(e) => {
                            let mut new_statements =
                                vec![ast::Statement::Impl(ast::Implementation {
                                    symbol: e.2,
                                    ..imp.clone()
                                })];
                            new_statements.extend(statements_iter.cloned());
                            return evaluate_and_execute(
                                (e.0, e.1),
                                &context,
                                globals,
                                new_statements,
                                start_group,
                            );
                        }
                    };

                let typ = match evaled {
                    Value::Str(s) => s,
                    _ => panic!("Expected type (string)"),
                };

                let scope = compile_scope(
                    &imp.members,
                    context.clone(),
                    Group { id: 0 },
                    globals,
                    &Value::Null,
                );

                match globals.implementations.get_mut(&typ) {
                    Some(implementation) => {
                        for (key, val) in scope.members.into_iter() {
                            (*implementation).insert(key, val);
                        }
                    }
                    None => {
                        (*globals).implementations.insert(typ, scope.members);
                    }
                }
            }
            ast::Statement::Call(call) => {
                let func =
                    match call
                        .function
                        .to_value(&(context.move_down()), globals, placeholder_value)
                    {
                        Literal(l) => l,
                        Evaluatable(e) => {
                            let mut new_statements =
                                vec![ast::Statement::Call(ast::Call { function: e.2 })];
                            new_statements.extend(statements_iter.cloned());
                            return evaluate_and_execute(
                                (e.0, e.1),
                                &context,
                                globals,
                                new_statements,
                                start_group,
                            );
                        }
                    };

                (*globals).obj_list.push(
                    GDObj {
                        obj_id: 1268,
                        groups: vec![start_group],
                        target: match func {
                            Value::Scope(s) => s.group,
                            Value::Group(g) => g,
                            _ => panic!("Not callable"),
                        },

                        ..context_trigger(context.clone())
                    }
                    .context_parameters(context.clone()),
                );
                context.y -= 30;
            }

            ast::Statement::Add(v) => {
                let val = match v.eval(&context, globals, placeholder_value) {
                    Literal(l) => l,
                    Evaluatable(e) => {
                        let mut new_statements = vec![ast::Statement::Add(e.2.to_expression())];
                        new_statements.extend(statements_iter.cloned());
                        return evaluate_and_execute(
                            (e.0, e.1),
                            &context,
                            globals,
                            new_statements,
                            start_group,
                        );
                    }
                };
                match val {
                    Value::Obj(obj) => {
                        (*globals).obj_list.push(
                            GDObj {
                                params: obj,
                                groups: vec![start_group],
                                ..context_trigger(context.clone())
                            }
                            .context_parameters(context.clone()),
                        );
                        context.y -= 30;
                    }

                    _ => panic!("Expected Object"),
                }
            }

            ast::Statement::Return(val) => {
                let return_info = match context.ret.clone() {
                    Some(ret) => ret,
                    None => panic!("Cannot return outside function defnition"),
                };
                let new_context = Context {
                    spawn_triggered: context.spawn_triggered,
                    added_groups: context.added_groups.clone(),
                    ..return_info.context
                };
                let return_placeholder = match val.eval(&context, globals, placeholder_value) {
                    Literal(l) => l,
                    Evaluatable(e) => {
                        let mut new_statements = vec![ast::Statement::Return(e.2.to_expression())];
                        new_statements.extend(statements_iter.cloned());
                        return evaluate_and_execute(
                            (e.0, e.1),
                            &context,
                            globals,
                            new_statements,
                            start_group,
                        );
                    }
                };

                compile_scope(
                    &return_info.statements,
                    new_context,
                    start_group,
                    globals,
                    &return_placeholder,
                );
            }

            ast::Statement::EOI => {}
        }
    }

    Scope {
        group: start_group,
        members: exports,
    }
}

pub fn import_module(path: &PathBuf, globals: &mut Globals) -> Value {
    let module_path = globals
        .path
        .clone()
        .parent()
        .expect("Your file must be in a folder to import modules!")
        .join(&path);
    let parsed = crate::parse_spwn(&module_path);
    let scope = compile_scope(
        &parsed,
        Context {
            x: 0,
            y: 300,
            added_groups: Vec::new(),
            spawn_triggered: false,
            variables: HashMap::new(),
            ret: None,
        },
        Group { id: 0 },
        globals,
        &Value::Null,
    );
    match scope.members.get("exports") {
        Some(value) => (value).clone(),
        None => panic!("No \"exports\" variable in module"),
    }
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
