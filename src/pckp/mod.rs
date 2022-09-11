pub mod util;
pub mod add;
pub mod remove;
pub mod restore;
pub mod package;
pub mod new;

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
        ("new", cmd) => {
            new::new(
                cmd.get_one::<String>("FOLDER").unwrap().to_string()
            );
        },
        (_,_) => unreachable!(),
    }

    return;
}
