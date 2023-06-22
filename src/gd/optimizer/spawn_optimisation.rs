use ahash::{AHashMap, AHashSet};

use super::optimize::{create_spawn_trigger, is_start_group, replace_groups, ToggleGroups};
use super::{
    obj_props, ReservedIds, Swaps, Trigger, TriggerNetwork, TriggerRole, Triggerlist, NO_GROUP,
};
use crate::gd::gd_object::ObjParam;
use crate::gd::ids::Id;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct SpawnDelay {
    pub(crate) delay: u32,
    pub(crate) epsiloned: bool,
}

#[derive(Debug)]
struct Connection {
    start_group: Id,
    end_group: Id,
    delay: SpawnDelay,
    trigger: Trigger,
}

#[derive(Debug)]
struct SpawnTrigger {
    target: Id,
    delay: SpawnDelay,
    trigger: Trigger,
}
// fn can_toggle_on(obj: &GdObj) -> bool {
//     if let Some(ObjParam::Number(obj_id)) = obj.params.get(&1) {
//         match *obj_id as u16 {
//             obj_ids::TOUCH
//             | obj_ids::COUNT
//             | obj_ids::INSTANT_COUNT
//             | obj_ids::COLLISION
//             | obj_ids::ON_DEATH => {
//                 if let Some(ObjParam::Bool(false)) | None =
//                     obj.params.get(&obj_props::ACTIVATE_GROUP)
//                 {
//                     false
//                 } else {
//                     matches!(obj.params.get(&obj_props::TARGET), Some(_))
//                 }
//             }
//             _ => false,
//         }
//     } else {
//         false
//     }
// }

// spawn trigger optimisation

// attempts to remove as many spawn triggers as possible by
// - combining them into a single spawn trigger (with their combined delay)
// - removing the spawn trigger if it is not needed (if it has 0 delay)

pub(crate) fn spawn_optimisation(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    reserved: &ReservedIds,
    toggle_groups: &ToggleGroups,
) {
    let mut spawn_connections = AHashMap::<Id, Vec<SpawnTrigger>>::default();
    let mut inputs = AHashSet::<Id>::default();
    let mut outputs = AHashSet::<Id>::default();

    let mut cycle_points = AHashSet::<Id>::default();
    let mut all = Vec::new();

    for (group, gang) in network.map.iter_mut() {
        let output_condition = gang.triggers.iter().any(|t| t.role != TriggerRole::Spawn);
        if output_condition {
            outputs.insert(*group);
        }
        for trigger in &mut gang.triggers {
            let obj = &objects[trigger.obj].params();

            if trigger.role == TriggerRole::Spawn {
                // dont include ones that dont activate a group

                let target = match obj.get(&obj_props::TARGET) {
                    Some(ObjParam::Group(g)) => *g,

                    _ => continue,
                };

                if gang.non_spawn_triggers_in || *group == NO_GROUP {
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
                trigger.deleted = true;

                if let Some(l) = spawn_connections.get_mut(group) {
                    l.push(SpawnTrigger {
                        target,
                        delay,
                        trigger: *trigger,
                    })
                } else {
                    spawn_connections.insert(
                        *group,
                        vec![SpawnTrigger {
                            target,
                            delay,
                            trigger: *trigger,
                        }],
                    );
                }
            }
        }
    }

    //bg!(&spawn_connections, &all);

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

    //dbg!(&spawn_connections, &all);

    // println!(
    //     "spawn_triggers: {:?}\n\n inputs: {:?}\n\n outputs: {:?}\n",
    //     spawn_connections, inputs, outputs
    // );

    // go from every trigger in an input group and get every possible path to an
    // output group (stopping if it reaches a group already visited)

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

    //dbg!(&all);

    let mut deduped = AHashMap::default();

    for Connection {
        start_group,
        end_group,
        delay,
        trigger,
    } in all
    {
        deduped.insert((start_group, end_group, delay), trigger);
    }

    let mut swaps = Swaps::default();

    // let mut start_counts = FnvHashMap::default();
    // let mut end_counts = FnvHashMap::default();

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
        let mut plain_trigger = |network| {
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
        };

        let mut insert_to_swaps = |a: Id, b: Id, objects: &mut Triggerlist| {
            let order = objects[trigger.obj].order;
            for v in swaps.values_mut() {
                if v.0 == a {
                    *v = (b, order);
                }
            }
            assert!(swaps.insert(a, (b, order)).is_none());
        };

        let default = &AHashSet::default();
        let targeters = network.connectors.get(&start).unwrap_or(default);

        let start_can_toggle_off = if let Some(togglers) = toggle_groups.toggles_off.get(&start) {
            // if the togglers are a subset of the targeters, we can safely remove the spawn trigger
            !togglers.iter().all(|x| targeters.contains(x))
        } else {
            false
        };

        // let end_can_toggle_off = if let Some(togglers) = toggle_groups.toggles_off.get(&end) {
        //     // if the togglers are a subset of the targeters, we can safely remove the spawn trigger
        //     !togglers.iter().all(|x| targeters.contains(x))
        // } else {
        //     false
        // };

        if start_can_toggle_off
            || (toggle_groups.toggles_on.contains_key(&start)
                && toggle_groups.toggles_off.contains_key(&end))
            || toggle_groups.stops.contains_key(&end)
        {
            plain_trigger(network)
        } else if d == 0 && !is_start_group(end, reserved) && network.map[&end].connections_in == 1
        {
            //dbg!(end, start);
            insert_to_swaps(end, start, objects);
        } else if d == 0 && !is_start_group(start, reserved)
                && network.map[&start].connections_in == 1 //??
                && (network.map[&start].triggers.is_empty()
                    || network.map[&start].triggers.iter().all(|t| t.deleted))
        {
            insert_to_swaps(start, end, objects);
        } else {
            plain_trigger(network)
        }
    }
    //dbg!(&swaps);

    replace_groups(swaps, objects);
}

// set triggers that make cycles to inputs and outputs
fn look_for_cycle(
    current: Id,
    ictriggers: &AHashMap<Id, Vec<SpawnTrigger>>,
    visited: &mut Vec<Id>,
    inputs: &mut AHashSet<Id>,
    outputs: &mut AHashSet<Id>,
    cycle_points: &mut AHashSet<Id>,
    all: &mut Vec<Connection>,
) {
    if let Some(connections) = ictriggers.get(&current) {
        for SpawnTrigger {
            target: g,
            delay,
            trigger,
        } in connections
        {
            if visited.contains(g) {
                //println!("cycle detected");
                outputs.insert(current);
                inputs.insert(*g);
                all.push(Connection {
                    start_group: current,
                    end_group: *g,
                    delay: *delay,
                    trigger: *trigger,
                });
                cycle_points.insert(current);

                return;
            }

            visited.push(current);

            look_for_cycle(*g, ictriggers, visited, inputs, outputs, cycle_points, all);

            assert_eq!(visited.pop(), Some(current));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn traverse(
    current: Id,
    origin: Id,
    total_delay: SpawnDelay, // delay from the origin to the current trigger
    trigger: Option<Trigger>,
    outputs: &AHashSet<Id>,
    cycle_points: &AHashSet<Id>,
    spawn_connections: &AHashMap<Id, Vec<SpawnTrigger>>,
    visited: &mut Vec<Id>,
    all: &mut Vec<Connection>,
) {
    if visited.contains(&current) {
        unreachable!()
    }

    if let Some(connections) = spawn_connections.get(&current) {
        for SpawnTrigger {
            target: g,
            delay: d,
            trigger: t2,
        } in connections
        {
            //println!("{:?} -> {:?}", current, g);
            let new_delay = SpawnDelay {
                delay: total_delay.delay + d.delay,
                epsiloned: total_delay.epsiloned || d.epsiloned,
            };
            visited.push(current);
            if outputs.contains(g) {
                all.push(Connection {
                    start_group: origin,
                    end_group: *g,
                    delay: new_delay,
                    trigger: trigger.unwrap_or(*t2),
                });

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
                    trigger,
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
        all.push(Connection {
            start_group: origin,
            end_group: current,
            delay: total_delay,
            trigger: t,
        }) //?
    } else {
        //unreachable!();
        assert!(outputs.contains(&current));
    }
}
