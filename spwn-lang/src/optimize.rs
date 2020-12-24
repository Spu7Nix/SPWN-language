use crate::ast::ObjectMode;
use crate::builtin::{Block, Group, Item, ID};
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
//                                     triggers      connections in
type TriggerNetwork = HashMap<Group, TriggerGang>;

#[derive(Debug, Clone)]
// what do you mean? its a trigger gang!
struct TriggerGang {
    triggers: Vec<Trigger>,
    connections_in: u32,
}

impl TriggerGang {
    fn new(triggers: Vec<Trigger>) -> Self {
        TriggerGang {
            triggers,
            connections_in: 0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Trigger {
    obj: ObjPtr,
    role: TriggerRole,
    order: usize,
    deleted: bool,
    optimized: bool,
}

struct Triggerlist<'a> {
    list: &'a mut Vec<FunctionID>,
}

impl<'a> std::ops::Index<ObjPtr> for Triggerlist<'a> {
    type Output = (GDObj, usize);

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
    let (obj, order) = objects[trigger.obj].clone();
    let obj_map = &obj.params;

    let obj = GDObj {
        params: obj_map.clone(),
        mode: ObjectMode::Trigger,
        func_id: obj.func_id,
        unique_id: obj.unique_id, //this might cause a problem in the future
    };
    let fn_id = obj.func_id;
    (*objects.list)[fn_id].obj_list.push((obj.clone(), order));
    let obj_index = (fn_id, objects.list[fn_id].obj_list.len() - 1);
    let trigger = Trigger {
        obj: obj_index,
        deleted: false,
        ..trigger
    };
    if let Some(ObjParam::Group(group)) = obj.params.get(&57) {
        match network.get_mut(group) {
            Some(l) => (*l).triggers.push(trigger),
            None => {
                network.insert(*group, TriggerGang::new(vec![trigger]));
            }
        }
    } else {
        match network.get_mut(&NO_GROUP) {
            Some(l) => (*l).triggers.push(trigger),
            None => {
                network.insert(NO_GROUP, TriggerGang::new(vec![trigger]));
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
        for (o, (obj, order)) in fnid.obj_list.iter().enumerate() {
            if let Some(ObjParam::Number(id)) = obj.params.get(&1) {
                let trigger = Trigger {
                    obj: (f, o),
                    role: get_role(*id as u16),
                    order: *order,
                    deleted: true,
                    optimized: false,
                };
                if let Some(ObjParam::Group(group)) = obj.params.get(&57) {
                    match network.get_mut(group) {
                        Some(l) => (*l).triggers.push(trigger),
                        None => {
                            network.insert(*group, TriggerGang::new(vec![trigger]));
                        }
                    }
                } else {
                    match network.get_mut(&NO_GROUP) {
                        Some(l) => (*l).triggers.push(trigger),
                        None => {
                            network.insert(NO_GROUP, TriggerGang::new(vec![trigger]));
                        }
                    }
                }
            }
        }
    }

    // count connection in for all triggers
    for fnid in &obj_in {
        for (obj, _) in &fnid.obj_list {
            if let Some(ObjParam::Number(id)) = obj.params.get(&1) {
                if get_role(*id as u16) != TriggerRole::Output {
                    if let Some(ObjParam::Group(id)) = obj.params.get(&51) {
                        if let Some(gang) = network.get_mut(id) {
                            (*gang).connections_in += 1;
                        }
                    }
                }
            }
        }
    }

    //optimize
    //optimize_network(&mut network);

    let mut objects = Triggerlist { list: &mut obj_in };

    // fix read write order
    // not an optimization, more like a consistancy fix
    // also, like nothing works without this, so i should probably move
    // this somewhere else if i want to add an option to not have optimization
    network = fix_read_write_order(&mut objects, &network, &mut closed_group);

    //println!("{:?}", network);

    //return rebuild(&network, &obj_in);

    let len = if let Some(gang) = network.get(&NO_GROUP) {
        gang.triggers.len()
    } else {
        0
    };
    for i in 0..len {
        let trigger = network[&NO_GROUP].triggers[i];

        // if trigger.optimized {
        //     continue;
        // }

        if trigger.role != TriggerRole::Output {
            optimize_from(&mut network, &mut objects, (NO_GROUP, i), &mut closed_group);
        } else {
            (*network.get_mut(&NO_GROUP).unwrap()).triggers[i].deleted = false;
        }
    }

    //for (g, len) in group_sizes {}

    // put into new fn ids and lists

    //profit

    rebuild(&network, &obj_in)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum IDData {
    Group(Group),
    Block(Block),
    Item(Item),
}

fn reads_writes(t: Trigger, objects: &Triggerlist) -> (Vec<IDData>, Vec<IDData>) {
    let role = t.role;
    let obj = &objects[t.obj].0;
    let mut out = (Vec::new(), Vec::new());
    for (key, val) in &obj.params {
        let id_data = match val {
            ObjParam::Group(g) => IDData::Group(*g),
            ObjParam::Block(b) => IDData::Block(*b),
            ObjParam::Item(i) => IDData::Item(*i),
            _ => continue,
        };
        // 77 is the "count" key, and will only be used by an output trigger
        // in a pickup trigger
        if (*key == 51 || *key == 80) && role == TriggerRole::Output {
            out.1.push(id_data);
        } else if *key != 57 {
            out.0.push(id_data);
        }
    }
    out
}

fn get_targets<'a>(
    network: &'a mut TriggerNetwork,
    objects: &'a mut Triggerlist,
    start: (Group, usize),
    delay: u32,
    ignore_optimized: bool,
    closed_group: &mut u16,
) -> Option<Vec<(Group, u32)>> {
    //u32: delay in millis

    if network[&start.0].triggers[start.1].optimized {
        if network[&start.0].triggers[start.1].deleted {
            return Some(Vec::new());
        } else {
            return None;
        }
    }
    (*network.get_mut(&start.0).unwrap()).triggers[start.1].optimized = true;
    let trigger = network.get(&start.0).unwrap().triggers[start.1];
    let start_obj = &objects[trigger.obj].0.params;

    println!("{}", network[&start.0].connections_in);

    let list: Vec<(usize, Group)>;

    if let Some(ObjParam::Group(g)) = start_obj.get(&51) {
        if let ID::Specific(_) = g.id {
            (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
            return None;
        }

        if let Some(gang) = network.get(g) {
            list = vec![*g; gang.triggers.len()]
                .iter()
                .copied()
                .enumerate()
                .collect();
        } else {
            //dangeling

            return Some(Vec::new());
        }
    } else {
        //dangeling

        return Some(Vec::new());
    }

    let added_delay = match start_obj.get(&63) {
        Some(ObjParam::Number(n)) => (*n * 1000.0) as u32,
        Some(ObjParam::Epsilon) => {
            if delay == 0 {
                50
            } else {
                0
            }
        }
        _ => 0,
    };

    let mut out = HashSet::<(Group, u32)>::new();

    for (i, g) in list {
        let trigger_ptr = (g, i);
        let trigger = network[&trigger_ptr.0].triggers[trigger_ptr.1];

        let full_delay = delay + added_delay;

        //let full_trigger_ptr = (trigger_ptr.0, trigger_ptr.1, full_delay);
        let target_out = (trigger_ptr.0, full_delay);

        if trigger.optimized && !ignore_optimized {
            if !trigger.deleted {
                out.insert(target_out);
            }
        } else if network[&trigger_ptr.0].connections_in > 1 {
            (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1].deleted = false;
            if optimize_from(network, objects, trigger_ptr, closed_group) {
                out.insert(target_out);
            } else {
                (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1].deleted = true;
            }
        } else {
            match trigger.role {
                TriggerRole::Output => {
                    (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1].deleted =
                        false;
                    out.insert(target_out);
                }
                TriggerRole::Func => {
                    if optimize_from(network, objects, trigger_ptr, closed_group) {
                        (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1]
                            .deleted = false;
                        out.insert(target_out);
                    }
                }
                TriggerRole::Spawn => {
                    match get_targets(
                        network,
                        objects,
                        trigger_ptr,
                        delay + added_delay,
                        ignore_optimized,
                        closed_group,
                    ) {
                        Some(children) => out.extend(children),
                        None => {
                            (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1]
                                .deleted = false;
                            out.insert(target_out);
                        }
                    }
                }
            }
        }
    }

    (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = true;

    Some(out.iter().copied().collect())
}

fn create_spawn_trigger(
    trigger: Trigger,
    target_group: Group,
    group: Option<Group>,
    delay: f64,
    objects: &mut Triggerlist,
    network: &mut TriggerNetwork,
    //         opt   del
    settings: (bool, bool),
) {
    let mut new_obj_map = HashMap::new();
    new_obj_map.insert(1, ObjParam::Number(1268.0));
    new_obj_map.insert(51, ObjParam::Group(target_group));
    new_obj_map.insert(63, ObjParam::Number(delay));

    if let Some(g) = group {
        new_obj_map.insert(57, ObjParam::Group(g));
    }

    let new_obj = GDObj {
        params: new_obj_map,
        func_id: trigger.obj.0,
        mode: ObjectMode::Trigger,
        unique_id: objects[trigger.obj].0.unique_id,
    };

    (*objects.list)[trigger.obj.0]
        .obj_list
        .push((new_obj.clone(), trigger.order));

    let obj_index = (
        trigger.obj.0,
        objects.list[trigger.obj.0].obj_list.len() - 1,
    );
    let new_trigger = Trigger {
        obj: obj_index,
        optimized: settings.0,
        deleted: settings.1,
        role: TriggerRole::Spawn,
        ..trigger
    };

    if let Some(ObjParam::Group(group)) = new_obj.params.get(&57) {
        match network.get_mut(group) {
            Some(gang) => (*gang).triggers.push(new_trigger),
            None => {
                network.insert(*group, TriggerGang::new(vec![new_trigger]));
            }
        }
    } else {
        match network.get_mut(&NO_GROUP) {
            Some(gang) => (*gang).triggers.push(new_trigger),
            None => {
                network.insert(NO_GROUP, TriggerGang::new(vec![new_trigger]));
            }
        }
    }
}

fn optimize_from<'a>(
    network: &'a mut TriggerNetwork,
    objects: &mut Triggerlist,
    start: (Group, usize),
    closed_group: &mut u16,
) -> bool {
    //returns weather to keep or delete the trigger

    let trigger = network[&start.0].triggers[start.1];
    if trigger.role == TriggerRole::Output {
        (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
        return true;
    }

    if trigger.optimized {
        return !trigger.deleted;
    }

    //let role = trigger.role;

    let targets = get_targets(network, objects, start, 0, false, closed_group);
    let trigger = network[&start.0].triggers[start.1];

    // {
    //     let object = &objects[trigger.obj];
    //     println!("\nsource");
    //     println!("Deleted: {}", trigger.deleted);
    //     println!("Optimized: {}", trigger.optimized);
    //     let mut paramlist = object.0.params.iter().collect::<Vec<(&u16, &ObjParam)>>();
    //     paramlist.sort_by(|a, b| (a.0).cmp(b.0));
    //     for (k, v) in &paramlist {
    //         println!("{}: {:?}", k, v);
    //     }
    // }

    if let Some(targets) = targets {
        if targets.is_empty() {
            return false;
        }
        // else {
        //     for (g, _) in &targets {
        //         {
        //             println!("\nTarget group: {:?}", g);
        //             for t in &network[&g].triggers {
        //                 let object = &objects[t.obj];
        //                 println!("\ntarget");
        //                 println!("Deleted: {}", t.deleted);
        //                 println!("Optimized: {}", t.optimized);
        //                 let mut paramlist =
        //                     object.0.params.iter().collect::<Vec<(&u16, &ObjParam)>>();
        //                 paramlist.sort_by(|a, b| (a.0).cmp(b.0));
        //                 for (k, v) in &paramlist {
        //                     println!("{}: {:?}", k, v);
        //                 }
        //             }
        //         }
        //     }
        // }

        if trigger.role == TriggerRole::Func && targets.len() == 1 && targets[0].1 == 0
        //&& network[&start.0].connections_in > 1
        {
            let new_trigger = clone_trigger(trigger, network, objects);
            objects[new_trigger.obj]
                .0
                .params
                .insert(51, ObjParam::Group(targets[0].0));
            return true;
        }

        let spawn_group = if trigger.role == TriggerRole::Func {
            (*closed_group) += 1;
            let new_group = Group {
                id: ID::Arbitrary(*closed_group),
            };

            objects[trigger.obj]
                .0
                .params
                .insert(51, ObjParam::Group(new_group));

            (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;

            Some(new_group)
        } else {
            match objects[trigger.obj].0.params.get(&57) {
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

        for (g, delay) in targets {
            // add spawn trigger to obj.fn_id with target group: g and delay: delay

            if delay == 0 && network[&g].connections_in == 1 {
                for trigger in &network[&g].triggers {
                    if let Some(gr) = spawn_group {
                        match objects[trigger.obj].0.params.get_mut(&57) {
                            Some(ObjParam::Group(g)) => (*g) = gr,
                            _ => {
                                objects[trigger.obj]
                                    .0
                                    .params
                                    .insert(57, ObjParam::Group(gr));
                            }
                        }
                    } else {
                        objects[trigger.obj].0.params.remove(&57);
                        objects[trigger.obj].0.params.remove(&62);
                    }
                }

                for trigger in &mut (*network.get_mut(&g).unwrap()).triggers {
                    (*trigger).optimized = true;
                }

            //continue;
            } else {
                create_spawn_trigger(
                    trigger,
                    g,
                    spawn_group,
                    delay as f64 / 1000.0,
                    objects,
                    network,
                    (true, false),
                )
                // let mut new_obj_map = HashMap::new();
                // new_obj_map.insert(1, ObjParam::Number(1268.0));
                // new_obj_map.insert(51, ObjParam::Group(g));
                // new_obj_map.insert(63, ObjParam::Number(delay as f64 / 1000.0));

                // if let Some(g) = spawn_group {
                //     new_obj_map.insert(57, ObjParam::Group(g));
                // }

                // let new_obj = GDObj {
                //     params: new_obj_map,
                //     func_id: trigger.obj.0,
                //     mode: ObjectMode::Trigger,
                //     unique_id: objects[trigger.obj].0.unique_id,
                // };

                // (*objects.list)[trigger.obj.0]
                //     .obj_list
                //     .push((new_obj.clone(), trigger.order));

                // let obj_index = (
                //     trigger.obj.0,
                //     objects.list[trigger.obj.0].obj_list.len() - 1,
                // );
                // let new_trigger = Trigger {
                //     obj: obj_index,
                //     optimized: true,
                //     deleted: false,
                //     ..trigger
                // };

                // if let Some(ObjParam::Group(group)) = new_obj.params.get(&57) {
                //     match network.get_mut(group) {
                //         Some(l) => (*l).triggers.push(new_trigger),
                //         None => {
                //             network.insert(*group, TriggerGang::new(vec![new_trigger]));
                //         }
                //     }
                // } else {
                //     match network.get_mut(&NO_GROUP) {
                //         Some(l) => (*l).triggers.push(new_trigger),
                //         None => {
                //             network.insert(NO_GROUP, TriggerGang::new(vec![new_trigger]));
                //         }
                //     }
                // }
            }
        }

        true
    } else {
        (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
        true
    }
}

fn rebuild(network: &TriggerNetwork, orig_structure: &[FunctionID]) -> Vec<FunctionID> {
    let mut out = orig_structure.to_vec();
    for el in &mut out {
        (*el).obj_list.clear();
    }

    for gang in network.values() {
        for trigger in &gang.triggers {
            //assert!(trigger.optimized);
            if trigger.deleted {
                continue;
            }
            let (obj, order) = &orig_structure[trigger.obj.0].obj_list[trigger.obj.1];
            let fn_id = &out[obj.func_id];
            // if it's already there, continue
            if fn_id
                .obj_list
                .iter()
                .any(|x| x.0.unique_id == obj.unique_id && &x.0 == obj)
            {
                continue;
            }
            out[obj.func_id].obj_list.push((obj.clone(), *order))
        }
    }

    out
}

fn fix_read_write_order(
    objects: &mut Triggerlist,
    network: &TriggerNetwork,
    closed_group: &mut u16,
) -> TriggerNetwork {
    let mut new_network = TriggerNetwork::new();
    for (group, gang) in network {
        let mut written_to = HashSet::new();
        let mut read_from = HashSet::new();
        let mut current_group = *group;

        new_network.insert(
            current_group,
            TriggerGang {
                triggers: Vec::new(),
                connections_in: gang.connections_in,
            },
        );
        let mut sorted = gang.triggers.clone();
        sorted.sort_by(|a, b| objects[a.obj].1.cmp(&objects[b.obj].1));

        //let mut previous_delays = Vec::new();

        for trigger in &gang.triggers {
            let (reads, writes) = reads_writes(*trigger, objects);

            if reads.iter().any(|x| written_to.contains(x))
                || writes.iter().any(|x| read_from.contains(x))
            {
                // add delay, reset lists

                // select new group
                (*closed_group) += 1;
                let new_group = Group {
                    id: ID::Arbitrary(*closed_group),
                };

                // add spawn trigger
                create_spawn_trigger(
                    *trigger,
                    new_group,
                    Some(current_group),
                    0.05,
                    objects,
                    &mut new_network,
                    (false, true),
                );

                current_group = new_group;
                new_network.insert(
                    current_group,
                    TriggerGang {
                        triggers: Vec::new(),
                        connections_in: 1,
                    },
                );
                written_to.clear();
                read_from.clear();

            // for obj in &previous_delays {
            //     if let Some(ObjParam::Number(d)) = objects[*obj].0.params.get_mut(&63) {
            //         (*d) += 0.05;
            //     } else {
            //         unreachable!()
            //     }
            // }
            } else {
                written_to.extend(writes);
                read_from.extend(reads);
            }

            // get mutable ref to delay
            // match trigger.role {
            //     TriggerRole::Func => {
            //         // add spawn trigger
            //         (*closed_group) += 1;
            //         let new_group = Group {
            //             id: ID::Arbitrary(*closed_group),
            //         };
            //         let target = if let ObjParam::Group(g) = &objects[trigger.obj].0.params[&51] {
            //             g
            //         } else {
            //             unreachable!()
            //         };

            //         create_spawn_trigger(
            //             *trigger,
            //             *target,
            //             Some(*group),
            //             0.0,
            //             objects,
            //             &mut new_network,
            //             (false, true),
            //         );

            //         (*objects[trigger.obj].0.params.get_mut(&51).unwrap()) =
            //             ObjParam::Group(new_group);

            //         new_network.insert(new_group, TriggerGang::new(vec![*trigger]));
            //     }
            //     TriggerRole::Spawn => {
            //         // use existing
            //         match objects[trigger.obj].0.params.get_mut(&63) {
            //             Some(ObjParam::Number(_)) => (),
            //             _ => {
            //                 objects[trigger.obj]
            //                     .0
            //                     .params
            //                     .insert(63, ObjParam::Number(0.0));
            //             }
            //         };
            //         previous_delays.push(trigger.obj);
            //         (*new_network.get_mut(&current_group).unwrap())
            //             .triggers
            //             .push(*trigger)
            //     }
            //     TriggerRole::Output => (*new_network.get_mut(&current_group).unwrap())
            //         .triggers
            //         .push(*trigger),
            // };

            (*new_network.get_mut(&current_group).unwrap())
                .triggers
                .push(*trigger);

            //change object group
            // TODO: enforce single group on trigger

            (*objects)[trigger.obj]
                .0
                .params
                .insert(57, ObjParam::Group(current_group));
        }
    }
    new_network
}
