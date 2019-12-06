//! Defining all native classes and functions
use crate::compiler::*;
use crate::levelstring::*;



#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct Group {
    pub id: u16
}

impl Group {
    
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






pub fn member(value: Value, member: String) -> Value {
    match value {
        Value::Group(group) => {
            
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
        },
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
                _ => panic!("Expected block")
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
                params: vec![(80, block_A_ID.id.to_string()), (95, block_B_ID.id.to_string())],
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

// pub fn check_type(values: Vec<Value>, correct: Vec<String>) -> Result<(), String> {
//     let arg_length = values.len();
//     let correct_length = correct.len();


//     if arg_length != correct_length {
//         return Err (format!("Expected {} values, recieved {}.", correct_length, arg_length));
//     }

//     for i in 0..arg_length {
//         if values[i].t != correct[i] {
//             return Err (format!("Expected value {} at position {}, got {}.", correct[i], i, values[i].t));
//         }
//     }

//     return Ok(());
// }