use crate::pckp::util;
use crate::pckp::restore::restore;
use crate::pckp::package;
use std::path::PathBuf;

pub async fn add(packages: Vec<&String>) {
    let mut pckp_file = util::get_pckp_file();
    for package in packages {
        if pckp_file.meta.dependencies.contains(&package.to_string()) {
            util::pckp_warn(format!("package `{}` already exists", package).as_str());
            continue;
        }

        let mut push_dependency = || pckp_file.meta.dependencies.push(package.to_string());

        if PathBuf::from(package).is_dir() || PathBuf::from(package).is_file() {
            push_dependency();
            continue;
        }

        let github_repo = &package.split(&['/','@'][..]).collect::<Vec<&str>>()[..];
        match github_repo {
            [ owner, repo ] => {
                package::RemotePackage::from_github_repo(owner.to_string(), repo.to_string(), None).await;
                push_dependency();
                continue;
            },
            [ owner, repo, version ] => {
                package::RemotePackage::from_github_repo(owner.to_string(), repo.to_string(), Some(version.to_string())).await;
                push_dependency();
                continue;
            },
            _ => (),
        }

        util::pckp_warn(format!("input package `{}` is not valid", package).as_str());
    }
    pckp_file.write_back();
    restore().await;
}