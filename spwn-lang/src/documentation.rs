// tools for generating documentation for SPWN libraries
//use crate::ast::*;
use crate::builtin::TYPE_MEMBER_NAME;
use crate::compiler::{import_module, RuntimeError};
use crate::compiler_types::{
    find_key_for_value, store_value, CompilerInfo, Context, Globals, Macro, Value,
};
use crate::parser::ParseNotes;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn document_lib(path: &PathBuf) -> Result<String, RuntimeError> {
    let mut globals = Globals::new(ParseNotes::new(), path.clone());
    let start_context = Context::new();

    store_value(Value::Builtins, &mut globals, &start_context);
    store_value(Value::Null, &mut globals, &start_context);

    let module = import_module(
        path,
        &start_context,
        &mut globals,
        CompilerInfo {
            depth: 0,
            path: vec!["main scope".to_string()],
            line: (0, 0),
            func_id: 0,
        },
    )?;

    if module.len() > 1 {
        return Err(RuntimeError::RuntimeError {
            message: "Documentation of context-splitting libraries is not yet supported!"
                .to_string(),
            info: CompilerInfo::new(),
        });
    }

    let mut doc = format!(
        "# Documentation for `{}` \n",
        path.file_stem().unwrap().to_str().unwrap()
    );

    let exports = globals.stored_values[module[0].0].clone();
    let implementations = &module[0].1.implementations;

    doc += "_This file was generated using `spwn doc [file name]`_\n";

    doc += &format!("## Exports:\n{}", document_val(&exports, &mut globals));

    doc += "## Type Implementations:\n";

    let mut list: Vec<(&u16, &HashMap<String, usize>)> = implementations.iter().collect();
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
        .filter(|x| match globals.stored_values[*x.1] {
            Value::Macro(_) => true,
            _ => false,
        })
        .collect();
    macro_list.sort_by(|a, b| a.0.cmp(&b.0));

    let mut val_list: Vec<(&String, &usize)> = dict
        .iter()
        .filter(|x| match globals.stored_values[*x.1] {
            Value::Macro(_) => false,
            _ => true,
        })
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
    match mac.tag.get_desc() {
        Some(s) => doc += &format!("## Description: \n _{}_\n", s),
        None => (),
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

            if let Some(def_val) = arg.1.clone() {
                arg_string += &format!(
                    "\n\n_Default value:_\n\n{}\n\n",
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
