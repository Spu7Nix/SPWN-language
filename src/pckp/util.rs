use std::io::Write;
use std::process::exit;
use std::fs::{read_to_string, File, remove_file};
use std::path::PathBuf;

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
    pub dependencies: Vec<String> // TODO: add dependencies
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