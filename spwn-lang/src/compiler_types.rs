use crate::ast;
use crate::levelstring::*;
use crate::native::*;
use std::boxed::Box;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::compiler::{compile_scope, import_module, next_free};

#[derive(Clone, Debug, PartialEq)]
pub struct Context {
    pub x: u32,
    pub y: u16,
    pub added_groups: Vec<Group>,
    pub spawn_triggered: bool,
    pub variables: HashMap<String, Value>,
    pub ret: Option<Box<Return>>,
}

impl Context {
    pub fn move_down(&self) -> Context {
        Context {
            y: self.y - 30,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Return {
    pub context: Context,
    pub statements: Vec<ast::Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub group: Group,
    pub members: HashMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Macro {
    pub args: Vec<(String, Option<Value>)>,
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
    Dict(HashMap<String, Value>),
    Macro(Macro),
    Str(String),
    Array(Vec<Value>),
    Obj(Vec<(u16, String)>),
    Null,
}

impl Value {
    pub fn to_variable(&self) -> ast::Variable {
        ast::Variable {
            value: ast::ValueLiteral::Resolved(self.clone()),
            path: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct Globals {
    pub closed_groups: Vec<u16>,
    pub closed_colors: Vec<u16>,
    pub closed_blocks: Vec<u16>,
    pub closed_items: Vec<u16>,
    pub path: PathBuf,
    pub obj_list: Vec<GDObj>,

    pub highest_x: u32,

    pub implementations: HashMap<String, HashMap<String, Value>>,
}

#[derive(Debug)]
pub enum ValSuccess {
    Literal(Value),
    Evaluatable(
        (
            Macro,                        //macro called
            Vec<(Option<String>, Value)>, //macro arguments
            ast::Variable,                //variable with PLACEHOLDER somewhere in it
        ),
    ),
}

use ValSuccess::{Evaluatable, Literal};

impl ast::Expression {
    pub fn eval(
        &self,
        context: &Context,
        globals: &mut Globals,
        placeholder_value: &Value,
    ) -> ValSuccess {
        let mut vals = self.values.iter();
        let mut acum = match vals
            .next()
            .unwrap()
            .to_value(context, globals, placeholder_value)
        {
            Literal(l) => l,
            Evaluatable(e) => {
                let mut return_expr = self.clone();
                return_expr.values[0] = e.2;
                return Evaluatable((
                    e.0,
                    e.1,
                    ast::Variable {
                        value: ast::ValueLiteral::Expression(return_expr),
                        path: Vec::new(),
                    },
                ));
            }
        };

        if self.operators.is_empty() {
            return Literal(acum);
        }

        for (i, var) in vals.enumerate() {
            let val = match var.to_value(context, globals, placeholder_value) {
                Literal(l) => l,
                Evaluatable(e) => {
                    let mut return_expr = self.clone();
                    return_expr.values[i] = e.2;
                    return Evaluatable((
                        e.0,
                        e.1,
                        ast::Variable {
                            value: ast::ValueLiteral::Expression(return_expr),
                            path: Vec::new(),
                        },
                    ));
                }
            };

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

        Literal(acum)
    }
}

pub fn evaluate_and_execute(
    (m, args): (Macro, Vec<(Option<String>, Value)>),
    context: &Context,
    globals: &mut Globals,
    statements: Vec<ast::Statement>, // with PLACEHOLDER in it
    start_group: Group,
) -> Scope {
    let mut new_context = context.clone();
    new_context.variables = m.def_context.variables;
    new_context.ret = Some(Box::new(Return {
        statements: statements.clone(),
        context: context.clone(),
    }));

    let mut new_variables: HashMap<String, Value> = HashMap::new();

    //parse each argument given into a local macro variable
    for (i, arg) in args.iter().enumerate() {
        match &arg.0 {
            Some(name) => {
                if m.args.iter().any(|e| e.0 == *name) {
                    new_variables.insert(name.clone(), arg.1.clone());
                } else {
                    panic!("This function has no argument with this name!")
                }
            }
            None => {
                new_variables.insert(m.args[i].0.clone(), arg.1.clone());
            }
        }
    }
    //insert defaults and check non-optional arguments
    for arg in m.args.iter() {
        if !new_variables.contains_key(&arg.0) {
            match &arg.1 {
                Some(default) => {
                    new_variables.insert(arg.0.clone(), default.clone());
                }

                None => panic!("Non-optional argument '{}' not satisfied!", arg.0),
            }
        }
    }

    new_context.variables.extend(new_variables);
    let ret_body = m.body.clone();
    new_context.y -= 30;
    let scope = compile_scope(&ret_body, new_context, start_group, globals, &Value::Null);

    return scope;
}

impl ast::Variable {
    pub fn to_value(
        &self,
        context: &Context,
        globals: &mut Globals,
        placeholder_value: &Value,
    ) -> ValSuccess {
        // TODO: Check if this variable has native functions called on it, and if not set this to false

        let mut final_value = match &self.value {
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
            ast::ValueLiteral::Dictionary(dict) => Value::Dict(dict.read(&context, globals)),
            ast::ValueLiteral::CmpStmt(cmp_stmt) => {
                Value::Scope(cmp_stmt.to_scope(context, globals))
            }

            ast::ValueLiteral::Expression(expr) => {
                match expr.eval(&context, globals, placeholder_value) {
                    Literal(l) => l,
                    Evaluatable(e) => {
                        return Evaluatable((e.0, e.1, {
                            let mut placeholder = e.2;
                            placeholder.path = self.path.clone(); //expr.eval never returns a variable with a path, so I can just replace this
                            placeholder
                        }));
                    }
                }
            }

            ast::ValueLiteral::Bool(b) => Value::Bool(*b),
            ast::ValueLiteral::Symbol(string) => match context.variables.get(string) {
                Some(value) => (value).clone(),
                None => panic!(format!(
                    "The variable {} does not exist in this scope.",
                    string
                )),
            },
            ast::ValueLiteral::Str(s) => Value::Str(s.clone()),
            ast::ValueLiteral::Array(a) => {
                let mut arr: Vec<Value> = Vec::new();
                let mut a_iter = a.iter();
                for val in &mut a_iter {
                    arr.push(match val.eval(&context, globals, placeholder_value) {
                        Literal(l) => l,
                        Evaluatable(e) => {
                            return Evaluatable((e.0, e.1, {
                                let mut new_arr: Vec<ast::Expression> = arr
                                    .iter()
                                    .map(|x| x.to_variable().to_expression())
                                    .collect();
                                new_arr.push(e.2.to_expression());
                                new_arr.extend(a_iter.cloned());
                                ast::Variable {
                                    value: ast::ValueLiteral::Array(new_arr),
                                    path: Vec::new(),
                                }
                            }))
                        }
                    });
                }
                Value::Array(arr)
            }
            ast::ValueLiteral::Import(i) => import_module(i, globals),
            ast::ValueLiteral::Obj(o) => Value::Obj({
                let mut obj: Vec<(u16, String)> = Vec::new();
                let mut o_iter = o.iter();
                for prop in &mut o_iter {
                    obj.push((
                        match prop.0.eval(&context, globals, placeholder_value) {
                            Literal(Value::Number(n)) => n as u16,
                            Evaluatable(e) => {
                                return Evaluatable((e.0, e.1, {
                                    let mut new_obj: Vec<(ast::Expression, ast::Expression)> = obj
                                        .clone()
                                        .iter()
                                        .map(|x| {
                                            (
                                                Value::Number(x.0 as f64)
                                                    .to_variable()
                                                    .to_expression(),
                                                Value::Str(x.1.clone())
                                                    .to_variable()
                                                    .to_expression(),
                                            )
                                        })
                                        .collect();
                                    new_obj.push((e.2.to_expression(), prop.1.clone()));
                                    new_obj.extend(o_iter.cloned());
                                    ast::ValueLiteral::Obj(new_obj).to_variable()
                                }))
                            }
                            _ => panic!("Expected number as object property"),
                        },
                        match prop.1.eval(&context, globals, placeholder_value) {
                            Literal(Value::Number(n)) => n.to_string(),
                            Literal(Value::Str(s)) => s,
                            Literal(Value::Scope(s)) => s.group.id.to_string(),

                            Literal(Value::Group(g)) => g.id.to_string(),
                            Literal(Value::Color(c)) => c.id.to_string(),
                            Literal(Value::Block(b)) => b.id.to_string(),
                            Literal(Value::Item(i)) => i.id.to_string(),

                            Evaluatable(e) => {
                                //literally just copy paste of the above because im lazy
                                return Evaluatable((e.0, e.1, {
                                    let mut new_obj: Vec<(ast::Expression, ast::Expression)> = obj
                                        .clone()
                                        .iter()
                                        .map(|x| {
                                            (
                                                Value::Number(x.0 as f64)
                                                    .to_variable()
                                                    .to_expression(),
                                                Value::Str(x.1.clone())
                                                    .to_variable()
                                                    .to_expression(),
                                            )
                                        })
                                        .collect();
                                    //since the previous value has already been determined, this can be improved by storing the previous
                                    //in a var and replacing it with the prop.0
                                    //          This 🠟
                                    new_obj.push((prop.0.clone(), e.2.to_expression()));
                                    new_obj.extend(o_iter.cloned());
                                    ast::ValueLiteral::Obj(new_obj).to_variable()
                                }));
                            }
                            //Value::Array(a) => {} TODO: Add this
                            x => panic!("{:?} is not a valid object value", x),
                        },
                    ))
                }
                obj
            }),

            ast::ValueLiteral::Macro(m) => Value::Macro(Macro {
                args: {
                    let mut args: Vec<(String, Option<Value>)> = Vec::new();
                    for arg in m.args.iter() {
                        args.push((
                            arg.0.clone(),
                            match &arg.1 {
                                Some(expr) => {
                                    Some(match expr.eval(&context, globals, placeholder_value) {
                                        Literal(l) => l,
                                        Evaluatable(e) => return Evaluatable(e),
                                    })
                                }
                                None => None,
                            },
                        ));
                    }

                    args
                },
                def_context: context.clone(),
                body: m.body.statements.clone(),
            }),

            ast::ValueLiteral::Resolved(r) => r.clone(),

            ast::ValueLiteral::Null => Value::Null,
            ast::ValueLiteral::PLACEHOLDER => placeholder_value.clone(),
        };
        let mut path_iter = self.path.iter();
        let mut parent = Value::Null;
        for p in &mut path_iter {
            let stored = final_value.clone();
            match p {
                ast::Path::Member(m) => final_value = final_value.member(m.clone(), globals),

                ast::Path::Index(i) => {
                    final_value = match &final_value {
                        Value::Array(a) => a[match i.eval(context, globals, placeholder_value) {
                            Literal(l) => match l {
                                Value::Number(n) => n,
                                _ => panic!("Expected number"),
                            },

                            Evaluatable(e) => {
                                let mut path: Vec<ast::Path> =
                                    vec![ast::Path::Index(e.2.to_expression())];

                                path.extend(path_iter.cloned());

                                return Evaluatable((
                                    e.0,
                                    e.1,
                                    ast::Variable {
                                        value: ast::ValueLiteral::Resolved(final_value),
                                        path,
                                    },
                                ));
                            }
                        } as usize]
                            .clone(),
                        _ => panic!("Can't index non-iteratable variable"),
                    }
                }

                ast::Path::Call(c) => {
                    let m = match final_value.clone() {
                        Value::Macro(m) => m,
                        _ => panic!("Not a macro!"),
                    };
                    return Evaluatable((
                        m.clone(),
                        {
                            let mut vals: Vec<(Option<String>, Value)> = Vec::new();
                            let mut c_iter = c.iter();
                            if !m.args.is_empty() && m.args[0].0 == "self" {
                                vals.push((None, parent))
                            }
                            for arg in &mut c_iter {
                                let arg_value =
                                    match arg.value.eval(&context, globals, placeholder_value) {
                                        Literal(l) => (arg.symbol.clone(), l),
                                        Evaluatable(e) => {
                                            let mut path: Vec<ast::Path> = vec![ast::Path::Call(
                                                [
                                                    vals.iter()
                                                        .map(|x| ast::Argument {
                                                            symbol: x.0.clone(),
                                                            value: x
                                                                .1
                                                                .to_variable()
                                                                .to_expression(),
                                                        })
                                                        .collect(),
                                                    vec![ast::Argument {
                                                        value: e.2.to_expression(),
                                                        ..arg.clone()
                                                    }],
                                                    c_iter.cloned().collect(),
                                                ]
                                                .concat(),
                                            )];

                                            path.extend(path_iter.cloned());

                                            return Evaluatable((
                                                e.0,
                                                e.1,
                                                ast::Variable {
                                                    value: ast::ValueLiteral::Resolved(final_value),
                                                    path,
                                                },
                                            ));
                                        }
                                    };

                                vals.push(arg_value);
                            }
                            vals
                        },
                        ast::Variable {
                            value: ast::ValueLiteral::PLACEHOLDER,
                            path: path_iter.map(|x| x.clone()).collect(),
                        },
                    ));
                }
            }
            parent = stored;
        }

        Literal(final_value)
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

        compile_scope(
            &self.statements,
            new_context,
            start_group,
            globals,
            &Value::Null,
        )
    }
}

impl ast::Dictionary {
    fn read(&self, context: &Context, globals: &mut Globals) -> HashMap<String, Value> {
        compile_scope(
            &self.members,
            context.clone(),
            Group { id: 0 },
            globals,
            &Value::Null,
        )
        .members
    }
}
