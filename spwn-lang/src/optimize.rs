use crate::ast::ObjectMode;
use crate::builtin::{Block, Group, Id, Item};
use crate::compiler_types::FunctionId;

//mod icalgebra;
use crate::levelstring::{GdObj, ObjParam};

use std::collections::{HashMap, HashSet};

pub type Swaps = HashMap<Group, Group>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum TriggerRole {
    // Spawn triggers have their own catagory
    // because they can be combined by adding their delays
    Spawn,

    // Triggers like move and rotate, which have some output in the level
    // and therefore cannot be optimized away
    Output,

    // Triggers that send a signal, but don't cause any side effects
    Func,
}

fn get_role(obj_id: u16, hd: bool) -> TriggerRole {
    match obj_id {
        1268 => {
            if hd {
                TriggerRole::Func
            } else {
                TriggerRole::Spawn
            }
        }
        1595 | 1611 | 1811 | 1815 | 1812 => TriggerRole::Func,
        _ => TriggerRole::Output,
    }
}

type ObjPtr = (usize, usize);
//                                     triggers      connections in
pub type TriggerNetwork = HashMap<Group, TriggerGang>;

#[derive(Debug, Clone)]
// what do you mean? its a trigger gang!
pub struct TriggerGang {
    pub triggers: Vec<Trigger>,
    pub connections_in: u32,
    // wether any of the connections in are not instant count triggers
    pub non_ic_triggers_in: bool,
}

impl TriggerGang {
    fn new(triggers: Vec<Trigger>) -> Self {
        TriggerGang {
            triggers,
            connections_in: 0,
            non_ic_triggers_in: false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Trigger {
    pub obj: ObjPtr,
    pub role: TriggerRole,
    pub order: usize,
    pub deleted: bool,
    pub optimized: bool,
}

pub struct Triggerlist<'a> {
    list: &'a mut Vec<FunctionId>,
}

impl<'a> std::ops::Index<ObjPtr> for Triggerlist<'a> {
    type Output = (GdObj, usize);

    fn index(&self, i: ObjPtr) -> &Self::Output {
        &self.list[i.0].obj_list[i.1]
    }
}
impl<'a> std::ops::IndexMut<ObjPtr> for Triggerlist<'a> {
    fn index_mut(&mut self, i: ObjPtr) -> &mut Self::Output {
        &mut self.list[i.0].obj_list[i.1]
    }
}

const NO_GROUP: Group = Group {
    id: Id::Specific(0),
};

pub fn optimize(mut obj_in: Vec<FunctionId>, mut closed_group: u16) -> Vec<FunctionId> {
    let mut network = TriggerNetwork::new();

    // sort all triggers by their group
    for (f, fnid) in obj_in.iter().enumerate() {
        for (o, (obj, order)) in fnid.obj_list.iter().enumerate() {
            if let Some(ObjParam::Number(id)) = obj.params.get(&1) {
                let mut hd = false;
                if let Some(ObjParam::Bool(hd_val)) = obj.params.get(&103) {
                    hd = *hd_val;
                }
                let trigger = Trigger {
                    obj: (f, o),
                    role: get_role(*id as u16, hd),
                    order: *order,
                    deleted: false,
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

    //optimize
    //optimize_network(&mut network);

    let mut objects = Triggerlist { list: &mut obj_in };

    clean_network(&mut network, &objects, true);

    // fix read write order
    // not an optimization, more like a consistancy fix
    // also, like nothing works without this, so i should probably move
    // this somewhere else if i want to add an option to not have optimization
    network = fix_read_write_order(&mut objects, &network, &mut closed_group);

    // round 1
    spawn_and_dead_code_optimization(&mut network, &mut objects, &mut closed_group);

    // clean_network(&mut network, &objects, false);

    // instant_count_optimization(&mut network, &mut objects, &mut closed_group);

    // // //cleanup

    // clean_network(&mut network, &objects, false);

    // // // // round 2

    // spawn_and_dead_code_optimization(&mut network, &mut objects, &mut closed_group);

    // clean_network(&mut network, &objects, false);

    //instant_count_optimization(&mut network, &mut objects, &mut closed_group);

    rebuild(&network, &obj_in)
}

fn spawn_and_dead_code_optimization(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    closed_group: &mut u16,
) {
    let mut swaps = HashMap::new();
    for (group, gang) in network.clone() {
        if let Id::Specific(_) = group.id {
            for (i, trigger) in gang.triggers.iter().enumerate() {
                if trigger.role != TriggerRole::Output {
                    let (_, new_swaps) = optimize_from(network, objects, (group, i), closed_group);
                    swaps.extend(new_swaps);
                } else {
                    (*network.get_mut(&group).unwrap()).triggers[i].deleted = false;
                }
            }
        }
    }
    replace_groups(swaps, objects);
}

fn clean_network(network: &mut TriggerNetwork, objects: &Triggerlist, delete_objects: bool) {
    let mut new_network = TriggerNetwork::new();

    for (_, gang) in network.iter() {
        let new_triggers: Vec<Trigger> = gang
            .triggers
            .iter()
            .filter(|a| !a.deleted)
            .map(|a| Trigger {
                optimized: false,
                deleted: delete_objects,
                ..*a
            })
            .collect();

        for trigger in new_triggers {
            let obj = &objects[trigger.obj].0;
            if let Some(ObjParam::Group(group)) = obj.params.get(&57) {
                match new_network.get_mut(group) {
                    Some(l) => (*l).triggers.push(trigger),
                    None => {
                        new_network.insert(*group, TriggerGang::new(vec![trigger]));
                    }
                }
            } else {
                match new_network.get_mut(&NO_GROUP) {
                    Some(l) => (*l).triggers.push(trigger),
                    None => {
                        new_network.insert(NO_GROUP, TriggerGang::new(vec![trigger]));
                    }
                }
            }
        }
    }

    for (_, gang) in new_network.clone() {
        for trigger in gang.triggers {
            let obj = &objects[trigger.obj].0;
            if let Some(ObjParam::Group(id)) = obj.params.get(&51) {
                if let Some(gang) = new_network.get_mut(id) {
                    (*gang).connections_in += 1;

                    if let Some(ObjParam::Number(objid)) = obj.params.get(&1) {
                        if *objid as i16 != 1811 {
                            (*gang).non_ic_triggers_in = true;
                        }
                    } else {
                        (*gang).non_ic_triggers_in = true;
                    }
                }
            }
        }
    }

    *network = new_network;
}

// fn instant_count_optimization(
//     network: &mut TriggerNetwork,
//     objects: &mut Triggerlist,
//     closed_group: &mut u16,
// ) {
//     use icalgebra::{build_ic_connections, get_all_ic_connections};
//     let c = get_all_ic_connections(network, &objects);
//     let swaps = build_ic_connections(network, objects, closed_group, c);
//     replace_groups(swaps, network, objects);
// }

pub fn replace_groups(table: Swaps, objects: &mut Triggerlist) {
    for fn_id in objects.list.iter_mut() {
        for (object, _) in &mut fn_id.obj_list {
            for param in &mut object.params.values_mut() {
                match param {
                    ObjParam::Group(g) => {
                        if let Some(to) = table.get(g) {
                            *g = *to;
                        }
                    }
                    ObjParam::GroupList(list) => {
                        for g in list {
                            if let Some(to) = table.get(g) {
                                *g = *to;
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    // let mut new_network = TriggerNetwork::new();
    // for (group, gang) in network.iter() {
    //     let new_group = if let Some(new) = table.get(group) {
    //         new
    //     } else {
    //         group
    //     };
    //     new_network.insert(*new_group, gang.clone());
    // }

    // *network = new_network;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum IdData {
    //Group(Group),
    Block(Block),
    Item(Item),
}

fn reads_writes(t: Trigger, objects: &Triggerlist) -> (Vec<IdData>, Vec<IdData>) {
    let role = t.role;
    let obj = &objects[t.obj].0;
    let mut out = (Vec::new(), Vec::new());
    for (key, val) in &obj.params {
        let id_data = match val {
            //ObjParam::Group(g) => IDData::Group(*g),
            ObjParam::Block(b) => IdData::Block(*b),
            ObjParam::Item(i) => IdData::Item(*i),
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
) -> (Option<Vec<(Group, u32)>>, Swaps) {
    //u32: delay in millis

    let mut swaps = HashMap::new();

    let trigger = network.get(&start.0).unwrap().triggers[start.1];
    let start_obj = &objects[trigger.obj].0.params;

    if network[&start.0].triggers[start.1].optimized {
        if network[&start.0].triggers[start.1].deleted {
            return (Some(Vec::new()), swaps);
        } else {
            // if its a spawn trigger, go to targets anyways
            return (None, swaps);
        }
    }

    (*network.get_mut(&start.0).unwrap()).triggers[start.1].optimized = true;

    //println!("{}", network[&start.0].connections_in);

    let list: Vec<(usize, Group)>;

    if let Some(ObjParam::Group(g)) = start_obj.get(&51) {
        if let Id::Specific(_) = g.id {
            //(*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
            return (Some(vec![(*g, delay)]), swaps);
        }

        if let Some(gang) = network.get(g) {
            if gang.triggers.is_empty() {
                return (Some(Vec::new()), swaps);
            }
            list = vec![*g; gang.triggers.len()]
                .iter()
                .copied()
                .enumerate()
                .collect();
        } else {
            //dangeling

            return (Some(Vec::new()), swaps);
        }
    } else {
        //dangeling

        return (Some(Vec::new()), swaps);
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
            let (keep, new_swaps) = optimize_from(network, objects, trigger_ptr, closed_group);

            swaps.extend(new_swaps);
            if keep {
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
                    let (keep, new_swaps) =
                        optimize_from(network, objects, trigger_ptr, closed_group);
                    swaps.extend(new_swaps);
                    if keep {
                        (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1]
                            .deleted = false;
                        out.insert(target_out);
                    }
                }
                TriggerRole::Spawn => {
                    let (result, new_swaps) = get_targets(
                        network,
                        objects,
                        trigger_ptr,
                        delay + added_delay,
                        ignore_optimized,
                        closed_group,
                    );
                    swaps.extend(new_swaps);
                    match result {
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

    (Some(out.iter().copied().collect()), swaps)
}

pub fn create_spawn_trigger(
    trigger: Trigger,
    target_group: Group,
    group: Group,
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

    new_obj_map.insert(57, ObjParam::Group(group));

    let new_obj = GdObj {
        params: new_obj_map,
        func_id: trigger.obj.0,
        mode: ObjectMode::Trigger,
        unique_id: objects[trigger.obj].0.unique_id,
        sync_group: 0,
        sync_part: 0,
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

#[must_use]
fn optimize_from<'a>(
    network: &'a mut TriggerNetwork,
    objects: &mut Triggerlist,
    start: (Group, usize),
    closed_group: &mut u16,
) -> (bool, Swaps) {
    //returns weather to keep or delete the trigger
    let mut swaps = HashMap::new();

    let trigger = network[&start.0].triggers[start.1];
    if trigger.role == TriggerRole::Output {
        (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
        return (true, swaps);
    }

    if trigger.optimized {
        return (!trigger.deleted, swaps);
    }

    //let role = trigger.role;

    let (targets, new_swaps) = get_targets(network, objects, start, 0, false, closed_group);
    let trigger = network[&start.0].triggers[start.1];

    swaps.extend(new_swaps);

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

    //println!("targets: {:?}", targets);

    if let Some(targets) = targets {
        if targets.is_empty() {
            return (false, swaps);
        }

        if (trigger.role == TriggerRole::Func) && targets.len() == 1 && targets[0].1 == 0
        //&& network[&start.0].connections_in > 1
        {
            //let new_trigger = clone_trigger(trigger, network, objects);
            objects[trigger.obj]
                .0
                .params
                .insert(51, ObjParam::Group(targets[0].0));
            (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
            (*network.get_mut(&start.0).unwrap()).triggers[start.1].optimized = true;
            return (true, swaps);
        }

        let spawn_group = if trigger.role == TriggerRole::Func {
            (*closed_group) += 1;
            let new_group = Group {
                id: Id::Arbitrary(*closed_group),
            };

            objects[trigger.obj]
                .0
                .params
                .insert(51, ObjParam::Group(new_group));

            (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;

            new_group
        } else {
            match objects[trigger.obj].0.params.get(&57) {
                Some(ObjParam::Group(g)) => *g,
                _ => NO_GROUP,
            }
        };
        let mut delay_map: HashMap<u32, Vec<Group>> = HashMap::new();
        for (group, delay) in targets {
            if let Some(list) = delay_map.get_mut(&delay) {
                list.push(group);
            } else {
                delay_map.insert(delay, vec![group]);
            }
        }

        for (delay, targets) in delay_map {
            // let can_be_combined = targets
            //     .iter()
            //     .filter(|a| network[a].connections_in == 1)
            //     .collect::<Vec<_>>();
            // if !can_be_combined.is_empty() {
            //     if delay == 0 {
            //         swaps.extend(can_be_combined.iter().map(|a| (**a, spawn_group)));
            //     } else {
            //         (*closed_group) += 1;
            //         let new_group = Group {
            //             id: Id::Arbitrary(*closed_group),
            //         };
            //         create_spawn_trigger(
            //             trigger,
            //             new_group,
            //             spawn_group,
            //             delay as f64 / 1000.0,
            //             objects,
            //             network,
            //             (true, false),
            //         );
            //         swaps.extend(can_be_combined.iter().map(|a| (**a, new_group)));
            //     }
            // }
            // can't be combined
            for g in targets
            // .iter()
            // .filter(|a| network[a].connections_in != 1)
            // .collect::<Vec<_>>()
            // .iter()
            {
                if network[&g].connections_in == 1 && delay == 0 {
                    swaps.insert(g, spawn_group);
                } else {
                    create_spawn_trigger(
                        trigger,
                        g,
                        spawn_group,
                        delay as f64 / 1000.0,
                        objects,
                        network,
                        (true, false),
                    );
                }
            }
        }

        (true, swaps)
    } else {
        (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
        (true, swaps)
    }
}

fn rebuild(network: &TriggerNetwork, orig_structure: &[FunctionId]) -> Vec<FunctionId> {
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
                ..*gang
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
                    id: Id::Arbitrary(*closed_group),
                };

                // add spawn trigger
                create_spawn_trigger(
                    *trigger,
                    new_group,
                    current_group,
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
                        non_ic_triggers_in: true,
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
