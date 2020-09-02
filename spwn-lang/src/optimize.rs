//post-compile-optimization of the triggers

use crate::levelstring::GDObj;

use std::collections::{HashMap, HashSet};

pub fn optimize(mut objects: Vec<GDObj>) -> Vec<GDObj> {
    let mut group_map = HashMap::<u16, Vec<usize>>::new();

    for (i, obj) in objects.iter().enumerate() {
        for group in &obj.groups {
            match group_map.get_mut(&group.id) {
                Some(list) => (*list).push(i),
                None => {
                    group_map.insert(group.id, vec![i]);
                }
            };
        }
    }

    let mut to_be_deleted = HashSet::<usize>::new();

    //find compressable chains and dangling chains

    for (i, _) in objects.iter().enumerate() {
        let chains = Vec::<Vec<(usize, bool)>>::new();

        let traverse = |current_chain: (Vec<usize>, bool)| -> Vec<(Vec<usize>, bool)> // (chain end group, delay)
         {
            let obj = objects[*current_chain.0.last().unwrap()];
            match obj.obj_id {
                1268 => {
                    //spawn trigger

                    match obj.params.get(&51) {
                        Some(group) => {
                            let objects_w_group = match group_map.get(group.parse()) {
                                Some(list) => {

                                }
                                None => (current_chain.0, false)
                            }
                        }
                        None =>  {
                            //DANGELING
                            (current_chain.0, false)
                        },
                    };
                }
                _ => current_chain,
            }
        };
    }

    objects
}
