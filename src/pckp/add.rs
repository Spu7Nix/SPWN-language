use crate::pckp::util;
use crate::pckp::restore::restore;

pub fn add(packages: Vec<&String>) {
    let mut pckp_file = util::get_pckp_file();
    for package in &packages {
        if ! pckp_file.meta.dependencies.contains(&package.to_string()) {
            pckp_file.meta.dependencies.push(package.to_string());
        } else {
            util::pckp_warn(format!("package {} already exists", package).as_str());
        }
    }
    pckp_file.write_back();
    restore();
}