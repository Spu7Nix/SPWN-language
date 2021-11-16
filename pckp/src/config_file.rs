use std::path::Path;
use std::path::PathBuf;
use std::fs;

use yaml_rust::{YamlLoader, Yaml};
use path_absolutize::*;

use crate::package::{Package, Dependency, DependencySource};
use crate::error::PckpError;

pub const CONFIG_NAME: &str = "pckp.yaml";

#[allow(dead_code)]
struct MarkerPub {
    index: usize,
    line: usize,
    col: usize
}

struct ScanErrorPub {
    pub mark: MarkerPub,
    pub info: String
}

impl ScanErrorPub {
    fn from(pri: yaml_rust::scanner::ScanError) -> Self {
        return unsafe {
             std::mem::transmute(pri)
        }; // this is horrible ik
    }
}

fn yaml_to_str(y: &Yaml) -> String {
    match y {
        Yaml::Real(s) => s.to_string(),
        Yaml::Integer(i) => i.to_string(),
        Yaml::String(s) => s.to_string(),
        Yaml::Boolean(b) => if *b {"true".to_string()} else {"false".to_string()},
        Yaml::Array(a) => format!("[{}]", a.into_iter().map(|x| yaml_to_str(x)).collect::<Vec<_>>().join(",")),
        Yaml::Hash(h) => format!("{{{}}}", h.into_iter().map(|(k, v)| format!("{}: {}", yaml_to_str(k), yaml_to_str(v))).collect::<Vec<_>>().join(",")),
        Yaml::Alias(a) => a.to_string(),
        Yaml::Null => "Null".to_string(),
        Yaml::BadValue => "`Invalid Value`".to_string(),
    }
}

struct YamlMap {
    pub internal: yaml_rust::yaml::Hash,
    pub cfg: PathBuf
}

impl YamlMap {
    fn from_hash(parent: &str, hash: &Yaml, cfg: &PathBuf) -> Result<Self, PckpError> {
        match hash.as_hash() {
            Some(a) => Ok(YamlMap {internal: a.clone(), cfg: cfg.clone()}),
            a => {
                Err(PckpError::config(format!("Expected dictionary in {}, found {:?}", parent, a), cfg.clone(), None))
            }
        }
    }

    fn get(&self, parent: &str, key: &str) -> Result<Yaml, PckpError> {
        match self.internal.get(&Yaml::from_str(key)) {
            Some(a) => Ok(a.clone()),
            None => Err(PckpError::config(format!("Expected to find key '{}' in '{}'", key, parent), self.cfg.clone(), None))
        }
    }

    fn get_or_else<F>(&self, key: &str, cb: F) -> Result<Yaml, PckpError> 
    where F: FnOnce(&Self) -> Result<Yaml, PckpError>

    {
        match self.internal.get(&Yaml::from_str(key)) {
            Some(a) => Ok(a.clone()),
            None => cb(self)
        }
    }
}
macro_rules! ensure_variant {
    ($val:expr, $variant_name:tt = $variant:ident, $key:tt from $parent:tt) => {
        match $val.get($parent, $key)? {
            Yaml::$variant(a) => Ok(a),
            _ => Err(PckpError::config(format!("Expected key '{}' to be of type {:?}", $key, $variant_name), $val.cfg.clone(), None))
        }
    };

    ($val:expr, $variant_name:tt = $variant:ident, $key:tt or $key2:tt from $parent:tt, enum $src:expr, $src2:expr) => {
        match $val.get_or_else($key, |v| match v.internal.get(&Yaml::from_str($key2)) {
            Some(a) => Ok(a.clone()),
            None => Err(PckpError::config(format!("Expected to find either keys '{}' or '{}' in '{}'", $key, $key2, $parent), v.cfg.clone(), None))
        })? {
            Yaml::$variant(a) => if $val.get($parent, $key).is_ok() {Ok($src(a))} else {Ok($src2(a))},
            _ => Err(PckpError::config(format!("Expected key '{}' to be of type {:?}", if $val.get($parent, $key).is_ok() {$key} else {$key2}, $variant_name), $val.cfg.clone(), None))
        }
    };

    ($val:expr, $variant_name:tt = $variant:ident, $key:tt? from $parent:tt) => {
        match $val.internal.get(&Yaml::from_str($key)) {
            Some(Yaml::$variant(a)) => Ok(Some(a)),
            Some(_) => Err(PckpError::config(format!("Expected key '{}' to be of type {:?}", $key, $variant_name), $val.cfg.clone(), None)),
            None => Ok(None)
        }
    }
}

pub fn get_config(opath: Option<PathBuf>) -> PathBuf {
    let mut path = opath.unwrap_or(PathBuf::new());
    path.push(CONFIG_NAME);
    path
}

fn check_invalid(n: &str) -> Option<char> {
    let mut b = [0; 4];
    let potential_invalid: Vec<char> = n.chars().filter(|x| !String::from("qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLXCVBNM_-.0123456789").contains(&*x.encode_utf8(&mut b))).collect();

    potential_invalid.first().map(|x| *x)
}

pub fn config_to_package(cfg: PathBuf) -> Result<Option<Package>, PckpError> {

    if cfg.exists() {
        match fs::read_to_string(cfg.clone()) {
            Ok(s) => {
                let yaml = match YamlLoader::load_from_str(&s) {
                    Ok(y) => y,
                    Err(e) => {
                        let public = ScanErrorPub::from(e);

                        return Err(PckpError::config(public.info, cfg, Some((public.mark.line, public.mark.col))))
                    }
                };
                if yaml.len() == 0 {
                    return Err(PckpError::config("Config file must include a package name".to_string(), cfg, None))
                }

                let ymap = YamlMap::from_hash("root", &yaml[0], &cfg)?;

                let package_name = ensure_variant!(ymap, "string" = String, "name" from "root")?;
                let version = ensure_variant!(ymap, "string" = String, "version" from "root")?;

                if let Some(bad) = check_invalid(&package_name) {
                    return Err(PckpError::config(format!("Invalid character {} in package name {}", bad, package_name), cfg, None));
                }
                if let Some(bad) = check_invalid(&version) {
                    return Err(PckpError::config(format!("Invalid character {} for version {} of package {}", bad, version, package_name), cfg, None));
                }

                let mut folders = Vec::new();

                let f_list = ensure_variant!(ymap, "list" = Array, "folders"? from "root");
                let f_str = ensure_variant!(ymap, "string" = String, "folders"? from "root");

                let folders_maybe = match (f_list, f_str) {
                    (_, Ok(b)) => Ok(b.map(|x| vec![Yaml::String(x.to_string())])),
                    (a, _) => a.map(|x| x.map(|y| y.clone()))
                }?;

                match folders_maybe {
                    Some(folds) => {
                        for v in folds {
                            folders.push(match v.clone() {
                                Yaml::String(a) => {
                                    match Path::new(&a).absolutize_virtually(cfg.parent().unwrap().as_os_str().to_str().unwrap()) {
                                        Ok(p) => {
                                            let s = PathBuf::from(p.to_str().unwrap());
                                            if !s.exists() {
                                                return Err(PckpError::config(
                                                    format!("Cannot access folder '{}' which doesn't exist", a),
                                                    cfg.clone(),
                                                    None
                                                ));
                                            } else if !s.is_dir() {
                                                return Err(PckpError::config(
                                                    format!("Expected '{}' to be a folder", a),
                                                    cfg.clone(),
                                                    None
                                                ));
                                            }
                                            s
                                        },
                                        Err(_) => return Err(PckpError::config(
                                            format!("Cannot access folder '{}' which is outside of package directory", a),
                                            cfg.clone(),
                                            None
                                        ))
                                    }
                                },
                                b => return Err(
                                    PckpError::config(
                                        format!("Expected list element {:?} to be of type string", b),
                                        cfg.clone(),
                                        None
                                    )
                                )
                            });
                        }
                        ()
                    },
                    None => folders.push(PathBuf::new().absolutize_virtually(cfg.parent().unwrap().as_os_str().to_str().unwrap()).unwrap().to_path_buf())
                }

                let depends = match ymap.internal.get(&Yaml::String("dependencies".to_string())) {
                    Some(depend_node) => {
                        match depend_node.clone() {
                            Yaml::Array(arr) => {
                                //arr
                                arr.into_iter().map(|d_node| {
                                    match d_node {
                                        Yaml::String(s) => {
                                            Ok(Dependency {
                                                source: DependencySource::Name(s),
                                                version: "latest".to_string()
                                            })
                                        },
                                        Yaml::Hash(h) => {
                                            let dmap = YamlMap::from_hash("dependencies", &Yaml::Hash(h.clone()), &cfg)?;

                                            Ok(Dependency {

                                                source: ensure_variant!(
                                                    dmap, 
                                                    "string" = String, 
                                                    "name" or "url" from "dependencies",
                                                    enum DependencySource::Name, DependencySource::Url
                                                )?,

                                                version: yaml_to_str(&dmap.get_or_else("version", |_| Ok(Yaml::String("latest".to_string())))?)
                                            })
                                        },
                                        c => Err(
                                            PckpError::config(
                                                format!("{:?} cannot be parsed as a dependency", c),
                                                cfg.clone(),
                                                None
                                            )
                                        )
                                    }
                                }).collect::<Result<Vec<_>, _>>()?
                            },
                            Yaml::String(s) => {
                                vec![Dependency {
                                    source: DependencySource::Name(s),
                                    version: "latest".to_string()
                                }]
                            },
                            Yaml::Hash(h) => {
                                let dmap = YamlMap::from_hash("dependencies", &Yaml::Hash(h.clone()), &cfg)?;

                                vec![Dependency {

                                    source: ensure_variant!(
                                        dmap, 
                                        "string" = String, 
                                        "name" or "url" from "dependencies",
                                        enum DependencySource::Name, DependencySource::Url
                                    )?,

                                    version: yaml_to_str(&dmap.get_or_else("version", |_| Ok(Yaml::String("latest".to_string())))?)
                                }]
                            },
                            c => return Err(
                                PckpError::config(
                                    format!("{:?} cannot be parsed as a dependency", c),
                                    cfg.clone(),
                                    None
                                )
                            )
                        }
                    },
                    None => Vec::new()
                }.into_iter()
                 .map(|dep| Package::dependency(dep))
                 .collect::<Vec<_>>();

                return Ok(Some(Package::local(package_name, version, folders, depends)));
            },
            Err(_) => {
                return Err(PckpError::config("Could not open configuration file".to_string(), cfg, None));
            }
        }
    } else {
        Ok(None)
    }
}