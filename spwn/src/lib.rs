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
use internment::LocalIntern;

pub use ::compiler::STD_PATH;

pub use ::parser::fmt;
pub use ::parser::parser;
pub use ::parser::parser::parse_spwn;

pub use errors;
pub use errors::compiler_info;
pub use shared;
use shared::SpwnSource;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Default)]
pub struct SpwnCache {
    files: HashMap<SpwnSource, Source>,
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

pub fn run_spwn(
    code: String,
    included: Vec<PathBuf>,
    optimize: bool,
) -> Result<[String; 2], String> {
    let source = SpwnSource::String(LocalIntern::new(code.clone()));
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

    let mut compiled = match compiler::compile_spwn(
        statements,
        source,
        included,
        notes,
        Default::default(),
        "".to_string(),
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

    let has_stuff = compiled.func_ids.iter().any(|x| !x.obj_list.is_empty());

    let reserved = optimizer::ReservedIds::from_objects(&compiled.objects, &compiled.func_ids);

    if has_stuff && optimize {
        compiled.func_ids =
            optimizer::optimize::optimize(compiled.func_ids, compiled.closed_groups, reserved);
    }

    let mut objects = leveldata::apply_fn_ids(&compiled.func_ids);

    objects.extend(compiled.objects);

    let (new_ls, _) = leveldata::append_objects(objects, &String::new())?;

    Ok([String::from_utf8_lossy(&std_out).to_string(), new_ls])
}
#[cfg(test)]
mod tests;

#[test]
pub fn run_all_doc_examples() {
    use shared::ImportType;
    use std::str::FromStr;

    let mut globals_path = std::env::current_dir().unwrap();
    globals_path.push("temp"); // this folder doesn't actually exist, but it needs to be there because .parent is called in import_module
    let mut std_out = Vec::<u8>::new();

    let permissions = builtins::BuiltinPermissions::new();

    let mut globals = globals::Globals::new(
        SpwnSource::File(globals_path),
        permissions.clone(),
        String::from(""),
        &mut std_out,
    );
    globals.includes.push(PathBuf::from("./"));

    let mut start_context = context::FullContext::new(&globals);

    let info = compiler_info::CompilerInfo::new();

    compiler::import_module(
        &ImportType::Lib(STD_PATH.to_string()),
        &mut start_context,
        &mut globals,
        info,
        false,
    )
    .unwrap();

    let exports = globals.stored_values[start_context.inner().return_value].clone();

    if let value::Value::Dict(d) = &exports {
        for (a, b, c) in d.iter().map(|(k, v)| (*k, *v, -1)) {
            start_context.inner().new_redefinable_variable(a, b, c)
        }
    } else {
        panic!("The standard library must return a dictionary");
    }

    let implementations = globals.implementations.clone();

    let mut all_tests = Vec::new();

    add_tests("std".to_string(), exports, &globals, &mut all_tests);

    for (typ, dict) in implementations {
        let type_name = value::find_key_for_value(&globals.type_ids, typ).unwrap();
        for (name, (val, _)) in dict {
            let val = globals.stored_values[val].clone();
            add_tests(
                format!("@{}::{}", type_name, name),
                val,
                &globals,
                &mut all_tests,
            );
        }
    }

    for (name, code) in builtins::BUILTIN_EXAMPLES {
        if permissions.is_allowed(builtins::Builtin::from_str(*name).unwrap()) {
            all_tests.push((format!("$.{}", name), code.to_string()));
        }
    }

    let mut all_failed_tests = Vec::new();

    //let storage = globals.stored_values.clone();

    for (name, code) in all_tests {
        //println!("Running test: `\n{}\n`", code);

        let source = SpwnSource::String(LocalIntern::new(code.clone()));
        let cache = SpwnCache::default();

        let info = compiler_info::CompilerInfo {
            ..compiler_info::CompilerInfo::from_area(errors::compiler_info::CodeArea {
                file: LocalIntern::new(source.clone()),
                pos: (0, 0),
            })
        };

        let (statements, _) = match parse_spwn(code, source.clone(), BUILTIN_NAMES) {
            Ok(a) => a,
            Err(e) => {
                let mut out = Vec::<u8>::new();
                create_report(ErrorReport::from(e))
                    .write(cache, &mut out)
                    .unwrap();
                all_failed_tests.push((name, String::from_utf8_lossy(&out).to_string()));
                continue;
            }
        };

        //globals.stored_values = storage.clone();
        globals.objects.clear();

        let mut contexts = start_context.clone();
        contexts.inner().root_context_ptr = &mut contexts;

        match compiler::compile_scope(&statements, &mut contexts, &mut globals, info.clone()) {
            Ok(_) => (),
            Err(e) => {
                let mut out = Vec::<u8>::new();
                create_report(ErrorReport::from(e))
                    .write(cache, &mut out)
                    .unwrap();
                all_failed_tests.push((name, String::from_utf8_lossy(&out).to_string()));
                continue;
            }
        }
    }

    if !all_failed_tests.is_empty() {
        eprintln!(
            "{} examples from the STD failed to compile:",
            all_failed_tests.len()
        );
        for (name, err) in all_failed_tests {
            eprintln!("{}:\n{}\n", name, err);
        }
        panic!("Some examples failed to compile");
    }
}

#[cfg(test)]
fn add_tests(
    name: String,
    val: value::Value,
    globals: &globals::Globals,
    all_tests: &mut Vec<(String, String)>,
) {
    match val {
        value::Value::Macro(m) => {
            if let Some(example) = m.tag.get_example(true) {
                all_tests.push((name, example));
            }
        }
        value::Value::Dict(d) => {
            for (prop_name, v) in d.iter() {
                add_tests(
                    format!("{}.{}", name, prop_name),
                    globals.stored_values[*v].clone(),
                    globals,
                    all_tests,
                );
            }
        }
        value::Value::Array(l) => {
            for (i, v) in l.iter().enumerate() {
                add_tests(
                    format!("{}[{}]", name, i),
                    globals.stored_values[*v].clone(),
                    globals,
                    all_tests,
                );
            }
        }
        _ => (),
    };
}
