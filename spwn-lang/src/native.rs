//! Defining all native classes and functions
use crate::compiler::*;
use crate::compiler_types::*;
use crate::levelstring::*;

use crate::ast;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Group {
    pub id: u16,
}

impl Group {
    pub fn native(
        &self,
        name: &String,
        arguments: Vec<Value>,
        context: Context,
        globals: &mut Globals,
        start_group: Group,
    ) -> bool {
        match name.as_str() {
            // group.move(x, y, time, ease_type, ease_value)
            "move" => {
                let mut args = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

                for i in 0..arguments.len() {
                    args[i] = match arguments[i] {
                        Value::Number(num) => num,
                        _ => panic!("Expected number"),
                    };
                }
                let trigger = GDObj {
                    obj_id: 901,
                    target: *self,
                    groups: vec![start_group],
                    params: vec![
                        (28, (args[0] * 3.0).to_string()),
                        (29, (args[1] * 3.0).to_string()),
                        (10, args[2].to_string()),
                        (30, args[3].to_string()),
                        (85, args[4].to_string()),
                    ],
                    ..context_trigger(context.clone())
                }
                .context_parameters(context.clone());

                (*globals).obj_list.push(trigger);
                true
            }

            "stop" => {
                let trigger = GDObj {
                    obj_id: 1616,
                    target: *self,
                    groups: vec![start_group],
                    ..context_trigger(context.clone())
                }
                .context_parameters(context.clone());

                (*globals).obj_list.push(trigger);
                true
            }

            "alpha" => {
                let mut args = vec![0.0, 0.0];

                for i in 0..arguments.len() {
                    args[i] = match arguments[i] {
                        Value::Number(num) => num,
                        _ => panic!("Expected number"),
                    };
                }

                let trigger = GDObj {
                    obj_id: 1007,
                    target: *self,
                    groups: vec![start_group],
                    params: vec![(35, args[0].to_string()), (10, args[1].to_string())],
                    ..context_trigger(context.clone())
                }
                .context_parameters(context.clone());

                (*globals).obj_list.push(trigger);
                true
            }

            "enable" => {
                let trigger = GDObj {
                    obj_id: 1649,
                    target: *self,
                    groups: vec![start_group],
                    params: vec![(56, "1".to_string())],
                    ..context_trigger(context.clone())
                }
                .context_parameters(context.clone());

                (*globals).obj_list.push(trigger);
                true
            }

            "disable" => {
                let trigger = GDObj {
                    obj_id: 1649,
                    target: *self,
                    groups: vec![start_group],
                    params: vec![(56, "0".to_string())],
                    ..context_trigger(context.clone())
                }
                .context_parameters(context.clone());

                (*globals).obj_list.push(trigger);
                true
            }
            // group.rotate(center, times 360, duration, easing, easing rate, lock object rotation)
            "rotate" => {
                let mut args = (Group { id: 0 }, 0.0, 0.0, 0.0, 2.0, false);
                if arguments.len() > 0 {
                    args.0 = match arguments[0] {
                        Value::Group(g) => g,
                        _ => panic!("Expected group, got {:?}", arguments[0]),
                    };
                }
                if arguments.len() > 1 {
                    args.1 = match arguments[1] {
                        Value::Number(n) => n,
                        _ => panic!("Expected number, got {:?}", arguments[1]),
                    };
                }
                if arguments.len() > 2 {
                    args.2 = match arguments[2] {
                        Value::Number(n) => n,
                        _ => panic!("Expected number, got {:?}", arguments[2]),
                    };
                }
                if arguments.len() > 3 {
                    args.3 = match arguments[3] {
                        Value::Number(n) => n,
                        _ => panic!("Expected number, got {:?}", arguments[3]),
                    };
                }
                if arguments.len() > 4 {
                    args.4 = match arguments[4] {
                        Value::Number(n) => n,
                        _ => panic!("Expected number, got {:?}", arguments[4]),
                    };
                }
                if arguments.len() > 5 {
                    args.5 = match arguments[5] {
                        Value::Bool(b) => b,
                        _ => panic!("Expected boolean, got {:?}", arguments[5]),
                    };
                }

                let trigger = GDObj {
                    obj_id: 1346,
                    target: *self,
                    groups: vec![start_group],
                    params: vec![
                        (71, args.0.id.to_string()),
                        (68, ((args.1 - args.1.floor()) * 360.0).to_string()),
                        (69, (args.1.floor()).to_string()),
                        (10, args.2.to_string()),
                        (30, args.3.to_string()),
                        (85, args.4.to_string()),
                        (70, (args.5 as u8).to_string()),
                    ],
                    ..context_trigger(context.clone())
                }
                .context_parameters(context.clone());

                (*globals).obj_list.push(trigger);
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]

pub struct Color {
    pub id: u16,
}

impl Color {
    pub fn native(
        &self,
        name: &String,
        arguments: Vec<Value>,
        context: Context,
        globals: &mut Globals,
        start_group: Group,
    ) -> bool {
        match name.as_str() {
            // group.move(r,g,b,duration,opacity,blending)
            "set" => {
                let mut number_args = vec![0.0, 0.0, 0.0, 0.0, 0.0];
                let mut blending = false;

                for i in 0..arguments.len() {
                    number_args[i] = match arguments[i] {
                        Value::Number(num) => num,
                        _ => panic!("Expected number"),
                    };
                }

                if arguments.len() > 5 {
                    blending = match arguments[5] {
                        Value::Bool(b) => b,
                        _ => panic!("Expected boolean"),
                    };
                }
                let trigger = GDObj {
                    obj_id: 899,
                    target: Group { id: 0 },
                    groups: vec![start_group],
                    params: vec![
                        (7, number_args[0].to_string()),
                        (8, number_args[1].to_string()),
                        (8, number_args[2].to_string()),
                        (10, number_args[3].to_string()),
                        (35, number_args[4].to_string()),
                        (17, (blending as u8).to_string()),
                    ],
                    ..context_trigger(context.clone())
                }
                .context_parameters(context.clone());

                (*globals).obj_list.push(trigger);
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Block {
    pub id: u16,
}

impl Block {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Item {
    pub id: u16,
}

impl Item {
    pub fn native(
        &self,
        name: &String,
        arguments: Vec<Value>,
        context: Context,
        globals: &mut Globals,
        start_group: Group,
    ) -> bool {
        match name.as_str() {
            // group.move(r,g,b,duration,opacity,blending)
            "add" => {
                if arguments.is_empty() {
                    panic!("Expected 1 argument")
                };

                let amount = match arguments[0] {
                    Value::Number(n) => n,
                    _ => panic!("Expected number"),
                };
                let trigger = GDObj {
                    obj_id: 1817,
                    target: Group { id: 0 },
                    groups: vec![start_group],
                    params: vec![(77, amount.to_string()), (80, self.id.to_string())],
                    ..context_trigger(context.clone())
                }
                .context_parameters(context.clone());

                (*globals).obj_list.push(trigger);
                true
            }

            "is" => {
                if arguments.len() < 3 {
                    panic!("Expected 3 arguments")
                };

                let operation = match arguments[0] {
                    Value::Number(n) => n,
                    _ => panic!("Expected number"),
                };

                let amount = match arguments[1] {
                    Value::Number(n) => n,
                    _ => panic!("Expected number"),
                };

                let func = match &arguments[2] {
                    Value::Scope(s) => s.group,
                    _ => panic!("Expected function"),
                };
                let trigger = GDObj {
                    obj_id: 1811,
                    target: func,
                    groups: vec![start_group],
                    params: vec![
                        (77, amount.to_string()),
                        (56, "1".to_string()),
                        (88, operation.to_string()),
                        (80, self.id.to_string()),
                    ],
                    ..context_trigger(context.clone())
                }
                .context_parameters(context.clone());

                (*globals).obj_list.push(trigger);
                true
            }
            _ => false,
        }
    }
}

pub fn context_trigger(context: Context) -> GDObj {
    GDObj {
        obj_id: 0,
        groups: context.added_groups,
        target: Group { id: 0 },
        spawn_triggered: context.spawn_triggered,
        params: Vec::new(),
        x: context.x,
        y: context.y,
    }
}

const TYPE_MEMBER_NAME: &str = "TYPE";
impl Value {
    pub fn member(&self, member: String, globals: &Globals) -> Value {
        let get_imp = |t: String, m: String| match globals.implementations.get(&(t)) {
            Some(imp) => match imp.get(&m) {
                Some(mem) => mem.clone(),
                None => panic!("{} does not have member", t),
            },
            None => panic!("{} does not have member", t),
        };
        let my_type = match self {
            Value::Scope(scope) => match scope.members.get(TYPE_MEMBER_NAME) {
                Some(value) => match (value).clone() {
                    Value::Str(s) => s,
                    _ => unreachable!(),
                },
                None => "function".to_string(),
            },
            Value::Dict(dict) => match dict.get(TYPE_MEMBER_NAME) {
                Some(value) => match (value).clone() {
                    Value::Str(s) => s,
                    _ => unreachable!(),
                },
                None => "dictionary".to_string(),
            },

            Value::Group(_) => "group".to_string(),
            Value::Color(_) => "color".to_string(),
            Value::Block(_) => "block".to_string(),
            Value::Item(_) => "item".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Macro(_) => "macro".to_string(),
            Value::Str(_) => "string".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Obj(_) => "object".to_string(),
            Value::Null => "null".to_string(),
        };
        if member == TYPE_MEMBER_NAME {
            return Value::Str(my_type);
        } else {
            match self {
                Value::Scope(scope) => match scope.members.get(&member) {
                    Some(value) => (value).clone(),
                    None => get_imp(my_type, member),
                },
                Value::Dict(dict) => match dict.get(&member) {
                    Some(value) => (value).clone(),
                    None => get_imp(my_type, member),
                },
                _ => get_imp(my_type, member),
            }
        }
    }
}

/*pub fn event(
    name: &String,
    args: Vec<Value>,
    context: Context,
    globals: &mut Globals,
    start_group: Group,
    activate_group: Group,
) {
    match name.as_ref() {
        "Collide" => {
            let block_a_id = match args[0] {
                Value::Block(b) => b,
                _ => panic!("Expected block, got {:?}", args[0]),
            };

            let block_b_id = match args[1] {
                Value::Block(b) => b,
                _ => panic!("Expected block"),
            };

            let group = activate_group;
            let trigger = GDObj {
                obj_id: 1815,
                groups: vec![start_group],
                target: group,
                params: vec![
                    (80, block_a_id.id.to_string()),
                    (95, block_b_id.id.to_string()),
                    (56, "1".to_string()),
                ],
                ..context_trigger(context.clone())
            }
            .context_parameters(context.clone());

            (*globals).obj_list.push(trigger);
        }
        "Touch" => {
            let group = activate_group;
            let trigger = GDObj {
                obj_id: 1595,
                groups: vec![start_group],
                target: group,
                params: vec![(82, "1".to_string()), (81, "1".to_string())],
                ..context_trigger(context.clone())
            }
            .context_parameters(context.clone());

            (*globals).obj_list.push(trigger);
        }

        "TouchEnd" => {
            let group = activate_group;
            let trigger = GDObj {
                obj_id: 1595,
                groups: vec![start_group],
                target: group,
                params: vec![(82, "2".to_string()), (81, "1".to_string())],
                ..context_trigger(context.clone())
            }
            .context_parameters(context.clone());

            (*globals).obj_list.push(trigger);
        }
        "Count" => {
            let item = match args[0] {
                Value::Item(i) => i,
                _ => panic!("Expected item, got {:?}", args[0]),
            };

            let target = match args[1] {
                Value::Number(n) => n,
                _ => panic!("Expected number, got {:?}", args[0]),
            };

            let group = activate_group;
            let trigger = GDObj {
                obj_id: 1611,
                groups: vec![start_group],
                target: group,
                params: vec![
                    (99, "1".to_string()),
                    (104, "1".to_string()), //multi activate
                    (56, "1".to_string()),  //activate group
                    (77, target.to_string()),
                    (80, item.id.to_string()),
                ],
                ..context_trigger(context.clone())
            }
            .context_parameters(context.clone());

            (*globals).obj_list.push(trigger);
        }
        _ => panic!("The event \"{}\" does not exist!", name),
    }
}

pub fn native_func(
    function: ast::Native,
    context: Context,
    globals: &mut Globals,
    start_group: Group,
) -> bool {
    let mut var = function.function;
    let args = function
        .args
        .iter()
        .map(|x| x.value.eval(&context, globals))
        .collect();

    let func_name: String;

    if var.path.is_empty() {
        func_name = match &var.value {
            ast::ValueLiteral::Symbol(s) => s.clone(),
            _ => panic!("Cannot take value as native function name"),
        }
    } else {
        func_name = match var.path[var.path.len() - 1].clone() {
            ast::Path::Member(m) => m,
            _ => panic!("will deprecate"),
        }
    }

    let mut value = Value::Null;
    if var.path.len() > 0 {
        var.path.pop();
        value = var.to_value(&context, globals);
    }

    match value {
        Value::Group(group) => group.native(&func_name, args, context, globals, start_group),
        Value::Scope(scope) => scope
            .group
            .native(&func_name, args, context, globals, start_group),
        Value::Color(color) => color.native(&func_name, args, context, globals, start_group),
        Value::Item(item) => item.native(&func_name, args, context, globals, start_group),
        Value::Null => {
            // not called on value
            match func_name.as_str() {
                // group.move(r,g,b,duration,opacity,blending)
                "wait" => {
                    if args.len() < 2 {
                        panic!("Expected 2 arguments")
                    };
                    let duration = match args[0] {
                        Value::Number(n) => n,
                        _ => panic!("Expected number"),
                    };
                    let func = match &args[1] {
                        Value::Scope(s) => s.group,
                        _ => panic!("Expected function"),
                    };
                    let trigger = GDObj {
                        obj_id: 1268,
                        target: func,
                        groups: vec![start_group],
                        params: vec![(63, duration.to_string())],
                        ..context_trigger(context.clone())
                    }
                    .context_parameters(context.clone());
                    (*globals).obj_list.push(trigger);
                    true
                }

                "print" => {
                    println!("{:?}", args[0]);
                    true
                }
                _ => false,
            }
        }
        _ => {
            panic!(format!(
                "This value ({:?}) has no native function ascosiated with it!",
                value
            ));
        }
    }
}*/
