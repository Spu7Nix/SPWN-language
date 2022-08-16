use ahash::{AHashMap, AHashSet};

use super::gd_types::Id;
use super::object_data::SPWN_SIGNATURE_GROUP;

pub fn remove_spwn_objects(file_content: &mut String) {
    let spwn_group = match SPWN_SIGNATURE_GROUP {
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
