use crate::ast::ObjectMode;
use crate::builtin::{Block, Group, Id, Item};
use crate::compiler_types::FunctionId;

//mod icalgebra;
use crate::levelstring::{GdObj, ObjParam};

use std::collections::{HashMap, HashSet};

pub type Swaps = HashMap<Group, Group>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum TriggerRole {
    // Spawn triggers have their own category
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
    pub non_spawn_triggers_in: bool,
}

impl TriggerGang {
    fn new(triggers: Vec<Trigger>) -> Self {
        TriggerGang {
            triggers,
            connections_in: 0,
            non_spawn_triggers_in: false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Trigger {
    pub obj: ObjPtr,
    pub role: TriggerRole,
    pub order: usize,
    pub deleted: bool,
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

pub fn optimize(
    mut obj_in: Vec<FunctionId>,
    mut closed_group: u16,
    reserved_groups: &HashSet<Group>,
) -> Vec<FunctionId> {
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
    // not an optimization, more like a consistency fix
    // also, like nothing works without this, so i should probably move
    // this somewhere else if i want to add an option to not have optimization
    network = fix_read_write_order(&mut objects, &network, &mut closed_group);

    // round 1
    dead_code_optimization(
        &mut network,
        &mut objects,
        &mut closed_group,
        reserved_groups,
    );

    clean_network(&mut network, &objects, false);

    spawn_optimisation(&mut network, &mut objects);

    // instant_count_optimization(&mut network, &mut objects, &mut closed_group);

    // // //cleanup

    // clean_network(&mut network, &objects, false);

    // // // // round 2

    // spawn_and_dead_code_optimization(&mut network, &mut objects, &mut closed_group);

    // clean_network(&mut network, &objects, false);

    //instant_count_optimization(&mut network, &mut objects, &mut closed_group);

    rebuild(&network, &obj_in)
}

fn dead_code_optimization(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    closed_group: &mut u16,
    reserved_groups: &HashSet<Group>,
) {
    for (group, gang) in network.clone() {
        if matches!(group.id, Id::Specific(_)) || reserved_groups.contains(&group) {
            for (i, trigger) in gang.triggers.iter().enumerate() {
                if trigger.role != TriggerRole::Output {
                    let mut visited = Vec::new();
                    if check_for_dead_code(
                        network,
                        objects,
                        (group, i),
                        closed_group,
                        reserved_groups,
                        &mut visited,
                        false,
                    ) {
                        (*network.get_mut(&group).unwrap()).triggers[i].deleted = false;
                    }
                } else {
                    (*network.get_mut(&group).unwrap()).triggers[i].deleted = false;
                }
            }
        }
    }
}

fn clean_network(network: &mut TriggerNetwork, objects: &Triggerlist, delete_objects: bool) {
    let mut new_network = TriggerNetwork::new();

    for (_, gang) in network.iter() {
        let new_triggers: Vec<Trigger> = gang
            .triggers
            .iter()
            .filter(|a| !a.deleted)
            .map(|a| Trigger {
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

                    if trigger.role != TriggerRole::Spawn {
                        (*gang).non_spawn_triggers_in = true;
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

#[must_use]
fn check_for_dead_code<'a>(
    network: &'a mut TriggerNetwork,
    objects: &mut Triggerlist,
    start: (Group, usize),
    closed_group: &mut u16,
    reserved_groups: &HashSet<Group>,
    visited_stack: &mut Vec<Group>,
    visited_output: bool,
) -> bool {
    //returns whether to keep or delete the trigger

    if visited_stack.contains(&start.0) {
        return visited_output;
    }

    // if trigger is an output trigger, keep this branch
    let trigger = network[&start.0].triggers[start.1];
    if trigger.role == TriggerRole::Output {
        (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
        return true;
    }

    let start_obj = &objects[trigger.obj].0.params;

    let mut has_output = visited_output;

    //println!("{}", network[&start.0].connections_in);

    let list: Vec<(usize, Group)> = if let Some(ObjParam::Group(g)) = start_obj.get(&51) {
        if matches!(g.id, Id::Specific(_)) || reserved_groups.contains(g) {
            //(*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
            return true;
        } else if let Some(gang) = network.get(g) {
            if gang.triggers.is_empty() {
                return false;
            }
            if !has_output && gang.triggers.iter().any(|t| t.role == TriggerRole::Output) {
                has_output = true
            }
            vec![*g; gang.triggers.len()]
                .iter()
                .copied()
                .enumerate()
                .collect()
        } else {
            //dangling

            return false;
        }
    } else {
        //dangling

        return false;
    };

    let mut out = false;

    visited_stack.push(start.0);

    for (i, g) in list {
        let trigger_ptr = (g, i);

        if check_for_dead_code(
            network,
            objects,
            trigger_ptr,
            closed_group,
            reserved_groups,
            visited_stack,
            has_output,
        ) {
            (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1].deleted = false;
            out = true;
        }
    }

    assert_eq!(visited_stack.pop(), Some(start.0));

    out
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

pub fn create_spawn_trigger(
    trigger: Trigger,
    target_group: Group,
    group: Group,
    delay: f64,
    objects: &mut Triggerlist,
    network: &mut TriggerNetwork,
    //         opt   del
    deleted: bool,
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

        deleted,
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
                    true,
                );

                current_group = new_group;
                new_network.insert(
                    current_group,
                    TriggerGang {
                        triggers: Vec::new(),
                        connections_in: 1,
                        non_spawn_triggers_in: true,
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct SpawnDelay {
    delay: u32,
    epsiloned: bool,
}

// spawn trigger optimisation
pub fn spawn_optimisation(network: &mut TriggerNetwork, objects: &mut Triggerlist) {
    let mut spawn_connections = HashMap::<Group, Vec<(Group, SpawnDelay, Trigger)>>::new();
    let mut inputs = HashSet::<Group>::new();
    let mut outputs = HashSet::<Group>::new();

    let mut to_be_subtracted_from = Vec::new();

    for (group, gang) in network.iter_mut() {
        let output_condition = gang.triggers.iter().any(|t| t.role != TriggerRole::Spawn);
        if output_condition {
            outputs.insert(*group);
        }
        for trigger in &mut gang.triggers {
            let obj = &objects[trigger.obj].0.params;

            if trigger.role == TriggerRole::Spawn {
                // dont include ones that dont activate a group

                let target = match obj.get(&51) {
                    Some(ObjParam::Group(g)) => *g,

                    _ => continue,
                };

                if gang.non_spawn_triggers_in
                    || *group
                        == (Group {
                            id: Id::Specific(0),
                        })
                {
                    inputs.insert(*group);
                }

                let delay = match obj.get(&63).unwrap_or(&ObjParam::Number(0.0)) {
                    ObjParam::Number(d) => SpawnDelay {
                        delay: (*d * 1000.0) as u32,
                        epsiloned: false,
                    },
                    ObjParam::Epsilon => SpawnDelay {
                        delay: 0,
                        epsiloned: true,
                    },
                    _ => SpawnDelay {
                        delay: 0,
                        epsiloned: false,
                    },
                };

                // delete trigger that will be rebuilt
                (*trigger).deleted = true;

                to_be_subtracted_from.push(target);

                if let Some(l) = spawn_connections.get_mut(group) {
                    l.push((target, delay, *trigger))
                } else {
                    spawn_connections.insert(*group, vec![(target, delay, *trigger)]);
                }
            }
        }
    }

    for g in &to_be_subtracted_from {
        (*network.get_mut(g).unwrap()).connections_in -= 1;
    }

    // println!(
    //     "spawn_triggers: {:?}\n\n inputs: {:?}\n\n outputs: {:?}\n",
    //     spawn_connections, inputs, outputs
    // );

    let mut all = Vec::new();
    // set triggers that make cycles to inputs and outputs
    fn look_for_cycle(
        current: Group,
        ictriggers: &HashMap<Group, Vec<(Group, SpawnDelay, Trigger)>>,
        visited: &mut Vec<Group>,
        inputs: &mut HashSet<Group>,
        outputs: &mut HashSet<Group>,
        all: &mut Vec<(Group, Group, SpawnDelay, Trigger)>,
    ) {
        if let Some(connections) = ictriggers.get(&current) {
            for (g, delay, trigger) in connections {
                if visited.contains(g) {
                    outputs.insert(current);
                    inputs.insert(*g);
                    all.push((current, *g, *delay, *trigger));

                    return;
                }

                visited.push(current);
                look_for_cycle(*g, ictriggers, visited, inputs, outputs, all);
                assert_eq!(visited.pop(), Some(current));
            }
        }
    }
    for start in inputs.clone() {
        let mut visited = Vec::new();
        look_for_cycle(
            start,
            &spawn_connections,
            &mut visited,
            &mut inputs,
            &mut outputs,
            &mut all,
        )
    }

    // go from every trigger in an input group and get every possible path to an
    // output group (stopping if it reaches a group already visited)

    fn traverse(
        current: Group,
        origin: Group,
        delay: SpawnDelay,
        trigger: Option<Trigger>,
        outputs: &HashSet<Group>,
        spawn_connections: &HashMap<Group, Vec<(Group, SpawnDelay, Trigger)>>,
        //visited: &mut Vec<Group>,
        all: &mut Vec<(Group, Group, SpawnDelay, Trigger)>,
    ) {
        // if visited.contains(&current) {
        //     unreachable!()
        // }

        if let Some(connections) = spawn_connections.get(&current) {
            for (g, d, trigger) in connections {
                //println!("{:?} -> {:?}", current, g);
                let new_delay = SpawnDelay {
                    delay: delay.delay + d.delay,
                    epsiloned: delay.epsiloned || d.epsiloned,
                };
                if outputs.contains(g) {
                    all.push((origin, *g, new_delay, *trigger));
                }
                //visited.push(current);
                traverse(
                    *g,
                    origin,
                    new_delay,
                    Some(*trigger),
                    outputs,
                    spawn_connections,
                    //visited,
                    all,
                );
                //assert_eq!(visited.pop(), Some(current));
            }
        } else if let Some(t) = trigger {
            all.push((origin, current, delay, t)) //?
        } else {
            unreachable!();
            //assert!(outputs.contains(&current));
        }
    }

    for start in inputs {
        //println!("<{:?}>", start);
        //let mut visited = Vec::new();
        traverse(
            start,
            start,
            SpawnDelay {
                delay: 0,
                epsiloned: false,
            },
            None,
            &outputs,
            &spawn_connections,
            //&mut visited,
            &mut all,
        )
        //println!("</{:?}>", start);
    }

    let mut deduped = HashMap::new();

    for (start, end, delay, trigger) in all {
        deduped.insert((start, end, delay), trigger);
    }

    //dbg!(&deduped);

    let mut swaps = HashMap::new();

    for ((start, end, delay), trigger) in deduped {
        let d = if delay.delay < 50 && delay.epsiloned {
            50
        } else {
            delay.delay
        };

        if d == 0 && network[&end].connections_in == 1 {
            assert!(swaps.insert(end, start).is_none());
        } else if d == 0 && network[&start].triggers.len() == 1 {
            assert!(swaps.insert(start, end).is_none());
        } else {
            create_spawn_trigger(
                trigger,
                end,
                start,
                d as f64 / 1000.0,
                objects,
                network,
                false,
            )
        }
    }

    replace_groups(swaps, objects);
}
