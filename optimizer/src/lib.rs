use compiler::{
    builtins::{Group, Id},
    compiler_types::{FunctionId, TriggerOrder},
    leveldata::{self, GdObj, ObjParam},
};
use fnv::{FnvHashMap, FnvHashSet};
use optimize::ToggleGroups;

mod dead_code;
mod group_toggling;
pub mod optimize;
mod spawn_optimisation;
mod trigger_dedup;

pub type Swaps = FnvHashMap<Group, Group>;

mod obj_ids {
    #![allow(dead_code)]
    pub const MOVE: u16 = 901;
    pub const ROTATE: u16 = 1346;
    pub const ANIMATE: u16 = 1585;
    pub const PULSE: u16 = 1006;
    pub const COUNT: u16 = 1611;
    pub const ALPHA: u16 = 1007;
    pub const TOGGLE: u16 = 1049;
    pub const FOLLOW: u16 = 1347;
    pub const SPAWN: u16 = 1268;
    pub const STOP: u16 = 1616;
    pub const TOUCH: u16 = 1595;
    pub const INSTANT_COUNT: u16 = 1811;
    pub const ON_DEATH: u16 = 1812;
    pub const FOLLOW_PLAYER_Y: u16 = 1814;
    pub const COLLISION: u16 = 1815;
    pub const PICKUP: u16 = 1817;
    pub const BG_EFFECT_ON: u16 = 1818;
    pub const BG_EFFECT_OFF: u16 = 1819;
    pub const SHAKE: u16 = 1520;
    pub const COLOR: u16 = 899;
    pub const ENABLE_TRAIL: u16 = 32;
    pub const DISABLE_TRAIL: u16 = 33;
    pub const HIDE: u16 = 1612;
    pub const SHOW: u16 = 1613;
}

pub mod obj_props {
    pub const TARGET: u16 = 51;
    pub const GROUPS: u16 = 57;
    pub const ACTIVATE_GROUP: u16 = 56;
}

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

#[derive(Debug)]
pub struct ReservedIds {
    pub object_groups: FnvHashSet<Id>,
    pub trigger_groups: FnvHashSet<Id>, // only includes the obj_props::GROUPS prop

    pub object_colors: FnvHashSet<Id>,

    pub object_blocks: FnvHashSet<Id>,

    pub object_items: FnvHashSet<Id>,
}

impl ReservedIds {
    pub fn from_objects(objects: &[GdObj], func_ids: &[FunctionId]) -> Self {
        let mut reserved = ReservedIds {
            object_groups: Default::default(),
            trigger_groups: Default::default(),
            object_colors: Default::default(),

            object_blocks: Default::default(),

            object_items: Default::default(),
        };
        for obj in objects {
            for param in obj.params.values() {
                match &param {
                    leveldata::ObjParam::Group(g) => {
                        reserved.object_groups.insert(g.id);
                    }
                    leveldata::ObjParam::GroupList(g) => {
                        reserved.object_groups.extend(g.iter().map(|g| g.id));
                    }

                    leveldata::ObjParam::Color(g) => {
                        reserved.object_colors.insert(g.id);
                    }

                    leveldata::ObjParam::Block(g) => {
                        reserved.object_blocks.insert(g.id);
                    }

                    leveldata::ObjParam::Item(g) => {
                        reserved.object_items.insert(g.id);
                    }
                    _ => (),
                }
            }
        }

        for fn_id in func_ids {
            for (trigger, _) in &fn_id.obj_list {
                for (prop, param) in trigger.params.iter() {
                    if *prop == 57 {
                        match &param {
                            leveldata::ObjParam::Group(g) => {
                                reserved.trigger_groups.insert(g.id);
                            }
                            leveldata::ObjParam::GroupList(g) => {
                                reserved.trigger_groups.extend(g.iter().map(|g| g.id));
                            }

                            _ => (),
                        }
                    }
                }
            }
        }
        reserved
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjPtr(usize, usize);
//                                     triggers      connections in
#[derive(Default)]
pub struct TriggerNetwork {
    map: FnvHashMap<Group, TriggerGang>,
    connectors: FnvHashMap<Group, FnvHashSet<ObjPtr>>,
}

#[derive(Debug, Clone)]
// what do you mean? its a trigger gang!
pub struct TriggerGang {
    pub triggers: Vec<Trigger>,
    pub connections_in: u32,
    // whether any of the connections in are not instant count triggers
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

pub fn get_role(obj: &GdObj, toggle_groups: &ToggleGroups) -> TriggerRole {
    if let Some(ObjParam::Number(obj_id)) = obj.params.get(&1) {
        let mut hd = false;
        if let Some(ObjParam::Bool(hd_val)) = obj.params.get(&103) {
            hd = *hd_val;
        }
        match *obj_id as u16 {
            obj_ids::SPAWN => {
                if let Some(ObjParam::Group(Group {
                    id: Id::Specific(_),
                })) = obj.params.get(&obj_props::TARGET)
                {
                    TriggerRole::Output
                } else if hd {
                    TriggerRole::Func
                } else {
                    TriggerRole::Spawn
                    // let togglable = match obj.params.get(&obj_props::GROUPS) {
                    //     Some(ObjParam::GroupList(g)) => {
                    //         g.iter().any(|g| toggle_groups.toggles_off.contains_key(g))
                    //     }
                    //     Some(ObjParam::Group(g)) => toggle_groups.toggles_off.contains_key(g),
                    //     _ => false,
                    // };

                    // if togglable {
                    //     TriggerRole::Func
                    // } else {
                    //     TriggerRole::Spawn
                    // }
                }
            }
            obj_ids::TOUCH => {
                if let Some(ObjParam::Group(g)) = obj.params.get(&obj_props::TARGET) {
                    if let Id::Specific(_) = g.id {
                        // might interact with triggers in the editor
                        TriggerRole::Output
                    } else {
                        TriggerRole::Func
                    }
                } else {
                    // the user didnt provide a target group, so fuck them no optimization for you >:D
                    TriggerRole::Output
                }
            }
            obj_ids::COUNT | obj_ids::COLLISION | obj_ids::INSTANT_COUNT | obj_ids::ON_DEATH => {
                if let Some(ObjParam::Bool(false)) | None =
                    obj.params.get(&obj_props::ACTIVATE_GROUP)
                {
                    // will toggle a group off
                    TriggerRole::Output
                } else if let Some(ObjParam::Group(g)) = obj.params.get(&obj_props::TARGET) {
                    if let Id::Specific(_) = g.id {
                        // might interact with triggers in the editor
                        TriggerRole::Output
                    } else {
                        TriggerRole::Func
                    }
                    // else if toggle_groups.contains(g) {
                    //     // might toggle a group on
                    //     TriggerRole::Output
                    // }
                } else {
                    // the user didnt provide a target group, so fuck them no optimization for you >:D
                    TriggerRole::Output
                }
            }
            _ => TriggerRole::Output,
        }
    } else {
        TriggerRole::Output
    }
}

pub const NO_GROUP: Group = Group {
    id: Id::Specific(0),
};
