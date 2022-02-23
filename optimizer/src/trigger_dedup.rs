use crate::optimize::clean_network;

use crate::optimize::replace_groups;

use crate::obj_ids;

use crate::optimize::is_start_group;

use compiler::builtins::Block;
use compiler::builtins::Color;
use compiler::builtins::Group;
use compiler::builtins::Item;
use fnv::FnvHashMap;

use crate::ReservedIds;

use crate::TriggerNetwork;

use crate::TriggerGang;

use crate::obj_props;

use crate::Triggerlist;

use crate::Trigger;

use compiler::builtins::Id;

use compiler::leveldata::ObjParam;

pub(crate) fn param_identifier(param: &ObjParam) -> String {
    let str = match param {
        ObjParam::Group(Group { id })
        | ObjParam::Color(Color { id })
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
use std::cmp::Ordering;
use std::collections::BTreeSet;
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct TriggerParam(u16, String);

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

#[derive(Debug)]
pub(crate) struct TriggerBehavior(BTreeSet<TriggerParam>, i64);

impl PartialEq for TriggerBehavior {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for TriggerBehavior {}

impl Ord for TriggerBehavior {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

// TODO: make this sort by trigger order as well
impl PartialOrd for TriggerBehavior {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let order_cmp = self.1.partial_cmp(&other.1).unwrap();
        if order_cmp != Ordering::Equal {
            return Some(order_cmp);
        }

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

pub(crate) fn get_trigger_behavior(t: Trigger, objects: &Triggerlist) -> TriggerBehavior {
    let mut set = BTreeSet::new();
    for (prop, param) in &objects[t.obj].0.params {
        if *prop == obj_props::GROUPS {
            // group
            continue;
        }
        set.insert(TriggerParam(*prop, param_identifier(param)));
    }
    TriggerBehavior(set, (objects[t.obj].1 .0 * 100000.0) as i64)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TriggerGangBehavior(BTreeSet<TriggerBehavior>);

pub(crate) fn get_triggergang_behavior(
    gang: &TriggerGang,
    objects: &Triggerlist,
) -> TriggerGangBehavior {
    let mut set = BTreeSet::new();

    for trigger in &gang.triggers {
        set.insert(get_trigger_behavior(*trigger, objects));
    }

    TriggerGangBehavior(set)
}

pub(crate) fn dedup_triggers(
    network: &mut TriggerNetwork,
    objects: &mut Triggerlist,
    reserved: &ReservedIds,
) {
    loop {
        let mut swaps = FnvHashMap::default();
        let mut representative_groups = Vec::<(TriggerGangBehavior, Group)>::new();

        for (group, gang) in network.iter_mut() {
            if is_start_group(*group, reserved) {
                continue;
            }
            let contains_stackable_trigger = gang.triggers.iter().any(|t| {
                let obj = &objects[t.obj].0;
                if let Some(ObjParam::Number(n)) = obj.params.get(&1) {
                    let id = *n as u16;
                    id == obj_ids::MOVE || id == 1817
                } else {
                    false
                }
            });
            if contains_stackable_trigger {
                continue;
            }
            let behavior = get_triggergang_behavior(gang, objects);

            let mut found = false;
            for (b, repr) in representative_groups.iter() {
                if b == &behavior {
                    for trigger in &mut gang.triggers {
                        (*trigger).deleted = true;
                    }
                    //dbg!(behavior, repr, group, &representative_groups);
                    assert!(swaps.insert(*group, *repr).is_none());

                    found = true;
                    break;
                }
            }
            if !found {
                representative_groups.push((behavior, *group));
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
