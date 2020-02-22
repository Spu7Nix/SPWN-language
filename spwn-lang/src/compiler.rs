//! Tools for compiling SPWN into GD object strings
use crate::ast;
use crate::levelstring::*;
use crate::native::*;

use std::collections::HashMap;
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

    compile_scope(&statements, vec![Context::new()], &mut globals);

    globals
}

pub fn compile_scope(
    statements: &Vec<ast::Statement>,
    mut contexts: Vec<Context>,
    globals: &mut Globals,
) -> (Vec<Context>, Returns) {
    /*
    context.x = globals.highest_x;
    if start_group.id != 0 {
        context.spawn_triggered = true;
    }

    (*globals).highest_x += 30;
    context.y -= 30;

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
    */
    let mut statements_iter = statements.iter();

    let mut returns: Returns = Vec::new();

    while let Some(statement) = statements_iter.next() {
        //find out what kind of statement this is

        match statement {
            ast::Statement::Expr(expr) => {
                let mut new_contexts: Vec<Context> = Vec::new();
                for context in contexts {
                    //we dont care about the return value in this case
                    new_contexts.extend(expr.eval(context, globals).iter().map(|x| x.1.clone()));
                }
                contexts = new_contexts;
            }

            ast::Statement::Definition(def) => {
                let mut all_values: Returns = Vec::new();
                for context in contexts {
                    all_values.extend(def.value.eval(context, globals));
                }
                contexts = Vec::new();
                for (val, mut context) in all_values {
                    if def.symbol == "*" {
                        match val {
                            Value::Dict(d) => {
                                context.variables.extend(d.clone());
                            }
                            _ => panic!("Only dict can have their values extracted"),
                        }
                    } else {
                        context
                            .variables
                            .insert(String::from(&def.symbol), val.clone());
                    }
                    contexts.push(context);
                }
            }

            ast::Statement::If(if_stmt) => {
                let mut all_values: Returns = Vec::new();
                for context in contexts {
                    all_values.extend(if_stmt.condition.eval(context, globals));
                }
                contexts = Vec::new();

                for (val, context) in all_values {
                    match val {
                        Value::Bool(b) => {
                            //internal if statement
                            if b {
                                compile_scope(&if_stmt.if_body, vec![context], globals);
                            // TODO: add the returns from these scopes
                            } else {
                                match &if_stmt.else_body {
                                    Some(body) => {
                                        compile_scope(body, vec![context], globals);
                                    }
                                    None => {}
                                };
                            }
                        }
                        _ => panic!("Expected boolean condition in if statement"),
                    }
                }
            }

            ast::Statement::Async(a) => {
                //just get value of a in every context ig
            }

            ast::Statement::Impl(imp) => {
                for context in &mut contexts {
                    let evaled = imp.symbol.to_value(context.clone(), globals);

                    for (typ, c) in evaled {
                        if let Value::Str(s) = typ {
                        } else {
                            panic!("Must implement on a type (a string)");
                        }
                    }
                }

                /*match globals.implementations.get_mut(&typ) {
                    Some(implementation) => {
                        for (key, val) in Func.members.into_iter() {
                            (*implementation).insert(key, val);
                        }
                    }
                    None => {
                        (*globals).implementations.insert(typ, Func.members);
                    }
                }*/
            }
            ast::Statement::Call(call) => {
                let mut all_values: Returns = Vec::new();
                for context in contexts {
                    all_values.extend(call.function.to_value(context, globals));
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
                                _ => panic!("Not callable"),
                            },

                            ..context_trigger(context.clone())
                        }
                        .context_parameters(context.clone()),
                    );
                }
            }

            ast::Statement::Add(v) => {
                let mut all_values: Returns = Vec::new();
                for context in contexts {
                    all_values.extend(v.eval(context, globals));
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

                        _ => panic!("Expected Object"),
                    }
                }
            }
            ast::Statement::For(f) => unimplemented!(),
            /*ast::Statement::For(f) => {
                let evaled = match f.array.eval(context, globals, placeholder_value) {
                    Literal(l) => l,
                    Evaluatable(e) => {
                        let mut new_statements = vec![ast::Statement::For(ast::For {
                            array: e.2.to_expression(),
                            ..f.clone()
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

                let array = match evaled {
                    Value::Array(a) => a,
                    _ => panic!("Non iteratable type"),
                };

                if !array.is_empty() {
                    //currently makes a macro and then calls that macro.
                    //Probably super innefiencent, but this is a job for future sput
                    let mut new_vars = context.variables.clone();
                    new_vars.insert(f.symbol.clone(), array[0].clone());

                    let mut body_with_ret = f.body.to_owned();
                    body_with_ret.push(ast::Statement::Return(
                        Value::Null.to_variable().to_expression(),
                    ));
                    let mut new_statements = vec![ast::Statement::Expr(
                        ast::Variable {
                            operator: None,
                            value: ast::ValueLiteral::Resolved(Value::Macro(Macro {
                                body: body_with_ret,
                                args: Vec::new(),
                                def_context: Context {
                                    variables: new_vars,
                                    ..context.clone()
                                },
                            })),
                            path: vec![ast::Path::Call(Vec::new())],
                        }
                        .to_expression(),
                    )];

                    if array.len() > 1 {
                        new_statements.push(ast::Statement::For(ast::For {
                            array: ast::ValueLiteral::Resolved(Value::Array(
                                array[1..(array.len() - 1)].to_vec(),
                            ))
                            .to_variable()
                            .to_expression(),
                            ..f.to_owned()
                        }));
                    }
                    new_statements.extend(statements_iter.cloned());

                    //println!("{:?}", new_statements);
                    return compile_scope(
                        &new_statements,
                        context,
                        start_group,
                        globals,
                        &Value::Null,
                    );
                }

                //internal for loop
                //TODO: make some deeply nested shit that deals with this or smth idk i need sleep gn
            }*/
            ast::Statement::Return(val) => {
                let mut all_values: Returns = Vec::new();
                for context in contexts.clone() {
                    all_values.extend(val.eval(context, globals));
                }

                returns.extend(all_values);
            }

            ast::Statement::EOI => {}
        }
    }

    //(*globals).highest_x = context.x;
    (contexts, returns)
}

pub fn import_module(path: &PathBuf, globals: &mut Globals) -> Value {
    let module_path = globals
        .path
        .clone()
        .parent()
        .expect("Your file must be in a folder to import modules!")
        .join(&path);
    let parsed = crate::parse_spwn(&module_path);
    let (contexts, _) = compile_scope(&parsed, vec![Context::new()], globals);
    if contexts.len() > 1 {
        panic!(
            "Imported files does not (currently) support context splitting in the main scope.
            Please remove any context splitting statements outside of function or macro definitions."
        )
    }
    match contexts[0].variables.get("exports") {
        Some(val) => val.clone(),
        None => Value::Null,
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
