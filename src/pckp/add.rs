use crate::pckp::util;
use crate::pckp::restore::restore;
use crate::pckp::package;

pub async fn add(packages: Vec<&String>) {
    let mut pckp_file = util::get_pckp_file();
    for package in &packages {
        if !pckp_file.meta.dependencies.contains(&package.to_string()) {
            let (owner, repo) = match package.split("/").collect::<Vec<&str>>()[..] {
                [owner, repo] => (owner, repo),
                _ => {
                    util::pckp_warn(format!("input package `{}` is not valid", package).as_str());
                    continue;
                }
            };
            package::RemotePackage::from_github_repo(owner.to_string(), repo.to_string()).await;
            pckp_file.meta.dependencies.push(package.to_string());
        } else {
            util::pckp_warn(format!("package `{}` already exists", package).as_str());
        }
    }
    pckp_file.write_back();
    restore();
}