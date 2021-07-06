use aes::cipher::generic_array::typenum::Less;

use crate::ast::ObjectMode;
use crate::builtin::{Block, Group, Item, ID};
use crate::compiler_types::FunctionID;
use crate::levelstring::{GDObj, ObjParam};
use std::cmp::{self, max, min};
use std::collections::btree_map::Range;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TriggerRole {
    // Spawn triggers have their own catagory
    // because they can be combined by adding their delays
    Spawn,

    // Instant count triggers have their own category
    // because they can be simplified by performing "algebra" on them
    Operator,

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
        1811 => TriggerRole::Operator,
        1595 | 1611 | 1815 | 1812 => TriggerRole::Func,
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
        sync_group: obj.sync_group,
        sync_part: obj.sync_part,
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
                let mut hd = false;
                if let Some(ObjParam::Bool(hd_val)) = obj.params.get(&103) {
                    hd = *hd_val;
                }
                let trigger = Trigger {
                    obj: (f, o),
                    role: get_role(*id as u16, hd),
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
            //if let Some(ObjParam::Number(id)) = obj.params.get(&1) {
            //if get_role(*id as u16) != TriggerRole::Output {
            if let Some(ObjParam::Group(id)) = obj.params.get(&51) {
                if let Some(gang) = network.get_mut(id) {
                    (*gang).connections_in += 1;
                }
            }
            //}
            //}
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

    // let len = if let Some(gang) = network.get(&NO_GROUP) {
    //     gang.triggers.len()
    // } else {
    //     0
    // };
    // for i in 0..len {
    //     let trigger = network[&NO_GROUP].triggers[i];

    //     // if trigger.optimized {
    //     //     continue;
    //     // }

    //     if trigger.role != TriggerRole::Output {
    //         optimize_from(&mut network, &mut objects, (NO_GROUP, i), &mut closed_group);
    //     } else {
    //         (*network.get_mut(&NO_GROUP).unwrap()).triggers[i].deleted = false;
    //     }
    // }

    for (group, gang) in network.clone() {
        if let ID::Specific(_) = group.id {
            for (i, trigger) in gang.triggers.iter().enumerate() {
                if trigger.role != TriggerRole::Output {
                    optimize_from(&mut network, &mut objects, (group, i), &mut closed_group);
                } else {
                    (*network.get_mut(&group).unwrap()).triggers[i].deleted = false;
                }
            }
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
            //ObjParam::Group(g) => IDData::Group(*g),
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

    //println!("{}", network[&start.0].connections_in);

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
                TriggerRole::Operator => {
                    if optimize_from(network, objects, trigger_ptr, closed_group) {
                        out.insert(target_out);
                    }
                }
                //  => {
                //     match get_instant_count_network(
                //         network,
                //         objects,
                //         trigger_ptr,
                //         ignore_optimized,
                //         closed_group,
                //         HashSet::new(),
                //     ) {
                //         None => {
                //             (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1]
                //                 .deleted = false;
                //             out.insert(target_out);
                //         }
                //         Some(target_list) => {
                //             for (target_group, expr) in target_list {
                //                 if build_instant_count_network(
                //                     network,
                //                     objects,
                //                     Some(start.0),
                //                     target_group,
                //                     expr,
                //                     trigger,
                //                     closed_group,
                //                 ) {
                //                     out.insert((target_group, delay));
                //                 }
                //             }
                //         }
                //     }
                // }
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
    reference_trigger: Trigger,
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
        func_id: reference_trigger.obj.0,
        mode: ObjectMode::Trigger,
        unique_id: objects[reference_trigger.obj].0.unique_id,
        sync_group: 0,
        sync_part: 0,
    };

    (*objects.list)[reference_trigger.obj.0]
        .obj_list
        .push((new_obj.clone(), reference_trigger.order));

    let obj_index = (
        reference_trigger.obj.0,
        objects.list[reference_trigger.obj.0].obj_list.len() - 1,
    );
    let new_trigger = Trigger {
        obj: obj_index,
        optimized: settings.0,
        deleted: settings.1,
        role: TriggerRole::Spawn,
        ..reference_trigger
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

    if trigger.role == TriggerRole::Operator {
        // check for other ic triggers in the same group
        let mut ic_triggers = Vec::new();

        for (i, t) in (*network.get_mut(&start.0).unwrap().triggers)
            .iter_mut()
            .enumerate()
        {
            if t.role == TriggerRole::Operator {
                ic_triggers.push(i);
            }
        }

        let mut networks = HashMap::<Group, IcExpr>::new();
        let mut insert_to_networks = |(g, expr): (Group, IcExpr)| {
            if let Some(start_expr) = networks.get_mut(&g) {
                *start_expr = IcExpr::Or(start_expr.clone().into(), expr.into());
            } else {
                networks.insert(g, expr);
            }
        };

        for i in ic_triggers {
            let targets = match get_instant_count_network(
                network,
                objects,
                (start.0, i),
                false,
                closed_group,
                HashSet::new(),
            ) {
                None => None,
                Some(target_list) => {
                    let len = target_list.len();
                    for target in target_list {
                        insert_to_networks(target)
                    }
                    Some(len)
                }
            };
            let t = network
                .get_mut(&start.0)
                .unwrap()
                .triggers
                .get_mut(i)
                .unwrap();
            (*t).optimized = true;

            if let Some(targets) = targets {
                if targets > 0 {
                    (*t).deleted = true;
                } else {
                    (*t).deleted = false;
                }
            } else {
                (*t).deleted = false;
            }
        }

        if networks.is_empty() {
            return false;
        }
        let mut count = 0;
        for (target_group, expr) in networks {
            let expr = simplify_ic_expr_full(expr);
            if expr != IcExpr::False
                && build_instant_count_network(
                    network,
                    objects,
                    Some(start.0),
                    target_group,
                    expr,
                    trigger,
                    closed_group,
                )
            {
                count += 1;
            }
        }

        count > 0
    } else {
        let targets = get_targets(network, objects, start, 0, false, closed_group);
        let trigger = network[&start.0].triggers[start.1];

        if let Some(targets) = targets {
            if targets.is_empty() {
                return false;
            }

            if (trigger.role == TriggerRole::Func) && targets.len() == 1 && targets[0].1 == 0 {
                let new_trigger = clone_trigger(trigger, network, objects);
                objects[new_trigger.obj]
                    .0
                    .params
                    .insert(51, ObjParam::Group(targets[0].0));
                return true;
            }
            // group that the trigger spawns
            let spawn_group =
                if trigger.role == TriggerRole::Func || trigger.role == TriggerRole::Operator {
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
                }
            }
            true
        } else {
            (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
            true
        }
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

// instant count algebra :pog:
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IcExpr {
    Or(Box<IcExpr>, Box<IcExpr>),
    And(Box<IcExpr>, Box<IcExpr>),
    True,
    False,
    Equals(Item, i32),
    MoreThan(Item, i32),
    LessThan(Item, i32),
}

impl IcExpr {
    fn flatten_and(&self) -> Vec<Self> {
        match &self {
            &Self::And(a, b) => a
                .flatten_and()
                .into_iter()
                .chain(b.flatten_and().into_iter())
                .collect(),
            a => vec![(*a).clone()],
        }
    }

    fn flatten_or(&self) -> Vec<Self> {
        match &self {
            &Self::Or(a, b) => a
                .flatten_or()
                .into_iter()
                .chain(b.flatten_or().into_iter())
                .collect(),
            a => vec![(*a).clone()],
        }
    }

    fn stack_and(list: Vec<Self>) -> Self {
        let mut iter = list.iter();
        let mut out = iter.next().unwrap().clone();
        for expr in list {
            out = Self::And(out.into(), expr.remove_duplicates().into());
        }
        out
    }
    fn stack_or(list: Vec<Self>) -> Self {
        let mut iter = list.iter();
        let mut out = iter.next().unwrap().clone();
        for expr in list {
            out = Self::Or(out.into(), expr.remove_duplicates().into());
        }
        out
    }

    fn remove_duplicates(&self) -> Self {
        match self {
            Self::And(_, _) => {
                let list = self.flatten_and();
                let set: HashSet<_> = list.iter().collect();
                let mut set_iter = set.iter().cloned();

                let mut out = (*set_iter.next().unwrap()).remove_duplicates();
                for expr in set_iter {
                    out = Self::And(out.into(), expr.remove_duplicates().into());
                }
                out
            }
            Self::Or(_, _) => {
                let list = self.flatten_or();
                let set: HashSet<_> = list.iter().collect();
                let mut set_iter = set.iter().cloned();

                let mut out = (*set_iter.next().unwrap()).remove_duplicates();
                for expr in set_iter {
                    out = Self::Or(out.into(), expr.remove_duplicates().into());
                }
                out
            }
            a => a.clone(),
        }
    }

    fn decrease_and(&self) -> Self {
        // remove duplicates before using this
        match self {
            Self::Or(a, b) => {
                if let Self::And(a1, b1) = *a.clone() {
                    if let Self::And(a2, b2) = *b.clone() {
                        let a1 = a1.decrease_and();
                        let b1 = b1.decrease_and();
                        let a2 = a2.decrease_and();
                        let b2 = b2.decrease_and();
                        if a1 == a2 {
                            Self::And(a1.into(), Self::Or(b1.into(), b2.into()).into())
                        } else if a1 == b2 {
                            Self::And(a1.into(), Self::Or(b1.into(), a2.into()).into())
                        } else if b1 == a2 {
                            Self::And(b1.into(), Self::Or(a1.into(), b2.into()).into())
                        } else if b1 == b2 {
                            Self::And(b1.into(), Self::Or(a1.into(), a2.into()).into())
                        } else {
                            Self::Or(a.decrease_and().into(), b.decrease_and().into())
                        }
                    } else {
                        Self::Or(a.decrease_and().into(), b.decrease_and().into())
                    }
                } else {
                    Self::Or(a.decrease_and().into(), b.decrease_and().into())
                }
            }
            Self::And(a, b) => Self::And(a.decrease_and().into(), b.decrease_and().into()),
            a => a.clone(),
        }
    }
}

// fn more_or_equally_limiting(expr1: IcExpr, expr2: IcExpr) -> Option<bool> {
//     //this function takes two icexprs that are NOT and nor or
//     // and have THE SAME ITEM

//     // true = first argument
//     // false = second argument
//     use IcExpr::*;
//     match (expr1, expr2) {
//         (Equals(_, num1), Equals(_, num2)) => {
//             if num1 == num2 {
//                 return Some(true);
//             } else {
//                 return None;
//             }
//         }
//         (MoreThan(_, num1), MoreThan(_, num2)) => Some(num1 > num2),
//         (LessThan(_, num1), LessThan(_, num2)) => Some(num1 < num2),
//         (LessThan(_, num1), MoreThan(_, num2)) | (MoreThan(_, num2), LessThan(_, num1)) => {
//             if num1 <= num2 + 1 {
//                 return None;
//             } else {
//                 return Some(true);
//             }
//         }
//         (Equals(_, num1), MoreThan(_, num2)) | (MoreThan(_, num2), Equals(_, num1)) => {
//             if num1 > num2 {
//                 return Some(true);
//             } else {
//                 return None;
//             }
//         }
//         (Equals(_, num1), LessThan(_, num2)) | (LessThan(_, num2), Equals(_, num1)) => {
//             if num1 < num2 {
//                 return Some(true);
//             } else {
//                 return None;
//             }
//         }
//         (_, _) => unreachable!(),
//     }
// }

fn overlap(expr: IcExpr) -> IcExpr {
    //only allow the most limiting condition for each item
    let mut most_limiting = IcExpr::True;

    use IcExpr::*;
    for c in expr.flatten_and() {
        let expr1 = match simplify_ic_expr(c) {
            True => continue,
            False => return False,
            a => a,
        };
        let expr2 = most_limiting.clone();
        let base_expr = And(Box::from(expr1.clone()), Box::from(expr2.clone()));
        match (expr1.clone(), expr2.clone()) {
            (Equals(item1, num1), Equals(item2, num2)) => {
                if item1 == item2 {
                    if num1 != num2 {
                        return False;
                    }
                } else {
                    most_limiting = base_expr;
                }
            }
            (MoreThan(item1, num1), MoreThan(item2, num2)) => {
                if item1 == item2 {
                    most_limiting = MoreThan(item1, max(num1, num2));
                } else {
                    most_limiting = base_expr;
                }
            }
            (LessThan(item1, num1), LessThan(item2, num2)) => {
                if item1 == item2 {
                    most_limiting = LessThan(item1, min(num1, num2));
                } else {
                    most_limiting = base_expr;
                }
            }
            (LessThan(item1, num1), MoreThan(item2, num2))
            | (MoreThan(item2, num2), LessThan(item1, num1)) => {
                if item1 == item2 && num1 <= num2 + 1 {
                    return False;
                } else if item1 == item2 && num1 == num2 + 2 {
                    most_limiting = Equals(item1, num2 + 1);
                } else {
                    most_limiting = base_expr;
                }
            }
            (Equals(item1, num1), MoreThan(item2, num2))
            | (MoreThan(item2, num2), Equals(item1, num1)) => {
                if item1 == item2 {
                    if num1 > num2 {
                        most_limiting = Equals(item1, num1);
                    } else {
                        return False;
                    }
                } else {
                    most_limiting = Equals(item1, num1);
                }
            }
            (Equals(item1, num1), LessThan(item2, num2))
            | (LessThan(item2, num2), Equals(item1, num1)) => {
                if item1 == item2 {
                    if num1 < num2 {
                        most_limiting = Equals(item1, num1);
                    } else {
                        return False;
                    }
                } else {
                    most_limiting = Equals(item1, num1);
                }
            }
            (Or(e1, e2), a) | (a, Or(e1, e2)) => {
                if overlap(And(a.clone().into(), e1)) == a
                    && overlap(And(a.clone().into(), e2)) == a
                {
                    most_limiting.insert(item, a);
                } else {
                    most_limiting.insert(item, base_expr);
                }
            }

            (_, _) => {
                most_limiting.insert(item, base_expr);
            }
        }
    }

    most_limiting
}

fn union(expr: IcExpr) -> IcExpr {
    //only allow the least limiting condition for each item
    let mut least_limiting = HashMap::<Item, IcExpr>::new();
    let mut out_conditions = Vec::new();
    use IcExpr::*;
    for c in expr.flatten_or() {
        let expr1 = match simplify_ic_expr(c) {
            False => continue,
            True => return True,
            a => a,
        };
        let (expr2, item) = match expr1 {
            Equals(item, _) | MoreThan(item, _) | LessThan(item, _) => {
                match least_limiting.get(&item) {
                    None => {
                        least_limiting.insert(item, expr1);
                        continue;
                    }
                    Some(e) => (e.clone(), item),
                }
            }
            a => {
                out_conditions.push(a);
                continue;
            }
        };
        let base_expr = Or(Box::from(expr1.clone()), Box::from(expr2.clone()));
        match (expr1.clone(), expr2.clone()) {
            (Equals(_, num1), Equals(_, num2)) => {
                if num1 != num2 {
                    least_limiting.insert(item, base_expr);
                }
            }
            (MoreThan(_, num1), MoreThan(_, num2)) => {
                least_limiting.insert(item, MoreThan(item, min(num1, num2)));
            }
            (LessThan(_, num1), LessThan(_, num2)) => {
                least_limiting.insert(item, LessThan(item, max(num1, num2)));
            }
            (LessThan(_, num1), MoreThan(_, num2)) | (MoreThan(_, num2), LessThan(_, num1)) => {
                if num1 == num2 - 1 {
                    return True;
                }
                least_limiting.insert(item, base_expr);
            }
            (Equals(_, num1), MoreThan(_, num2)) | (MoreThan(_, num2), Equals(_, num1)) => {
                if num1 > num2 {
                    least_limiting.insert(item, MoreThan(item, num2));
                } else {
                    least_limiting.insert(item, base_expr);
                }
            }
            (Equals(_, num1), LessThan(_, num2)) | (LessThan(_, num2), Equals(_, num1)) => {
                if num1 < num2 {
                    least_limiting.insert(item, LessThan(item, num2));
                } else {
                    least_limiting.insert(item, base_expr);
                }
            }
            (And(e1, e2), a) | (a, And(e1, e2)) => {
                if union(And(a.clone().into(), e1)) == a || union(And(a.clone().into(), e2)) == a {
                    least_limiting.insert(item, a);
                } else {
                    least_limiting.insert(item, base_expr);
                }
            }

            (_, _) => {
                least_limiting.insert(item, base_expr);
            }
        }
    }
    for (_, expr) in least_limiting {
        out_conditions.push(expr);
    }

    IcExpr::stack_or(out_conditions)
}

fn simplify_ic_expr(expr: IcExpr) -> IcExpr {
    use IcExpr::*;
    match expr {
        And(_, _) => overlap(expr),
        Or(_, _) => union(expr),
        a => a,
    }
}

fn simplify_ic_expr_full(mut expr: IcExpr) -> IcExpr {
    println!("\n\nstart: {:?}\n", expr);
    expr = expr.remove_duplicates();
    expr = expr.decrease_and();
    expr = simplify_ic_expr(expr);
    println!("end: {:?}", expr);
    expr
}

fn build_instant_count_network<'a>(
    network: &'a mut TriggerNetwork,
    objects: &'a mut Triggerlist,
    start_group: Option<Group>,
    target: Group,
    expr: IcExpr,
    reference_trigger: Trigger,
    closed_group: &mut u16,
) -> bool {
    match expr {
        IcExpr::Equals(item, num) | IcExpr::MoreThan(item, num) | IcExpr::LessThan(item, num) => {
            create_instant_count_trigger(
                reference_trigger,
                target,
                start_group,
                match expr {
                    IcExpr::Equals(_, _) => 0,
                    IcExpr::MoreThan(_, _) => 1,
                    IcExpr::LessThan(_, _) => 2,
                    _ => unreachable!(),
                },
                num,
                item,
                objects,
                network,
                (true, false),
            );
            true
        }

        IcExpr::True => {
            // This can be optimized
            create_spawn_trigger(
                reference_trigger,
                target,
                start_group,
                0.0,
                objects,
                network,
                (true, false),
            );
            true
        }

        IcExpr::And(expr1, expr2) => {
            (*closed_group) += 1;
            let middle_group = Group {
                id: ID::Arbitrary(*closed_group),
            };
            if build_instant_count_network(
                network,
                objects,
                start_group,
                middle_group,
                *expr1,
                reference_trigger,
                closed_group,
            ) {
                build_instant_count_network(
                    network,
                    objects,
                    Some(middle_group),
                    target,
                    *expr2,
                    reference_trigger,
                    closed_group,
                )
            } else {
                false
            }
        }

        IcExpr::Or(expr1, expr2) => {
            let result1 = build_instant_count_network(
                network,
                objects,
                start_group,
                target,
                *expr1,
                reference_trigger,
                closed_group,
            );
            let result2 = build_instant_count_network(
                network,
                objects,
                start_group,
                target,
                *expr2,
                reference_trigger,
                closed_group,
            );
            result1 || result2
        }
        _ => unreachable!(),
    }
}

fn get_instant_count_network<'a>(
    network: &'a mut TriggerNetwork,
    objects: &'a mut Triggerlist,
    start: (Group, usize),
    ignore_optimized: bool,
    closed_group: &mut u16,
    mut visited: HashSet<(Group, usize)>,
) -> Option<Vec<(Group, IcExpr)>> {
    //u32: delay in millis
    let trigger = network.get(&start.0).unwrap().triggers[start.1];

    if visited.contains(&start) {
        if network[&start.0].triggers[start.1].deleted {
            return Some(Vec::new());
        } else {
            return None;
        }
    }

    visited.insert(start);
    let start_obj = &objects[trigger.obj].0.params;

    //println!("{}", network[&start.0].connections_in);
    assert_eq!(start_obj.get(&1), Some(&ObjParam::Number(1811.0)));
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
    let start_item = if let ObjParam::Item(i) =
        start_obj.get(&80).unwrap_or(&ObjParam::Item(Item {
            id: ID::Specific(0),
        })) {
        *i
    } else {
        Item {
            id: ID::Specific(0),
        }
    };
    let start_num =
        if let ObjParam::Number(a) = start_obj.get(&77).unwrap_or(&ObjParam::Number(0.0)) {
            *a as i32
        } else {
            0
        };
    let start_expr = match start_obj.get(&88) {
        Some(ObjParam::Number(1.0)) => IcExpr::MoreThan(start_item, start_num),
        Some(ObjParam::Number(2.0)) => IcExpr::LessThan(start_item, start_num),
        _ => IcExpr::Equals(start_item, start_num),
    };

    let mut out = HashSet::<(Group, IcExpr)>::new();

    for (i, g) in list {
        let trigger_ptr = (g, i);
        let trigger = network[&trigger_ptr.0].triggers[trigger_ptr.1];

        //let full_trigger_ptr = (trigger_ptr.0, trigger_ptr.1, full_delay);
        let target_out = (trigger_ptr.0, start_expr.clone());

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
                TriggerRole::Operator => {
                    match get_instant_count_network(
                        network,
                        objects,
                        trigger_ptr,
                        ignore_optimized,
                        closed_group,
                        visited.clone(),
                    ) {
                        Some(children) => {
                            for el in children.iter().map(|(g, expr)| {
                                (
                                    *g,
                                    IcExpr::And(
                                        Box::from(start_expr.clone()),
                                        Box::from(expr.clone()),
                                    ),
                                )
                            }) {
                                out.insert(el);
                            }
                        }
                        None => {
                            (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1]
                                .deleted = false;
                            out.insert(target_out);
                        }
                    }
                }

                _ => {
                    if optimize_from(network, objects, trigger_ptr, closed_group) {
                        (*network.get_mut(&trigger_ptr.0).unwrap()).triggers[trigger_ptr.1]
                            .deleted = false;
                        out.insert(target_out);
                    }
                }
            }
        }
    }

    (*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = true;

    Some(out.iter().map(|(a, b)| (*a, b.clone())).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let A = Item {
            id: ID::Specific(1),
        };
        let B = Item {
            id: ID::Specific(2),
        };
        let C = Item {
            id: ID::Specific(3),
        };
        use IcExpr::*;

        let mut expr = And(
            Or(LessThan(A, 1).into(), MoreThan(A, 1).into()).into(),
            Equals(A, 1).into(),
        );
        println!("start: {:?}\n", expr);
        expr = expr.remove_duplicates();
        println!("duplicates removed: {:?}\n", expr);
        expr = expr.decrease_and();
        println!("ands decreased: {:?}\n", expr);

        /*
        duplicates removed:
        Or(And(MoreThan(B, 2), Equals(C, 2)), And(LessThan(C, 2), MoreThan(B, 2)))

        ands decreased: And(MoreThan(B, 2), Or(Equals(C, 2), LessThan(C, 2)))

        simplified: Some(And(MoreThan(B, 2), LessThan(C, 3)))

        ((B > 2) && (C == 2)) || ((B > 2) && (C < 2))

        (B > 2) && ((C == 2) || (C < 2))

        (B > 2) && (C < 3)

        thats pretty epic

        */

        println!("simplified: {:?}\n", simplify_ic_expr(expr));
    }
}

fn create_instant_count_trigger(
    reference_trigger: Trigger,
    target_group: Group,
    group: Option<Group>,
    operation: u8,
    num: i32,
    item: Item,
    objects: &mut Triggerlist,
    network: &mut TriggerNetwork,
    //         opt   del
    settings: (bool, bool),
) {
    let mut new_obj_map = HashMap::new();
    new_obj_map.insert(1, ObjParam::Number(1811.0));
    new_obj_map.insert(51, ObjParam::Group(target_group));
    new_obj_map.insert(80, ObjParam::Item(item));
    new_obj_map.insert(77, ObjParam::Number(num.into()));
    new_obj_map.insert(88, ObjParam::Number(operation.into()));

    if let Some(g) = group {
        new_obj_map.insert(57, ObjParam::Group(g));
    }

    let new_obj = GDObj {
        params: new_obj_map,
        func_id: reference_trigger.obj.0,
        mode: ObjectMode::Trigger,
        unique_id: objects[reference_trigger.obj].0.unique_id,
        sync_group: 0,
        sync_part: 0,
    };

    (*objects.list)[reference_trigger.obj.0]
        .obj_list
        .push((new_obj.clone(), reference_trigger.order));

    let obj_index = (
        reference_trigger.obj.0,
        objects.list[reference_trigger.obj.0].obj_list.len() - 1,
    );
    let new_trigger = Trigger {
        obj: obj_index,
        optimized: settings.0,
        deleted: settings.1,
        role: TriggerRole::Operator,
        ..reference_trigger
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
