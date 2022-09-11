use crate::pckp::util;
use crate::pckp::restore::restore;

pub fn remove(packages: Vec<&String>) {
    let mut pckp_file = util::get_pckp_file();

    pckp_file.meta.dependencies = pckp_file.meta.dependencies
        .iter()
        .filter(|dep| !packages.contains(dep))
        .map(|s| s.clone())
        .collect();

    pckp_file.write_back();
    restore();
}