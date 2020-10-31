// tools for generating documentation for SPWN libraries
//use crate::ast::*;
use crate::builtin::TYPE_MEMBER_NAME;
use crate::compiler::{import_module, RuntimeError};
use crate::compiler_types::{
    find_key_for_value, CompilerInfo, Context, Globals, ImportType, Macro, Value,
};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn document_lib(path: &str) -> Result<String, RuntimeError> {
    let mut globals = Globals::new(PathBuf::new());

    let start_context = Context::new();

    // store_value(Value::Builtins, 1, &mut globals, &start_context);
    // store_value(Value::Null, 1, &mut globals, &start_context);

    let module = import_module(
        &ImportType::Lib(path.to_string()),
        &start_context,
        &mut globals,
        CompilerInfo::new(),
    )?;

    if module.len() > 1 {
        return Err(RuntimeError::RuntimeError {
            message: "Documentation of context-splitting libraries is not yet supported!"
                .to_string(),
            info: CompilerInfo::new(),
        });
    }

    let mut doc = format!("# Documentation for `{}` \n", path);

    let exports = globals.stored_values[module[0].0].clone();
    let implementations = globals.implementations.clone();

    doc += "_This file was generated using `spwn doc [file name]`_\n";

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

    doc += &format!("## Exports:\n{}", document_val(&exports, &mut globals));

    doc += "## Type Implementations:\n";

    let mut list: Vec<(&u16, HashMap<String, usize>)> = implementations
        .iter()
        .map(|(key, val)| {
            (
                key,
                val.iter()
                    .map(|(key, val)| (key.clone(), val.0))
                    .collect::<HashMap<String, usize>>(),
            )
        })
        .collect();
    list.sort_by(|a, b| a.0.cmp(b.0));

    for (typ, dict) in list.iter() {
        let type_name = find_key_for_value(&globals.type_ids, **typ)
            .expect("Implemented type was not found!")
            .clone();

        doc += &format!(
            "### **@{}**: \n {}",
            type_name,
            document_dict(dict, &mut globals)
        );
    }

    Ok(doc)
}

fn document_dict(dict: &HashMap<String, usize>, globals: &mut Globals) -> String {
    let mut doc = String::from("<details>\n<summary> View members </summary>\n");

    let mut macro_list: Vec<(&String, &usize)> = dict
        .iter()
        .filter(|x| matches!(globals.stored_values[*x.1], Value::Macro(_)))
        .collect();
    macro_list.sort_by(|a, b| a.0.cmp(&b.0));

    let mut val_list: Vec<(&String, &usize)> = dict
        .iter()
        .filter(|x| !matches!(globals.stored_values[*x.1], Value::Macro(_)))
        .collect();
    val_list.sort_by(|a, b| a.0.cmp(&b.0));

    let mut document_member = |key: &String, val: &usize| -> String {
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
**`{}`**:

{}
>
"#,
            key, formatted
        );
        member_doc
    };
    if !macro_list.is_empty() {
        if !val_list.is_empty() {
            doc += "\n\n## Macros:\n\n";
        }
        for (key, val) in macro_list.iter() {
            doc += &document_member(*key, *val)
        }
    }
    if !val_list.is_empty() {
        if !macro_list.is_empty() {
            doc += "## Other values:\n\n<details>\n<summary> View </summary>\n";
        }
        for (key, val) in val_list.iter() {
            doc += &document_member(*key, *val)
        }
        if !macro_list.is_empty() {
            doc += "\n\n</details>\n\n";
        }
    }
    if !macro_list.is_empty() {
        doc += "</details>\n\n";
    }
    doc
}

fn document_macro(mac: &Macro, globals: &mut Globals) -> String {
    //description
    let mut doc = String::new();
    if let Some(s) = mac.tag.get_desc() {
        doc += &format!("## Description: \n _{}_\n", s)
    };

    if !(mac.args.is_empty() || (mac.args.len() == 1 && mac.args[0].0 == "self")) {
        doc += "## Arguments:\n";

        for arg in &mac.args {
            let mut arg_string = String::new();

            if arg.0 == "self" {
                continue;
            }

            if arg.1 != None {
                arg_string += &format!(" _`{}` (optional)_ ", arg.0);
            } else {
                arg_string += &format!(" **`{}`** _(obligatory)_", arg.0);
            }

            if let Some(desc) = arg.2.get_desc() {
                arg_string += &format!(": _{}_", desc);
            }

            if let Some(def_val) = arg.1 {
                let val = &globals.stored_values[def_val].clone();
                arg_string +=
                    &format!("\n\n_Default value:_\n\n{}\n\n", document_val(val, globals));
            }

            add_arrows(&mut arg_string);
            doc += &arg_string;

            doc += "\n\n\n\n\n";
        }
    }

    //arguments

    doc
}

fn document_val(val: &Value, globals: &mut Globals) -> String {
    let mut doc = String::new();
    let typ_index = val
        .member(TYPE_MEMBER_NAME.to_string(), &Context::new(), globals)
        .unwrap();
    let type_id = match globals.stored_values[typ_index] {
        Value::TypeIndicator(t) => t,
        _ => unreachable!(),
    };

    let type_name =
        find_key_for_value(&globals.type_ids, type_id).expect("Implemented type was not found!");

    doc += &format!("**Type:** `{}` \n\n", type_name);
    let literal = val.to_str(globals);
    if literal.lines().count() > 1 {
        doc += &format!("**Literal:** \n\n ```\n\n{}\n\n``` \n\n", literal);
    } else {
        doc += &format!("**Literal:** ```{}``` \n\n", literal);
    }

    doc += &match &val {
        Value::Dict(d) => document_dict(d, globals),
        Value::Macro(m) => document_macro(m, globals),
        _ => String::new(),
    };

    //add_arrows(&mut doc);

    doc += "\n";
    doc
}

fn add_arrows(string: &mut String) {
    let mut formatted = String::new();

    for line in string.lines() {
        formatted += &format!(">{}\n", line);
    }

    formatted.pop();
    (*string) = formatted
}
