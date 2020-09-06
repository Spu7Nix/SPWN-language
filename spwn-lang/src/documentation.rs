// tools for generating documentation for SPWN libraries
use crate::ast::*;
use crate::builtin::TYPE_MEMBER_NAME;
use crate::compiler::{import_module, RuntimeError};
use crate::compiler_types::{find_key_for_value, CompilerInfo, Context, Globals, Macro, Value};
use crate::parser::ParseNotes;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn document_lib(path: &PathBuf) -> Result<String, RuntimeError> {
    let mut globals = Globals::new(ParseNotes::new(), path.clone());

    let (exports, implementations) = import_module(
        path,
        &mut globals,
        CompilerInfo {
            depth: 0,
            path: vec!["main scope".to_string()],
            line: (0, 0),
            func_id: 0,
        },
    )?;

    let mut doc = format!(
        "# Documentation for {} \n",
        path.file_stem().unwrap().to_str().unwrap()
    );

    doc += &format!("## Exports:\n{}", document_val(&exports, &globals));

    doc += "## Type Implementations:\n";

    let mut list: Vec<(&u16, &HashMap<String, u32>)> = implementations.iter().collect();
    list.sort_by(|a, b| a.0.cmp(b.0));

    for (typ, dict) in list.iter() {
        let type_name =
            find_key_for_value(&globals.type_ids, **typ).expect("Implemented type was not found!");

        doc += &format!(
            "### **@{}**: \n {}",
            type_name,
            document_dict(dict, &globals)
        );
    }

    Ok(doc)
}

fn document_dict(dict: &HashMap<String, u32>, globals: &Globals) -> String {
    let mut doc = String::from("<details>\n<summary> View members </summary>\n");

    let mut list: Vec<(&String, &u32)> = dict.iter().collect();
    list.sort_by(|a, b| a.0.cmp(&b.0));

    for (key, val) in list.iter() {
        let val_str = document_val(globals.stored_values.get(**val as usize).unwrap(), globals);

        let mut formatted = String::new();

        for line in val_str.lines() {
            formatted += &format!(">{}\n", line);
        }
        formatted.pop();

        doc += &format!(
            r#"
> `{}`:
>
{}

"#,
            key, formatted
        )
    }

    doc += "</details>\n\n";
    doc
}

fn document_macro(mac: &Macro, globals: &Globals) -> String {
    //description
    let mut doc = String::new();
    match mac.tag.get_desc() {
        Some(s) => doc += &format!("## Description: \n _{}_\n", s),
        None => (),
    };

    if !mac.args.is_empty() {
        doc += "## Arguments:\n";

        for arg in &mac.args {
            let mut arg_string = String::new();

            if arg.0 == "self" {
                continue;
            }

            if arg.1 != None {
                arg_string += &format!("### _{} (optional)_ ", arg.0);
            } else {
                arg_string += &format!("### **{}** _(obligatory)_", arg.0);
            }

            if let Some(desc) = arg.2.get_desc() {
                arg_string += &format!(": _{}_", desc);
            }

            if let Some(def_val) = arg.1.clone() {
                arg_string += &format!(
                    "\n\n_Default value:_\n\n{}",
                    document_val(&def_val, globals)
                );
            }

            add_arrows(&mut arg_string);
            doc += &arg_string;

            doc += "\n\n\n\n\n";
        }
    }

    //arguments

    doc
}

fn document_val(val: &Value, globals: &Globals) -> String {
    let mut doc = String::new();

    let type_id = match val
        .member(TYPE_MEMBER_NAME.to_string(), &Context::new(), globals)
        .unwrap()
    {
        Value::TypeIndicator(t) => t,
        _ => unreachable!(),
    };

    let type_name =
        find_key_for_value(&globals.type_ids, type_id).expect("Implemented type was not found!");

    doc += &format!("**Type:** `{}` \n\n", type_name);
    doc += &format!("**Literal:** \n `{}` \n\n", val.to_str(globals));

    doc += &match &val {
        Value::Dict(d) => document_dict(d, globals),
        Value::Macro(m) => document_macro(m, globals),
        _ => String::new(),
    };

    add_arrows(&mut doc);

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
