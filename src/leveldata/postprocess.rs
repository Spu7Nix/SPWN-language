use ahash::AHashMap;

use crate::{
    interpreter::value::Id,
    leveldata::{
        object_data::{serialize_obj, ObjParam},
        preprocess::get_used_ids,
    },
};
pub type ArbitraryId = u16;
pub type SpecificId = u16;
use super::object_data::GdObj;

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
                    id = vec![g];
                }

                ObjParam::GroupList(l) => {
                    class_index = 0;
                    id = l.iter().collect();
                }
                ObjParam::Color(g) => {
                    class_index = 1;
                    id = vec![g];
                }
                ObjParam::Block(g) => {
                    class_index = 2;
                    id = vec![g];
                }
                ObjParam::Item(g) => {
                    class_index = 3;
                    id = vec![g];
                }
                _ => continue,
            }
            for id in id {
                match id {
                    Id::Specific(i) => {
                        closed_ids[class_index].insert(*i);
                    }
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
                }
                ObjParam::GroupList(g) => {
                    class_index = 0;
                    ids = g.iter_mut().collect();
                }
                ObjParam::Color(g) => {
                    class_index = 1;
                    ids = vec![g];
                }
                ObjParam::Block(g) => {
                    class_index = 2;
                    ids = vec![g];
                }
                ObjParam::Item(g) => {
                    class_index = 3;
                    ids = vec![g];
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
