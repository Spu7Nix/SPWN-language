use errors::RuntimeError;
use internment::LocalIntern;
use shared::{ImportType, SpwnSource, StoredValue};

// tools for generating documentation for SPWN libraries
//use crate::ast::*;

use compiler::builtins::BuiltinPermissions;
use compiler::compiler::import_module;
use errors::compiler_info::CompilerInfo;

use compiler::context::FullContext;
use compiler::globals::Globals;
use compiler::{type_id, value::*};

use std::fs::File;

use fnv::FnvHashMap;
use std::env::current_dir;
use std::path::PathBuf;
fn create_doc_file(mut dir: PathBuf, mut name: String, content: &str) -> String {
    use std::io::Write;

    find_avaliable_name(&mut dir, &mut name);
    let mut output_file = File::create(&dir).unwrap();
    output_file.write_all(content.as_bytes()).unwrap();
    println!("written to {:?}", dir);
    name
}

fn find_avaliable_name(dir: &mut PathBuf, name: &mut String) {
    dir.push(format!("{}.md", name));
    while dir.exists() {
        dir.pop();
        name.push('_');
        dir.push(format!("{}.md", name));
    }
}
pub fn document_lib(path: &str) -> Result<(), RuntimeError> {
    let mut globals_path = std::env::current_dir().unwrap();
    globals_path.push("temp"); // this folder doesn't actually exist, but it needs to be there because .parent is called in import_module
    let mut std_out = std::io::stdout();
    let mut globals = Globals::new(
        SpwnSource::File(globals_path),
        BuiltinPermissions::new(),
        String::from(""),
        &mut std_out,
    );

    let mut start_context = FullContext::new(&globals);

    // store_value(Value::Builtins, 1, &mut globals, &start_context);
    // store_value(Value::Null, 1, &mut globals, &start_context);

    let mut output_path = current_dir().unwrap();
    let is_module = path.contains('.');

    let name: String = if is_module {
        let p = PathBuf::from(path);
        // let mut new_path = globals.path.as_ref().clone();
        // new_path.push(p.clone());
        // globals.path = LocalIntern::new(new_path);
        p.file_stem()
            .expect("invalid module path")
            .to_string_lossy()
            .to_string()
    } else {
        path.to_string()
    };
    let folder_name = format!("{}-docs", name);
    output_path.push(PathBuf::from(&folder_name));
    if !output_path.exists() {
        std::fs::create_dir(output_path.clone()).unwrap();
    } else {
        // delete all files in the directory
        for entry in std::fs::read_dir(output_path.clone()).unwrap() {
            let entry = entry.unwrap();
            std::fs::remove_file(entry.path()).unwrap();
        }
    }
    let info: CompilerInfo = CompilerInfo::new();
    globals
        .includes
        .push(std::env::current_dir().expect("Cannot access current directory"));
    globals.includes.push(
        std::env::current_exe()
            .expect("Cannot access directory of executable")
            .parent()
            .expect("Executable must be in some directory")
            .to_path_buf(),
    );

    import_module(
        &if is_module {
            ImportType::Script(PathBuf::from(path))
        } else {
            ImportType::Lib(path.to_string())
        },
        &mut start_context,
        &mut globals,
        info,
        false,
    )?;

    if let FullContext::Split(_, _) = start_context {
        return Err(RuntimeError::CustomError(errors::create_error(
            CompilerInfo::new(),
            "Documentation of context-splitting libraries is not yet supported!",
            &[],
            None,
        )));
    }

    let mut doc = format!("# Documentation for `{}`\n\n", name);

    let main_file = format!("{}-docs", name);

    let mut sidebar = "[_go back_](/)\n- **Exports**\n".to_string();

    globals.push_new_preserved();
    globals.push_preserved_val(start_context.inner().return_value);

    let exports = globals.stored_values[start_context.inner().return_value].clone();
    let implementations = globals.implementations.clone();

    doc += "_Generated using `spwn doc [file name]`_\n";

    let used_groups = globals.closed_groups;
    let used_colors = globals.closed_colors;
    let used_blocks = globals.closed_blocks;
    let used_items = globals.closed_items;

    let total_objects = globals.func_ids.iter().fold(0, |mut sum, val| {
        sum += val.obj_list.len();
        sum
    }) + globals.objects.len();

    //if used_groups > 0 || used_colors > 0 || used_blocks > 0 || used_used > 0 {
    doc += "\n## Info\n";
    //}

    doc += &format!(
        "
- Uses {} groups
- Uses {} colors
- Uses {} block IDs
- Uses {} item IDs

- Adds {} objects
",
        used_groups, used_colors, used_blocks, used_items, total_objects
    );

    let mut type_links = FnvHashMap::<u16, String>::default();
    let mut type_paths = FnvHashMap::<u16, String>::default();

    let mut impl_list = Vec::new();
    let doc_implementations =
        !implementations.is_empty() && implementations.iter().any(|(_, a)| !a.is_empty());
    if doc_implementations {
        impl_list = implementations
            .into_iter()
            .map(|(a, map)| {
                (
                    a,
                    if is_module {
                        map.into_iter().filter(|(_, (_, a))| *a).collect()
                    } else {
                        map
                    },
                )
            })
            .filter(|(_, a)| !a.is_empty())
            .map(|(key, val)| {
                (
                    key,
                    val.iter()
                        .map(|(key, val)| (*key, val.0))
                        .collect::<FnvHashMap<LocalIntern<String>, StoredValue>>(),
                )
            })
            .collect();
        impl_list.sort_by(|a, b| a.0.cmp(&b.0));
        for (typ, _) in impl_list.iter() {
            let mut type_name = find_key_for_value(&globals.type_ids, *typ)
                .expect("Implemented type was not found!")
                .clone();
            let orig_name = type_name.clone();
            find_avaliable_name(&mut output_path.clone(), &mut type_name);
            let path = format!("{}/{}", folder_name, type_name);

            //sidebar += &format!("- [**@{}**]({})\n", type_name, path);

            type_links.insert(*typ, format!("[`@{}`]({})", orig_name, path));
            type_paths.insert(*typ, path);
        }
    }

    let (doc_content, sidebar_content) = document_val(
        &exports,
        &mut globals,
        &mut start_context,
        &type_links,
        &format!("{}/{}", folder_name, main_file),
        None,
    )?;

    sidebar += &sidebar_content
        .lines()
        .map(|l| format!("\t{}\n", l))
        .collect::<Vec<_>>()
        .join("");

    doc += &format!("\n## Exports\n\n{}", doc_content);

    if doc_implementations {
        for (typ, dict) in impl_list.iter() {
            let type_name = find_key_for_value(&globals.type_ids, *typ)
                .expect("Implemented type was not found!")
                .clone();
            let (doc_content, sidebar_content) = document_dict(
                dict,
                &mut globals,
                &mut start_context,
                &type_links,
                &type_paths[typ],
                Some(&type_name),
            )?;

            sidebar += &format!("- **@{}**\n", type_name.replace("_", "\\_"));

            sidebar += &sidebar_content
                .lines()
                .map(|l| format!("\t{}\n", l))
                .collect::<Vec<_>>()
                .join("");

            let content = &if let Some(desc) = globals.type_descriptions.get(typ).cloned() {
                format!("# **@{}**\n?> {}\n{}", type_name, desc, doc_content)
            } else {
                format!("# **@{}**\n{}", type_name, doc_content)
            };

            create_doc_file(output_path.clone(), type_name, content);
        }
    }

    globals.pop_preserved();

    create_doc_file(output_path.clone(), main_file, &doc);
    create_doc_file(output_path, "_sidebar".to_string(), &sidebar);
    Ok(())
}

fn document_dict(
    dict: &FnvHashMap<LocalIntern<String>, StoredValue>,
    globals: &mut Globals,
    full_context: &mut FullContext,
    type_links: &FnvHashMap<u16, String>,
    path: &str,
    type_name: Option<&str>,
) -> Result<(String, String), RuntimeError> {
    let mut doc = String::new(); //String::from("<details>\n<summary> View members </summary>\n");
    let mut inner_sidebar = String::new();

    type ValList = Vec<(LocalIntern<String>, StoredValue)>;
    let mut categories = [
        ("Constructors", ValList::new()),
        ("Macros", ValList::new()),
        ("Operator Implementations", ValList::new()),
        ("Values", ValList::new()),
    ];

    for (name, x) in dict {
        if let Value::Macro(m) = &globals.stored_values[*x] {
            if m.tag.get("constructor").is_some() {
                categories[0].1.push((*name, *x));
            } else if name.starts_with('_') && name.ends_with('_') {
                categories[2].1.push((*name, *x));
            } else {
                categories[1].1.push((*name, *x));
            }
        } else {
            categories[3].1.push((*name, *x));
        }
    }

    for (_, list) in &mut categories {
        list.sort_by_key(|a| a.0);
    }

    let mut document_member = |key: &String,
                               val: &StoredValue,
                               inner_sidebar: &mut String|
     -> Result<String, RuntimeError> {
        *inner_sidebar += &if let Some(type_name) = type_name {
            format!(
                "\t- [`@{}::`{}]({}?id={})\n",
                type_name,
                key.replace("_", "\\_"),
                path,
                key
            )
        } else {
            format!("\t- [{}]({}?id={})\n", key.replace("_", "\\_"), path, key)
        };
        let mut member_doc = String::new();
        let inner_val = globals.stored_values[*val].clone();
        let (val_str, _) = document_val(
            &inner_val,
            globals,
            full_context,
            type_links,
            path,
            type_name,
        )?;
        let mut formatted = String::new();

        for line in val_str.lines() {
            formatted += &format!(">{}\n", line);
        }
        formatted.pop();

        member_doc += &format!(
            r#"
### {}

{}
>
"#,
            key.replace("_", "\\_"),
            formatted
        );
        Ok(member_doc)
    };

    for (category, list) in categories {
        if !list.is_empty() {
            doc += &format!("\n## {}\n", category);
            inner_sidebar += &format!("- {}\n", category);

            for (key, val) in list.iter() {
                doc += &document_member(key.as_ref(), val, &mut inner_sidebar)?
            }
        }
    }

    Ok((doc, inner_sidebar))
}

fn document_macro(
    mac: &Macro,
    globals: &mut Globals,
    full_context: &mut FullContext,
    type_links: &FnvHashMap<u16, String>,
) -> Result<String, RuntimeError> {
    //description
    let mut doc = String::new();
    if let Some(s) = mac.tag.get_desc() {
        doc += &format!("\n**Description:**\n\n_{}_\n", s)
    };

    if let Some(example) = mac.tag.get_example(false) {
        doc += &format!("\n**Example:**\n\n```spwn\n{}\n```\n\n", example)
    }

    if let Some(ret) = mac.ret_pattern {
        match globals.stored_values[ret].clone() {
            Value::Pattern(ret) => {
                if ret != Pattern::Type(type_id!(NULL)) {
                    doc += &format!(
                        "\n**Returns:** \n{}\n",
                        display_pattern(&ret, full_context, globals, type_links)?
                    );
                }
            }
            Value::TypeIndicator(t) => {
                if t != type_id!(NULL) {
                    doc += &format!(
                        "\n**Returns:** \n{}\n",
                        type_links.get(&t).cloned().unwrap_or_else(|| {
                            String::from("`@")
                                + &find_key_for_value(&globals.type_ids, t).unwrap().clone()
                                + "`"
                        })
                    );
                }
            }
            a => {
                doc += &format!(
                    "\n**Returns:** \n{}\n",
                    a.display(full_context, globals, &CompilerInfo::new())?
                        .replace("|", "\\|")
                );
            }
        }
    }

    if !(mac.args.is_empty()
        || (mac.args.len() == 1 && mac.args[0].name == globals.SELF_MEMBER_NAME))
    {
        doc += "\n**Arguments:**\n";
        doc += "
| # | name | type | default value | description |
| - | ---- | ---- | ------------- | ----------- |
";
        let mut i = 0;
        for arg in mac.args.iter() {
            let mut arg_string = String::new();

            if arg.name == globals.SELF_MEMBER_NAME {
                continue;
            }
            i += 1;

            arg_string += &format!("| {} | `{}` |", i, arg.name);

            if let Some(typ) = arg.pattern {
                let val = &globals.stored_values[typ].clone();
                arg_string += &format!(
                    " {} |",
                    match val {
                        Value::Pattern(p) => display_pattern(p, full_context, globals, type_links)?
                            .replace('_', "\\_"),
                        Value::TypeIndicator(t) =>
                            type_links.get(t).cloned().unwrap_or_else(|| {
                                String::from("`@")
                                    + &find_key_for_value(&globals.type_ids, *t).unwrap().clone()
                                    + "`"
                            }),
                        _ => format!(
                            "`{}`",
                            val.display(full_context, globals, &CompilerInfo::new())?
                                .replace("|", "\\|")
                        ),
                    }
                );
            } else {
                arg_string += "any |";
            }

            if let Some(def_val) = arg.default {
                let val = &globals.stored_values[def_val].clone();
                arg_string += &format!(
                    " `{}` |",
                    val.display(full_context, globals, &CompilerInfo::new())?
                        .replace("\n", "")
                );
            } else {
                arg_string += " |";
            }

            if let Some(desc) = arg.attribute.get_desc() {
                arg_string += &format!("{} |\n", desc);
            } else {
                arg_string += " |\n";
            }

            //add_arrows(&mut arg_string);

            doc += arg_string.as_ref();

            //doc += "\n  ";
        }
    }

    //arguments

    Ok(doc)
}

fn display_pattern(
    pat: &Pattern,
    full_context: &mut FullContext,
    globals: &mut Globals,
    type_links: &FnvHashMap<u16, String>,
) -> Result<String, RuntimeError> {
    Ok(match pat {
        Pattern::Type(type_id) => type_links.get(type_id).cloned().unwrap_or_else(|| {
            String::from("`@")
                + find_key_for_value(&globals.type_ids, *type_id)
                    .expect("Implemented type was not found!")
                + "`"
        }),
        Pattern::Array(a) => {
            format!(
                "an {} of {} elements",
                type_links
                    .get(&type_id!(array))
                    .unwrap_or(&"`@array`".to_string()),
                display_pattern(&a[0], full_context, globals, type_links)?
            )
        }
        Pattern::Either(p1, p2) => format!(
            "{} or {}",
            display_pattern(p1, full_context, globals, type_links)?,
            display_pattern(p2, full_context, globals, type_links)?
        ),
        Pattern::Both(p1, p2) => format!(
            "{} and {}",
            display_pattern(p1, full_context, globals, type_links)?,
            display_pattern(p2, full_context, globals, type_links)?
        ),
        Pattern::Not(p) => format!(
            "not {}",
            display_pattern(&**p, full_context, globals, type_links)?
        ),
        Pattern::Any => "any".to_string(),

        Pattern::Macro { args, ret } => {
            let mut arg_list = String::new();
            for arg in args.iter() {
                arg_list += &format!(
                    "{}, ",
                    display_pattern(arg, full_context, globals, type_links)?
                );
            }
            arg_list.pop();
            arg_list.pop();
            format!(
                "a {} that returns {} and takes {} as {}",
                type_links
                    .get(&type_id!(macro))
                    .unwrap_or(&"`@macro`".to_string()),
                display_pattern(&**ret, full_context, globals, type_links)?,
                arg_list,
                if args.len() == 1 {
                    "an argument"
                } else {
                    "arguments"
                }
            )
        }
        _ => Value::Pattern(pat.clone()).display(full_context, globals, &CompilerInfo::new())?,
    })
}

fn document_val(
    val: &Value,
    globals: &mut Globals,
    full_context: &mut FullContext,
    type_links: &FnvHashMap<u16, String>,
    path: &str,
    tn: Option<&str>,
) -> Result<(String, String), RuntimeError> {
    let mut doc = String::new();
    let typ_index = val
        .member(
            globals.TYPE_MEMBER_NAME,
            full_context.inner(),
            globals,
            CompilerInfo::new(),
        )
        .unwrap();
    let type_id = match globals.stored_values[typ_index] {
        Value::TypeIndicator(t) => t,
        _ => unreachable!(),
    };
    let literal = val.display(full_context, globals, &CompilerInfo::new())?;

    let type_name = type_links.get(&type_id).cloned().unwrap_or_else(|| {
        String::from("`@")
            + find_key_for_value(&globals.type_ids, type_id)
                .expect("Implemented type was not found!")
            + "`"
    });

    if literal.len() < 300 {
        doc += &format!(
            "**Printed**\n\n```spwn\n{}\n```\n\n**Type:** {}\n",
            literal, type_name
        );
    } else {
        doc += &format!("**Type:** {}\n", type_name);
    }

    let (new_doc, sidebar) = &match &val {
        Value::Dict(d) => document_dict(d, globals, full_context, type_links, path, tn)?,
        Value::Macro(m) => (
            document_macro(m, globals, full_context, type_links)?,
            String::new(),
        ),
        _ => (String::new(), String::new()),
    };
    doc += new_doc;

    //add_arrows(&mut doc);

    //doc += "\n  ";
    Ok((doc, sidebar.clone()))
}

// fn add_arrows(string: &mut String) {
//     let mut formatted = String::new();

//     for line in string.lines() {
//         formatted += &format!(">{}\n", line);
//     }

//     formatted.pop();
//     (*string) = formatted
// }
