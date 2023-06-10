use ahash::{AHashMap, AHashSet};

use super::optimize::is_start_group;
use super::{
    obj_ids, obj_props, ObjPtr, ReservedIds, Trigger, TriggerGang, TriggerNetwork, TriggerRole,
    Triggerlist, NO_GROUP,
};
use crate::gd::gd_object::{GdObject, ObjParam, TriggerObject, TriggerOrder};
use crate::gd::ids::Id;
use crate::parsing::ast::ObjectType;

pub fn group_toggling(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    reserved: &ReservedIds,
    closed_group: &mut u16,
) {
    let mut visited = AHashSet::default();
    for group in network.map.clone().keys() {
        if is_start_group(*group, reserved) {
            intraframe_grouping(
                network,
                objects,
                reserved,
                closed_group,
                GroupingInput::Group(*group),
                Vec::new(),
                &mut visited,
                None,
            );
        }
    }
}

// intraframe sync grouping :pog:

#[derive(Debug)]
enum GroupingInput {
    Group(Id),
    ObjList(Vec<(Trigger, f64)>, Id), // main group
}
#[allow(clippy::too_many_arguments)]
fn intraframe_grouping(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    reserved: &ReservedIds,
    closed_group: &mut u16,
    input: GroupingInput,
    additional_groups: Vec<Id>,
    visited: &mut AHashSet<Id>,
    toggle_groups: Option<(Id, Id)>,
) {
    // if let GroupingInput::ObjList(li, _) = &input {
    //     let disp = li
    //         .iter()
    //         .map(|(t, b)| (*t, objects[t.obj].1, *b))
    //         .collect::<Vec<_>>();
    //     dbg!(disp);
    // }
    //dbg!(&input);
    let (sorted, main_group) = match input {
        GroupingInput::Group(input) => {
            if visited.contains(&input) {
                return;
            }
            visited.insert(input);
            let gang = &network.map[&input];
            let mut sorted = gang.triggers.clone();
            sorted.sort_by(|a, b| {
                objects[a.obj]
                    .order
                    .0
                    .partial_cmp(&objects[b.obj].order.0)
                    .unwrap()
            });
            let mut with_betweens = Vec::new();
            for i in 0..(sorted.len() - 1) {
                with_betweens.push((
                    sorted[i],
                    objects[sorted[i + 1].obj].order.0 - objects[sorted[i].obj].order.0,
                ))
            }

            with_betweens.push((*sorted.last().unwrap(), 1.0));
            (with_betweens, input)
        },
        GroupingInput::ObjList(l, main) => {
            // if visited.contains(&main) {
            //     return;
            // }
            let mut sorted = l;
            sorted.sort_by(|a, b| {
                objects[a.0.obj]
                    .order
                    .0
                    .partial_cmp(&objects[b.0.obj].order.0)
                    .unwrap()
            });
            (sorted, main)
        },
    };

    let mut groupable_triggers = Vec::new();
    let mut ungroupable = Vec::new();

    for (trigger, between) in &sorted {
        let mut grouped = false;
        if let Some(ObjParam::Number(id)) = objects[trigger.obj].params().get(&1) {
            if *id as u16 == 1811 {
                // only works with instant count

                if let Some(ObjParam::Group(target)) =
                    objects[trigger.obj].params().get(&obj_props::TARGET)
                {
                    if !is_start_group(*target, reserved)
                        && network.map[target].connections_in == 1
                        && network.map[target].triggers.iter().all(|t| {
                            t.role == TriggerRole::Output
                                || if let Some(ObjParam::Number(n)) =
                                    objects[t.obj].params().get(&1)
                                {
                                    let id = *n as u16;
                                    id == 1811 || id == 1268
                                } else {
                                    false
                                }
                        })
                    {
                        groupable_triggers.push((*trigger, *between));
                        grouped = true;
                    }
                }
            }
        };
        if !grouped {
            ungroupable.push((*trigger, *between));
        }
    }

    //dbg!(&groupable_triggers, &ungroupable);

    if groupable_triggers.len() >= 3 {
        group_triggers(
            groupable_triggers,
            network,
            objects,
            main_group,
            closed_group,
            reserved,
            additional_groups,
            visited,
            toggle_groups,
        );
    } else {
        ungroupable.extend(groupable_triggers);
    }

    for (trigger, _) in ungroupable {
        if trigger.role == TriggerRole::Func || trigger.role == TriggerRole::Spawn {
            let obj = &objects[trigger.obj];
            if let Some(&ObjParam::Group(g)) = obj.params().get(&obj_props::TARGET) {
                if !is_start_group(g, reserved) {
                    intraframe_grouping(
                        network,
                        objects,
                        reserved,
                        closed_group,
                        GroupingInput::Group(g),
                        Vec::new(),
                        visited,
                        None,
                    );
                }
            }
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn group_triggers(
    triggers: Vec<(Trigger, f64)>, // sorted
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    group: Id,
    closed_group: &mut u16,
    reserved: &ReservedIds,
    additional_groups: Vec<Id>,
    visited: &mut AHashSet<Id>,
    toggle_groups: Option<(Id, Id)>,
) {
    let mut get_new_group = || {
        (*closed_group) += 1;
        Id::Arbitrary(*closed_group)
    };

    // let disp2 = triggers
    //     .iter()
    //     .map(|(t, b)| (*t, objects[t.obj].1, *b))
    //     .collect::<Vec<_>>();
    // dbg!(disp2);

    // let mut add_group = |trigger, group| {
    //     if let Some(param) = objects[trigger].0.params.get_mut(&obj_props::GROUPS) {
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
    let (swapping_group, output_group) = if let Some(a) = toggle_groups {
        a
    } else {
        (get_new_group(), get_new_group())
    };

    let can_recurse_further = additional_groups.len() < 5;

    let recursion_groups = (get_new_group(), get_new_group());

    for (trigger, between) in triggers.iter() {
        let trigger = &trigger.obj;
        let mut all_outputs = Vec::<Trigger>::new();

        let order = objects[*trigger].order;

        if let Some(ObjParam::Group(target)) =
            objects[*trigger].params_mut().get_mut(&obj_props::TARGET)
        {
            all_outputs.extend(network.map[target].triggers.iter().copied());
            for t in &mut network.map.get_mut(target).unwrap().triggers {
                t.deleted = true;
            }

            *target = output_group; // enable output
        } else {
            unreachable!()
        };
        for output in all_outputs.iter_mut() {
            output.deleted = false;
            let new_obj = objects[output.obj].clone();

            (*objects.list).push(new_obj);

            let obj_index = ObjPtr(objects.list.len() - 1);

            output.obj = obj_index;
        }
        for trigger in all_outputs.iter() {
            if let Some(param) = objects[trigger.obj]
                .params_mut()
                .get_mut(&obj_props::GROUPS)
            {
                // check if it already has multiple groups
                let mut groups = vec![main_group, output_group, swapping_group];
                groups.extend(additional_groups.iter().copied());
                *param = ObjParam::GroupList(groups);
            }
        }

        all_outputs.sort();

        //dbg!(&all_outputs);
        let spacing = 0.0001;
        let mut current_order = order.0 + spacing;
        let delta = (between - spacing * 2.0) / (all_outputs.len() as f64);

        for trigger in all_outputs.iter() {
            objects[trigger.obj].order = TriggerOrder(current_order);
            current_order += delta;
        }

        if can_recurse_further {
            let mut new_add_groups = additional_groups.clone();
            new_add_groups.push(main_group);
            new_add_groups.push(output_group);
            new_add_groups.push(swapping_group);
            intraframe_grouping(
                network,
                objects,
                reserved,
                closed_group,
                GroupingInput::ObjList(
                    all_outputs.iter().map(|a| (*a, delta)).collect(),
                    main_group,
                ),
                new_add_groups,
                visited,
                Some(recursion_groups),
            );
        } else {
            for output in all_outputs.iter().copied() {
                if output.role == TriggerRole::Func || output.role == TriggerRole::Spawn {
                    // let spawn_delay = objects[output.obj].0.params.get(&63).cloned();
                    // let is_instant = match (output.role, spawn_delay) {
                    //     (TriggerRole::Spawn, Some(ObjParam::Number(n))) => n < 0.001,
                    //     (TriggerRole::Spawn, Some(_)) => false,
                    //     (TriggerRole::Spawn, None) => true,
                    //     _ => true,
                    // };

                    if let Some(ObjParam::Group(g)) =
                        objects[output.obj].params_mut().get_mut(&obj_props::TARGET)
                    {
                        if !is_start_group(*g, reserved) {
                            let orig_group = *g;
                            // if network[g].connections_in == 1 && !visited.contains(g) && is_instant
                            // {
                            //     let shared_group = recursion_groups.0;
                            //     *g = shared_group;
                            //     let mut obj_list = network[&orig_group].triggers.clone();
                            //     for t in obj_list.iter().copied() {
                            //         match objects[t.obj].0.params.get_mut(&obj_props::GROUPS) {
                            //             Some(ObjParam::GroupList(l)) => {
                            //                 for g in l {
                            //                     if *g == orig_group {
                            //                         *g = shared_group;
                            //                     }
                            //                 }
                            //             }
                            //             Some(ObjParam::Group(g)) => *g = shared_group,
                            //             _ => (),
                            //         };
                            //     }

                            //     obj_list.sort_by(|a, b| {
                            //         objects[a.obj].1.partial_cmp(&objects[b.obj].1).unwrap()
                            //     });
                            //     let mut with_betweens = Vec::new();
                            //     for i in 0..(obj_list.len() - 1) {
                            //         with_betweens.push((
                            //             obj_list[i],
                            //             objects[obj_list[i + 1].obj].1 .0
                            //                 - objects[obj_list[i].obj].1 .0,
                            //         ))
                            //     }
                            //     visited.insert(orig_group);
                            //     with_betweens.push((*obj_list.last().unwrap(), 1.0));

                            //     intraframe_grouping(
                            //         network,
                            //         objects,
                            //         reserved,
                            //         closed_group,
                            //         GroupingInput::ObjList(with_betweens, shared_group),
                            //         Vec::new(),
                            //         visited,
                            //         None,
                            //     );
                            // } else {
                            intraframe_grouping(
                                network,
                                objects,
                                reserved,
                                closed_group,
                                GroupingInput::Group(orig_group),
                                Vec::new(),
                                visited,
                                None,
                            );
                            //}
                        }
                    }
                }
            }
        }

        network
            .map
            .get_mut(&main_group)
            .unwrap()
            .triggers
            .extend(all_outputs);
        let delta = spacing / 17.0;

        let mut toggle_trigger_groups = vec![main_group];
        toggle_trigger_groups.extend(additional_groups.iter().copied());
        // create toggle triggers
        create_toggle_trigger(
            swapping_group,
            toggle_trigger_groups.clone(),
            false,
            objects,
            network,
            TriggerOrder(order.0 - delta), // before the function trigger
        );
        create_toggle_trigger(
            output_group,
            toggle_trigger_groups.clone(),
            false,
            objects,
            network,
            TriggerOrder(order.0 - delta), // before the function trigger
        );
        create_toggle_trigger(
            swapping_group,
            toggle_trigger_groups.clone(),
            true,
            objects,
            network,
            TriggerOrder(order.0 + delta), // after the function trigger
        );
    }
}

pub fn create_toggle_trigger(
    target_group: Id,
    groups: Vec<Id>,
    enable: bool,
    objects: &mut Triggerlist,
    network: &mut TriggerNetwork,
    order: TriggerOrder,
) {
    let mut new_obj_map = AHashMap::default();
    new_obj_map.insert(1, ObjParam::Number(obj_ids::TOGGLE as f64));
    new_obj_map.insert(obj_props::TARGET, ObjParam::Group(target_group));
    new_obj_map.insert(56, ObjParam::Bool(enable));

    new_obj_map.insert(obj_props::GROUPS, ObjParam::GroupList(groups));

    let new_obj = GdObject {
        params: new_obj_map,
        //func_id: obj.0,
        mode: ObjectType::Trigger,
        //unique_id: objects[obj].0.unique_id,
    };

    (*objects.list).push(TriggerObject {
        obj: new_obj.clone(),
        order,
    });

    let obj_index = ObjPtr(objects.list.len() - 1);
    let new_trigger = Trigger {
        obj: obj_index,

        deleted: false,
        role: TriggerRole::Output,
    };

    if let Some(ObjParam::Group(group)) = new_obj.params.get(&obj_props::GROUPS) {
        match network.map.get_mut(group) {
            Some(gang) => gang.triggers.push(new_trigger),
            None => {
                network
                    .map
                    .insert(*group, TriggerGang::new(vec![new_trigger]));
            },
        }
    } else {
        match network.map.get_mut(&NO_GROUP) {
            Some(gang) => gang.triggers.push(new_trigger),
            None => {
                network
                    .map
                    .insert(NO_GROUP, TriggerGang::new(vec![new_trigger]));
            },
        }
    }
}
