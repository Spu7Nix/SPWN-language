pub mod util;
pub mod add;
pub mod remove;
pub mod restore;
pub mod package;

use clap::ArgMatches;

pub async fn run(args: &ArgMatches) {
    match args.subcommand().unwrap() {
        ("add", cmd) => {
            add::add(
                cmd.get_many::<String>("LIBRARIES").unwrap().collect()
            ).await;
        },
        ("remove", cmd) => {
            remove::remove(
                cmd.get_many::<String>("LIBRARIES").unwrap().collect()
            );
        },
        ("restore", _cmd) => {
            restore::restore();
        },
        (_,_) => unreachable!(),
    }

    return;
}
