use std::fs::{create_dir, read_to_string};
use async_recursion::async_recursion;

use super::{util::{get_pckp_file, PckpMeta}, package::RemotePackage};

pub fn restore() {
    let meta = get_pckp_file();
    
    for dep in meta.meta.dependencies {

    }
}

// TODO: circular dependencies = you are dead
// this might be fixed by the `create_dir(path).unwrap()`
#[async_recursion]
async fn calculate_dependencies(root_pkg: String) -> Vec<String> {
    let owner = root_pkg.split("/").collect::<Vec<_>>()[0].to_string();
    let repo = root_pkg.split("/").collect::<Vec<_>>()[1].to_string();
    let root = RemotePackage::from_github_repo(owner, repo).await;
    let mut path = get_pckp_file().path;
    let mut pkgs = Vec::new();
    path.pop();
    path.push(".pckp");
    path.push("tmp");
    path.push(root_pkg.replace("/", "."));
    create_dir(&path).unwrap();
    root.download(&path);
    path.push("pckp.yaml");
    let root_pckp = serde_yaml::from_str::<PckpMeta>(read_to_string(&path).unwrap().as_str()).unwrap(); // TODO: this errors on a non-pckp dependency

    for dep in root_pckp.dependencies {
        let dependencies = calculate_dependencies(dep).await;
        for other_dep in dependencies {
            let owner = other_dep.split(".").collect::<Vec<_>>()[0].to_string();
            let repo = other_dep.split(".").collect::<Vec<_>>()[1].to_string();
            let pack = RemotePackage::from_github_repo(owner.clone(), repo.clone()).await;
            path.pop();
            path.pop();
            path.push(format!("{}.{}", owner, repo));
            pack.download(&path);
            pkgs.push(format!("{}.{}", owner, repo));
        }
    }

    return pkgs;
}