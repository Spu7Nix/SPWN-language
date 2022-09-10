use crate::pckp::util;
use crate::pckp::restore::restore;

pub fn remove(packages: Vec<&String>) {
    let mut pckp_file = util::get_pckp_file();

    let mut idx = 0;
    for package in packages {
        if pckp_file.meta.dependencies.contains(&package.to_string()) {
            pckp_file.meta.dependencies.remove(idx);
            // compensate for item shift
            if idx != 0 {
                idx -= 1;
            }
        } else {
            util::pckp_warn(format!("package {} doesn't exist", package).as_str())
        }
        idx += 1;
    }
    pckp_file.write_back();
    restore();
}