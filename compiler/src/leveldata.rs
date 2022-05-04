// useful things for dealing with gd level data

use crate::builtins::*;
use crate::compiler_types::FunctionId;
use crate::context::Context;
use fnv::{FnvHashMap, FnvHashSet};
use parser::ast::ObjectMode;
use std::hash::Hash;

pub struct TriggerOrder(f32);

#[derive(Clone, PartialEq, Debug)]
pub enum ObjParam {
    Group(Group),
    Color(Color),
    Block(Block),
    Item(Item),
    Number(f64),
    Bool(bool),
    Text(String),
    GroupList(Vec<Group>),
    Epsilon,
}
// this is so bruh
#[allow(clippy::derive_hash_xor_eq)]
impl Hash for ObjParam {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ObjParam::Group(v) => v.hash(state),
            ObjParam::Color(v) => v.hash(state),
            ObjParam::Block(v) => v.hash(state),
            ObjParam::Item(v) => v.hash(state),
            ObjParam::Number(v) => ((*v * 100000.0) as usize).hash(state),
            ObjParam::Bool(v) => v.hash(state),
            ObjParam::Text(v) => v.hash(state),
            ObjParam::GroupList(v) => v.hash(state),
            ObjParam::Epsilon => "epsilon".hash(state),
        }
    }
}

impl std::cmp::PartialOrd for GdObj {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        for param in [1, 51, 57].iter() {
            if let Some(p1) = self.params.get(param) {
                if let Some(p2) = other.params.get(param) {
                    match (p1, p2) {
                        (ObjParam::Number(n1), ObjParam::Number(n2)) => {
                            return (*n1).partial_cmp(n2)
                        }
                        (ObjParam::Group(g1), ObjParam::Group(g2)) => {
                            let num1 = match g1.id {
                                Id::Arbitrary(n) => n,
                                Id::Specific(n) => n,
                            };

                            let num2 = match g2.id {
                                Id::Arbitrary(n) => n,
                                Id::Specific(n) => n,
                            };

                            return num1.partial_cmp(&num2);
                        }
                        (_, _) => (),
                    }
                }
            }
        }
        Some(std::cmp::Ordering::Equal)
    }
}

use std::fmt;

impl fmt::Display for ObjParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjParam::Group(Group { id })
            | ObjParam::Color(Color { id })
            | ObjParam::Block(Block { id })
            | ObjParam::Item(Item { id }) => match id {
                Id::Specific(id) => write!(f, "{}", id),
                _ => write!(f, "0"),
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
                    if let Id::Specific(id) = g.id {
                        out += &(id.to_string() + ".")
                    } else {
                        out += "0."
                    };
                }
                out.pop();
                write!(f, "{}", out)
            }
            ObjParam::Epsilon => write!(f, "0.05"),
        }
    }
}
#[derive(Clone, PartialEq, Debug)]
pub struct GdObj {
    /*pub obj_id: u16,
    pub groups: Vec<Group>,
    pub target: Group,
    pub spawn_triggered: bool,*/
    pub func_id: usize,
    pub params: FnvHashMap<u16, ObjParam>,
    pub mode: ObjectMode,
    pub unique_id: usize,
}

impl GdObj {
    pub fn context_parameters(&mut self, context: &Context) -> GdObj {
        self.params.insert(57, ObjParam::Group(context.start_group));

        (*self).clone()
    }
}

pub fn get_used_ids(ls: &str) -> [FnvHashSet<u16>; 4] {
    let mut out = [
        FnvHashSet::<u16>::default(),
        FnvHashSet::<u16>::default(),
        FnvHashSet::<u16>::default(),
        FnvHashSet::<u16>::default(),
    ];
    let objects = ls.split(';');
    for obj in objects {
        let props: Vec<&str> = obj.split(',').collect();
        let mut map = FnvHashMap::default();

        for i in (0..props.len() - 1).step_by(2) {
            map.insert(props[i], props[i + 1]);
        }

        for (key, value) in &map {
            match *key {
                "57" => {
                    //GROUPS
                    let groups = value.split('.');
                    for g in groups {
                        let group = g.parse().unwrap();

                        out[0].insert(group);
                    }
                }
                "51" => {
                    match (map.get("1"), map.get("52")) {
                        (Some(&"1006"), Some(&"1")) => out[0].insert(value.parse().unwrap()),
                        (Some(&"1006"), _) => out[1].insert(value.parse().unwrap()),
                        _ => out[0].insert(value.parse().unwrap()),
                    };
                }
                "71" => {
                    out[0].insert(value.parse().unwrap());
                }
                //colors
                "21" => {
                    out[1].insert(value.parse().unwrap());
                }
                "22" => {
                    out[1].insert(value.parse().unwrap());
                }
                "23" => {
                    out[1].insert(value.parse().unwrap());
                }

                "80" => {
                    match map.get("1") {
                        //if collision trigger or block, add block id
                        Some(&"1815") | Some(&"1816") => out[2].insert(value.parse().unwrap()),
                        //counter display => do nothing
                        Some(&"1615") => false,
                        // else add item id
                        _ => out[3].insert(value.parse().unwrap()),
                    };
                }

                "95" => {
                    out[2].insert(value.parse().unwrap());
                }
                //some of these depends on what object it is
                //pulse target depends on group mode/color mode
                //figure this out, future me
                _ => (),
            }
        }
    }
    out
}

const START_HEIGHT: u16 = 10;
const MAX_HEIGHT: u16 = 40;

const DELTA_X: u16 = 1;

pub const SPWN_SIGNATURE_GROUP: Group = Group {
    id: Id::Specific(1001),
};
//use crate::ast::ObjectMode;

pub fn remove_spwn_objects(file_content: &mut String) {
    let spwn_group = match SPWN_SIGNATURE_GROUP.id {
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

//returns the string to be appended to the old string
pub fn append_objects(
    mut objects: Vec<GdObj>,
    old_ls: &str,
) -> Result<(String, [usize; 4]), String> {
    let mut closed_ids = get_used_ids(old_ls);

    //collect all specific ids mentioned into closed_[id] lists
    for obj in &objects {
        for prop in obj.params.values() {
            let class_index;
            let id;
            match prop {
                ObjParam::Group(g) => {
                    class_index = 0;
                    id = vec![g.id];
                }

                ObjParam::GroupList(l) => {
                    class_index = 0;

                    id = l.iter().map(|g| g.id).collect();
                }
                ObjParam::Color(g) => {
                    class_index = 1;
                    id = vec![g.id];
                }
                ObjParam::Block(g) => {
                    class_index = 2;
                    id = vec![g.id];
                }
                ObjParam::Item(g) => {
                    class_index = 3;
                    id = vec![g.id];
                }
                _ => continue,
            }
            for id in id {
                match id {
                    Id::Specific(i) => {
                        closed_ids[class_index].insert(i);
                    }
                    _ => continue,
                }
            }
        }
    }

    //find new ids for all the arbitrary ones
    let mut id_maps: [FnvHashMap<ArbitraryId, SpecificId>; 4] = [
        FnvHashMap::default(),
        FnvHashMap::default(),
        FnvHashMap::default(),
        FnvHashMap::default(),
    ];

    const ID_MAX: u16 = 999;

    for obj in &mut objects {
        for prop in obj.params.values_mut() {
            let class_index;
            let ids: Vec<&mut Id>;
            match prop {
                ObjParam::Group(g) => {
                    class_index = 0;
                    ids = vec![&mut g.id];
                }
                ObjParam::GroupList(g) => {
                    class_index = 0;
                    ids = g.iter_mut().map(|x| &mut x.id).collect();
                }
                ObjParam::Color(g) => {
                    class_index = 1;
                    ids = vec![&mut g.id];
                }
                ObjParam::Block(g) => {
                    class_index = 2;
                    ids = vec![&mut g.id];
                }
                ObjParam::Item(g) => {
                    class_index = 3;
                    ids = vec![&mut g.id];
                }
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
                                    return Err(format!(
                                        "This level exceeds the {} limit!",
                                        ["group", "color", "block ID", "item ID"][class_index]
                                    ));
                                }
                            }
                        })
                    }
                    _ => continue,
                }
            }
        }
    }
    for (i, list) in closed_ids.iter_mut().enumerate() {
        list.remove(&0);
        if list.len() > ID_MAX as usize {
            return Err(format!(
                "This level exceeds the {} limit! ({}/{})",
                ["group", "color", "block ID", "item ID"][i],
                list.len(),
                ID_MAX
            ));
        }
    }

    //println!("group_map: {:?}", id_maps[0]);

    fn serialize_obj(mut trigger: GdObj) -> String {
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

pub fn apply_fn_ids(func_ids: &[FunctionId]) -> Vec<GdObj> {
    //println!("{:?}", trigger);

    let mut objectlist = Vec::new();

    for func_id in func_ids.iter() {
        objectlist.extend(func_id.obj_list.clone());
    }

    let mut full_obj_list = Vec::<GdObj>::new();

    /*if !id.obj_list.is_empty() {
        //add label
        obj_string += &format!(
            "1,914,2,{},3,{},31,{},32,0.5;",
            x_offset * 30 + 15,
            ((81 - START_HEIGHT) - y_offset) * 30 + 15,
            base64::encode(id.name.as_bytes())
        );
    }*/

    //add top layer
    let possible_height = MAX_HEIGHT - START_HEIGHT; //30 is max (TODO: case for if y_offset is more than 30)
    objectlist.sort_by(|x, y| x.1.partial_cmp(&y.1).unwrap());

    for (i, (obj, _)) in objectlist.iter().enumerate() {
        match obj.mode {
            ObjectMode::Object => {
                full_obj_list.push(obj.clone());
            }
            ObjectMode::Trigger => {
                let y_pos = (i as u16) % possible_height + START_HEIGHT;
                let x_pos = 0;

                let spawned = match obj.params.get(&62) {
                    Some(ObjParam::Bool(b)) => *b,
                    _ => match obj.params.get(&57) {
                        None => false,
                        // Some(ObjParam::GroupList(l)) => {
                        //     l.iter().any(|x| x.id != ID::Specific(0))
                        // }
                        Some(ObjParam::Group(g)) => g.id != Id::Specific(0),
                        Some(ObjParam::GroupList(g)) => g[0].id != Id::Specific(0),
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
            }
        }
    }
    // if !objectlist.is_empty() {
    //     current_x += (objectlist.len() as f64 / possible_height as f64).floor() as u32 + 1;
    // }

    //add all children
    // for (i, func_id) in func_ids.iter().enumerate() {
    //     if func_id.parent == Some(id_index) {
    //         let (obj, new_length) = apply_fn_id(i, func_ids, current_x + x_offset, y_offset);
    //         objects.extend(obj);

    //         if new_length > 0 {
    //             current_x += new_length + 1;
    //         }
    //     }
    // }

    full_obj_list
}
