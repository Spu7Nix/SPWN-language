use errors::RuntimeError;
use internment::Intern;
use shared::BreakType;
use shared::ImportType;
use shared::SpwnSource;
use shared::StoredValue;

///types and functions used by the compiler
use crate::builtins::*;
use crate::compiler::BUILTIN_STORAGE;
use crate::compiler::NULL_STORAGE;
use crate::context::VariableData;
use errors::compiler_info::CodeArea;

use crate::context::FullContext;
use crate::leveldata::GdObj;

use crate::compiler_types::*;
use crate::value::*;

use fnv::FnvHashMap;

//use std::boxed::Box;
use crate::value_storage::*;
use errors::compiler_info::CompilerInfo;

use std::io::Write;
use std::path::PathBuf;

#[allow(non_snake_case)]
pub struct Globals<'a> {
    //counters for arbitrary groups
    pub closed_groups: u16,
    pub closed_colors: u16,
    pub closed_blocks: u16,
    pub closed_items: u16,

    pub path: Intern<SpwnSource>,

    pub lowest_y: FnvHashMap<u32, u16>,
    pub stored_values: ValStorage,
    pub val_id: StoredValue,

    pub type_ids: FnvHashMap<String, (u16, CodeArea)>,
    pub type_id_count: u16,

    pub func_ids: Vec<FunctionId>,
    pub objects: Vec<GdObj>,

    pub prev_imports: FnvHashMap<ImportType, (StoredValue, Implementations)>,

    pub trigger_order: f64,

    pub uid_counter: usize,
    pub implementations: Implementations,

    pub sync_groups: Vec<SyncGroup>,
    pub includes: Vec<PathBuf>,

    pub permissions: BuiltinPermissions,

    pub TYPE_MEMBER_NAME: Intern<String>,
    pub SELF_MEMBER_NAME: Intern<String>,
    pub OR_BUILTIN: Intern<String>,
    pub AND_BUILTIN: Intern<String>,
    pub ASSIGN_BUILTIN: Intern<String>,
    pub OBJ_KEY_ID: Intern<String>,
    pub OBJ_KEY_PATTERN: Intern<String>,
    // the path to a potential executable built-in path
    pub built_in_path: Option<PathBuf>,
    pub std_out: &'a mut dyn Write,
}

impl<'a> Globals<'a> {
    pub fn get_val_fn_context(
        &self,
        p: StoredValue,
        info: CompilerInfo,
    ) -> Result<Group, RuntimeError> {
        match self.stored_values.map.get(&p) {
            Some(val) => Ok(val.fn_context),
            None => Err(RuntimeError::CustomError(errors::create_error(
                info,
                "Pointer points to no data! (this is probably a bug, please contact a developer)",
                &[],
                None,
            ))),
        }
    }
    pub fn is_mutable(&self, p: StoredValue) -> bool {
        match self.stored_values.map.get(&p) {
            Some(val) => val.mutable,
            None => unreachable!("{}", p),
        }
    }

    pub fn can_mutate(&self, p: StoredValue) -> bool {
        self.is_mutable(p)
    }

    // pub fn get_lifetime(&self, p: StoredValue) -> Option<u16> {
    //     match self.stored_values.map.get(&p) {
    //         Some(val) => val.lifetime,
    //         None => unreachable!(),
    //     }
    // }

    // pub fn get_fn_context(&self, p: StoredValue) -> Group {
    //     match self.stored_values.map.get(&p) {
    //         Some(val) => val.fn_context,
    //         None => unreachable!(),
    //     }
    // }

    pub fn get_area(&self, p: StoredValue) -> CodeArea {
        match self.stored_values.map.get(&p) {
            Some(val) => val.def_area,
            None => unreachable!(),
        }
    }

    pub fn get_type_str(&self, p: StoredValue) -> String {
        let val = &self.stored_values[p];
        let typ = match val {
            Value::Dict(d) => {
                if let Some(s) = d.get(&self.TYPE_MEMBER_NAME) {
                    match self.stored_values[*s] {
                        Value::TypeIndicator(t) => t,
                        _ => unreachable!(),
                    }
                } else {
                    val.to_num(self)
                }
            }
            _ => val.to_num(self),
        };
        find_key_for_value(&self.type_ids, typ).unwrap().clone()
    }

    pub fn new(
        path: SpwnSource,
        permissions: BuiltinPermissions,
        std_out: &'a mut impl Write,
    ) -> Self {
        let storage = ValStorage::new();
        let mut globals = Globals {
            closed_groups: 0,
            closed_colors: 0,
            closed_blocks: 0,
            closed_items: 0,
            path: Intern::new(path),

            lowest_y: FnvHashMap::default(),

            type_ids: FnvHashMap::default(),

            prev_imports: FnvHashMap::default(),
            type_id_count: 0,
            trigger_order: 0.0,
            uid_counter: 0,

            val_id: storage.map.len() as StoredValue,
            stored_values: storage,
            func_ids: vec![FunctionId {
                parent: None,
                width: None,
                obj_list: Vec::new(),
            }],
            objects: Vec::new(),
            implementations: FnvHashMap::default(),
            sync_groups: vec![SyncGroup {
                parts: vec![0],
                groups_used: Vec::new(),
            }],
            includes: Vec::new(),

            permissions,
            TYPE_MEMBER_NAME: Intern::new(String::from("type")),
            SELF_MEMBER_NAME: Intern::new(String::from("self")),
            OR_BUILTIN: Intern::new(String::from("_or_")),
            AND_BUILTIN: Intern::new(String::from("_and_")),
            ASSIGN_BUILTIN: Intern::new(String::from("_assign_")),
            OBJ_KEY_ID: Intern::new(String::from("id")),
            OBJ_KEY_PATTERN: Intern::new(String::from("pattern")),
            built_in_path: None,
            std_out,
        };

        let mut add_type = |name: &str, id: u16| {
            globals
                .type_ids
                .insert(String::from(name), (id, CodeArea::new()))
        };

        add_type("group", 0);
        add_type("color", 1);
        add_type("block", 2);
        add_type("item", 3);
        add_type("number", 4);
        add_type("bool", 5);
        add_type("trigger_function", 6);
        add_type("dictionary", 7);
        add_type("macro", 8);
        add_type("string", 9);
        add_type("array", 10);
        add_type("object", 11);
        add_type("spwn", 12);
        add_type("builtin", 13);
        add_type("type_indicator", 14);
        add_type("NULL", 15);
        add_type("trigger", 16);
        add_type("range", 17);
        add_type("pattern", 18);
        add_type("object_key", 19);
        add_type("epsilon", 20);

        globals.type_id_count = globals.type_ids.len() as u16;

        globals
    }

    // pub fn clean_up(&mut self, full_context: &mut FullContext, mut removed: FnvHashSet<usize>) {
    //     let mut used_values = FnvHashSet::new();

    //     // for l in self.implementations.values() {
    //     //     for (v, _) in l.values() {
    //     //         used_values.insert(*v);
    //     //     }
    //     // }
    //     for c in full_context.with_breaks() {
    //         used_values.extend(c.inner().variables.iter().map(|(_, (a, _))| *a));
    //         // used_values.insert(c.inner().return_value);
    //         // used_values.insert(c.inner().return_value2);
    //         match c.inner().broken {
    //             Some((BreakType::Macro(Some(v), _), _)) => {
    //                 used_values.insert(v);
    //             }
    //             Some((BreakType::Switch(v), _)) => {
    //                 used_values.insert(v);
    //             }
    //             _ => (),
    //         };
    //     }
    //     let mut all_used_values = FnvHashSet::new();
    //     for v in used_values {
    //         all_used_values.extend(get_all_ptrs_used(v, self));
    //     }

    //     // for v in all_used_values.iter() {
    //     //     dbg!(v, self.stored_values[*v].clone());
    //     // }
    //     all_used_values.insert(BUILTIN_STORAGE);
    //     all_used_values.insert(NULL_STORAGE);

    //     removed.retain(|a| !all_used_values.contains(a));

    //     self.stored_values
    //         .map
    //         .retain(|a, _| -> bool { !removed.contains(a) });
    // }
    pub fn push_new_preserved(&mut self) {
        self.stored_values.preserved_stack.push(Vec::new());
    }

    pub fn pop_preserved(&mut self) {
        self.stored_values.preserved_stack.pop();
    }

    pub fn push_preserved_val(&mut self, val: StoredValue) {
        self.stored_values
            .preserved_stack
            .last_mut()
            .unwrap()
            .push(val);
    }

    pub fn collect_garbage(&mut self, contexts: &mut FullContext) {
        //gc

        //mark
        self.stored_values.mark(NULL_STORAGE);
        self.stored_values.mark(BUILTIN_STORAGE);

        unsafe {
            let root_context = contexts
                .with_breaks()
                .next()
                .unwrap()
                .inner()
                .root_context_ptr
                .as_mut()
                .unwrap();

            for c in root_context.with_breaks() {
                for stack in c.inner().get_variables().values() {
                    for VariableData { val: v, .. } in stack.iter() {
                        self.stored_values.mark(*v);
                    }
                }

                match c.inner().broken {
                    Some((BreakType::Macro(Some(v), _), _)) | Some((BreakType::Switch(v), _)) => {
                        self.stored_values.mark(v);
                    }
                    _ => (),
                }

                // for split contexts
                self.stored_values.mark(c.inner().return_value);
                self.stored_values.mark(c.inner().return_value2);
            }
            for s in self.stored_values.preserved_stack.clone() {
                for v in s {
                    self.stored_values.mark(v);
                }
            }
            for imp in self.implementations.values() {
                for (v, _) in imp.values() {
                    self.stored_values.mark(*v);
                }
            }
        }

        for (v, imp) in self.prev_imports.values() {
            for imp in imp.values() {
                for (v, _) in imp.values() {
                    self.stored_values.mark(*v);
                }
            }

            self.stored_values.mark(*v);
        }
        //dbg!(&self.stored_values.map);

        //sweep
        self.stored_values.sweep();

        self.stored_values.prev_value_count = self.stored_values.map.len() as u32;
    }
}
