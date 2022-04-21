use compiler::builtins::{Group, Id};
use compiler::compiler_types::{FunctionId, TriggerOrder};
use parser::ast::ObjectMode;

use crate::{
    dead_code, get_role, group_toggling, obj_ids, obj_props, spawn_optimisation, trigger_dedup,
    ObjPtr, ReservedIds, Swaps, Trigger, TriggerGang, TriggerNetwork, TriggerRole, Triggerlist,
    NO_GROUP,
};

//mod icalgebra;
use compiler::leveldata::{GdObj, ObjParam};

use fnv::FnvHashMap;

pub fn optimize(
    mut obj_in: Vec<FunctionId>,
    mut closed_group: u16,
    mut reserved: ReservedIds,
) -> Vec<FunctionId> {
    let mut network = TriggerNetwork::default();

    let toggle_groups = get_toggle_groups(&obj_in);

    // sort all triggers by their group
    for (f, fnid) in obj_in.iter().enumerate() {
        for (o, (obj, _)) in fnid.obj_list.iter().enumerate() {
            //if let Some(ObjParam::Number(id)) = obj.params.get(&1) {
            // let mut hd = false;
            // if let Some(ObjParam::Bool(hd_val)) = obj.params.get(&103) {
            //     hd = *hd_val;
            // }
            let trigger = Trigger {
                obj: ObjPtr(f, o),
                role: get_role(obj),
                deleted: false,
            };
            if let Some(ObjParam::Group(group)) = obj.params.get(&obj_props::GROUPS) {
                match network.map.get_mut(group) {
                    Some(l) => (*l).triggers.push(trigger),
                    None => {
                        network.map.insert(*group, TriggerGang::new(vec![trigger]));
                    }
                }
            } else {
                match network.map.get_mut(&NO_GROUP) {
                    Some(l) => (*l).triggers.push(trigger),
                    None => {
                        network
                            .map
                            .insert(NO_GROUP, TriggerGang::new(vec![trigger]));
                    }
                }
            }
            //}
        }
    }

    let mut objects = Triggerlist { list: &mut obj_in };

    //optimize
    //optimize_network(&mut network);

    // fix read write order
    // not an optimization, more like a consistency fix
    // also, like nothing works without this, so i should probably move
    // this somewhere else if i want to add an option to not have optimization
    //network = fix_read_write_order(&mut objects, &network, &mut closed_group);

    // round 1
    //dbg!(&objects.list);

    for _ in 0..10 {
        clean_network(&mut network, &objects, true);

        dead_code::dead_code_optimization(&mut network, &mut objects, &reserved);

        clean_network(&mut network, &objects, false);
        //dbg!(&objects.list);

        spawn_optimisation::spawn_optimisation(
            &mut network,
            &mut objects,
            &reserved,
            &toggle_groups,
        );

        clean_network(&mut network, &objects, false);
        //dbg!(&objects.list);

        update_reserved(&mut network, &mut objects, &mut reserved);
    }

    clean_network(&mut network, &objects, false);

    trigger_dedup::dedup_triggers(&mut network, &mut objects, &reserved);

    clean_network(&mut network, &objects, false);

    group_toggling::group_toggling(&mut network, &mut objects, &reserved, &mut closed_group);
    //dbg!(&network);

    let zero_group = Group {
        id: Id::Specific(0),
    };
    if let Some(gang) = network.map.get(&zero_group) {
        if gang.triggers.len() > 1 {
            closed_group += 1;
            let new_start_group = Group {
                id: Id::Arbitrary(closed_group),
            };

            let mut swaps = Swaps::default();
            swaps.insert(zero_group, (new_start_group, TriggerOrder(0.0)));

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
    }

    rebuild(&network, &obj_in)
}

pub fn is_start_group(g: Group, reserved: &ReservedIds) -> bool {
    matches!(g.id, Id::Specific(_)) || reserved.object_groups.contains(&g.id)
}

#[derive(Default)]
pub struct ToggleGroups {
    pub toggles_on: fnv::FnvHashMap<Group, Vec<ObjPtr>>,
    pub toggles_off: fnv::FnvHashMap<Group, Vec<ObjPtr>>,
    pub stops: fnv::FnvHashMap<Group, Vec<ObjPtr>>,
}

fn get_toggle_groups(objects: &[FunctionId]) -> ToggleGroups {
    let mut toggle_groups = ToggleGroups::default();
    for (i, fnid) in objects.iter().enumerate() {
        for (j, (obj, _)) in fnid.obj_list.iter().enumerate() {
            if let Some(ObjParam::Number(id)) = obj.params.get(&1) {
                if let obj_ids::COUNT
                | obj_ids::INSTANT_COUNT
                | obj_ids::COLLISION
                | obj_ids::ON_DEATH
                | obj_ids::TOGGLE = *id as u16
                {
                    if let Some(ObjParam::Group(target)) = obj.params.get(&obj_props::TARGET) {
                        if let Some(ObjParam::Bool(false)) | None =
                            obj.params.get(&obj_props::ACTIVATE_GROUP)
                        {
                            &mut toggle_groups.toggles_off
                        } else {
                            &mut toggle_groups.toggles_on
                        }
                        .entry(*target)
                        .or_default()
                        .push(ObjPtr(i, j));
                    }
                } else if *id as u16 == obj_ids::TOUCH {
                    // touch triggers are kinda quirky uwu owo
                    if let Some(ObjParam::Group(target)) = obj.params.get(&obj_props::TARGET) {
                        for a in [
                            &mut toggle_groups.toggles_off,
                            &mut toggle_groups.toggles_on,
                        ] {
                            a.entry(*target).or_default().push(ObjPtr(i, j));
                        }
                    }
                } else if *id as u16 == obj_ids::STOP {
                    if let Some(ObjParam::Group(target)) = obj.params.get(&obj_props::TARGET) {
                        toggle_groups
                            .stops
                            .entry(*target)
                            .or_default()
                            .push(ObjPtr(i, j));
                    }
                }
            }
        }
    }
    toggle_groups
}

fn update_reserved(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,

    reserved: &mut ReservedIds,
) {
    reserved.trigger_groups.clear();

    for gang in network.map.values() {
        for trigger in gang.triggers.iter() {
            for (prop, param) in objects[trigger.obj].0.params.iter() {
                if *prop == obj_props::GROUPS {
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

pub fn clean_network(network: &mut TriggerNetwork, objects: &Triggerlist, delete_objects: bool) {
    let mut new_network = TriggerNetwork::default();

    for (_, gang) in network.map.iter() {
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
            if let Some(ObjParam::Group(group)) = obj.params.get(&obj_props::GROUPS) {
                match new_network.map.get_mut(group) {
                    Some(l) => (*l).triggers.push(trigger),
                    None => {
                        new_network
                            .map
                            .insert(*group, TriggerGang::new(vec![trigger]));
                    }
                }
            } else {
                match new_network.map.get_mut(&NO_GROUP) {
                    Some(l) => (*l).triggers.push(trigger),
                    None => {
                        new_network
                            .map
                            .insert(NO_GROUP, TriggerGang::new(vec![trigger]));
                    }
                }
            }
        }
    }

    // if let (TriggerRole::Func, Some(ObjParam::Group(target))) =
    //     (trigger.role, obj.params.get(&obj_props::TARGET))
    // {
    //     network
    //         .connectors
    //         .entry(*target)
    //         .or_default()
    //         .insert(trigger.obj);
    // }

    for (_, gang) in new_network.map.clone() {
        for trigger in gang.triggers {
            let obj = &objects[trigger.obj].0;
            if let (TriggerRole::Func | TriggerRole::Spawn, Some(ObjParam::Group(id))) =
                (trigger.role, obj.params.get(&obj_props::TARGET))
            {
                if let Some(gang) = new_network.map.get_mut(id) {
                    (*gang).connections_in += 1;

                    if trigger.role != TriggerRole::Spawn {
                        (*gang).non_spawn_triggers_in = true;
                    }
                }

                new_network
                    .connectors
                    .entry(*id)
                    .or_default()
                    .insert(trigger.obj);
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
    let mut map: FnvHashMap<Group, (Vec<ObjPtr>, Group, TriggerOrder)> = table
        .into_iter()
        .map(|(a, (b, c))| (a, (vec![], b, c)))
        .collect();

    for (i, fn_id) in objects.list.iter_mut().enumerate() {
        for (j, (object, _)) in &mut fn_id.obj_list.iter_mut().enumerate() {
            for (prop, param) in &mut object.params.iter_mut() {
                match param {
                    ObjParam::Group(g) => {
                        if let Some(to) = map.get_mut(g) {
                            *g = to.1;
                            if *prop == obj_props::GROUPS {
                                to.0.push(ObjPtr(i, j));
                            }
                        }
                    }
                    ObjParam::GroupList(list) => {
                        for g in list {
                            if let Some(to) = map.get_mut(g) {
                                *g = to.1;
                                if *prop == obj_props::GROUPS {
                                    to.0.push(ObjPtr(i, j));
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    //dbg!(&map);

    for (_, (mut triggers, _, order)) in map.into_iter() {
        // sort objects by trigger order
        triggers.sort_by(|a, b| objects[*a].1 .0.partial_cmp(&objects[*b].1 .0).unwrap());
        const ORDER_WINDOW: f64 = 0.1;
        let delta = ORDER_WINDOW / triggers.len() as f64;
        for (i, trigger) in triggers.iter().enumerate() {
            let new_order = TriggerOrder(order.0 + i as f64 * delta + delta);

            objects[*trigger].1 = new_order;
        }
    }
}

fn rebuild(network: &TriggerNetwork, orig_structure: &[FunctionId]) -> Vec<FunctionId> {
    let mut out = orig_structure.to_vec();
    for el in &mut out {
        (*el).obj_list.clear();
    }

    for gang in network.map.values() {
        for trigger in &gang.triggers {
            //assert!(trigger.optimized);
            if trigger.deleted {
                continue;
            }
            let (obj, order) = &orig_structure[trigger.obj.0].obj_list[trigger.obj.1];
            //let fn_id = &out[obj.func_id];
            // if it's already there, continue
            // if fn_id
            //     .obj_list
            //     .iter()
            //     .any(|x| x.0.unique_id == obj.unique_id && &x.0 == obj)
            // {
            //     continue;
            // }
            out[obj.func_id].obj_list.push((obj.clone(), *order))
        }
    }

    out
}
#[allow(clippy::too_many_arguments)]
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
    let mut new_obj_map = FnvHashMap::default();
    new_obj_map.insert(1, ObjParam::Number(1268.0));
    new_obj_map.insert(obj_props::TARGET, ObjParam::Group(target_group));
    new_obj_map.insert(63, ObjParam::Number(delay));

    new_obj_map.insert(obj_props::GROUPS, ObjParam::Group(group));

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

    if let Some(ObjParam::Group(group)) = new_obj.params.get(&obj_props::GROUPS) {
        match network.map.get_mut(group) {
            Some(gang) => (*gang).triggers.push(new_trigger),
            None => {
                network
                    .map
                    .insert(*group, TriggerGang::new(vec![new_trigger]));
            }
        }
    } else {
        match network.map.get_mut(&NO_GROUP) {
            Some(gang) => (*gang).triggers.push(new_trigger),
            None => {
                network
                    .map
                    .insert(NO_GROUP, TriggerGang::new(vec![new_trigger]));
            }
        }
    }
}
