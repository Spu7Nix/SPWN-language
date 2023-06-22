use super::optimize::is_start_group;
use super::{obj_props, ReservedIds, TriggerNetwork, TriggerRole, Triggerlist};
use crate::gd::gd_object::ObjParam;
use crate::gd::ids::Id;

// performes a DFS from each start group, and removes all triggers that:
// - dont lead to an output trigger
// - are not reachable from a start group

pub fn dead_code_optimization(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    //closed_group: &mut u16,
    reserved: &ReservedIds,
) {
    for (group, gang) in network.map.clone() {
        if is_start_group(group, reserved) {
            for (i, _) in gang.triggers.iter().enumerate() {
                let mut visited = Vec::new();
                if check_for_dead_code(
                    network,
                    objects,
                    (group, i),
                    //closed_group,
                    reserved,
                    &mut visited,
                ) == DeadCodeResult::Keep
                {
                    network.map.get_mut(&group).unwrap().triggers[i].deleted = false;
                }
            }
        }
    }
}

#[derive(PartialEq, Eq)]
enum DeadCodeResult {
    Keep,
    Delete,
}

#[must_use]
fn check_for_dead_code(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    start: (Id, usize),
    //closed_group: &mut u16,
    reserved: &ReservedIds,
    visited_stack: &mut Vec<(Id, usize)>,
) -> DeadCodeResult {
    use DeadCodeResult::*;
    //returns whether to keep or delete the trigger
    let trigger = network.map[&start.0].triggers[start.1];
    if !trigger.deleted {
        return Keep;
    }

    if trigger.role == TriggerRole::Output {
        if let Some(ObjParam::Group(i @ Id::Arbitrary(_))) =
            objects[trigger.obj].params().get(&obj_props::TARGET)
        {
            if !reserved.object_groups.contains(i) && !reserved.trigger_groups.contains(i) {
                return Delete;
            }
        }
        network.map.get_mut(&start.0).unwrap().triggers[start.1].deleted = false;
        return Keep;
    }

    if visited_stack.contains(&start) {
        return Keep; // keep all loops
    }

    // if trigger is an output trigger, keep this branch

    let start_obj = &objects[trigger.obj].params();

    //println!("{}", network[&start.0].connections_in);

    let list: Vec<(usize, Id)> = if let Some(ObjParam::Group(g)) = start_obj.get(&obj_props::TARGET)
    {
        if is_start_group(*g, reserved) {
            //(*network.get_mut(&start.0).unwrap()).triggers[start.1].deleted = false;
            return Keep;
        } else if let Some(gang) = network.map.get(g) {
            if gang.triggers.is_empty() {
                return Delete;
            }

            vec![*g; gang.triggers.len()]
                .iter()
                .copied()
                .enumerate()
                .collect()
        } else {
            //dangling

            return Delete;
        }
    } else {
        //dangling

        return Delete;
    };

    let mut out = Delete;

    visited_stack.push(start);

    for (i, g) in list {
        let trigger_ptr = (g, i);

        if check_for_dead_code(
            network,
            objects,
            trigger_ptr,
            //closed_group,
            reserved,
            visited_stack,
        ) == Keep
        {
            network.map.get_mut(&trigger_ptr.0).unwrap().triggers[trigger_ptr.1].deleted = false;
            out = Keep;
        }
    }

    assert_eq!(visited_stack.pop(), Some(start));

    out
}
