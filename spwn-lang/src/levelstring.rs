// useful things for dealing with gd level data
use crate::native::*;

use crate::compiler::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct GDTrigger {
    pub objID: u16,
    pub groups: Vec<Group>,
    pub target: Group,
    pub spawnTriggered: bool,
    pub x: u32,
    pub y: u16,
    pub params: Vec<(u16, String)>
}

impl GDTrigger {
    fn context_parameters(&mut self, context: Context) {
        self.x = context.x;
        for g in context.added_groups.iter(){
            self.groups.push(*g);
        }
        self.spawnTriggered = context.spawn_triggered;
        
    }
}

pub fn serialize_trigger(trigger: GDTrigger) -> String {
    fn group_string(list: Vec<Group>) -> String{
        let mut string = String::new();
        for group in list.iter() {
            string += &(group.id.to_string() + ".");
        }
        string.pop();
        string
    }

    let mut obj_string = format!("1,{},2,{},3,{},51,{}",

    trigger.objID, trigger.x, trigger.y, trigger.target.id);

    if trigger.spawnTriggered {
        obj_string += ",62,1,87,1";
    }

    if !trigger.groups.is_empty() {
        obj_string += &(String::from(",57,") + &group_string(trigger.groups));
    }

    for param in trigger.params {
        obj_string += &(String::from(",") + &param.0.to_string() + "," + &param.1);
    }

    obj_string + ";"
}