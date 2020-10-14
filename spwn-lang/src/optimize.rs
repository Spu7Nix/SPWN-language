use crate::ast::ObjectMode;
use crate::builtin::{Group, ID};
use crate::compiler_types::FunctionID;
use crate::levelstring::{GDObj, ObjParam};
use std::collections::HashMap;

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

pub fn optimize(mut obj_in: Vec<FunctionID>) -> Vec<FunctionID> {
    let mut network = TriggerNetwork::new();

    let no_group = Group {
        id: ID::Specific(0),
    };

    // sort all triggers by their group
    for (f, fnid) in obj_in.iter().enumerate() {
        for (o, obj) in fnid.obj_list.iter().enumerate() {
            if let Some(ObjParam::Number(id)) = obj.params.get(&1) {
                let trigger = Trigger {
                    obj: (f, o),
                    role: get_role(*id as u16),
                    connections_in: 0,
                    optimized: false,
                    deleted: false,
                };
                if let Some(ObjParam::GroupList(list)) = obj.params.get(&57) {
                    if list.is_empty() {
                        match network.get_mut(&no_group) {
                            Some(l) => (*l).push(trigger),
                            None => {
                                network.insert(no_group, vec![trigger]);
                            }
                        }
                        continue;
                    }

                    for group in list {
                        match network.get_mut(group) {
                            Some(l) => (*l).push(trigger),
                            None => {
                                network.insert(*group, vec![trigger]);
                            }
                        }
                    }
                } else {
                    match network.get_mut(&no_group) {
                        Some(l) => (*l).push(trigger),
                        None => {
                            network.insert(no_group, vec![trigger]);
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

    //optimize
    //optimize_network(&mut network);
    let group_sizes: Vec<(Group, usize)> = network
        .iter()
        .map(|(group, list)| (*group, list.len()))
        .collect();

    let objects = Triggerlist { list: &mut obj_in };

    for (g, len) in group_sizes {
        for i in 0..len {
            let trigger = network.get(&g).unwrap()[i];
            if trigger.role == TriggerRole::Output || trigger.optimized || trigger.deleted
            //|| (trigger.role == TriggerRole::Spawn && g != no_group)
            {
                continue;
            }

            let targets = optimize_from(&mut network, &objects, (g, i), 0.0, false);
            let start_trigger = network.get(&g).unwrap()[i];
            let obj = objects[trigger.obj].clone();
            if let Some(targets) = targets {
                for (g, _, delay) in targets {
                    // add spawn trigger to obj.fn_id with target group: g and delay: delay
                    let mut obj_map = HashMap::new();

                    obj_map.extend(obj.params.clone());
                    obj_map.insert(1, ObjParam::Number(1268.0));
                    obj_map.insert(51, ObjParam::Group(g));
                    obj_map.insert(63, ObjParam::Number(delay as f64));

                    let obj = GDObj {
                        params: obj_map.clone(),
                        mode: ObjectMode::Trigger,
                        func_id: obj.func_id,
                    };
                    let fn_id = obj.func_id;
                    (*objects.list)[fn_id].obj_list.push(obj);
                    let obj_index = (fn_id, objects.list[fn_id].obj_list.len() - 1);
                    let trigger = Trigger {
                        obj: obj_index,
                        optimized: true,
                        deleted: false,
                        ..start_trigger
                    };
                    if let Some(ObjParam::GroupList(list)) = &obj_map.get(&57) {
                        for group in list {
                            match network.get_mut(group) {
                                Some(list) => (*list).push(trigger),
                                None => unreachable!(),
                            }
                        }
                    }
                }
            }

            //create spawn triggers
            //TODO: keep track of delay
        }
    }

    // put into new fn ids and lists

    //profit

    rebuild(&network, &obj_in)
}

fn optimize_from<'a>(
    network: &'a mut TriggerNetwork,
    objects: &Triggerlist,
    start: (Group, usize),
    delay: f32,
    ignore_otimized: bool,
) -> Option<Vec<(Group, usize, f32)>> {
    // optimize from that trigger

    //(*start).optimized = true;
    let start_obj = &objects[network.get(&start.0).unwrap()[start.1].obj].params;

    (*network.get_mut(&start.0).unwrap())[start.1].optimized = true;

    match get_role(match *start_obj.get(&1).unwrap() {
        ObjParam::Number(n) => n,
        _ => unreachable!(),
    } as u16)
    {
        TriggerRole::Func => None,
        TriggerRole::Spawn => {
            let list: Vec<(Group, usize)>;

            if let Some(ObjParam::Group(g)) = start_obj.get(&51) {
                if let ID::Specific(_) = g.id {
                    return None;
                }

                if let Some(l) = network.get_mut(g) {
                    list = (0..l.len()).map(|x| (*g, x)).collect();
                } else {
                    //dangeling
                    (*network.get_mut(&start.0).unwrap())[start.1].deleted = true;
                    return Some(Vec::new());
                }
            } else {
                //dangeling
                (*network.get_mut(&start.0).unwrap())[start.1].deleted = true;
                return Some(Vec::new());
            }

            let added_delay = if let Some(ObjParam::Number(n)) = start_obj.get(&63) {
                *n as f32
            } else {
                0.0
            };

            let mut out = Vec::new();

            for trigger_ptr in list {
                let trigger = network.get_mut(&trigger_ptr.0).unwrap()[trigger_ptr.1];
                if trigger.deleted {
                    continue;
                }

                let full_trigger_ptr = (trigger_ptr.0, trigger_ptr.1, delay + added_delay);

                if trigger.connections_in > 1
                    || (trigger.optimized && !ignore_otimized && trigger.role != TriggerRole::Spawn)
                {
                    out.push(full_trigger_ptr)
                } else if trigger.role == TriggerRole::Spawn && trigger.optimized {
                    // if its already optimized, redo
                    match optimize_from(network, objects, trigger_ptr, delay + added_delay, true) {
                        Some(children) => out.extend(children),
                        None => (),
                    }
                } else {
                    match trigger.role {
                        TriggerRole::Output | TriggerRole::Func => out.push(full_trigger_ptr),
                        TriggerRole::Spawn => {
                            match optimize_from(
                                network,
                                objects,
                                trigger_ptr,
                                delay + added_delay,
                                ignore_otimized,
                            ) {
                                Some(children) => out.extend(children),
                                None => (),
                            }
                        }
                    }
                }
            }

            //if out.is_empty() {
            (*network.get_mut(&start.0).unwrap())[start.1].deleted = true;
            //}

            Some(out)
        }
        TriggerRole::Output => unreachable!(),
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
