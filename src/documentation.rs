use internment::Intern;

// tools for generating documentation for SPWN libraries
//use crate::ast::*;

use crate::builtin::BuiltinPermissions;
use crate::compiler::{import_module, RuntimeError};
use crate::compiler_info::CompilerInfo;
use crate::compiler_types::ImportType;
use crate::context::FullContext;
use crate::globals::Globals;
use crate::value::*;
use crate::value_storage::StoredValue;
use std::fs::File;

use std::path::PathBuf;
use std::{collections::HashMap, env::current_dir};
fn create_doc_file(mut dir: PathBuf, name: String, content: &str) {
    use std::io::Write;
    dir.push(format!("{}.md", name));
    let mut output_file = File::create(&dir).unwrap();
    output_file.write_all(content.as_bytes()).unwrap();
    println!("written to {:?}", dir);
}
pub fn document_lib(path: &str) -> Result<(), RuntimeError> {
    let mut globals = Globals::new(PathBuf::new(), BuiltinPermissions::new());

    let mut start_context = FullContext::new();

    // store_value(Value::Builtins, 1, &mut globals, &start_context);
    // store_value(Value::Null, 1, &mut globals, &start_context);

    let mut output_path = current_dir().unwrap();
    output_path.push(PathBuf::from(format!("{}-docs", path)));
    if !output_path.exists() {
        std::fs::create_dir(output_path.clone()).unwrap();
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
        &ImportType::Lib(path.to_string()),
        &mut start_context,
        &mut globals,
        info,
        false,
    )?;

    if let FullContext::Split(_, _) = start_context {
        return Err(RuntimeError::CustomError(crate::compiler::create_error(
            CompilerInfo::new(),
            "Documentation of context-splitting libraries is not yet supported!",
            &[],
            None,
        )));
    }

    let mut doc = format!("# Documentation for `{}` \n", path);

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
    doc += "## Info:\n";
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
    if !implementations.is_empty() && implementations.iter().any(|(_, a)| !a.is_empty()) {
        doc += "# Type Implementations:\n";

        let mut list: Vec<_> = implementations
            .iter()
            .filter(|(_, a)| !a.is_empty())
            .map(|(key, val)| {
                (
                    key,
                    val.iter()
                        .map(|(key, val)| (*key, val.0))
                        .collect::<HashMap<Intern<String>, StoredValue>>(),
                )
            })
            .collect();
        list.sort_by(|a, b| a.0.cmp(b.0));
        for (typ, dict) in list.iter() {
            let type_name = find_key_for_value(&globals.type_ids, **typ)
                .expect("Implemented type was not found!")
                .clone();

            doc += &format!("- [**@{1}**]({}-docs/{1}.md)\n", path, type_name);

            let content = &format!(
                "  \n# **@{}**: \n {}",
                type_name,
                document_dict(dict, &mut globals)
            );

            create_doc_file(output_path.clone(), type_name, content);
        }
    }

    doc += &format!("# Exports:\n{}", document_val(&exports, &mut globals));

    create_doc_file(output_path, format!("{}-docs", path), &doc);
    Ok(())
}

fn document_dict(dict: &HashMap<Intern<String>, StoredValue>, globals: &mut Globals) -> String {
    let mut doc = String::new(); //String::from("<details>\n<summary> View members </summary>\n");
    type ValList = Vec<(Intern<String>, StoredValue)>;
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
            } else if name.starts_with("_") && name.ends_with("_") {
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

    let mut document_member = |key: &String, val: &StoredValue| -> String {
        let mut member_doc = String::new();
        let inner_val = globals.stored_values[*val].clone();
        let val_str = document_val(&inner_val, globals);
        let mut formatted = String::new();

        for line in val_str.lines() {
            formatted += &format!(">{}\n", line);
        }
        formatted.pop();

        member_doc += &format!(
            r#"
## **{}**:

{}
>
"#,
            key.replace("_", "\\_"),
            formatted
        );
        member_doc
    };

    for list in categories {
        if !list.1.is_empty() {
            doc += &format!("\n## {}:\n", list.0);

            for (key, val) in list.1.iter() {
                doc += &document_member(key.as_ref(), val)
            }
        }
    }

    doc
}

fn document_macro(mac: &Macro, globals: &mut Globals) -> String {
    //description
    let mut doc = String::new();
    if let Some(s) = mac.tag.get_desc() {
        doc += &format!("## Description: \n _{}_\n", s)
    };

    if let Some(example) = mac.tag.get_example() {
        doc += &format!("### Example: \n```spwn\n {}\n```\n", example)
    }

    if !(mac.args.is_empty() || (mac.args.len() == 1 && mac.args[0].0 == globals.SELF_MEMBER_NAME))
    {
        doc += "## Arguments:\n";
        doc += "
| # | name | type | default value | description |
| - | ---- | ---- | ------------- | ----------- |
";
        let mut i = 0;
        for arg in mac.args.iter() {
            let mut arg_string = String::new();

            if arg.0 == globals.SELF_MEMBER_NAME {
                continue;
            }
            i += 1;

            if arg.1 != None {
                arg_string += &format!("| {} | `{}` |", i, arg.0);
            } else {
                arg_string += &format!("| {} | **`{}`** |", i, arg.0);
            }

            if let Some(typ) = arg.3 {
                let val = &globals.stored_values[typ].clone();
                arg_string += &format!(" {} |", val.to_str(globals).replace("|", "or"));
            } else {
                arg_string += "any |";
            }

            if let Some(def_val) = arg.1 {
                let val = &globals.stored_values[def_val].clone();
                arg_string += &format!(" `{}` |", val.to_str(globals).replace("\n", ""));
            } else {
                arg_string += " |";
            }

            if let Some(desc) = arg.2.get_desc() {
                arg_string += &format!("{} |\n", desc);
            } else {
                arg_string += " |\n";
            }

            //add_arrows(&mut arg_string);

            doc += &arg_string;

            //doc += "\n  ";
        }
    }

    //arguments

    doc
}

fn document_val(val: &Value, globals: &mut Globals) -> String {
    let mut doc = String::new();
    let mut full_context = FullContext::new();
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

    let type_name =
        find_key_for_value(&globals.type_ids, type_id).expect("Implemented type was not found!");

    let literal = val.to_str(globals);
    if literal.len() < 300 {
        doc += &format!(
            " **Value:** \n```spwn\n{}\n``` \n**Type:** `@{}` \n",
            literal, type_name
        );
    } else {
        doc += &format!(" **Type:** `@{}` \n", type_name);
    }

    doc += &match &val {
        Value::Dict(d) => document_dict(d, globals),
        Value::Macro(m) => document_macro(m, globals),
        _ => String::new(),
    };

    //add_arrows(&mut doc);

    //doc += "\n  ";
    doc
}

// fn add_arrows(string: &mut String) {
//     let mut formatted = String::new();

//     for line in string.lines() {
//         formatted += &format!(">{}\n", line);
//     }

//     formatted.pop();
//     (*string) = formatted
// }
