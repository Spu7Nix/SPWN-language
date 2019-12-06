//! Tools for compiling SPWN into GD object strings
use crate::levelstring::*;
use crate::native::*;
use crate::ast;
use std::collections::HashMap;


#[derive(Clone)]
pub struct Context {
    pub x: u32,
    pub added_groups: Vec<Group>,
    pub spawn_triggered: bool,
    pub variables: HashMap<String, Value>
}

#[derive(Clone)]
pub struct Scope {
    pub group: Group,
    pub members: HashMap<String, Value>
}

#[derive(Clone)]
pub enum Value {
    Group (Group),
    Color (Color),
    Block (Block),
    Item  (Item),
    Number(f64),
    Bool  (bool),
    Scope (Scope)
}





pub struct Globals {
    pub closed_groups: Vec<u16>,
    pub closed_colors: Vec<u16>,
    pub closed_blocks: Vec<u16>,
    pub closed_items:  Vec<u16>,

    pub obj_list: Vec<GDTrigger>
}



pub fn Compile (statements: Vec<ast::Statement>) -> Vec<GDTrigger> {

    //context at the start of the program
    let start_context = Context {
        x : 0,
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
        obj_list: Vec::new()
    };
    
    compile_scope(&statements, start_context, Group {id: 0}, &mut globals);

    globals.obj_list

}

pub fn compile_scope (statements: &Vec<ast::Statement>, mut context: Context, startGroup: Group, globals: &mut Globals) -> Scope {

    

    for statement in statements.iter() {
        //find out what kind of statement this is

        match statement {

            ast::Statement::Definition(def) => {
                //insert the variable into the variable list
                let v = def.value.to_value(&context, globals);
                (&mut context).variables.insert(String::from(&def.symbol), v);
            },

            ast::Statement::Event(e) => {
                let pair = event(&e.symbol, e.args.iter().map(|x| x.to_value(&context, globals)).collect(), context.clone(), globals);

                let scope = compile_scope(&e.cmp_stmt.statements, pair.0, pair.1, globals);
            },

            ast::Statement::Call(call) => {
                let func = call.function.to_value(&(context.clone()), globals);
                
            },

            ast::Statement::Native(func) => {

            },

            ast::Statement::EOI => {

            }
        }
    }

    Scope {
        group: startGroup,
        members: context.clone().variables
    }
}


impl ast::Variable {
    pub fn to_value(&self, context: &Context, globals: &mut Globals) -> Value {
        let baseValue = match &self.value {
            ast::ValueLiteral::ID(ID) => {
                match ID.class_name.as_ref() {
                    "g" => Value::Group(Group {id: ID.number}),
                    "c" => Value::Color(Color {id: ID.number}),
                    "b" => Value::Block(Block {id: ID.number}),
                    "i" => Value::Item (Item  {id: ID.number}),
                    _ => unreachable!()
                }
            },
            ast::ValueLiteral::Number(num) =>       Value::Number(*num),
            ast::ValueLiteral::CmpStmt(cmp_stmt) => Value::Scope(cmp_stmt.to_scope(context, globals)),
            ast::ValueLiteral::Bool(Bool) =>        Value::Bool(*Bool),
            ast::ValueLiteral::Symbol(string) => {
                match context.variables.get(string){
                    Some (value) => (value).clone(),
                    None => panic!(format!("The variable {} does not exist in this scope.", string))
                }
            }
        };

        let mut finalValue = baseValue;

        for mem in self.symbols.iter() {
            finalValue = member(finalValue, mem.clone());
        }

        finalValue
    }
}

impl ast::CompoundStatement {
    fn to_scope (&self, context: &Context, globals: &mut Globals) -> Scope {
        //pick a new group for the function
        let group = Group{id: nextFree(&mut globals.closed_groups)};

        let mut added_groups = context.added_groups.clone();
        added_groups.push(group);

        //create the function context
        let newContext = Context {
            
            spawn_triggered: true,
            added_groups: added_groups,
            variables: context.variables.clone(),
            ..*context
        };

        //pick a start group
        let startGroup = Group{id: nextFree(&mut globals.closed_groups)};

        compile_scope(&self.statements, newContext, startGroup, globals)
    }
}

const ID_MAX: u16 = 999;

pub fn nextFree(ids: &mut Vec<u16>) -> u16 {
    for i in 1..ID_MAX {
        if !ids.contains(&i) {
            (*ids).push(i);
            return i;
        }
    }
    panic!("All ids of this t are used up!");
}

