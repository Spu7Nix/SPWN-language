//! Tools for compiling SPWN into GD object strings
use crate::levelstring::*;
use crate::native::*;
use crate::ast;
use std::collections::HashMap;


#[derive(Clone)]
pub struct Context {
    pub x: u32,
    pub y: u16,
    pub added_groups: Vec<Group>,
    pub spawn_triggered: bool,
    pub variables: HashMap<String, Value>
}

impl Context {
    pub fn move_down (&self) -> Context {
        Context {
            y: self.y - 30,
            ..self.clone()
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Scope {
    pub start_group: Group,
    pub group: Group,
    pub members: HashMap<String, Value>
}

#[derive(Clone)]
#[derive(Debug)]
pub enum Value {
    Group (Group),
    Color (Color),
    Block (Block),
    Item  (Item),
    Number(f64),
    Bool  (bool),
    Scope (Scope),
    Null
}





pub struct Globals {
    pub closed_groups: Vec<u16>,
    pub closed_colors: Vec<u16>,
    pub closed_blocks: Vec<u16>,
    pub closed_items:  Vec<u16>,

    pub obj_list: Vec<GDTrigger>,

    pub highest_x: u32
}



pub fn compile_spwn (statements: Vec<ast::Statement>) -> Vec<GDTrigger> {

    //context at the start of the program
    let start_context = Context {
        x : 0,
        y : 300,
        added_groups : Vec::new(),
        spawn_triggered : false,
        variables : HashMap::new()
    };

    //variables that get changed throughout the compiling
    let mut globals = Globals {
        closed_groups: Vec::new(),
        closed_colors: Vec::new(),
        closed_blocks: Vec::new(),
        closed_items: Vec::new(),
        obj_list: Vec::new(),

        highest_x: 0
    };
    
    compile_scope(&statements, start_context, Group {id: 0}, &mut globals);

    globals.obj_list

}

pub fn compile_scope (statements: &Vec<ast::Statement>, mut context: Context, start_group: Group, globals: &mut Globals) -> Scope {
    context.x = globals.highest_x;

    (*globals).highest_x += 30;

    let mut context_clone = context.clone();

    let scope_specific_group = match context_clone.added_groups.last(){
        Some(g) => g,
        None => &Group {id: 0}
    };

    

    for statement in statements.iter() {
        //find out what kind of statement this is

        match statement {

            ast::Statement::Definition(def) => {
                //insert the variable into the variable list
                let v = def.value.to_value(&context.move_down(), globals);
                (&mut context).variables.insert(String::from(&def.symbol), v);
            },

            ast::Statement::Event(e) => {

                let func = match e.func.to_value(&context.move_down(), globals) {
                    Value::Scope(s) => s.start_group,
                    Value::Group(g) => g,
                    _ => panic!("Not callable")
                };

                event(&e.symbol, e.args.iter().map(|x| x.to_value(&context, globals)).collect(), context_clone.clone(), globals, start_group, func);
                context_clone.y -= 30;
            },

            ast::Statement::Call(call) => {
                let func = call.function.to_value(&(context.move_down()), globals);
                
                (*globals).obj_list.push(GDTrigger {
                    obj_id: 1268,
                    groups: vec![start_group],
                    target: match func {
                        Value::Scope(s) => s.start_group,
                        Value::Group(g) => g,
                        _ => panic!("Not callable")
                    },
                    
                    ..context_trigger(context_clone.clone())
                }.context_parameters(context_clone.clone()));
                context_clone.y -= 30;
            },

            ast::Statement::Native(call) => {
                native_func(call.clone(), context_clone.clone(), globals, start_group);
                context_clone.y -= 30;
            },

            ast::Statement::EOI => {

            }
        }
    }

    Scope {
        start_group: start_group,
        group: *scope_specific_group,
        members: context.clone().variables
    }
}


impl ast::Variable {
    pub fn to_value(&self, context: &Context, globals: &mut Globals) -> Value {
        let base_value = match &self.value {
            ast::ValueLiteral::ID(id) => {
                match id.class_name.as_ref() {
                    "g" => {
                        if id.unspecified{
                            Value::Group( Group {id: next_free(&mut globals.closed_groups) })
                        } else {
                            Value::Group(Group {id: id.number})
                        }
                    },
                    "c" => {
                        if id.unspecified{
                            Value::Color( Color {id: next_free(&mut globals.closed_colors) })
                        } else {
                            Value::Color(Color {id: id.number})
                        }
                    },
                    "b" => {
                        if id.unspecified{
                            Value::Block( Block {id: next_free(&mut globals.closed_blocks) })
                        } else {
                            Value::Block(Block {id: id.number})
                        }
                    },
                    "i" => {
                        if id.unspecified{
                            Value::Item( Item {id: next_free(&mut globals.closed_items) })
                        } else {
                            Value::Item (Item  {id: id.number})
                        }
                    },
                    _ => unreachable!()
                }
            },
            ast::ValueLiteral::Number(num) =>       Value::Number(*num),
            ast::ValueLiteral::CmpStmt(cmp_stmt) => Value::Scope(cmp_stmt.to_scope(context, globals)),
            ast::ValueLiteral::Bool(b) =>        Value::Bool(*b),
            ast::ValueLiteral::Symbol(string) => {
                match context.variables.get(string){
                    Some (value) => (value).clone(),
                    None => panic!(format!("The variable {} does not exist in this scope.", string))
                }
            }
        };

        let mut final_value = base_value;

        for mem in self.symbols.iter() {
            final_value = member(final_value, mem.clone());
        }

        final_value
    }
}

impl ast::CompoundStatement {
    fn to_scope (&self, context: &Context, globals: &mut Globals) -> Scope {
        //pick a new group for the function
        let group = Group{id: next_free(&mut globals.closed_groups)};
        

        let mut added_groups = context.added_groups.clone();
        added_groups.push(group);

        //create the function context
        let new_context = Context {
            
            spawn_triggered: true,
            added_groups: added_groups,
            variables: context.variables.clone(),
            ..*context
        };

        //pick a start group
        let start_group = Group{id: next_free(&mut globals.closed_groups)};

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

