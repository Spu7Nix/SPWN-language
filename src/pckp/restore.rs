use std::fs::{create_dir, read_to_string};
use std::path::PathBuf;
use async_recursion::async_recursion;

use super::{
    util::{get_pckp_file, PckpMeta, PckpDependency, PckpLockFile},
    package::RemotePackage
};

pub async fn restore() {
    calculate_dependencies("SpeckyYT/ANMT".to_string()).await;
}

// TODO: circular dependencies = you are dead
// this might be fixed by the `create_dir(path).unwrap()`
#[async_recursion]
async fn calculate_dependencies(root_pkg: String) -> PckpLockFile {
    let owner = root_pkg.split('/').collect::<Vec<_>>()[0];
    let repo = root_pkg.split('/').collect::<Vec<_>>()[1];
    let version = Option::None; // TODO: fix add version to this
    let root = RemotePackage::from_github_repo(owner.into(), repo.into(), None).await; // TODO: version
    let mut pkgs = PckpLockFile { dependencies: Vec::new() };
    
    let cwd = std::env::current_dir().unwrap();
    let pckp_folder = cwd.join(".pckp");
    let pckp_temp_folder = pckp_folder.join("tmp");
    let root_pkg_folder = pckp_temp_folder.join(github_folder_name(owner, repo, version));
    let root_pkg_pckp = root_pkg_folder.join("pckp.yaml");

    summon_folder(&pckp_folder);
    summon_folder(&pckp_temp_folder);
    summon_folder(&root_pkg_folder);

    root.download(&root_pkg_folder).await;

    let root_pckp = serde_yaml::from_str::<PckpMeta>(read_to_string(&root_pkg_pckp).unwrap().as_str()).unwrap(); // TODO: this errors on a non-pckp dependency

    for dep in root_pckp.dependencies {
        let dependencies = calculate_dependencies(dep).await.dependencies;
        for other_dep in dependencies {
            if pkgs.dependencies.contains(&other_dep) { continue } // recursion fix (in theory)

            match other_dep {
                PckpDependency::GitHub {
                    owner,
                    repo,
                    version,
                } => {
                    let pack = RemotePackage::from_github_repo(owner.clone(), repo.clone(), version.clone()).await;
                    pack.download(&root_pkg_folder).await;
                    pkgs.dependencies.push(PckpDependency::GitHub {
                        owner,
                        repo,
                        version,
                    });
                },
                PckpDependency::Path {
                    name,
                    path,
                } => {
                    pkgs.dependencies.push(PckpDependency::Path { name, path })
                },
            }
        }
    }

    pkgs
}

#[inline]
fn github_folder_name(owner: &str, repo: &str, version: Option<String>) -> String {
    format!("{}_{}.{}", owner, repo, version.unwrap_or("master".to_string()))
}

fn summon_folder(folder: &PathBuf) {
    if !folder.exists() { create_dir(folder).unwrap() }
}
