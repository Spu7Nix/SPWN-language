///types and functions used by the compiler
use crate::builtin::*;
use crate::compiler_info::CodeArea;
use crate::levelstring::GdObj;

use crate::compiler_types::*;
use crate::value::*;

//use std::boxed::Box;
use crate::compiler_info::CompilerInfo;
use crate::value_storage::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::compiler::RuntimeError;

pub struct Globals {
    //counters for arbitrary groups
    pub closed_groups: u16,
    pub closed_colors: u16,
    pub closed_blocks: u16,
    pub closed_items: u16,

    pub path: PathBuf,

    pub lowest_y: HashMap<u32, u16>,
    pub stored_values: ValStorage,
    pub val_id: usize,

    pub type_ids: HashMap<String, (u16, PathBuf, (usize, usize))>,
    pub type_id_count: u16,

    pub func_ids: Vec<FunctionId>,
    pub objects: Vec<GdObj>,

    pub prev_imports: HashMap<ImportType, (Value, Implementations)>,

    pub trigger_order: usize,

    pub uid_counter: usize,
    pub implementations: Implementations,

    pub sync_groups: Vec<SyncGroup>,
}

impl Globals {
    pub fn get_val_fn_context(
        &self,
        p: StoredValue,
        info: CompilerInfo,
    ) -> Result<Group, RuntimeError> {
        match self.stored_values.map.get(&p) {
            Some(val) => Ok(val.fn_context),
            None => Err(RuntimeError::RuntimeError {
                message: "Pointer points to no data!".to_string(),
                info,
            }),
        }
    }
    pub fn is_mutable(&self, p: StoredValue) -> bool {
        match self.stored_values.map.get(&p) {
            Some(val) => val.mutable,
            None => unreachable!(),
        }
    }

    pub fn can_mutate(&self, p: StoredValue) -> bool {
        self.is_mutable(p)
    }

    pub fn increment_implementations(&mut self) {
        let mut incremented = HashSet::new();
        for imp in self.implementations.values() {
            for (val, _) in imp.values() {
                self.stored_values
                    .increment_single_lifetime(*val, 1, &mut incremented);
            }
        }
    }

    // pub fn get_fn_context(&self, p: StoredValue) -> Group {
    //     match self.stored_values.map.get(&p) {
    //         Some(val) => val.fn_context,
    //         None => unreachable!(),
    //     }
    // }

    pub fn get_lifetime(&self, p: StoredValue) -> u16 {
        match self.stored_values.map.get(&p) {
            Some(val) => val.lifetime,
            None => unreachable!(),
        }
    }

    pub fn get_area(&self, p: StoredValue) -> CodeArea {
        match self.stored_values.map.get(&p) {
            Some(val) => val.area.clone(),
            None => unreachable!(),
        }
    }

    pub fn get_type_str(&self, p: StoredValue) -> String {
        let val = &self.stored_values[p];
        let typ = match val {
            Value::Dict(d) => {
                if let Some(s) = d.get(TYPE_MEMBER_NAME) {
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

    pub fn new(path: PathBuf) -> Self {
        let storage = ValStorage::new();
        let mut globals = Globals {
            closed_groups: 0,
            closed_colors: 0,
            closed_blocks: 0,
            closed_items: 0,
            path,

            lowest_y: HashMap::new(),

            type_ids: HashMap::new(),

            prev_imports: HashMap::new(),
            type_id_count: 0,
            trigger_order: 0,
            uid_counter: 0,

            val_id: storage.map.len(),
            stored_values: storage,
            func_ids: vec![FunctionId {
                parent: None,
                width: None,
                obj_list: Vec::new(),
            }],
            objects: Vec::new(),
            implementations: HashMap::new(),
            sync_groups: vec![SyncGroup {
                parts: vec![0],
                groups_used: Vec::new(),
            }],
        };

        let mut add_type = |name: &str, id: u16| {
            globals
                .type_ids
                .insert(String::from(name), (id, PathBuf::new(), (0, 0)))
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
}
