pub use ::compiler::builtins;
use ::compiler::builtins::BUILTIN_NAMES;

use ::compiler::builtins::get_lib_file;
pub use ::compiler::compiler;
pub use ::compiler::compiler_types;
pub use ::compiler::context;
pub use ::compiler::globals;
pub use ::compiler::leveldata;
pub use ::compiler::value;
pub use ::compiler::value_storage;
pub use ::docgen::documentation;
pub use ::parser::ast;
use ariadne::Source;
use errors::create_report;
use errors::ErrorReport;
use internment::Intern;

pub use ::compiler::STD_PATH;

pub use ::parser::fmt;
pub use ::parser::parser;
pub use ::parser::parser::parse_spwn;

pub use errors::compiler_info;
pub use optimize;
use shared::SpwnSource;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use std::io;

pub struct SpwnCache {
    files: HashMap<SpwnSource, Source>,
}

impl Default for SpwnCache {
    fn default() -> Self {
        Self {
            files: HashMap::default(),
        }
    }
}

impl ariadne::Cache<SpwnSource> for SpwnCache {
    fn fetch(&mut self, source: &SpwnSource) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        Ok(match self.files.entry(source.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Source::from(match source {
                SpwnSource::File(path) => fs::read_to_string(path).map_err(|e| Box::new(e) as _)?,
                SpwnSource::BuiltIn(path) => match get_lib_file(path) {
                    Some(file) => match file.contents_utf8() {
                        Some(c) => c.to_string(),
                        None => return Err(Box::new("Invalid built in file content")),
                    },
                    _ => return Err(Box::new("Could not find built in file")),
                },
                SpwnSource::String(a) => a.as_ref().clone(),
            })),
        })
    }
    fn display<'a>(&self, source: &'a SpwnSource) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match source {
            SpwnSource::File(path) | SpwnSource::BuiltIn(path) => Some(Box::new(path.display())),
            SpwnSource::String(_) => Some(Box::new("source")),
        }
    }
}

pub fn run_spwn(code: String, included: Vec<PathBuf>) -> Result<String, String> {
    let source = SpwnSource::String(Intern::new(code.clone()));
    let cache = SpwnCache::default();
    let (statements, notes) = match parse_spwn(code, source.clone(), BUILTIN_NAMES) {
        Ok(a) => a,
        Err(e) => {
            let mut out = Vec::<u8>::new();
            create_report(ErrorReport::from(e))
                .write(cache, &mut out)
                .unwrap();
            return Err(String::from_utf8_lossy(&out).to_string());
        }
    };

    let mut std_out = Vec::<u8>::new();

    let compiled = match compiler::compile_spwn(
        statements,
        source,
        included,
        notes,
        Default::default(),
        &mut std_out,
    ) {
        Ok(a) => a,
        Err(e) => {
            let mut out = Vec::<u8>::new();
            create_report(ErrorReport::from(e))
                .write(cache, &mut out)
                .unwrap();
            return Err(String::from_utf8_lossy(&out).to_string());
        }
    };

    //let has_stuff = compiled.func_ids.iter().any(|x| !x.obj_list.is_empty());
    // if opti_enabled && has_stuff {
    //     compiled.func_ids =
    //         optimizer::optimize::optimize(compiled.func_ids, compiled.closed_groups, );
    // }

    // let mut objects = leveldata::apply_fn_ids(&compiled.func_ids);

    // objects.extend(compiled.objects);

    // let (new_ls, _) = leveldata::append_objects(objects, &String::new())?;

    Ok(String::from_utf8_lossy(&std_out).to_string())
}

#[test]
fn run_test() {
    dbg!(run_spwn(
        "$.print('Hello')".to_string(),
        vec![std::env::current_dir().expect("Cannot access current directory")],
    ));
}
