//! Defining all native classes and functions
use crate::compiler::*;
use crate::levelstring::*;

use crate::ast;


#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct Group {
    pub id: u16
}

impl Group {
    pub fn native(&self, name:  &String, arguments: Vec<Value>, context: Context, globals: &mut Globals) -> Context {
        match name.as_str() {

            
            // group.move(x, y, time, ease_type, ease_value)
            "move" => {
                let mut args = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

                for i in 0..arguments.len() {
                    args[i] = match arguments[i] {
                        Value::Number(num) => num,
                        _ => panic!("Expected number")
                    };
                }
                
                let trigger = GDTrigger {
                    objID: 901,
                    target: *self,
                    params: vec![(28, (args[0] * 3.0).to_string()),
                                 (29, (args[1] * 3.0).to_string()), 
                                 (10, args[2].to_string()), 
                                 (30, args[3].to_string()), 
                                 (85, args[4].to_string())],
                    ..context_trigger(context.clone())
                };

                (*globals).obj_list.push(trigger);
                context
            },

            _ => {
                panic!("Group has no native function with this name");
            }

        }
    }
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct Color {
    pub id: u16
}

impl Color {
    
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct Block {
    pub id: u16
}

impl Block {
    
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct Item {
    pub id: u16
}

impl Item {
    
}


pub fn context_trigger (context: Context) -> GDTrigger {
    GDTrigger {
        objID: 0,
        groups: context.added_groups,
        target: Group {id: 0},
        spawnTriggered: context.spawn_triggered,
        params: Vec::new(),
        x: context.x,
        y: context.y
    }
}





pub fn member(value: Value, member: String) -> Value {
    match value {
        /*Value::Group(group) => {
            
            Value::Number(20.0)
            
        },
        Value::Color(color) => {
            Value::Number(20.0)
        },
        Value::Block(block) => {
            Value::Number(20.0)
        },
        Value::Item(item) => {
            Value::Number(20.0)
        },*/
        Value::Scope(scope) => {
            match scope.members.get(&member){
                Some (value) => (value).clone(),
                None => panic!("Variable does not have member")
            }
        },

        _ => {
            panic!("Object does not have member!")
        }
        
    }
}

pub fn event(name: &String, args: Vec<Value>, context: Context, globals: &mut Globals) -> (Context, Group){
    match name.as_ref() {
        "Collide" => {
            let block_A_ID = match args[0] {
                Value::Block(Block) => Block,
                _ => panic!("Expected block, got {:?}", args[0])
            };

            let block_B_ID = match args[1] {
                Value::Block(Block) => Block,
                _ => panic!("Expected block")
            };

            let mut ag = context.added_groups;

            let group = Group {id: nextFree(&mut globals.closed_groups)};
            let trigger = GDTrigger {
                objID: 1815,
                groups: ag.clone(),
                target: group,
                spawnTriggered: context.spawn_triggered,
                params: vec![(80, block_A_ID.id.to_string()), (95, block_B_ID.id.to_string()), (56, "1".to_string())],
                x: context.x,
                y: 0
            };

            ag.push(group);
            
            (*globals).obj_list.push(trigger);

            (Context {
                spawn_triggered: true,
                added_groups: ag,
                ..context
            }, group)


        },
        _ => {
            panic!("This event does not exist!")
        }
    }
}

pub fn nativeFunc (mut function: ast::Native, context: Context, globals: &mut Globals) -> Context {
    let mut var = function.function;
    let args = function.args.iter().map(|x| x.to_value(&context, globals)).collect();

    let funcName = var.symbols[var.symbols.len() - 1].clone();
    let mut value = Value::Null;
    
    if var.symbols.len() > 0 {
        var.symbols.pop();
        value = var.to_value(&context, globals);
    }

    match value {
        Value::Group(group) => group.native(&funcName, args, context, globals),
        /*Value::Color(color) => {

        },
        Value::Block(block) => {

        },
        Value::Item(item) => {

        },
        Value::Null => {
            // not called on value
        },*/
        _ => {
            panic!(format!("This value ({:?}) has no native function ascosiated with it!", value));
        }
    }
}