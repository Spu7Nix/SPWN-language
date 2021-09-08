pub use ::compiler::builtins;
pub use ::compiler::compiler;
pub use ::compiler::compiler_types;
pub use ::compiler::context;
pub use ::compiler::globals;
pub use ::compiler::leveldata;
pub use ::compiler::value;
pub use ::compiler::value_storage;
pub use ::docgen::documentation;
pub use ::parser::ast;

pub use ::compiler::STD_PATH;

pub use ::parser::fmt;
pub use ::parser::parser;
pub use ::parser::parser::parse_spwn;

pub use errors::compiler_info;
pub use optimize;

use std::path::PathBuf;

use std::io;

#[derive(Clone)]
pub struct Compiler {
    opti_enabled: bool,
    included_paths: Vec<PathBuf>,
    unparsed_code: String,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            opti_enabled: true,
            included_paths: vec![
                std::env::current_dir().expect("Cannot access current directory"),
                std::env::current_exe()
                    .expect("Cannot access directory of executable")
                    .parent()
                    .expect("Executable must be in some directory")
                    .to_path_buf(),
            ],
            unparsed_code: String::new(),
        }
    }
    pub fn add_include(&mut self, path_str: String) -> io::Result<()> {
        self.included_paths.push({
            let path = PathBuf::from(path_str);
            path.read_dir()?;
            path
        });
        Ok(())
    }
    pub fn set_code(&mut self, code: String) {
        self.unparsed_code = code;
    }
    /*pub fn _run(code: String, included_paths: Vec<PathBuf>, opti_enabled: bool) -> Result<String, Box<dyn std::error::Error>> {
        let file = NamedTempFile::new()?;
        let mut pbuf = PathBuf::new();
        pbuf.push(file.path());
        let (statements, notes) = parse_spwn(code, pbuf.clone())?;

        let mut compiled = compiler::compile_spwn(
            statements,
            pbuf,
            included_paths,
            notes,
        )?;

        let has_stuff = compiled.func_ids.iter().any(|x| !x.obj_list.is_empty());
        if opti_enabled && has_stuff {
            compiled.func_ids = optimize(compiled.func_ids, compiled.closed_groups);
        }

        let mut objects = levelstring::apply_fn_ids(&compiled.func_ids);

        objects.extend(compiled.objects);

        let (new_ls, _) = levelstring::append_objects(objects, &String::new())?;

        Ok(new_ls)
    }*/
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
