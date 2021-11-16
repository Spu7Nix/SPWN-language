use std::path::{PathBuf};
use std::fs;


use git2::{Repository};

use crate::error::PckpError;
use crate::version::{get_version_file, import_version, export_version};
use crate::config_file::{config_to_package, get_config};


use fs_extra::dir as fs_dir;

pub const PACKAGE_DIR: &str = "pckp_libraries";

#[derive(PartialEq, Clone, Debug)]
pub enum DependencySource {
	Name(String),
	Url(String)
}

#[derive(Clone, PartialEq, Debug)]
pub struct Dependency {
	pub source: DependencySource,
	pub version: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LocalPackage {
	pub name: String,
	pub version: String,
	pub paths: Vec<PathBuf>,
	pub dependencies: Vec<Package>
}

#[derive(Clone, PartialEq, Debug)]
pub enum PackageType {
	Local(LocalPackage),
	External(Dependency)
}

#[derive(Clone, PartialEq, Debug)]
pub struct Package {
	internal: PackageType
}

impl Package {
	pub fn local(name: String, version: String, paths: Vec<PathBuf>, dependencies: Vec<Package>) -> Package {
		Package {
			internal: PackageType::Local(LocalPackage {
				name, version, paths, dependencies
			})
		}
	}

	pub fn dependency(dep: Dependency) -> Package {
		Package {
			internal: PackageType::External(dep)
		}
	}

	pub fn install_dependencies(&self, path: PathBuf) -> Result<(), PckpError> {
		match &self.internal {
			PackageType::Local(root) => {
				for x in &root.dependencies {
				    x.install(&root.name, path.clone(), false)?;
				}
				Ok(())
			},
			_ => unreachable!("ensure_local")
		}
	}

	#[allow(dead_code)]
	fn get_version(&self) -> String {
		match &self.internal {
			PackageType::Local(p) => p.version.clone(),
			PackageType::External(d) => d.version.clone()
		}
	}
	pub fn install(&self, parent_name: &str, path: PathBuf, ignore_version: bool) -> Result<(), PckpError> {
		match &self.internal {
			PackageType::Local(p) => {
				/*for folder in &p.paths {
					// folder guaranteed to exist from config parser

				}*/
				let mut dest = path.to_path_buf();
				dest.push(PACKAGE_DIR);

				if !dest.exists() {
					fs::create_dir(&dest).unwrap();
				}

				let version_file = get_version_file(path.clone());
				let mut version_info = import_version(&version_file);

				if !version_info.iter().any(|(n, v)| n == &p.name && v == &p.version) {
					println!("Installing {}", p.name);
					
					let new_path = if ignore_version {
						p.name.to_string()
					} else {
						format!("{}@{}", p.name, p.version)
					};
					dest.push(new_path.to_string());

					for folder in &p.paths {
						let mut opts = fs_dir::CopyOptions::new();
						opts.content_only = true;
						fs_dir::copy(folder, &dest, &opts).unwrap();
					}

					version_info.push((p.name.clone(), p.version.clone()));
					//println!("package {:#?}", p);
				}

				for dep in &p.dependencies {
					dep.install(&p.name, path.clone(), false)?;
				}

				export_version(version_info, &version_file);
				Ok(())
			},
			PackageType::External(d) => {
				let source_url = d.source.to_string(parent_name.to_string())?;

				let tmp_path = PathBuf::from(".pckp_tmp");

				if tmp_path.exists() {
					fs_dir::remove(&tmp_path).unwrap();
				}
				let repo = match Repository::clone(&source_url, &tmp_path) {
				    Ok(repo) => repo,
				    Err(e) => {
				    	return Err(PckpError::custom(format!("Unable to clone package '{}'. Reason: {}", source_url, e), Some(parent_name.to_string())))
				    },
				};

				if d.version != "latest" {
					let tag_object = match repo.revparse_single(&("refs/tags/".to_string() + &d.version)) {
						Ok(x) => x,
						Err(_) => {
							return Err(PckpError::custom(format!("Unable to find version {} for package {}", d.version, source_url), Some(parent_name.to_string())))
						}
					};

					repo.checkout_tree(
					    &tag_object,
					    None
					).unwrap();
				}

				let cfg_dir = get_config(Some(tmp_path));
				let local_package = if cfg_dir.exists() {
					config_to_package(cfg_dir)?.unwrap()
				} else {
					return Err(PckpError::custom(format!("Package at {} does not have config file", source_url), Some(parent_name.to_string())))
				};

				local_package.install(parent_name, path, d.version == "latest")
				//todo!("download and stuff");
			}
		}
	}
}