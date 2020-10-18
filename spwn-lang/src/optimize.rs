use crate::ast::ObjectMode;
use crate::builtin::{Group, ID};
use crate::compiler_types::FunctionID;
use crate::levelstring::{GDObj, ObjParam};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TriggerRole {
    // Spawn triggers have their own catagory
    // because they can be combined by adding their delays
    Spawn,

    // Triggers like move and rotate, which have some output in the level
    // and therefore cannot be optimized away
    Output,

    // Triggers that send a signal, but don't cause any side effects
    Func,
}

fn get_role(obj_id: u16) -> TriggerRole {
    match obj_id {
        1268 => TriggerRole::Spawn,
        1595 | 1611 | 1811 | 1815 | 1812 => TriggerRole::Func,
        _ => TriggerRole::Output,
    }
}

type ObjPtr = (usize, usize);
type TriggerNetwork = HashMap<Group, Vec<Trigger>>;

#[derive(Debug, Copy, Clone)]
struct Trigger {
    obj: ObjPtr,
    role: TriggerRole,
    connections_in: u32,
    optimized: bool,
    deleted: bool,
}

struct Triggerlist<'a> {
    list: &'a mut Vec<FunctionID>,
}

impl<'a> std::ops::Index<ObjPtr> for Triggerlist<'a> {
    type Output = GDObj;

    fn index(&self, i: ObjPtr) -> &Self::Output {
        &self.list[i.0].obj_list[i.1]
    }
}
impl<'a> std::ops::IndexMut<ObjPtr> for Triggerlist<'a> {
    fn index_mut(&mut self, i: ObjPtr) -> &mut Self::Output {
        &mut self.list[i.0].obj_list[i.1]
    }
}

fn clone_trigger(
    trigger: Trigger,
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
) -> Trigger {
    let obj = objects[trigger.obj].clone();
    let obj_map = &obj.params;
    let obj = GDObj {
        params: obj_map.clone(),
        mode: ObjectMode::Trigger,
        func_id: obj.func_id,
    };
    let fn_id = obj.func_id;
    (*objects.list)[fn_id].obj_list.push(obj.clone());
    let obj_index = (fn_id, objects.list[fn_id].obj_list.len() - 1);
    let trigger = Trigger {
        obj: obj_index,
        optimized: true,
        deleted: false,
        ..trigger
    };
    if let Some(ObjParam::Group(group)) = obj.params.get(&57) {
        match network.get_mut(group) {
            Some(l) => (*l).push(trigger),
            None => {
                network.insert(*group, vec![trigger]);
            }
        }
    } else {
        match network.get_mut(&NO_GROUP) {
            Some(l) => (*l).push(trigger),
            None => {
                network.insert(NO_GROUP, vec![trigger]);
            }
        }
    }
    trigger
}

const NO_GROUP: Group = Group {
    id: ID::Specific(0),
};

pub fn optimize(mut obj_in: Vec<FunctionID>, mut closed_group: u16) -> Vec<FunctionID> {
    let mut network = TriggerNetwork::new();

    // sort all triggers by their group
    for (f, fnid) in obj_in.iter().enumerate() {
        for (o, obj) in fnid.obj_list.iter().enumerate() {
            if let Some(ObjParam::Number(id)) = obj.params.get(&1) {
                let trigger = Trigger {
                    obj: (f, o),
                    role: get_role(*id as u16),
                    connections_in: 0,
                    optimized: false,
                    deleted: true,
                };
                if let Some(ObjParam::Group(group)) = obj.params.get(&57) {
                    match network.get_mut(group) {
                        Some(l) => (*l).push(trigger),
                        None => {
                            network.insert(*group, vec![trigger]);
                        }
                    }
                } else {
                    match network.get_mut(&NO_GROUP) {
                        Some(l) => (*l).push(trigger),
                        None => {
                            network.insert(NO_GROUP, vec![trigger]);
                        }
                    }
                }
            }
        }
    }

    // count connection in for all triggers
    for fnid in &obj_in {
        for obj in &fnid.obj_list {
            if let Some(ObjParam::Number(id)) = obj.params.get(&1) {
                if get_role(*id as u16) != TriggerRole::Output {
                    if let Some(ObjParam::Group(id)) = obj.params.get(&51) {
                        if let Some(list) = network.get_mut(id) {
                            for t in list {
                                (*t).connections_in += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    /*
    Alright scrap this method
    store the functions applied with the delay
    merge threads
    rebuild whole thread
    */
    //optimize
    //optimize_network(&mut network);

    let mut objects = Triggerlist { list: &mut obj_in };
    let len = network[&NO_GROUP].len();
    for i in 0..len {
        let trigger = network[&NO_GROUP][i];

        // if trigger.optimized {
        //     continue;
        // }
        if trigger.role != TriggerRole::Output {
            //let result =
            optimize_from(&mut network, &mut objects, (NO_GROUP, i), &mut closed_group);
        /*if result {
            (*network.get_mut(&NO_GROUP).unwrap())[i].deleted = false;
        }*/
        } else {
            (*network.get_mut(&NO_GROUP).unwrap())[i].deleted = false;
        }
        //create spawn triggers
        //TODO: keep track of delay
    }

    //for (g, len) in group_sizes {}

    // put into new fn ids and lists

    //profit

    rebuild(&network, &obj_in)
}
fn get_targets<'a>(
    network: &'a mut TriggerNetwork,
    objects: &'a mut Triggerlist,
    start: (Group, usize),
    delay: u32,
    ignore_otimized: bool,
    closed_group: &mut u16,
) -> Option<Vec<(Group, usize, u32)>> {
    //u32: delay in millis

    if network[&start.0][start.1].optimized {
        if network[&start.0][start.1].deleted {
            return Some(Vec::new());
        } else {
            return None;
        }
    }
    (*network.get_mut(&start.0).unwrap())[start.1].optimized = true;
    let trigger = network.get(&start.0).unwrap()[start.1];
    let start_obj = &objects[trigger.obj].params;

    let list: Vec<(Group, usize)>;

    if let Some(ObjParam::Group(g)) = start_obj.get(&51) {
        if let ID::Specific(_) = g.id {
            (*network.get_mut(&start.0).unwrap())[start.1].deleted = false;
            return None;
        }

        if let Some(l) = network.get_mut(g) {
            list = (0..l.len()).map(|x| (*g, x)).collect();
        } else {
            //dangeling

            return Some(Vec::new());
        }
    } else {
        //dangeling

        return Some(Vec::new());
    }

    let added_delay = if let Some(ObjParam::Number(n)) = start_obj.get(&63) {
        (*n * 1000.0) as u32
    } else {
        0
    };

    let mut out = HashSet::new();

    for trigger_ptr in list {
        let trigger = network[&trigger_ptr.0][trigger_ptr.1];

        let full_trigger_ptr = (trigger_ptr.0, trigger_ptr.1, delay + added_delay);

        if trigger.optimized && !ignore_otimized {
            out.insert(full_trigger_ptr);
        } else if trigger.connections_in > 1 {
            if optimize_from(network, objects, trigger_ptr, closed_group) {
                out.insert(full_trigger_ptr);
            }
        }
        /* else if trigger.role == TriggerRole::Spawn && trigger.optimized {
            // if its already optimized, redo
            match get_targets(
                network,
                objects,
                trigger_ptr,
                delay + added_delay,
                true,
                closed_group,
            ) {
                Some(children) => out.extend(children),
                None => out.push(full_trigger_ptr),
            }
        }*/
        else {
            match trigger.role {
                TriggerRole::Output => {
                    (*network.get_mut(&trigger_ptr.0).unwrap())[trigger_ptr.1].deleted = false;
                    out.insert(full_trigger_ptr);
                }
                TriggerRole::Func => {
                    if optimize_from(network, objects, trigger_ptr, closed_group) {
                        (*network.get_mut(&trigger_ptr.0).unwrap())[trigger_ptr.1].deleted = false;
                        out.insert(full_trigger_ptr);
                    }
                }
                TriggerRole::Spawn => {
                    match get_targets(
                        network,
                        objects,
                        trigger_ptr,
                        delay + added_delay,
                        ignore_otimized,
                        closed_group,
                    ) {
                        Some(children) => out.extend(children),
                        None => {
                            (*network.get_mut(&trigger_ptr.0).unwrap())[trigger_ptr.1].deleted =
                                false;
                            out.insert(full_trigger_ptr);
                        }
                    }
                }
            }
        }
    }

    (*network.get_mut(&start.0).unwrap())[start.1].deleted = true;

    Some(out.iter().copied().collect())
}

fn optimize_from<'a>(
    network: &'a mut TriggerNetwork,
    objects: &mut Triggerlist,
    start: (Group, usize),
    closed_group: &mut u16,
) -> bool {
    //returns weather to keep or delete the trigger

    let trigger = network.get(&start.0).unwrap()[start.1];
    if trigger.role == TriggerRole::Output {
        (*network.get_mut(&start.0).unwrap())[start.1].deleted = false;
        return true;
    }

    if trigger.optimized {
        return !trigger.deleted;
    }

    //let role = trigger.role;

    let targets = get_targets(network, objects, start, 0, false, closed_group);
    let trigger = network.get(&start.0).unwrap()[start.1];
    // {
    //     let object = &objects[trigger.obj];
    //     println!("source\n");
    //     println!("Deleted: {}", trigger.deleted);
    //     println!("Optimized: {}", trigger.optimized);
    //     let mut paramlist = object.params.iter().collect::<Vec<(&u16, &ObjParam)>>();
    //     paramlist.sort_by(|a, b| (a.0).cmp(b.0));
    //     for (k, v) in &paramlist {
    //         println!("{}: {:?}", k, v);
    //     }
    // }

    if let Some(targets) = targets {
        // {
        //     for (g, i, d) in &targets {
        //         let obj = &objects[network[g][*i].obj];

        //         println!("TARGET\n");
        //         println!("Deleted: {}", network[g][*i].deleted);
        //         println!("Optimized: {}", network[g][*i].optimized);
        //         let mut paramlist = obj.params.iter().collect::<Vec<(&u16, &ObjParam)>>();
        //         paramlist.sort_by(|a, b| (a.0).cmp(b.0));
        //         for (k, v) in &paramlist {
        //             println!("{}: {:?}", k, v);
        //         }
        //         println!("\n")
        //     }
        // }

        if targets.is_empty() {
            return false;
        }
        if trigger.role == TriggerRole::Func
            && targets.len() == 1
            && targets[0].2 == 0
            && trigger.connections_in > 1
        {
            let new_trigger = clone_trigger(trigger, network, objects);
            objects[new_trigger.obj]
                .params
                .insert(51, ObjParam::Group(targets[0].0));
            return true;
        }

        // if trigger.role == TriggerRole::Spawn && targets.len() == 1 {
        //     let new_trigger = clone_trigger(trigger, network, objects);
        //     objects[new_trigger.obj]
        //         .params
        //         .insert(51, ObjParam::Group(targets[0].0));
        //     objects[new_trigger.obj]
        //         .params
        //         .insert(63, ObjParam::Number(targets[0].2 as f64 / 1000.0));
        //     return true;
        // }

        let spawn_group = if trigger.role == TriggerRole::Func {
            (*closed_group) += 1;
            let new_group = Group {
                id: ID::Arbitrary(*closed_group),
            };

            objects[trigger.obj]
                .params
                .insert(51, ObjParam::Group(new_group));

            (*network.get_mut(&start.0).unwrap())[start.1].deleted = false;

            Some(new_group)
        } else {
            match objects[trigger.obj].params.get(&57) {
                Some(ObjParam::Group(g)) => {
                    if *g == NO_GROUP {
                        None
                    } else {
                        Some(*g)
                    }
                }
                _ => None,
            }
        };

        //println!("SPAWN GROUP {:?}", spawn_group);

        for (g, i, delay) in targets {
            //println!("DELAY {:?}, spwg: {:?}", delay, spawn_group);
            // add spawn trigger to obj.fn_id with target group: g and delay: delay
            if delay == 0 && network[&g][i].connections_in == 1 {
                if let Some(gr) = spawn_group {
                    match objects[network[&g][i].obj].params.get_mut(&57) {
                        Some(ObjParam::Group(g)) => (*g) = gr,
                        _ => {
                            objects[network[&g][i].obj]
                                .params
                                .insert(57, ObjParam::Group(gr));
                        }
                    }
                } else {
                    objects[network[&g][i].obj].params.remove(&57);
                    objects[network[&g][i].obj].params.remove(&62);
                }

                (*network.get_mut(&g).unwrap())[i].optimized = true;

            //continue;
            } else {
                let mut new_obj_map = HashMap::new();
                new_obj_map.insert(1, ObjParam::Number(1268.0));
                new_obj_map.insert(51, ObjParam::Group(g));
                new_obj_map.insert(63, ObjParam::Number(delay as f64 / 1000.0));

                if let Some(g) = spawn_group {
                    new_obj_map.insert(57, ObjParam::Group(g));
                }

                let new_obj = GDObj {
                    params: new_obj_map,
                    func_id: trigger.obj.0,
                    mode: ObjectMode::Trigger,
                };

                (*objects.list)[trigger.obj.0]
                    .obj_list
                    .push(new_obj.clone());

                let obj_index = (
                    trigger.obj.0,
                    objects.list[trigger.obj.0].obj_list.len() - 1,
                );
                let new_trigger = Trigger {
                    obj: obj_index,
                    optimized: true,
                    deleted: false,
                    ..trigger
                };

                if let Some(ObjParam::Group(group)) = new_obj.params.get(&57) {
                    match network.get_mut(group) {
                        Some(l) => (*l).push(new_trigger),
                        None => {
                            network.insert(*group, vec![new_trigger]);
                        }
                    }
                } else {
                    match network.get_mut(&NO_GROUP) {
                        Some(l) => (*l).push(new_trigger),
                        None => {
                            network.insert(NO_GROUP, vec![new_trigger]);
                        }
                    }
                }
            }
        }

        true
    } else {
        (*network.get_mut(&start.0).unwrap())[start.1].deleted = false;
        true
    }
}

fn rebuild(network: &TriggerNetwork, orig_structure: &Vec<FunctionID>) -> Vec<FunctionID> {
    let mut out = orig_structure.clone();
    for el in &mut out {
        (*el).obj_list.clear();
    }

    for (_, list) in network {
        for trigger in list {
            //assert!(trigger.optimized);
            if trigger.deleted {
                continue;
            }
            let obj = &orig_structure[trigger.obj.0].obj_list[trigger.obj.1];
            let fn_id = &out[obj.func_id];
            // if it's already there, continue
            if fn_id.obj_list.iter().any(|x| x == obj) {
                continue;
            }
            out[obj.func_id].obj_list.push(obj.clone())
        }
    }

    out
}
