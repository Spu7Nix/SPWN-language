use crate::package::PACKAGE_DIR;
use std::path::{Path, PathBuf};
use std::fs;

pub const VERSION_FILE_NAME: &'static str = ".vrsn";
type VersionFile = Vec<(String, String)>;


pub fn get_version_file(mut pckp_dir: PathBuf) -> PathBuf {
	pckp_dir.push(PACKAGE_DIR);
	pckp_dir.push(VERSION_FILE_NAME);
	pckp_dir
}

pub fn export_version(version: VersionFile, path: &Path) {
	let output = version.into_iter().map(|(n, v)| format!("{}:{}", n, v)).collect::<Vec<_>>().join(",");
	fs::write(path, output).unwrap();
}

pub fn import_version(path: &Path) -> VersionFile {
	if !path.exists() {
		export_version(VersionFile::new(), path);
	}
	let a = fs::read(path).unwrap();

	if a.is_empty() {
		return VersionFile::new();
	}

	let input = std::str::from_utf8(&a).unwrap();
	input
		.split(",")
		.into_iter()
		.map(|x| {
			let mut b = x.split(":");
			(b.next().unwrap().to_string(), b.next().unwrap().to_string())
		}).collect::<VersionFile>()
}