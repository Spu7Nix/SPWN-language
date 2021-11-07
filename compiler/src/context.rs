use crate::builtins::*;
use crate::compiler_types::*;
use crate::globals::Globals;
use crate::leveldata::*;
use crate::value::{strict_value_equality, Value};
use crate::value_storage::{clone_value, store_val_m};
use errors::compiler_info::CodeArea;

//use std::boxed::Box;
use fnv::FnvHashMap;

use internment::LocalIntern;
use shared::{BreakType, StoredValue};

#[derive(Debug, Clone, PartialEq)]
pub struct VariableData {
    pub val: StoredValue,
    pub layers: i16,
    pub redefinable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    // broken doesn't mean something is wrong with it, it just means
    // a break statement ( or similar) has been used :)
    pub broken: Option<(BreakType, CodeArea)>,
    pub start_group: Group,
    pub func_id: FnIdPtr,
    pub fn_context_change_stack: Vec<CodeArea>,
    variables: FnvHashMap<LocalIntern<String>, Vec<VariableData>>,
    pub return_value: StoredValue,
    pub return_value2: StoredValue,
    pub root_context_ptr: *mut FullContext,
}

#[derive(Debug, Clone)]
pub enum FullContext {
    Single(Context),
    Split(Box<FullContext>, Box<FullContext>),
}

impl FullContext {
    pub fn new(globals: &Globals) -> Self {
        let mut new = FullContext::Single(Context::new(globals));
        new.inner().root_context_ptr = &mut new;
        new
    }
    pub fn inner(&mut self) -> &mut Context {
        match self {
            Self::Single(c) => c,
            _ => unreachable!("Called 'inner' on a split value"),
        }
    }

    pub fn inner_value(&mut self) -> (&mut Context, StoredValue) {
        let context = self.inner();
        let val = context.return_value;
        (context, val)
    }

    pub fn stack(list: &mut impl Iterator<Item = Self>) -> Option<Self> {
        let first = list.next()?;
        match Self::stack(list) {
            Some(second) => Some(FullContext::Split(first.into(), second.into())),
            None => Some(first),
        }
    }

    pub fn enter_scope(&mut self) {
        for context in self.with_breaks() {
            for stack in context.inner().variables.values_mut() {
                for VariableData { layers, .. } in stack.iter_mut() {
                    *layers += 1;
                }
            }
        }
    }
    pub fn exit_scope(&mut self) {
        for context in self.with_breaks() {
            for stack in context.inner().variables.values_mut() {
                for VariableData { layers, .. } in stack.iter_mut() {
                    *layers -= 1;
                }
            }

            for stack in context.inner().variables.values_mut() {
                if stack.last().unwrap().layers < 0 {
                    stack.pop();
                }
            }
            context.inner().variables.retain(|_, s| !s.is_empty())
        }
        // let mut removed = FnvHashSet::new();
        // for context in self.with_breaks() {
        //     for (_, layers) in context.inner().variables.values_mut() {
        //         *layers -= 1;
        //     }
        //     removed.extend(context.inner().variables.values().filter_map(|(v, l)| {
        //         if *l < 0 {
        //             Some(v)
        //         } else {
        //             None
        //         }
        //     }));
        //     context.inner().variables.retain(|_, (_, l)| *l >= 0)
        // }
        // let mut all_removed = FnvHashSet::new();
        // for v in removed {
        //     all_removed.extend(get_all_ptrs_used(v, globals));
        // }
        // all_removed
    }

    pub fn reset_return_vals(&mut self, globals: &Globals) {
        for c in self.with_breaks() {
            let c = c.inner();
            (*c).return_value = globals.NULL_STORAGE;
            (*c).return_value2 = globals.NULL_STORAGE;
        }
    }

    pub fn set_variable_and_clone(
        &mut self,
        name: LocalIntern<String>,
        val: StoredValue,
        layer: i16,
        constant: bool,
        globals: &mut Globals,
        area: CodeArea,
    ) {
        for c in self.iter() {
            // reset all variables per context
            let fn_context = c.inner().start_group;
            (*c.inner()).new_variable(
                name,
                clone_value(val, globals, fn_context, constant, area),
                layer,
            );
        }
    }

    pub fn set_variable_and_store(
        &mut self,
        name: LocalIntern<String>,
        val: Value,
        layer: i16,
        constant: bool,
        globals: &mut Globals,
        area: CodeArea,
    ) {
        for c in self.iter() {
            // reset all variables per context
            let fn_context = c.inner().start_group;
            (*c.inner()).new_variable(
                name,
                store_val_m(val.clone(), globals, fn_context, constant, area),
                layer,
            );
        }
    }

    pub fn disable_breaks(&mut self, breaktype: BreakType) {
        for fc in self.with_breaks() {
            if let Some((b, _)) = &mut fc.inner().broken {
                if std::mem::discriminant(b) == std::mem::discriminant(&breaktype) {
                    (*fc.inner()).broken = None;
                }
            }
        }
    }

    pub fn with_breaks(&mut self) -> ContextIterWithBreaks {
        ContextIterWithBreaks::new(self)
    }

    pub fn iter(&mut self) -> ContextIter {
        ContextIter::new(self)
    }

    pub fn from_ptr(ptr: *mut FullContext) -> &'static mut FullContext {
        unsafe { ptr.as_mut().unwrap() }
    }
}

/// Iterator type for a binary tree.
/// This is a generator that progresses through an in-order traversal.
pub struct ContextIter<'a> {
    right_nodes: Vec<&'a mut FullContext>,
    current_node: Option<&'a mut FullContext>,
}

pub struct ContextIterWithBreaks<'a> {
    right_nodes: Vec<&'a mut FullContext>,
    current_node: Option<&'a mut FullContext>,
}

impl<'a> ContextIter<'a> {
    fn new(node: &'a mut FullContext) -> ContextIter<'a> {
        let mut iter = ContextIter {
            right_nodes: vec![],
            current_node: None,
        };
        iter.add_left_subtree(node);
        iter
    }

    /// Consume a binary tree node, traversing its left subtree and
    /// adding all branches to the right to the `right_nodes` field
    /// while setting the current node to the left-most child.
    fn add_left_subtree(&mut self, mut node: &'a mut FullContext) {
        loop {
            match node {
                FullContext::Split(left, right) => {
                    self.right_nodes.push(&mut **right);
                    node = &mut **left;
                }
                val @ FullContext::Single(_) => {
                    self.current_node = Some(val);
                    break;
                }
            }
        }
    }
}

impl<'a> ContextIterWithBreaks<'a> {
    fn new(node: &'a mut FullContext) -> ContextIterWithBreaks<'a> {
        let mut iter = ContextIterWithBreaks {
            right_nodes: vec![],
            current_node: None,
        };
        iter.add_left_subtree(node);
        iter
    }

    /// Consume a binary tree node, traversing its left subtree and
    /// adding all branches to the right to the `right_nodes` field
    /// while setting the current node to the left-most child.
    fn add_left_subtree(&mut self, mut node: &'a mut FullContext) {
        loop {
            match node {
                FullContext::Split(left, right) => {
                    self.right_nodes.push(&mut **right);
                    node = &mut **left;
                }
                val @ FullContext::Single(_) => {
                    self.current_node = Some(val);
                    break;
                }
            }
        }
    }
}

impl<'a> Iterator for ContextIter<'a> {
    type Item = &'a mut FullContext;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the item we're going to return.
        let result = self.current_node.take();

        // Now add the next left subtree
        // (this is the "recursive call")
        if let Some(node) = self.right_nodes.pop() {
            self.add_left_subtree(node);
        }
        match result {
            Some(c) => {
                if c.inner().broken.is_some() {
                    self.next()
                } else {
                    Some(c)
                }
            }
            None => None,
        }
    }
}

impl<'a> Iterator for ContextIterWithBreaks<'a> {
    type Item = &'a mut FullContext;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the item we're going to return.
        let result = self.current_node.take();

        // Now add the next left subtree
        // (this is the "recursive call")
        if let Some(node) = self.right_nodes.pop() {
            self.add_left_subtree(node);
        }
        result
    }
}

impl Context {
    fn new(globals: &Globals) -> Context {
        Context {
            start_group: Group::new(0),
            //spawn_triggered: false,
            variables: FnvHashMap::default(),
            //return_val: Box::new(Value::Null),

            //self_val: None,
            func_id: 0,
            broken: None,

            fn_context_change_stack: Vec::new(),
            return_value: globals.NULL_STORAGE,
            return_value2: globals.NULL_STORAGE,
            root_context_ptr: std::ptr::null_mut(),
        }
    }

    pub fn next_fn_id(&mut self, globals: &mut Globals) {
        (*globals).func_ids.push(FunctionId {
            parent: Some(self.func_id),
            obj_list: Vec::new(),
            width: None,
        });

        self.func_id = globals.func_ids.len() - 1;
    }

    pub fn get_variable(&self, name: LocalIntern<String>) -> Option<StoredValue> {
        self.variables.get(&name).map(|a| a.last().unwrap().val)
    }

    pub fn is_redefinable(&self, name: LocalIntern<String>) -> Option<bool> {
        self.variables
            .get(&name)
            .map(|a| a.last().unwrap().redefinable)
    }

    fn new_variable_full(
        &mut self,
        name: LocalIntern<String>,
        val: StoredValue,
        layer: i16,
        redefinable: bool,
    ) {
        match self.variables.get_mut(&name) {
            Some(stack) => stack.push(VariableData {
                val,
                layers: layer,
                redefinable,
            }),
            None => {
                self.variables.insert(
                    name,
                    vec![VariableData {
                        val,
                        layers: layer,
                        redefinable,
                    }],
                );
            }
        }
    }

    pub fn new_variable(&mut self, name: LocalIntern<String>, val: StoredValue, layer: i16) {
        self.new_variable_full(name, val, layer, false)
    }

    // only used in extract statements
    pub fn new_redefinable_variable(
        &mut self,
        name: LocalIntern<String>,
        val: StoredValue,
        layer: i16,
    ) {
        self.new_variable_full(name, val, layer, true)
    }

    pub fn get_variables(&self) -> &FnvHashMap<LocalIntern<String>, Vec<VariableData>> {
        &self.variables
    }

    pub fn set_all_variables(&mut self, vars: FnvHashMap<LocalIntern<String>, Vec<VariableData>>) {
        (*self).variables = vars;
    }
}

//will merge one set of context, returning false if no mergable contexts were found
pub fn merge_contexts(
    contexts: &mut Vec<Context>,
    globals: &mut Globals,
    check_return_vals: bool,
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

            if check_return_vals
                && !strict_value_equality(c.return_value, ref_c.return_value, globals)
            {
                not_eq = true;
            } else {
                //check variables are equal
                for (key, stack) in &c.variables {
                    for (i, VariableData { val, .. }) in stack.iter().enumerate() {
                        if !strict_value_equality(ref_c.variables[key][i].val, *val, globals) {
                            not_eq = true;
                            break;
                        }
                    }
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
        let mut params = FnvHashMap::default();
        params.insert(51, ObjParam::Group(new_group));
        params.insert(1, ObjParam::Number(1268.0));
        (*globals).trigger_order += 1.0;

        (*globals).func_ids[context.func_id].obj_list.push((
            GdObj {
                params,

                ..context_trigger(context, &mut globals.uid_counter)
            }
            .context_parameters(context),
            TriggerOrder(globals.trigger_order),
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

//will merge one set of context, returning false if no mergable contexts were found
