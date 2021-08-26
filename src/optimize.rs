use crate::ast::ObjectMode;
use crate::builtin::{Block, Group, Id, Item};
use crate::compiler_types::FunctionId;

//mod icalgebra;
use crate::levelstring::{GdObj, ObjParam, TriggerOrder};

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};

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

#[derive(Debug, Clone)]
pub struct ReservedIds {
    pub object_groups: HashSet<Id>,
    pub trigger_groups: HashSet<Id>, // only includes the 57 prop

    pub object_colors: HashSet<Id>,

    pub object_blocks: HashSet<Id>,

    pub object_items: HashSet<Id>,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjPtr(usize, usize);
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
    pub deleted: bool,
}

pub struct Triggerlist<'a> {
    list: &'a mut Vec<FunctionId>,
}

impl<'a> std::ops::Index<ObjPtr> for Triggerlist<'a> {
    type Output = (GdObj, TriggerOrder);

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
    mut reserved: ReservedIds,
) -> Vec<FunctionId> {
    let mut network = TriggerNetwork::new();

    // sort all triggers by their group
    for (f, fnid) in obj_in.iter().enumerate() {
        for (o, (obj, _)) in fnid.obj_list.iter().enumerate() {
            if let Some(ObjParam::Number(id)) = obj.params.get(&1) {
                let mut hd = false;
                if let Some(ObjParam::Bool(hd_val)) = obj.params.get(&103) {
                    hd = *hd_val;
                }
                let trigger = Trigger {
                    obj: ObjPtr(f, o),
                    role: get_role(*id as u16, hd),
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
    //network = fix_read_write_order(&mut objects, &network, &mut closed_group);

    // round 1

    dead_code_optimization(&mut network, &mut objects, &mut closed_group, &reserved);

    clean_network(&mut network, &objects, false);

    spawn_optimisation(&mut network, &mut objects, &reserved);

    clean_network(&mut network, &objects, true);

    update_reserved(&mut network, &mut objects, &mut reserved);

    dead_code_optimization(&mut network, &mut objects, &mut closed_group, &reserved);

    clean_network(&mut network, &objects, false);

    spawn_optimisation(&mut network, &mut objects, &reserved);

    clean_network(&mut network, &objects, false);

    // dedup_triggers(&mut network, &mut objects, reserved);

    // clean_network(&mut network, &objects, false);

    intraframe_grouping(&mut network, &mut objects, &reserved, &mut closed_group);
    let zero_group = Group {
        id: Id::Specific(0),
    };
    if network[&zero_group].triggers.len() > 1 {
        closed_group += 1;
        let new_start_group = Group {
            id: Id::Arbitrary(closed_group),
        };

        let mut swaps = Swaps::new();
        swaps.insert(zero_group, new_start_group);

        replace_groups(swaps, &mut objects);

        create_spawn_trigger(
            Trigger {
                obj: ObjPtr(0, 0), // arbitrary object
                role: TriggerRole::Spawn,
                deleted: false,
            },
            new_start_group,
            zero_group,
            0.0,
            &mut objects,
            &mut network,
            TriggerRole::Spawn,
            false,
        );
    }

    rebuild(&network, &obj_in)
}

fn is_start_group(g: Group, reserved: &ReservedIds) -> bool {
    matches!(g.id, Id::Specific(_)) || reserved.object_groups.contains(&g.id)
}

fn dead_code_optimization(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    closed_group: &mut u16,
    reserved: &ReservedIds,
) {
    for (group, gang) in network.clone() {
        if is_start_group(group, reserved) {
            for (i, _) in gang.triggers.iter().enumerate() {
                let mut visited = Vec::new();
                if check_for_dead_code(
                    network,
                    objects,
                    (group, i),
                    closed_group,
                    reserved,
                    &mut visited,
                    0,
                ) {
                    (*network.get_mut(&group).unwrap()).triggers[i].deleted = false;
                }
            }
        }
    }
}

fn update_reserved(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,

    reserved: &mut ReservedIds,
) {
    reserved.trigger_groups.clear();

    for gang in network.values() {
        for trigger in gang.triggers.iter() {
            for (prop, param) in objects[trigger.obj].0.params.iter() {
                if *prop == 57 {
                    match &param {
                        ObjParam::Group(g) => {
                            reserved.trigger_groups.insert(g.id);
                        }
                        ObjParam::GroupList(g) => {
                            reserved.trigger_groups.extend(g.iter().map(|g| g.id));
                        }

                        _ => (),
                    }
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
    reserved: &ReservedIds,
    visited_stack: &mut Vec<(Group, usize)>,
    d: u32,
) -> bool {
    //returns whether to keep or delete the trigger
    let trigger = network[&start.0].triggers[start.1];
    if !trigger.deleted {
        return true;
    }

    if trigger.role == TriggerRole::Output {
        if let Some(ObjParam::Group(Group {
            id: i @ Id::Arbitrary(_),
        })) = objects[trigger.obj].0.params.get(&51)
        {
            // maybe restrict this to only stop triggers and toggle triggers
            if !reserved.trigger_groups.contains(i) && !reserved.object_groups.contains(i) {
                return false;
            }
        }
        (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
        return true;
    }

    if visited_stack.contains(&start) {
        return true; // keep all loops
    }

    // if trigger is an output trigger, keep this branch

    let start_obj = &objects[trigger.obj].0.params;

    //println!("{}", network[&start.0].connections_in);

    let list: Vec<(usize, Group)> = if let Some(ObjParam::Group(g)) = start_obj.get(&51) {
        if is_start_group(*g, reserved) {
            //(*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
            return true;
        } else if let Some(gang) = network.get(g) {
            if gang.triggers.is_empty() {
                return false;
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

    visited_stack.push(start);

    for (i, g) in list {
        let trigger_ptr = (g, i);

        if check_for_dead_code(
            network,
            objects,
            trigger_ptr,
            closed_group,
            reserved,
            visited_stack,
            d + 1,
        ) {
            (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1].deleted = false;
            out = true;
        }
    }

    assert_eq!(visited_stack.pop(), Some(start));

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
    role: TriggerRole,
    deleted: bool,
) {
    let mut new_obj_map = HashMap::new();
    new_obj_map.insert(1, ObjParam::Number(1268.0));
    new_obj_map.insert(51, ObjParam::Group(target_group));
    new_obj_map.insert(63, ObjParam::Number(delay));

    new_obj_map.insert(57, ObjParam::Group(group));

    let order = objects[trigger.obj].1;

    let new_obj = GdObj {
        params: new_obj_map,
        func_id: trigger.obj.0,
        mode: ObjectMode::Trigger,
        unique_id: objects[trigger.obj].0.unique_id,
    };

    (*objects.list)[trigger.obj.0]
        .obj_list
        .push((new_obj.clone(), order));

    let obj_index = ObjPtr(
        trigger.obj.0,
        objects.list[trigger.obj.0].obj_list.len() - 1,
    );
    let new_trigger = Trigger {
        obj: obj_index,

        deleted,
        role,
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

// not needed lol
fn fix_read_write_order(
    objects: &mut Triggerlist,
    network: &TriggerNetwork,
    closed_group: &mut u16,
) -> TriggerNetwork {
    let mut new_network = TriggerNetwork::new();
    for (group, gang) in network {
        let mut written_to = HashSet::new();
        let mut read_from = HashSet::new();
        let current_group = *group;

        new_network.insert(
            current_group,
            TriggerGang {
                triggers: Vec::new(),
                ..*gang
            },
        );
        let mut sorted = gang.triggers.clone();
        sorted.sort_by(|a, b| objects[a.obj].1.partial_cmp(&objects[b.obj].1).unwrap());

        for trigger in &sorted {
            let (reads, writes) = reads_writes(*trigger, objects);
            written_to.extend(writes);
            read_from.extend(reads);
        }

        //let mut previous_delays = Vec::new();

        for trigger in &sorted {
            dbg!(objects[trigger.obj].1 .0);
            let (_, writes) = reads_writes(*trigger, objects);
            if writes.iter().any(|x| read_from.contains(x)) {
                // put trigger behinf a func spawn trigger

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
                    0.0,
                    objects,
                    &mut new_network,
                    TriggerRole::Func,
                    true,
                );

                (*objects)[trigger.obj]
                    .0
                    .params
                    .insert(57, ObjParam::Group(new_group));

                new_network.insert(
                    new_group,
                    TriggerGang {
                        triggers: vec![*trigger],
                        connections_in: 1,
                        non_spawn_triggers_in: true,
                    },
                );
            } else {
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
    }
    new_network
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct SpawnDelay {
    delay: u32,
    epsiloned: bool,
}

// spawn trigger optimisation
pub fn spawn_optimisation(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    reserved: &ReservedIds,
) {
    let mut spawn_connections = HashMap::<Group, Vec<(Group, SpawnDelay, Trigger)>>::new();
    let mut inputs = HashSet::<Group>::new();
    let mut outputs = HashSet::<Group>::new();

    let mut cycle_points = HashSet::<Group>::new();
    let mut all = Vec::new();

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

                if let Some(l) = spawn_connections.get_mut(group) {
                    l.push((target, delay, *trigger))
                } else {
                    spawn_connections.insert(*group, vec![(target, delay, *trigger)]);
                }
            }
        }
    }

    // set triggers that make cycles to inputs and outputs
    fn look_for_cycle(
        current: Group,
        ictriggers: &HashMap<Group, Vec<(Group, SpawnDelay, Trigger)>>,
        visited: &mut Vec<Group>,
        inputs: &mut HashSet<Group>,
        outputs: &mut HashSet<Group>,
        cycle_points: &mut HashSet<Group>,
        all: &mut Vec<(Group, Group, SpawnDelay, Trigger)>,
    ) {
        if let Some(connections) = ictriggers.get(&current) {
            for (g, delay, trigger) in connections {
                if visited.contains(g) {
                    //println!("cycle detected");
                    outputs.insert(current);
                    inputs.insert(*g);
                    all.push((current, *g, *delay, *trigger));
                    cycle_points.insert(current);

                    return;
                }

                visited.push(current);

                look_for_cycle(*g, ictriggers, visited, inputs, outputs, cycle_points, all);

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
            &mut cycle_points,
            &mut all,
        )
    }

    // println!(
    //     "spawn_triggers: {:?}\n\n inputs: {:?}\n\n outputs: {:?}\n",
    //     spawn_connections, inputs, outputs
    // );

    // go from every trigger in an input group and get every possible path to an
    // output group (stopping if it reaches a group already visited)
    #[allow(clippy::too_many_arguments)]
    fn traverse(
        current: Group,
        origin: Group,
        delay: SpawnDelay,
        trigger: Option<Trigger>,
        outputs: &HashSet<Group>,
        cycle_points: &HashSet<Group>,
        spawn_connections: &HashMap<Group, Vec<(Group, SpawnDelay, Trigger)>>,
        visited: &mut Vec<Group>,
        all: &mut Vec<(Group, Group, SpawnDelay, Trigger)>,
    ) {
        if visited.contains(&current) {
            unreachable!()
        }

        if let Some(connections) = spawn_connections.get(&current) {
            for (g, d, trigger) in connections {
                //println!("{:?} -> {:?}", current, g);
                let new_delay = SpawnDelay {
                    delay: delay.delay + d.delay,
                    epsiloned: delay.epsiloned || d.epsiloned,
                };
                visited.push(current);
                if outputs.contains(g) {
                    all.push((origin, *g, new_delay, *trigger));

                    /*
                    in cases like this:

                    1i.if_is(SMALLER_THAN, 1, !{

                        2i.if_is(EQUAL_TO, 0, !{
                            2i.add(1)
                            1i.if_is(SMALLER_THAN, 0, !{
                                -> BG.pulse(0, 0, 255, fade_out = 0.5)
                            })
                        })

                    })

                    we can't simplify the three expressions together, because we need the result of the 2nd one to happen before it's result
                    therefore, the chain is split before the third expression

                    it cannot add the new inputs to the set because it's used in the current loop, but it doesn't matter since the set is not used after this.

                    (this is copied from the ic trigger thing, but it makes sense here too (I think, bf doesnt work without it))
                    */

                    // avoid infinite loop
                    if !cycle_points.contains(g) {
                        traverse(
                            *g,
                            *g,
                            SpawnDelay {
                                delay: 0,
                                epsiloned: false,
                            },
                            None,
                            outputs,
                            cycle_points,
                            spawn_connections,
                            visited,
                            all,
                        );
                    }
                } else {
                    traverse(
                        *g,
                        origin,
                        new_delay,
                        Some(*trigger),
                        outputs,
                        cycle_points,
                        spawn_connections,
                        visited,
                        all,
                    );
                }
                assert_eq!(visited.pop(), Some(current));
            }
        } else if let Some(t) = trigger {
            all.push((origin, current, delay, t)) //?
        } else {
            //unreachable!();
            assert!(outputs.contains(&current));
        }
    }

    for start in inputs {
        //println!("<{:?}>", start);
        let mut visited = Vec::new();
        traverse(
            start,
            start,
            SpawnDelay {
                delay: 0,
                epsiloned: false,
            },
            None,
            &outputs,
            &cycle_points,
            &spawn_connections,
            &mut visited,
            &mut all,
        );
        //println!("</{:?}>", start);
    }

    let mut deduped = HashMap::new();

    for (start, end, delay, trigger) in all {
        deduped.insert((start, end, delay), trigger);
    }

    let mut swaps = HashMap::new();

    let mut insert_to_swaps = |a: Group, b: Group| {
        for v in swaps.values_mut() {
            if *v == a {
                *v = b;
            }
        }
        assert!(swaps.insert(a, b).is_none());
    };
    // let mut start_counts = HashMap::new();
    // let mut end_counts = HashMap::new();

    // for ((start, end, _), _) in deduped.iter() {
    //     start_counts
    //         .entry(start)
    //         .and_modify(|c| *c += 1)
    //         .or_insert(1);

    //     end_counts.entry(end).and_modify(|c| *c += 1).or_insert(1);
    // }

    for ((start, end, delay), trigger) in deduped {
        let d = if delay.delay < 50 && delay.epsiloned {
            50
        } else {
            delay.delay
        };
        if d == 0 && !is_start_group(end, reserved) && network[&end].connections_in == 1 {
            insert_to_swaps(end, start);
        } else if d == 0 && !is_start_group(start, reserved)
            && network[&start].connections_in == 1 //??
            && (network[&start].triggers.is_empty()
                || network[&start].triggers.iter().all(|t| t.deleted))
        {
            insert_to_swaps(start, end);
        } else {
            create_spawn_trigger(
                trigger,
                end,
                start,
                d as f64 / 1000.0,
                objects,
                network,
                TriggerRole::Spawn,
                false,
            )
        }
    }

    replace_groups(swaps, objects);
}

// trigger gang dedup :pog:

fn param_identifier(param: &ObjParam) -> String {
    let str = match param {
        ObjParam::Group(Group { id })
        | ObjParam::Color(crate::builtin::Color { id })
        | ObjParam::Block(Block { id })
        | ObjParam::Item(Item { id }) => match id {
            Id::Specific(id) => format!("{}", id),
            Id::Arbitrary(id) => format!("?{}", id),
        },
        ObjParam::Number(n) => {
            if (n.round() - n).abs() < 0.001 {
                format!("{}", *n as i32)
            } else {
                format!("{:.1$}", n, 3)
            }
        }
        ObjParam::Bool(b) => (if *b { "1" } else { "0" }).to_string(),
        ObjParam::Text(t) => t.to_string(),
        ObjParam::GroupList(list) => {
            let mut out = String::new();

            for g in list {
                match g.id {
                    Id::Specific(id) => out += &format!("{}.", id),
                    Id::Arbitrary(id) => out += &format!("?{}.", id),
                }
            }
            out.pop();
            out
        }
        ObjParam::Epsilon => "0.050".to_string(),
    };
    str
    // use std::collections::hash_map::DefaultHasher;
    // use std::hash::{Hash, Hasher};

    // let mut hasher = DefaultHasher::new();

    // str.hash(&mut hasher);
    // hasher.finish()
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct TriggerParam(u16, String);
impl PartialOrd for TriggerParam {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let first = self.0.cmp(&other.0);
        Some(if first == Ordering::Equal {
            self.1.cmp(&other.1)
        } else {
            first
        })
    }
}
impl Ord for TriggerParam {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct TriggerBehavior(BTreeSet<TriggerParam>);

impl Ord for TriggerBehavior {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

// TODO: make this sort by trigger order as well
impl PartialOrd for TriggerBehavior {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let mut iter1 = self.0.iter();
        let mut iter2 = other.0.iter();
        loop {
            if let Some(val1) = iter1.next() {
                if let Some(val2) = iter2.next() {
                    let cmp = val1.cmp(val2);
                    if cmp != Ordering::Equal {
                        return Some(cmp);
                    }
                } else {
                    return Some(Ordering::Greater);
                }
            } else {
                return Some(Ordering::Less);
            }
        }
    }
}

fn get_trigger_behavior(t: Trigger, objects: &Triggerlist) -> TriggerBehavior {
    let mut set = BTreeSet::new();
    for (prop, param) in &objects[t.obj].0.params {
        if *prop == 57 {
            // group
            continue;
        }
        set.insert(TriggerParam(*prop, param_identifier(param)));
    }
    TriggerBehavior(set)
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct TriggerGangBehavior(BTreeSet<TriggerBehavior>);

fn get_triggergang_behavior(gang: &TriggerGang, objects: &Triggerlist) -> TriggerGangBehavior {
    let mut set = BTreeSet::new();

    for trigger in &gang.triggers {
        set.insert(get_trigger_behavior(*trigger, objects));
    }

    TriggerGangBehavior(set)
}

pub fn dedup_triggers(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    reserved: &ReservedIds,
) {
    loop {
        let mut swaps = HashMap::new();
        let mut representative_groups = HashMap::<TriggerGangBehavior, Group>::new();

        for (group, gang) in network.iter_mut() {
            if is_start_group(*group, reserved) {
                continue;
            }
            let contains_stackable_trigger = gang.triggers.iter().any(|t| {
                let obj = &objects[t.obj].0;
                if let Some(ObjParam::Number(n)) = obj.params.get(&1) {
                    let id = *n as u16;
                    id == 901 || id == 1817
                } else {
                    false
                }
            });
            if contains_stackable_trigger {
                continue;
            }
            let behavior = get_triggergang_behavior(gang, objects);
            if let Some(repr) = representative_groups.get(&behavior) {
                // discard this gang and add a swap
                for trigger in &mut gang.triggers {
                    (*trigger).deleted = true;
                }
                //dbg!(behavior, repr, group, &representative_groups);
                assert!(swaps.insert(*group, *repr).is_none());
            } else {
                representative_groups.insert(behavior, *group);
            }
        }

        //dbg!(&swaps);

        if swaps.is_empty() {
            break;
        }
        replace_groups(swaps, objects);
        clean_network(network, objects, false);
    }
}

// intraframe sync grouping :pog:

pub fn intraframe_grouping(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    reserved: &ReservedIds,
    closed_group: &mut u16,
) {
    for (group, gang) in network.clone() {
        let mut sorted = gang.triggers;
        sorted.sort_by(|a, b| objects[a.obj].1.partial_cmp(&objects[b.obj].1).unwrap());

        let mut groupable_triggers = Vec::new();

        for trigger in &sorted {
            if let Some(ObjParam::Number(id)) = objects[trigger.obj].0.params.get(&1) {
                if *id as u16 == 1811 {
                    // only works with instant count

                    if let Some(ObjParam::Group(target)) = objects[trigger.obj].0.params.get(&51) {
                        if !is_start_group(*target, reserved)
                            && network[target].connections_in == 1
                            && network[target].triggers.iter().all(|t| {
                                t.role == TriggerRole::Output
                                // || if let Some(ObjParam::Number(n)) =
                                //     objects[t.obj].0.params.get(&1)
                                // {
                                //     let id = *n as u16;
                                //     id == 1811 || id == 1268
                                // } else {
                                //     false
                                // }
                            })
                        {
                            groupable_triggers.push(trigger.obj);
                        }
                    }
                }
            };
        }

        if groupable_triggers.len() > 4 {
            group_triggers(groupable_triggers, network, objects, group, closed_group);
        }
    }
}

fn group_triggers(
    triggers: Vec<ObjPtr>, // sorted
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    group: Group,
    closed_group: &mut u16,
) {
    let mut get_new_group = || {
        (*closed_group) += 1;
        Group {
            id: Id::Arbitrary(*closed_group),
        }
    };

    // let mut add_group = |trigger, group| {
    //     if let Some(param) = objects[trigger].0.params.get_mut(&57) {
    //         *param = ObjParam::GroupList(vec![
    //             match param {
    //                 ObjParam::Group(g) => *g,

    //                 _ => unreachable!(),
    //             },
    //             group,
    //         ])
    //     }
    // };
    let main_group = group;
    let swapping_group = get_new_group();
    let output_group = get_new_group();

    for trigger in triggers.iter() {
        let mut all_outputs = Vec::<Trigger>::new();

        let order = objects[*trigger].1;

        if let Some(ObjParam::Group(target)) = objects[*trigger].0.params.get_mut(&51) {
            all_outputs.extend(network[target].triggers.iter().copied());
            for t in &mut network.get_mut(target).unwrap().triggers {
                t.deleted = true;
            }

            *target = output_group; // enable output
        } else {
            unreachable!()
        };
        for output in all_outputs.iter_mut() {
            let new_obj = (
                GdObj {
                    func_id: trigger.0,
                    ..objects[output.obj].0.clone()
                },
                objects[output.obj].1,
            );

            (*objects.list)[trigger.0].obj_list.push(new_obj);

            let obj_index = ObjPtr(trigger.0, objects.list[trigger.0].obj_list.len() - 1);

            (*output).obj = obj_index;
        }
        for trigger in all_outputs.iter() {
            if let Some(param) = objects[trigger.obj].0.params.get_mut(&57) {
                // check if it already has multiple groups
                *param = ObjParam::GroupList(vec![main_group, output_group, swapping_group])
            }
        }

        {
            all_outputs.sort();
            let mut current_order = order.0 + 0.2;
            let delta = 0.6 / all_outputs.len() as f32;
            for trigger in all_outputs.iter() {
                current_order += delta;
                objects[trigger.obj].1 = TriggerOrder(current_order);
            }
        }
        network
            .get_mut(&main_group)
            .unwrap()
            .triggers
            .extend(all_outputs);

        // create toggle triggers
        create_toggle_trigger(
            *trigger,
            swapping_group,
            main_group,
            false,
            objects,
            network,
            TriggerOrder(order.0 - 0.1), // before the function trigger
        );
        create_toggle_trigger(
            *trigger,
            output_group,
            main_group,
            false,
            objects,
            network,
            TriggerOrder(order.0 - 0.1), // before the function trigger
        );
        create_toggle_trigger(
            *trigger,
            swapping_group,
            main_group,
            true,
            objects,
            network,
            TriggerOrder(order.0 + 0.1), // after the function trigger
        );
    }
}

pub fn create_toggle_trigger(
    obj: ObjPtr,
    target_group: Group,
    group: Group,
    enable: bool,
    objects: &mut Triggerlist,
    network: &mut TriggerNetwork,
    order: TriggerOrder,
) {
    let mut new_obj_map = HashMap::new();
    new_obj_map.insert(1, ObjParam::Number(1049.0));
    new_obj_map.insert(51, ObjParam::Group(target_group));
    new_obj_map.insert(56, ObjParam::Bool(enable));

    new_obj_map.insert(57, ObjParam::Group(group));

    let new_obj = GdObj {
        params: new_obj_map,
        func_id: obj.0,
        mode: ObjectMode::Trigger,
        unique_id: objects[obj].0.unique_id,
    };

    (*objects.list)[obj.0]
        .obj_list
        .push((new_obj.clone(), order));

    let obj_index = ObjPtr(obj.0, objects.list[obj.0].obj_list.len() - 1);
    let new_trigger = Trigger {
        obj: obj_index,

        deleted: false,
        role: TriggerRole::Output,
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
