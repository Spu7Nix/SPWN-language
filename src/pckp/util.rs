use std::io::Write;
use std::process::exit;
use std::fs::{read_to_string, File, remove_file};
use std::path::PathBuf;
use regex::Regex;

use ansi_term::Colour::{Yellow, Red};

use serde::{Serialize, Deserialize};

pub struct PckpFile {
    pub path: PathBuf,
    pub meta: PckpMeta
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PckpMeta {
    pub name: String,
    pub version: String, // this should ALWAYS point to a valid tag on the remote
    pub folders: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>, // TODO: add dependencies
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum PckpDependency {
    GitHub {
        owner: String,
        repo: String,
        version: Option<String>,
    },
    Path {
        name: String,
        path: PathBuf,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PckpLockFile {
    pub dependencies: Vec<PckpDependency>,
}

impl PckpFile {
    pub fn new(path: PathBuf) -> Self {
        let code = read_to_string(&path).unwrap();
        Self {
            path,
            meta: serde_yaml::from_str::<PckpMeta>(&code).unwrap()
        }
    }
    pub fn write_back(&self) {
        let code = serde_yaml::to_string(&self.meta).unwrap();
        remove_file(&self.path).unwrap();
        let mut file = File::create(&self.path).unwrap();
        file.write_all(code.as_bytes()).unwrap();
    }
}

pub fn github_to_string(owner: &str, repo: &str, version: Option<&str>) -> String {
    // owner/repo@version
    format!("{}/{}", owner, repo) + &version.map(|s| format!("@{}",s)).unwrap_or_default()
}

impl PckpDependency {
    pub fn new(string: &str) -> PckpDependency {
        let github_regex = Regex::new(r"^(.+)/(.+)(@.+)?$").unwrap();
        let github_captures = github_regex.captures(string);

        if let Some(captures) = github_captures {
            PckpDependency::GitHub {
                owner: captures.get(0).unwrap().as_str().into(),
                repo: captures.get(1).unwrap().as_str().into(),
                version: captures.get(2).map(|s| s.as_str()[1..].to_string()), // removes @
            }
        } else {
            let lib_path = PathBuf::from(string);
            PckpDependency::Path {
                name: lib_path.file_name().unwrap().to_str().unwrap().to_string(),
                path: lib_path,
            }
        }
    }
}

pub fn get_pckp_file() -> PckpFile {
    let mut pwd = std::env::current_dir().unwrap();

    loop {
        pwd.push("pckp.yaml");
        if pwd.exists() {
            break PckpFile::new(pwd);
        }
        pwd.pop();
        pwd.pop();
        if pwd.as_os_str().len() == 3 || pwd.as_os_str().len() == 1 {
            pckp_error("Cant find pckp.yaml in any of the parent paths")
        } 
    }
}

/// use for unrecoverable errors
pub fn pckp_error(msg: &str) {
    println!("pckp {}: {}",
        Red.paint("ERR"),
        Red.paint(msg)
    );
    exit(1);
}

/// use for warnings / "recoverable" errors
pub fn pckp_warn(msg: &str) {
    println!("pckp {}: {}",
        Yellow.paint("WARN"),
        Yellow.paint(msg)
    );
}