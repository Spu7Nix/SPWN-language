//! Tools for compiling SPWN into GD object strings
use crate::ast;
use crate::levelstring::*;
use crate::native::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Context {
    pub x: u32,
    pub y: u16,
    pub added_groups: Vec<Group>,
    pub spawn_triggered: bool,
    pub variables: HashMap<String, Value>,
}

impl Context {
    pub fn move_down(&self) -> Context {
        Context {
            y: self.y - 30,
            ..self.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub group: Group,
    pub members: HashMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Macro {
    pub args: Vec<String>,
    pub def_context: Context,
    pub body: Vec<ast::Statement>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Group(Group),
    Color(Color),
    Block(Block),
    Item(Item),
    Number(f64),
    Bool(bool),
    Scope(Scope),
    Macro(Macro),
    Str(String),
    Null,
}

#[derive(Clone)]
pub struct Globals {
    pub closed_groups: Vec<u16>,
    pub closed_colors: Vec<u16>,
    pub closed_blocks: Vec<u16>,
    pub closed_items: Vec<u16>,
    pub path: PathBuf,
    pub obj_list: Vec<GDTrigger>,

    pub highest_x: u32,
}

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
    };

    compile_scope(&statements, start_context, Group { id: 0 }, &mut globals);

    globals
}

pub fn compile_scope(
    statements: &Vec<ast::Statement>,
    mut context: Context,
    mut start_group: Group,
    globals: &mut Globals,
) -> Scope {
    context.x = globals.highest_x;

    (*globals).highest_x += 30;

    context.added_groups.sort_by(|a, b| a.id.cmp(&b.id));
    context.added_groups.dedup();

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
    let statements_iter = statements.iter();

    for statement in statements_iter {
        //find out what kind of statement this is

        match statement {
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
                        ))
                    }

                    _ => def.value.eval(&context.move_down(), globals),
                };
                if def.symbol == "*" {
                    match val {
                        Value::Scope(s) => {
                            context.variables.extend(s.members);
                        }
                        _ => panic!("Only compound statements can have their values extracted"),
                    }
                } else {
                    context.variables.insert(String::from(&def.symbol), val);
                }
                //TODO: Account for dicts
            }

            ast::Statement::Event(e) => {
                let func = match e.func.to_value(&context.move_down(), globals) {
                    Value::Scope(s) => s.group,
                    Value::Group(g) => g,
                    _ => panic!("Not callable"),
                };

                event(
                    &e.symbol,
                    e.args.iter().map(|x| x.eval(&context, globals)).collect(),
                    context.clone(),
                    globals,
                    start_group,
                    func,
                );
                context.y -= 30;
            }

            ast::Statement::Call(call) => {
                let func = call.function.to_value(&(context.move_down()), globals);

                (*globals).obj_list.push(
                    GDTrigger {
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

            ast::Statement::Native(call) => {
                let native = native_func(call.clone(), context.clone(), globals, start_group);
                if !native {
                    match call.function.to_value(&context, globals) {
                        Value::Macro(m) => {
                            let mut new_context = context.clone();
                            new_context.variables = m.def_context.variables;
                            for (i, arg) in m.args.iter().enumerate() {
                                new_context
                                    .variables
                                    .insert(arg.clone(), call.args[i].eval(&context, globals));
                            }
                            compile_scope(&m.body, new_context, start_group, globals);
                        }
                        _ => panic!("Not a function!"),
                    }
                }

                context.y -= 30;
            }

            ast::Statement::Macro(m) => {
                context.variables.insert(
                    String::from(&m.name),
                    Value::Macro(Macro {
                        args: m.args.clone(),
                        def_context: context.clone(),
                        body: m.body.statements.clone(),
                    }),
                );
            }

            ast::Statement::EOI => {}
        }
    }

    Scope {
        group: start_group,
        members: context.clone().variables,
    }
}
impl ast::Expression {
    pub fn eval(&self, context: &Context, globals: &mut Globals) -> Value {
        let mut vals = self.values.iter();
        let mut acum = vals.next().unwrap().to_value(context, globals);

        if self.operators.is_empty() {
            return acum;
        }

        for (i, var) in vals.enumerate() {
            let val = var.to_value(context, globals);

            match self.operators[i].as_ref() {
                "||" | "&&" => {
                    //boolean operations
                    if let Value::Bool(b) = val {
                        if let Value::Bool(a) = acum {
                            match self.operators[i].as_ref() {
                                "||" => acum = Value::Bool(a || b),
                                "&&" => acum = Value::Bool(a && b),
                                _ => unreachable!(),
                            }
                        } else {
                            panic!("Right side must be boolean")
                        }
                    } else {
                        panic!("Both sides must be boolean")
                    }
                }

                ">" | "<" | ">=" | "<=" | "/" | "*" | "+" | "-" => {
                    //number operations
                    if let Value::Number(num) = val {
                        if let Value::Number(a) = acum {
                            match self.operators[i].as_ref() {
                                ">" => acum = Value::Bool(a > num),
                                "<" => acum = Value::Bool(a < num),
                                ">=" => acum = Value::Bool(a >= num),
                                "<=" => acum = Value::Bool(a <= num),
                                "/" => acum = Value::Number(a / num),
                                "*" => acum = Value::Number(a * num),
                                "+" => acum = Value::Number(a + num),
                                "-" => acum = Value::Number(a - num),
                                _ => unreachable!(),
                            }
                        } else {
                            panic!("Right side must be number")
                        }
                    } else {
                        panic!("Both sides must be numbers")
                    }
                }

                //any
                "==" => {
                    acum = Value::Bool(val == acum);
                }

                "!=" => {
                    acum = Value::Bool(val != acum);
                }

                _ => unreachable!(),
            }
        }

        acum
    }
}

fn import_module(path: &PathBuf, globals: &mut Globals) -> Value {
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
        },
        Group { id: 0 },
        globals,
    );
    match scope.members.get("exports") {
        Some(value) => (value).clone(),
        None => panic!("No \"exports\" variable in module"),
    }
}

impl ast::Variable {
    pub fn to_value(&self, context: &Context, globals: &mut Globals) -> Value {
        // TODO: Check if this variable has native functions called on it, and if not set this to false

        let base_value = match &self.value {
            ast::ValueLiteral::ID(id) => match id.class_name.as_ref() {
                "g" => {
                    if id.unspecified {
                        Value::Group(Group {
                            id: next_free(&mut globals.closed_groups),
                        })
                    } else {
                        Value::Group(Group { id: id.number })
                    }
                }
                "c" => {
                    if id.unspecified {
                        Value::Color(Color {
                            id: next_free(&mut globals.closed_colors),
                        })
                    } else {
                        Value::Color(Color { id: id.number })
                    }
                }
                "b" => {
                    if id.unspecified {
                        Value::Block(Block {
                            id: next_free(&mut globals.closed_blocks),
                        })
                    } else {
                        Value::Block(Block { id: id.number })
                    }
                }
                "i" => {
                    if id.unspecified {
                        Value::Item(Item {
                            id: next_free(&mut globals.closed_items),
                        })
                    } else {
                        Value::Item(Item { id: id.number })
                    }
                }
                _ => unreachable!(),
            },
            ast::ValueLiteral::Number(num) => Value::Number(*num),
            ast::ValueLiteral::CmpStmt(cmp_stmt) => {
                Value::Scope(cmp_stmt.to_scope(context, globals))
            }
            ast::ValueLiteral::Expression(expr) => expr.eval(&context, globals),
            ast::ValueLiteral::Bool(b) => Value::Bool(*b),
            ast::ValueLiteral::Symbol(string) => match context.variables.get(string) {
                Some(value) => (value).clone(),
                None => panic!(format!(
                    "The variable {} does not exist in this scope.",
                    string
                )),
            },
            ast::ValueLiteral::Str(s) => Value::Str(s.clone()),
            ast::ValueLiteral::Import(i) => import_module(i, globals),
        };

        let mut final_value = base_value;

        for mem in self.symbols.iter() {
            final_value = member(final_value, mem.clone());
        }

        final_value
    }
}

impl ast::CompoundStatement {
    fn to_scope(&self, context: &Context, globals: &mut Globals) -> Scope {
        //create the function context
        let mut new_context = context.clone();

        new_context.spawn_triggered = true;

        //pick a start group
        let start_group = Group {
            id: next_free(&mut globals.closed_groups),
        };

        compile_scope(&self.statements, new_context, start_group, globals)
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
    panic!("All ids of this t are used up!");
}
