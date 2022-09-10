use crate::pckp::util;
use crate::pckp::restore::restore;
use crate::pckp::package;

pub async fn add(packages: Vec<&String>) {
    let mut pckp_file = util::get_pckp_file();
    for package in &packages {
        if ! pckp_file.meta.dependencies.contains(&package.to_string()) {
            pckp_file.meta.dependencies.push(package.to_string());
            let owner = package.split("/").collect::<Vec<_>>()[0].to_string();
            let repo = package.split("/").collect::<Vec<_>>()[1].to_string();
            package::RemotePackage::from_github_repo(owner, repo).await;
        } else {
            util::pckp_warn(format!("package {} already exists", package).as_str());
        }
    }
    pckp_file.write_back();
    restore();
}