//post-compile-optimization of the triggers

use crate::levelstring::{GDObj, SPWN_SIGNATURE_GROUP};

use std::collections::{HashMap, HashSet};

fn traverse(
    current_chain: Vec<usize>,
    objects: &Vec<(GDObj, bool)>,
    group_map: &HashMap<String, Vec<usize>>,
) -> Vec<(Vec<usize>, bool)> {
    let obj = &objects[*current_chain.last().unwrap()];
    if obj.1 {
        return vec![(current_chain, true)];
    }
    match obj.0.obj_id() {
        "1268" => {
            //spawn trigger

            match obj.0.params.get(&51) {
                Some(group) => match group_map.get(group) {
                    Some(list) => {
                        let mut chains = Vec::new();
                        for o in list {
                            chains.extend(traverse(
                                {
                                    let mut new_chain = current_chain.clone();
                                    new_chain.push(*o);
                                    new_chain
                                },
                                objects,
                                group_map,
                            ))
                        }
                        chains
                    }
                    None => vec![(current_chain, false)],
                },
                None => {
                    //DANGELING
                    vec![(current_chain, false)]
                }
            }
        }
        _ => (vec![(current_chain, true)]),
    }
}

pub fn optimize(obj_in: Vec<GDObj>) -> Vec<GDObj> {
    let mut objects: Vec<(GDObj, bool)> = obj_in.iter().map(|x| (x.clone(), false)).collect();
    let mut group_map = HashMap::<String, Vec<usize>>::new();

    for (i, (obj, _)) in objects.iter().enumerate() {
        for group in &obj.get_groups() {
            match group_map.get_mut(&group.id.to_string()) {
                Some(list) => (*list).push(i),
                None => {
                    group_map.insert(group.id.to_string(), vec![i]);
                }
            };
        }
    }

    let mut to_be_deleted = HashSet::<usize>::new();

    //find compressable chains and dangling chains

    let mut to_be_added = Vec::<GDObj>::new();

    for (i, _) in objects.clone().iter().enumerate() {
        let chains = traverse(vec![i], &objects, &group_map);
        //SPAWN TRIGGERS OVERLAP
        //ADD NEW ONES INSTEAD OF CHANGING THEM!!
        //BASICALLY REMOVE to_be_changed
        for chain in chains {
            if chain.1 {
                if chain.0.len() > 2 {
                    let mut combined_delay = 0.0;

                    for o in &chain.0 {
                        let obj = &objects[*o];
                        match obj.0.obj_id() {
                            "1268" => {
                                //spawn trigger
                                //final object will never be a spawn trigger here
                                match obj.0.params.get(&63) {
                                    Some(d) => combined_delay += d.parse::<f64>().unwrap(),
                                    None => (),
                                }
                            }
                            _ => (),
                        };
                    }
                    let final_trigger_group = objects[chain.0[chain.0.len() - 2]]
                        .0
                        .params
                        .get(&51)
                        .unwrap()
                        .clone();

                    let first_trigger_group =
                        objects[chain.0[0]].0.params.get(&51).unwrap().clone();

                    let mut params = HashMap::new();
                    let pos = objects[chain.0[0]].0.get_pos();
                    params.insert(1, "1268".to_string());
                    if pos.0 == 0.0 {
                        params.insert(2, (pos.0).to_string());
                    } else {
                        params.insert(2, (pos.0 + 30.0).to_string());
                    }

                    params.insert(3, (pos.1).to_string());
                    params.insert(57, first_trigger_group.clone() + "." + SPWN_SIGNATURE_GROUP);
                    params.insert(51, final_trigger_group);
                    match objects[chain.0[0]].0.params.get(&62) {
                        Some(v) => {
                            if v == "1" {
                                params.insert(62, "1".to_string());
                            }
                        }
                        None => (),
                    };

                    //params.insert(87, "1".to_string());

                    if combined_delay < 0.0001 {
                        //skip all spawn triggers

                        for i in 0..(chain.0.len() - 1) {
                            objects[chain.0[i]].1 = true;
                            to_be_deleted.insert(chain.0[i]);
                        }
                    } else {
                        //have one spawn trigger with all the delay

                        params.insert(63, combined_delay.to_string());

                        for i in 0..(chain.0.len() - 1) {
                            objects[chain.0[i]].1 = true;
                            to_be_deleted.insert(chain.0[i]);
                        }
                    }

                    to_be_added.push(GDObj { params, func_id: 0 });

                    match group_map.get_mut(&first_trigger_group) {
                        Some(list) => (*list).push(objects.len() - 1 + to_be_added.len()),
                        None => {
                            group_map.insert(
                                first_trigger_group,
                                vec![objects.len() - 1 + to_be_added.len()],
                            );
                        }
                    }
                }
            } else {
                //dangling chain, delete last member
                //i think the whole thing would get deleted eventually, not sure tho
                //to_be_deleted.insert(*chain.0.last().unwrap());
                to_be_deleted.extend(chain.0)
            }
        }
    }

    let mut tbd_list = to_be_deleted.iter().map(|x| *x).collect::<Vec<usize>>();
    tbd_list.sort_by(|a, b| b.cmp(a));

    for o in tbd_list {
        objects.remove(o);
    }

    let mut obj_out: Vec<GDObj> = objects.iter().map(|x| x.0.clone()).collect();

    obj_out.extend(to_be_added);

    obj_out
}
