use crate::builtin::*;
use crate::compiler_info::{CodeArea, CompilerInfo};
use crate::compiler_types::*;
use crate::globals::Globals;
use crate::levelstring::*;
use crate::value_storage::StoredValue;

//use std::boxed::Box;
use std::collections::HashMap;

use smallvec::SmallVec;

use crate::compiler::CONTEXT_MAX;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Context {
    pub start_group: Group,
    pub fn_context_change_stack: Vec<CodeArea>,
    //pub spawn_triggered: bool,
    pub variables: HashMap<String, StoredValue>,
    //pub self_val: Option<StoredValue>,
    pub func_id: FnIdPtr,

    // info stores the info for the break statement if the context is "broken"
    // broken doesn't mean something is wrong with it, it just means
    // a break statement has been used :)
    pub broken: Option<(CompilerInfo, BreakType)>,

    pub sync_group: usize,
    pub sync_part: SyncPartId,
}

impl Context {
    pub fn new() -> Context {
        Context {
            start_group: Group::new(0),
            //spawn_triggered: false,
            variables: HashMap::new(),
            //return_val: Box::new(Value::Null),

            //self_val: None,
            func_id: 0,
            broken: None,

            sync_group: 0,
            sync_part: 0,
            fn_context_change_stack: Vec::new(),
        }
    }

    pub fn next_fn_id(&self, globals: &mut Globals) -> Context {
        (*globals).func_ids.push(FunctionId {
            parent: Some(self.func_id),
            obj_list: Vec::new(),
            width: None,
        });

        let mut out = self.clone();
        out.func_id = globals.func_ids.len() - 1;
        out
    }
}

//will merge one set of context, returning false if no mergable contexts were found
pub fn merge_contexts(
    contexts: &mut SmallVec<[Context; CONTEXT_MAX]>,
    globals: &mut Globals,
) -> bool {
    let mut mergable_ind = Vec::<usize>::new();
    let mut ref_c = 0;
    loop {
        if ref_c >= contexts.len() {
            return false;
        }
        for (i, c) in contexts.iter().enumerate() {
            if i == ref_c {
                continue;
            }
            let ref_c = &contexts[ref_c];

            if (ref_c.broken == None) != (c.broken == None) {
                continue;
            }
            let mut not_eq = false;

            //check variables are equal
            for (key, val) in &c.variables {
                if globals.stored_values[ref_c.variables[key]] != globals.stored_values[*val] {
                    not_eq = true;
                    break;
                }
            }
            if not_eq {
                continue;
            }
            //check implementations are equal
            // for (key, val) in &c.implementations {
            //     for (key2, val) in val {
            //         if globals.stored_values[ref_c.implementations[key][key2]] != globals.stored_values[*val] {
            //             not_eq = true;
            //             break;
            //         }
            //     }
            // }
            // if not_eq {
            //     continue;
            // }

            //everything is equal, add to list
            mergable_ind.push(i);
        }
        if mergable_ind.is_empty() {
            ref_c += 1;
        } else {
            break;
        }
    }

    let new_group = Group::next_free(&mut globals.closed_groups);
    //add spawn triggers
    let mut add_spawn_trigger = |context: &Context| {
        let mut params = HashMap::new();
        params.insert(51, ObjParam::Group(new_group));
        params.insert(1, ObjParam::Number(1268.0));
        (*globals).trigger_order += 1;

        (*globals).func_ids[context.func_id].obj_list.push((
            GdObj {
                params,

                ..context_trigger(&context, &mut globals.uid_counter)
            }
            .context_parameters(&context),
            globals.trigger_order,
        ))
    };
    add_spawn_trigger(&contexts[ref_c]);
    for i in mergable_ind.iter() {
        add_spawn_trigger(&contexts[*i])
    }

    (*contexts)[ref_c].start_group = new_group;
    (*contexts)[ref_c].next_fn_id(globals);

    for i in mergable_ind.iter().rev() {
        (*contexts).swap_remove(*i);
    }

    true
}
