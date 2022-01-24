use crate::builtins::{Block, Group, Id, Item};
use crate::{builtins::Color, leveldata::ObjParam, value::Value};
use errors::RuntimeError;
use parser::ast::ObjectMode;

pub fn parse_levelstring(ls: &str) -> Result<Vec<Value>, RuntimeError> {
    let mut obj_strings = ls.split(';');
    obj_strings.next(); // skip the header
    let mut objs = Vec::new();
    for obj_string in obj_strings {
        if obj_string.is_empty() {
            continue;
        }
        let key_val = obj_string.split(',').collect::<Vec<&str>>();
        let mut group_51 = false;

        {
            let mut key_val_iter = key_val.iter();
            while let Some(key) = key_val_iter.next() {
                let val = key_val_iter.next().unwrap();
                if *key == "52" && *val == "1" {
                    group_51 = true;
                }
            }
        }

        let mut obj = Vec::new();
        let mut obj_id = 0;

        let mut key_val_iter = key_val.iter();

        while let Some(key) = key_val_iter.next() {
            let key = key.parse::<u16>().unwrap();
            let val = key_val_iter.next().unwrap();

            let prop = match key {
                1 => {
                    obj_id = val.parse::<u16>().unwrap();
                    ObjParam::Number(obj_id as f64)
                }
                4 | 5 | 11 | 13 | 15 | 16 | 17 | 34 | 41 | 42 | 48 | 56 | 58 | 59 | 60 | 62
                | 64 | 65 | 66 | 67 | 70 | 81 | 86 | 87 | 89 | 93 | 94 | 96 | 98 | 104 | 100
                | 102 | 103 | 106 | 36 => ObjParam::Bool(val.trim() == "1"),
                21 | 22 | 23 | 50 => ObjParam::Color(Color {
                    id: Id::Specific(val.parse::<u16>().unwrap()),
                }),
                31 | 43 | 44 | 49 => ObjParam::Text(val.to_string()),
                71 => ObjParam::Group(Group {
                    id: Id::Specific(val.parse::<u16>().unwrap()),
                }),
                95 => ObjParam::Block(Block {
                    id: Id::Specific(val.parse::<u16>().unwrap()),
                }),

                57 => ObjParam::GroupList(
                    val.split('.')
                        .map(|g| Group {
                            id: Id::Specific(g.parse::<u16>().unwrap()),
                        })
                        .collect::<Vec<_>>(),
                ),
                80 => match obj_id {
                    1815 => ObjParam::Block(Block {
                        id: Id::Specific(val.parse::<u16>().unwrap()),
                    }),
                    _ => ObjParam::Item(Item {
                        id: Id::Specific(val.parse::<u16>().unwrap()),
                    }),
                },
                51 => match obj_id {
                    1006 => {
                        if group_51 {
                            ObjParam::Group(Group {
                                id: Id::Specific(val.parse::<u16>().unwrap()),
                            })
                        } else {
                            ObjParam::Color(Color {
                                id: Id::Specific(val.parse::<u16>().unwrap()),
                            })
                        }
                    }
                    899 => ObjParam::Color(Color {
                        id: Id::Specific(val.parse::<u16>().unwrap()),
                    }),
                    _ => ObjParam::Group(Group {
                        id: Id::Specific(val.parse::<u16>().unwrap()),
                    }),
                },
                _ => ObjParam::Number(val.parse::<f64>().unwrap()),
            };
            obj.push((key, prop));
        }
        objs.push(Value::Obj(obj, ObjectMode::Object));
    }
    Ok(objs)
}
