use errors::RuntimeError;
use internment::LocalIntern;
use shared::BreakType;
use shared::ImportType;
use shared::SpwnSource;
use shared::StoredValue;

///types and functions used by the compiler
use crate::builtins::*;

use crate::context::VariableData;
use errors::compiler_info::CodeArea;

use crate::context::FullContext;
use crate::leveldata::GdObj;

use crate::compiler_types::*;
use crate::value::*;

use ahash::AHashMap;

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

    pub path: LocalIntern<SpwnSource>,

    pub lowest_y: AHashMap<u32, u16>,
    pub stored_values: ValStorage,
    //pub val_id: StoredValue,
    pub type_ids: AHashMap<String, (u16, CodeArea)>,
    pub type_id_count: u16,

    pub type_descriptions: AHashMap<u16, String>,

    pub func_ids: Vec<FunctionId>,
    pub objects: Vec<GdObj>,
    pub initial_string: String,
    pub initial_objects: Option<StoredValue>,

    pub prev_imports: AHashMap<ImportType, (StoredValue, Implementations)>,

    pub trigger_order: f64,

    pub uid_counter: usize,
    pub implementations: Implementations,

    pub sync_groups: Vec<SyncGroup>,
    pub includes: Vec<PathBuf>,

    pub permissions: BuiltinPermissions,

    pub TYPE_MEMBER_NAME: LocalIntern<String>,
    pub SELF_MEMBER_NAME: LocalIntern<String>,
    pub OR_BUILTIN: LocalIntern<String>,
    pub AND_BUILTIN: LocalIntern<String>,
    pub ASSIGN_BUILTIN: LocalIntern<String>,
    pub OBJ_KEY_ID: LocalIntern<String>,
    pub OBJ_KEY_PATTERN: LocalIntern<String>,
    // the path to a potential executable built-in path
    pub built_in_path: Option<PathBuf>,
    pub std_out: &'a mut dyn Write,

    pub BUILTIN_STORAGE: StoredValue,
    pub NULL_STORAGE: StoredValue,
}

impl<'a> Globals<'a> {
    pub fn get_val_fn_context(
        &self,
        p: StoredValue,
        info: CompilerInfo,
    ) -> Result<Group, RuntimeError> {
        match self.stored_values.map.get(p) {
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
        match self.stored_values.map.get(p) {
            Some(val) => val.mutable,
            None => unreachable!("{:?}", p),
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
        match self.stored_values.map.get(p) {
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
        initial_string: String,
        std_out: &'a mut impl Write,
    ) -> Self {
        let (storage, builtin_storage, null_storage) = ValStorage::new();
        let mut type_descriptions = AHashMap::<u16, String>::default();

        type_descriptions.insert(type_id!(group), "
Groups are references to one or more Geometry Dash objects, and are used to make these objects do things like moving, rotating and pulsing.
They can also be used to mark a specific object, as for example a rotation center or a target position to move to.
Groups are also used to send signals between **triggers**, and are therefore quite closely related to trigger functions.
        ".trim().to_string());

        type_descriptions.insert(
            type_id!(color),
            "
Colors are references to color channels in a Geometry Dash level.
        "
            .trim()
            .to_string(),
        );

        type_descriptions.insert(
            type_id!(item),
            "
Item IDs are references to numbers that exist during the runtime of a Geometry Dash level. 
Items are used to store information that is changed during the level's runtime in an unpredictable way, like information about user input.
        "
            .trim().to_string()
            ,
        );

        type_descriptions.insert(
            type_id!(block),
            "
Collision Block IDs are references to one or more collision blocks, and are useful for tracking collisions between these.
        "
        .trim().to_string()
            ,
        );

        type_descriptions.insert(
            type_id!(number),
            "
Numbers are used to store numbers at compile time (if you want to have numbers at runtime, see [`@counter`](std-docs/counter) or [`@item`](std-docs/item))
        "
        .trim().to_string()
            ,
        );

        type_descriptions.insert(
            type_id!(bool),
            "
Booleans are used to store boolean values (`true` or `false`) at compile time (if you want to have booleans at runtime, see [`@counter`](std-docs/counter))
        "
        .trim().to_string()
            ,
        );

        type_descriptions.insert(
            type_id!(dictionary),
            "
Dictionaries are used to store key-value pairs at compile time.
        "
            .trim()
            .to_string(),
        );

        type_descriptions.insert(
            type_id!(string),
            "
Strings are used to store text at compile time.
        "
            .trim()
            .to_string(),
        );

        type_descriptions.insert(
            type_id!(array),
            "
Arrays are used to store lists of values at compile time.
        "
            .trim()
            .to_string(),
        );

        let mut globals = Globals {
            closed_groups: 0,
            closed_colors: 0,
            closed_blocks: 0,
            closed_items: 0,
            path: LocalIntern::new(path),

            lowest_y: AHashMap::default(),

            type_ids: AHashMap::default(),

            prev_imports: AHashMap::default(),
            type_id_count: 0,
            trigger_order: 0.0,
            uid_counter: 0,

            //val_id: storage.map.len() as StoredValue,
            stored_values: storage,
            func_ids: vec![FunctionId {
                parent: None,
                width: None,
                obj_list: Vec::new(),
            }],
            objects: Vec::new(),
            initial_string,
            implementations: AHashMap::default(),
            sync_groups: vec![SyncGroup {
                parts: vec![0],
                groups_used: Vec::new(),
            }],
            includes: Vec::new(),

            permissions,
            TYPE_MEMBER_NAME: LocalIntern::new(String::from("type")),
            SELF_MEMBER_NAME: LocalIntern::new(String::from("self")),
            BUILTIN_STORAGE: builtin_storage,
            NULL_STORAGE: null_storage,
            OR_BUILTIN: LocalIntern::new(String::from("_or_")),
            AND_BUILTIN: LocalIntern::new(String::from("_and_")),
            ASSIGN_BUILTIN: LocalIntern::new(String::from("_assign_")),
            OBJ_KEY_ID: LocalIntern::new(String::from("id")),
            OBJ_KEY_PATTERN: LocalIntern::new(String::from("pattern")),
            built_in_path: None,
            std_out,
            type_descriptions,
            initial_objects: None,
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
        //println!("before: {}", self.stored_values.map.len());

        //mark
        self.stored_values.mark(self.NULL_STORAGE);
        self.stored_values.mark(self.BUILTIN_STORAGE);

        if let Some(v) = self.initial_objects {
            self.stored_values.mark(v);
        }

        let root_context = unsafe {
            FullContext::from_ptr(
                contexts
                    .with_breaks()
                    .next()
                    .unwrap()
                    .inner()
                    .root_context_ptr,
            )
        };

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
        //}

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

        //println!("after: {}", self.stored_values.map.len());

        self.stored_values.prev_value_count = self.stored_values.map.len() as u32;
    }
}
