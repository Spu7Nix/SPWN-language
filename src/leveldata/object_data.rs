use std::{collections::HashMap, fmt};

use crate::interpreter::value::Id;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ObjParam {
    Group(Id),
    Color(Id),
    Block(Id),
    Item(Id),
    Number(f64),
    Bool(bool),
    Text(String),
    GroupList(Vec<Id>),
    Epsilon,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Copy, Hash)]
pub enum ObjectMode {
    Object,
    Trigger,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GdObj {
    pub params: HashMap<u16, ObjParam>,
    pub mode: ObjectMode,
}

impl fmt::Display for ObjParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjParam::Group(id)
            | ObjParam::Color(id)
            | ObjParam::Block(id)
            | ObjParam::Item(id) => match id {
                Id::Specific(id) => write!(f, "{}", id),
                Id::Arbitrary(id) => write!(f, "{}?", id),
            },
            ObjParam::Number(n) => {
                if n.fract().abs() < 0.001 {
                    write!(f, "{}", *n as i32)
                } else {
                    write!(f, "{:.1$}", n, 3)
                }
            }
            ObjParam::Bool(b) => write!(f, "{}", if *b { "1" } else { "0" }),
            ObjParam::Text(t) => write!(f, "{}", t),
            ObjParam::GroupList(list) => {
                let mut out = String::new();

                for g in list {
                    match g {
                        Id::Specific(id) => out += &(id.to_string() + "."),
                        Id::Arbitrary(id) => out += &(id.to_string() + "?."),
                    };
                }
                out.pop();
                write!(f, "{}", out)
            }
            ObjParam::Epsilon => write!(f, "0.05"),
        }
    }
}

pub const SPWN_SIGNATURE_GROUP: Id = Id::Specific(1001);

pub fn serialize_obj(mut trigger: GdObj) -> String {
    let mut obj_string = String::new();
    match trigger.mode {
        ObjectMode::Object => {
            match trigger.params.get_mut(&57) {
                Some(ObjParam::GroupList(l)) => (*l).push(SPWN_SIGNATURE_GROUP),
                Some(ObjParam::Group(g)) => {
                    let group = *g;
                    trigger
                        .params
                        .insert(57, ObjParam::GroupList(vec![group, SPWN_SIGNATURE_GROUP]));
                }
                _ => {
                    trigger
                        .params
                        .insert(57, ObjParam::Group(SPWN_SIGNATURE_GROUP));
                }
            };

            let mut param_list = trigger.params.iter().collect::<Vec<(&u16, &ObjParam)>>();

            param_list.sort_by(|a, b| (*a.0).cmp(b.0));

            for param in param_list {
                obj_string += &format!("{},{},", param.0, param.1);
            }

            obj_string + ";"
        }
        ObjectMode::Trigger => {
            match trigger.params.get_mut(&57) {
                Some(ObjParam::GroupList(l)) => {
                    (*l).push(SPWN_SIGNATURE_GROUP);
                    //list
                }
                Some(ObjParam::Group(g)) => {
                    let group = *g;
                    trigger
                        .params
                        .insert(57, ObjParam::GroupList(vec![group, SPWN_SIGNATURE_GROUP]));
                }
                _ => {
                    trigger
                        .params
                        .insert(57, ObjParam::Group(SPWN_SIGNATURE_GROUP));
                    //Vec::new()
                }
            };

            /*let spawned = match trigger.params.get(&62) {
                Some(ObjParam::Bool(b)) => *b,
                _ => groups.iter().any(|x| x.id != ID::Specific(0)),
            };
            if spawned {
                obj_string += "87,1,";
            }*/

            let mut param_list = trigger.params.iter().collect::<Vec<(&u16, &ObjParam)>>();

            param_list.sort_by(|a, b| (*a.0).cmp(b.0));

            for param in param_list {
                obj_string += &format!("{},{},", param.0, param.1);
            }
            obj_string + "108,1;" //linked group
        }
    }
}
