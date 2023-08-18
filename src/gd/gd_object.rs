use std::fmt;
use std::hash::Hash;

use ahash::{AHashMap, AHashSet};

use super::error::LevelError;
use super::ids::*;
use crate::interpreting::vm::Vm;
use crate::parsing::ast::ObjectType;

#[derive(Clone, Copy, Debug, Default)]
pub struct TriggerOrder(pub f64);

impl TriggerOrder {
    pub fn new() -> Self {
        Self(0.0)
    }

    pub fn next(&mut self) -> Self {
        self.0 += 1.0;
        *self
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ObjParam {
    Group(Id),
    Channel(Id),
    Block(Id),
    Item(Id),
    Number(f64),
    Bool(bool),
    Text(String),
    GroupList(Vec<Id>),
    Epsilon,
}

impl Hash for ObjParam {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            ObjParam::Group(id)
            | ObjParam::Channel(id)
            | ObjParam::Block(id)
            | ObjParam::Item(id) => id.hash(state),
            ObjParam::Number(n) => n.to_bits().hash(state),
            ObjParam::Bool(b) => b.hash(state),
            ObjParam::Text(t) => t.hash(state),
            ObjParam::GroupList(list) => list.hash(state),
            ObjParam::Epsilon => (),
        }
    }
}

impl fmt::Display for ObjParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjParam::Group(id)
            | ObjParam::Channel(id)
            | ObjParam::Block(id)
            | ObjParam::Item(id) => match id {
                Id::Specific(id) => write!(f, "{id}"),
                _ => write!(f, "0"),
            },
            ObjParam::Number(n) => {
                if n.fract().abs() < 0.001 {
                    write!(f, "{}", *n as i32)
                } else {
                    write!(f, "{:.1$}", n, 3)
                }
            },
            ObjParam::Bool(b) => write!(f, "{}", if *b { "1" } else { "0" }),
            ObjParam::Text(t) => write!(f, "{t}"),
            ObjParam::GroupList(list) => {
                let mut out = String::new();

                for g in list {
                    if let Id::Specific(id) = g {
                        out += &(id.to_string() + ".")
                    } else {
                        out += "0."
                    };
                }
                out.pop();
                write!(f, "{out}")
            },
            ObjParam::Epsilon => write!(f, "0.05"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct GdObject {
    pub params: AHashMap<u8, ObjParam>,
    pub mode: ObjectType,
}

impl Hash for GdObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut params = self.params.iter().collect::<Vec<_>>();
        params.sort_by_key(|(k, _)| *k);
        params.hash(state);
        self.mode.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct TriggerObject {
    pub obj: GdObject,
    pub order: TriggerOrder,
}

impl TriggerObject {
    pub fn apply_context(mut self, context: Id) -> Self {
        self.obj.params.insert(57, ObjParam::Group(context));
        self
    }

    pub fn params(&self) -> &AHashMap<u8, ObjParam> {
        &self.obj.params
    }

    pub fn params_mut(&mut self) -> &mut AHashMap<u8, ObjParam> {
        &mut self.obj.params
    }
}

pub fn make_spawn_trigger(context: Id, target: Id, vm: &mut Vm) -> TriggerObject {
    let mut obj = GdObject {
        params: AHashMap::default(),
        mode: ObjectType::Trigger,
    };

    obj.params.insert(1, ObjParam::Number(1268.0));
    //obj.params.insert(63, ObjParam::Number(0.0));
    obj.params.insert(51, ObjParam::Group(target));

    TriggerObject {
        obj,
        order: vm.trigger_order_count.next(),
    }
    .apply_context(context)
}

pub fn get_used_ids(ls: &str) -> [AHashSet<u16>; 4] {
    let mut out = [
        AHashSet::<u16>::default(),
        AHashSet::<u16>::default(),
        AHashSet::<u16>::default(),
        AHashSet::<u16>::default(),
    ];

    let objects = ls.split(';');

    for obj in objects {
        let props: Vec<&str> = obj.split(',').collect();
        let mut map = AHashMap::default();

        for i in (0..props.len() - 1).step_by(2) {
            map.insert(props[i], props[i + 1]);
        }

        for (key, value) in &map {
            match *key {
                // groups
                "57" => {
                    let groups = value.split('.');
                    for g in groups {
                        let group = g.parse().unwrap();

                        out[0].insert(group);
                    }
                },
                // target ids
                "51" => {
                    match (map.get("1"), map.get("52")) {
                        (Some(&"1006"), Some(&"1")) => out[0].insert(value.parse().unwrap()),
                        (Some(&"1006"), _) => out[1].insert(value.parse().unwrap()),
                        _ => out[0].insert(value.parse().unwrap()),
                    };
                },
                // target position, follow, center
                "71" => {
                    out[0].insert(value.parse().unwrap());
                },
                // channels
                "21" | "22" | "23" => {
                    out[1].insert(value.parse().unwrap());
                },

                "80" => {
                    match map.get("1") {
                        //if collision trigger or block, add block id
                        Some(&"1815") | Some(&"1816") => out[2].insert(value.parse().unwrap()),
                        //counter display => do nothing
                        Some(&"1615") => false,
                        // else add item id
                        _ => out[3].insert(value.parse().unwrap()),
                    };
                },

                "95" => {
                    out[2].insert(value.parse().unwrap());
                },
                //some of these depends on what object it is
                //pulse target depends on group mode/color mode
                //figure this out, future me
                _ => (),
            }
        }
    }
    out
}

pub const SPWN_SIGNATURE_GROUP: u16 = 1001;
pub const SPWN_SIGNATURE_GROUP_ID: Id = Id::Specific(SPWN_SIGNATURE_GROUP);

pub fn remove_spwn_objects(file_content: &mut String) {
    let spwn_group = match SPWN_SIGNATURE_GROUP_ID {
        Id::Specific(n) => n.to_string(),
        _ => unreachable!(),
    };
    (*file_content) = file_content
        //remove previous spwn objects
        .split(';')
        .map(|obj| {
            let key_val: Vec<&str> = obj.split(',').collect();
            let mut ret = obj;
            for i in (0..key_val.len()).step_by(2) {
                if key_val[i] == "57" {
                    let mut groups = key_val[i + 1].split('.');
                    if groups.any(|x| x == spwn_group) {
                        ret = "";
                    }
                }
            }
            ret
        })
        .collect::<Vec<&str>>()
        .join(";");
}

pub fn append_objects(
    mut objects: Vec<GdObject>,
    old_ls: &str,
) -> Result<(String, [usize; 4]), LevelError> {
    let mut closed_ids = get_used_ids(old_ls);

    //collect all specific ids mentioned into closed_[id] lists
    for obj in &objects {
        for prop in obj.params.values() {
            let class_index;
            let id;
            match prop {
                ObjParam::Group(g) => {
                    class_index = 0;
                    id = vec![g];
                },

                ObjParam::GroupList(l) => {
                    class_index = 0;

                    id = l.iter().collect();
                },
                ObjParam::Channel(g) => {
                    class_index = 1;
                    id = vec![g];
                },
                ObjParam::Block(g) => {
                    class_index = 2;
                    id = vec![g];
                },
                ObjParam::Item(g) => {
                    class_index = 3;
                    id = vec![g];
                },
                _ => continue,
            }
            for id in id {
                match id {
                    Id::Specific(i) => {
                        closed_ids[class_index].insert(*i);
                    },
                    _ => continue,
                }
            }
        }
    }

    //find new ids for all the arbitrary ones
    let mut id_maps: [AHashMap<ArbitraryId, SpecificId>; 4] = [
        AHashMap::default(),
        AHashMap::default(),
        AHashMap::default(),
        AHashMap::default(),
    ];

    const ID_MAX: u16 = 999;

    for obj in &mut objects {
        for prop in obj.params.values_mut() {
            let class_index;
            let ids: Vec<&mut Id>;
            match prop {
                ObjParam::Group(g) => {
                    class_index = 0;
                    ids = vec![g];
                },
                ObjParam::GroupList(g) => {
                    class_index = 0;
                    ids = g.iter_mut().collect();
                },
                ObjParam::Channel(g) => {
                    class_index = 1;
                    ids = vec![g];
                },
                ObjParam::Block(g) => {
                    class_index = 2;
                    ids = vec![g];
                },
                ObjParam::Item(g) => {
                    class_index = 3;
                    ids = vec![g];
                },
                _ => continue,
            }
            for id in ids {
                match &id {
                    Id::Arbitrary(i) => {
                        *id = Id::Specific(match id_maps[class_index].get(i) {
                            Some(a) => *a,
                            None => {
                                let mut out = None;
                                for i in 1..10000 {
                                    if !closed_ids[class_index].contains(&i) {
                                        out = Some(i);
                                        closed_ids[class_index].insert(i);
                                        break;
                                    }
                                }
                                if let Some(id) = out {
                                    id_maps[class_index].insert(*i, id);
                                    id
                                } else {
                                    return Err(LevelError::ExceedsIDLimit(
                                        ["group", "color", "block ID", "item ID"][class_index],
                                    ));
                                }
                            },
                        })
                    },
                    _ => continue,
                }
            }
        }
    }
    for (i, list) in closed_ids.iter_mut().enumerate() {
        list.remove(&0);
        if list.len() > ID_MAX as usize {
            return Err(LevelError::ExceedsIDLimitByAmount {
                id: ["group", "color", "block ID", "item ID"][i],
                max: ID_MAX,
                amount: list.len(),
            });
        }
    }

    //println!("group_map: {:?}", id_maps[0]);

    fn serialize_obj(mut trigger: GdObject) -> String {
        let mut obj_string = String::new();
        match trigger.mode {
            ObjectType::Object => {
                match trigger.params.get_mut(&57) {
                    Some(ObjParam::GroupList(l)) => (*l).push(SPWN_SIGNATURE_GROUP_ID),
                    Some(ObjParam::Group(g)) => {
                        let group = *g;
                        trigger.params.insert(
                            57,
                            ObjParam::GroupList(vec![group, SPWN_SIGNATURE_GROUP_ID]),
                        );
                    },
                    _ => {
                        trigger
                            .params
                            .insert(57, ObjParam::Group(SPWN_SIGNATURE_GROUP_ID));
                    },
                };

                let mut param_list = trigger.params.iter().collect::<Vec<(&u8, &ObjParam)>>();

                param_list.sort_by(|a, b| (*a.0).cmp(b.0));

                for param in param_list {
                    obj_string += &format!("{},{},", param.0, param.1);
                }

                obj_string + ";"
            },
            ObjectType::Trigger => {
                match trigger.params.get_mut(&57) {
                    Some(ObjParam::GroupList(l)) => {
                        (*l).push(SPWN_SIGNATURE_GROUP_ID);
                        //list
                    },
                    Some(ObjParam::Group(g)) => {
                        let group = *g;
                        trigger.params.insert(
                            57,
                            ObjParam::GroupList(vec![group, SPWN_SIGNATURE_GROUP_ID]),
                        );
                    },
                    _ => {
                        trigger
                            .params
                            .insert(57, ObjParam::Group(SPWN_SIGNATURE_GROUP_ID));
                        //Vec::new()
                    },
                };

                /*let spawned = match trigger.params.get(&62) {
                    Some(ObjParam::Bool(b)) => *b,
                    _ => groups.iter().any(|x| x != ID::Specific(0)),
                };
                if spawned {
                    obj_string += "87,1,";
                }*/

                let mut param_list = trigger.params.iter().collect::<Vec<(&u8, &ObjParam)>>();

                param_list.sort_by(|a, b| (*a.0).cmp(b.0));

                for param in param_list {
                    obj_string += &format!("{},{},", param.0, param.1);
                }
                obj_string + "108,1;" //linked group
            },
        }
    }

    let mut full_obj_string = String::new();

    for obj in objects {
        full_obj_string += &serialize_obj(obj)
    }
    Ok((
        full_obj_string,
        [
            closed_ids[0].len(),
            closed_ids[1].len(),
            closed_ids[2].len(),
            closed_ids[3].len(),
        ],
    ))
}

const START_HEIGHT: u16 = 10;
const MAX_HEIGHT: u16 = 40;

const DELTA_X: u16 = 1;

pub fn apply_triggers(mut triggers: Vec<TriggerObject>) -> Vec<GdObject> {
    //println!("{:?}", trigger);

    let mut full_obj_list = Vec::<GdObject>::new();

    //add top layer
    let possible_height = MAX_HEIGHT - START_HEIGHT; //30 is max (TODO: case for if y_offset is more than 30)
    triggers.sort_by(|x, y| x.order.0.partial_cmp(&y.order.0).unwrap());

    for (i, TriggerObject { obj, .. }) in triggers.iter().enumerate() {
        match obj.mode {
            ObjectType::Object => {
                full_obj_list.push(obj.clone());
            },
            ObjectType::Trigger => {
                let y_pos = (i as u16) % possible_height + START_HEIGHT;
                let x_pos = 0;

                let spawned = match obj.params.get(&62) {
                    Some(ObjParam::Bool(b)) => *b,
                    _ => match obj.params.get(&57) {
                        None => false,
                        // Some(ObjParam::GroupList(l)) => {
                        //     l.iter().any(|x| x.id != ID::Specific(0))
                        // }
                        Some(ObjParam::Group(g)) => *g != Id::Specific(0),
                        Some(ObjParam::GroupList(g)) => g[0] != Id::Specific(0),
                        _ => unreachable!(),
                    },
                };

                let mut new_obj = obj.clone();

                if spawned {
                    new_obj.params.insert(62, ObjParam::Bool(true));
                    new_obj.params.insert(87, ObjParam::Bool(true));
                }

                new_obj.params.insert(
                    2,
                    if spawned {
                        ObjParam::Number(
                            (x_pos * (MAX_HEIGHT - START_HEIGHT) as u32 * DELTA_X as u32
                                + 15
                                + i as u32 * DELTA_X as u32) as f64,
                        )
                    } else {
                        ObjParam::Number(0.0)
                    },
                );
                new_obj
                    .params
                    .insert(3, ObjParam::Number(((80 - y_pos) * 30 + 15) as f64));
                full_obj_list.push(new_obj);
            },
        }
    }

    full_obj_list
}
