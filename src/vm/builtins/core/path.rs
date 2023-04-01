use std::env;
use std::fs;
use std::path::Component;
use std::path::PathBuf;

use crate::sources::CodeArea;
use crate::vm::builtins::builtin_utils::impl_type;
use crate::vm::value::{Value, StoredValue};

impl_type! {
    impl Path {
        Constants:

        Functions(vm, call_area):

        fn new(path: String) -> Path {
            Value::Path(PathBuf::from(path.0.iter().collect::<String>()))
        }

        fn cwd() -> Path {
            Value::Path(env::current_dir().unwrap_or(PathBuf::from("./")))
        }

        // MODIFY 
        // returned
        fn join(Path(path) as self, sub_path: String | Path) -> Path {
            let sub_path = match sub_path {
                SubPathValue::String(string) => PathBuf::from(string.iter().collect::<String>()),
                SubPathValue::Path(path) => path.to_path_buf(),
            };
            Value::Path(path.join(sub_path))
        }
        fn parent(Path(path) as self) -> Path {
            let mut path = path.clone();
            path.pop();
            Value::Path(path)
        }
        // in-place
        fn push(slf: &Path, sub_path: String | Path) {
            let path = match sub_path {
                SubPathValue::String(string) => PathBuf::from(string.iter().collect::<String>()),
                SubPathValue::Path(path) => path.to_path_buf(),
            };
            slf.get_mut_ref(vm).push(path);
            Value::Empty
        }
        fn pop(slf: &Path) {
            slf.get_mut_ref(vm).pop();
            Value::Empty
        }

        fn is_absolute(Path(path) as self) -> Bool {
            Value::Bool(path.is_absolute())
        }
        fn is_relative(Path(path) as self) -> Bool {
            Value::Bool(path.is_relative())
        }

        fn split(Path(path) as self) -> Array {
            Value::Array(
                path
                .components()
                .enumerate()
                .filter_map(|(i, component)|
                    match component {
                        Component::RootDir => if i == 0 { Some("/") } else { None },
                        Component::CurDir => Some("./"),
                        Component::ParentDir => Some(".."),
                        Component::Normal(string) => Some(string.to_str().unwrap()),
                        Component::Prefix(prefix) => Some(prefix.as_os_str().to_str().unwrap()),
                    }
                )
                .map(|string| string.chars().collect::<Vec<char>>())
                .map(|string| StoredValue {
                    value: Value::String(string),
                    area: CodeArea {
                        src: crate::sources::SpwnSource::Core(Default::default()),
                        span: Default::default(),
                    },
                })
                .map(|stored_value| vm.memory.insert(stored_value))
                .collect()
            )
        }

        // FS DATA METHODS
        fn exists(Path(path) as self) -> Bool {
            Value::Bool(path.exists())
        }
        fn kind(Path(path) as self) -> String {
            match path.metadata() {
                Ok(metadata) => {
                    Value::String(match metadata.file_type() {
                        meta if meta.is_file() => "file",
                        meta if meta.is_dir() => "dir",
                        _ => "unknown",
                    }.chars().collect())
                },
                Err(err) => {
                    todo!()
                },
            }
        }
        fn metadata(Path(path) as self) -> Dict {
            match path.metadata() {
                Ok(metadata) => {
                    dict!{
                        length: Value::Int(metadata.len() as i64),
                        kind: Value::String(format!("{:?}", metadata.file_type()).chars().collect::<Vec<_>>()),
                    }
                },
                Err(err) => {
                    todo!()
                    // Value::Empty
                },
            }
        }

        // FILE METHODS
        fn write(Path(path) as self, content: String) {
            fs::write(path, content.0.iter().collect::<String>()).unwrap();
            Value::Empty
        }
        fn read(Path(path) as self) -> String {
            Value::String(fs::read(path).unwrap().iter().map(|byte| *byte as char).collect())
        }
        fn remove(Path(path) as self) {
            fs::remove_file(path).unwrap();
            Value::Empty
        }

        // FOLDER METHODS
        fn read_dir(Path(path) as self) -> Array {
            match path.read_dir() {
                Ok(dirs) => {
                    Value::Array(
                        dirs
                        .into_iter()
                        .filter_map(|f| f.ok().map(|f| vm.memory.insert(StoredValue {
                            value: Value::Path(f.path()),
                            area: CodeArea {
                                src: crate::sources::SpwnSource::Core(Default::default()),
                                span: Default::default(),
                            }
                        })))
                        .collect::<Vec<_>>()
                    )
                },
                Err(err) => {
                    todo!()
                },
            }
        }
        fn create_dir(Path(path) as self, all: Bool = {false}) {
            let error = if *all {
                fs::create_dir_all(path)
            } else {
                fs::create_dir(path)
            };
            Value::Empty
        }
        fn remove_dir(Path(path) as self, all: Bool = {false}) {
            let error = if *all {
                fs::remove_dir_all(path)
            } else {
                fs::remove_dir(path)
            };
            Value::Empty
        }
    }
}