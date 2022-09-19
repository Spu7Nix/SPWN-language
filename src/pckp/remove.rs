use crate::pckp::util;
use crate::pckp::restore::restore;

pub async fn remove(packages: Vec<&String>) {
    let mut pckp_file = util::get_pckp_file();

    pckp_file.meta.dependencies = pckp_file.meta.dependencies
        .iter()
        .filter(|dep| !packages.contains(dep))
        .cloned()
        .collect();

    pckp_file.write_back();
    restore().await;
}