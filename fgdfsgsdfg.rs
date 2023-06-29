// Recursive expansion of Subcommand macro
// ========================================

#[allow(dead_code, unreachable_code, unused_variables, unused_braces)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting
)]
#[deny(clippy::correctness)]
impl clap::FromArgMatches for Command {
    fn from_arg_matches(
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        Self::from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }

    fn from_arg_matches_mut(
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        #![allow(deprecated)]
        if let Some((__clap_name, mut __clap_arg_sub_matches)) =
            __clap_arg_matches.remove_subcommand()
        {
            let __clap_arg_matches = &mut __clap_arg_sub_matches;
            if __clap_name == "build" && !__clap_arg_matches.contains_id("") {
                return ::std::result::Result::Ok(Self::Build {
                    file: __clap_arg_matches
                        .remove_one::<PathBuf>("file")
                        .ok_or_else(|| {
                            clap::Error::raw(
                                clap::error::ErrorKind::MissingRequiredArgument,
                                format!(
                                    "The following required argument was not provided: {}",
                                    "file"
                                ),
                            )
                        })?,
                    settings: <BuildSettings as clap::FromArgMatches>::from_arg_matches_mut(
                        __clap_arg_matches,
                    )?,
                });
            }
            if __clap_name == "doc" && !__clap_arg_matches.contains_id("") {
                return ::std::result::Result::Ok(Self::Doc {
                    settings: <DocSettings as clap::FromArgMatches>::from_arg_matches_mut(
                        __clap_arg_matches,
                    )?,
                });
            }
            ::std::result::Result::Err(clap::Error::raw(
                clap::error::ErrorKind::InvalidSubcommand,
                format!("The subcommand '{}' wasn't recognized", __clap_name),
            ))
        } else {
            ::std::result::Result::Err(clap::Error::raw(
                clap::error::ErrorKind::MissingSubcommand,
                "A subcommand is required but one was not provided.",
            ))
        }
    }

    fn update_from_arg_matches(
        &mut self,
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        self.update_from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }

    fn update_from_arg_matches_mut<'b>(
        &mut self,
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        #![allow(deprecated)]
        if let Some(__clap_name) = __clap_arg_matches.subcommand_name() {
            match self {
                Self::Build { file, settings } if "build" == __clap_name => {
                    let (_, mut __clap_arg_sub_matches) =
                        __clap_arg_matches.remove_subcommand().unwrap();
                    let __clap_arg_matches = &mut __clap_arg_sub_matches;
                    {
                        if __clap_arg_matches.contains_id("file") {
                            *file = __clap_arg_matches
                                .remove_one::<PathBuf>("file")
                                .ok_or_else(|| {
                                    clap::Error::raw(
                                        clap::error::ErrorKind::MissingRequiredArgument,
                                        format!(
                                            "The following required argument was not provided: {}",
                                            "file"
                                        ),
                                    )
                                })?
                        }
                        {
                            <BuildSettings as clap::FromArgMatches>::update_from_arg_matches_mut(
                                settings,
                                __clap_arg_matches,
                            )?;
                        }
                    }
                },
                Self::Doc { settings } if "doc" == __clap_name => {
                    let (_, mut __clap_arg_sub_matches) =
                        __clap_arg_matches.remove_subcommand().unwrap();
                    let __clap_arg_matches = &mut __clap_arg_sub_matches;
                    {
                        {
                            <DocSettings as clap::FromArgMatches>::update_from_arg_matches_mut(
                                settings,
                                __clap_arg_matches,
                            )?;
                        }
                    }
                },
                s => {
                    *s = <Self as clap::FromArgMatches>::from_arg_matches_mut(__clap_arg_matches)?;
                },
            }
        }
        ::std::result::Result::Ok(())
    }
}
#[allow(dead_code, unreachable_code, unused_variables, unused_braces)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting
)]
#[deny(clippy::correctness)]
impl clap::Subcommand for Command {
    fn augment_subcommands<'b>(__clap_app: clap::Command) -> clap::Command {
        let __clap_app = __clap_app;
        let __clap_app = __clap_app.subcommand({
            let __clap_subcommand = clap::Command::new("build");
            {
                let __clap_subcommand =
                    __clap_subcommand.group(clap::ArgGroup::new("Build").multiple(true).args({
                        let members: [clap::Id; 0] = [];
                        members
                    }));
                let __clap_subcommand = __clap_subcommand.arg({
                    #[allow(deprecated)]
                    let arg = clap::Arg::new("file")
                        .value_name("FILE")
                        .required(true && clap::ArgAction::Set.takes_values())
                        .value_parser(clap::value_parser!(PathBuf))
                        .action(clap::ArgAction::Set);
                    let arg = arg;
                    let arg = arg;
                    arg
                });
                let __clap_subcommand = __clap_subcommand;
                let __clap_subcommand =
                    <BuildSettings as clap::Args>::augment_args(__clap_subcommand);
                __clap_subcommand
            }
        });
        let __clap_app = __clap_app.subcommand({
            let __clap_subcommand = clap::Command::new("doc");
            {
                let __clap_subcommand =
                    __clap_subcommand.group(clap::ArgGroup::new("Doc").multiple(true).args({
                        let members: [clap::Id; 0] = [];
                        members
                    }));
                let __clap_subcommand = __clap_subcommand;
                let __clap_subcommand =
                    <DocSettings as clap::Args>::augment_args(__clap_subcommand);
                __clap_subcommand
            }
        });
        __clap_app
    }

    fn augment_subcommands_for_update<'b>(__clap_app: clap::Command) -> clap::Command {
        let __clap_app = __clap_app;
        let __clap_app = __clap_app.subcommand({
            let __clap_subcommand = clap::Command::new("build");
            {
                let __clap_subcommand =
                    __clap_subcommand.group(clap::ArgGroup::new("Build").multiple(true).args({
                        let members: [clap::Id; 0] = [];
                        members
                    }));
                let __clap_subcommand = __clap_subcommand.arg({
                    #[allow(deprecated)]
                    let arg = clap::Arg::new("file")
                        .value_name("FILE")
                        .required(true && clap::ArgAction::Set.takes_values())
                        .value_parser(clap::value_parser!(PathBuf))
                        .action(clap::ArgAction::Set);
                    let arg = arg;
                    let arg = arg.required(false);
                    arg
                });
                let __clap_subcommand = __clap_subcommand;
                let __clap_subcommand =
                    <BuildSettings as clap::Args>::augment_args_for_update(__clap_subcommand);
                __clap_subcommand
            }
        });
        let __clap_app = __clap_app.subcommand({
            let __clap_subcommand = clap::Command::new("doc");
            {
                let __clap_subcommand =
                    __clap_subcommand.group(clap::ArgGroup::new("Doc").multiple(true).args({
                        let members: [clap::Id; 0] = [];
                        members
                    }));
                let __clap_subcommand = __clap_subcommand;
                let __clap_subcommand =
                    <DocSettings as clap::Args>::augment_args_for_update(__clap_subcommand);
                __clap_subcommand
            }
        });
        __clap_app
    }

    fn has_subcommand(__clap_name: &str) -> bool {
        if "build" == __clap_name {
            return true;
        }
        if "doc" == __clap_name {
            return true;
        }
        false
    }
}

// =================================




😭


// Recursive expansion of Args macro
// ==================================

#[allow(dead_code, unreachable_code, unused_variables, unused_braces)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting
)]
#[deny(clippy::correctness)]
impl clap::FromArgMatches for DocSettings {
    fn from_arg_matches(
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        Self::from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }

    fn from_arg_matches_mut(
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        #![allow(deprecated)]
        let v = DocSettings {
            module: __clap_arg_matches.remove_one::<String>("module"),
            lib: __clap_arg_matches.remove_one::<String>("lib"),
            target_dir: __clap_arg_matches.remove_one::<String>("target_dir"),
        };
        ::std::result::Result::Ok(v)
    }

    fn update_from_arg_matches(
        &mut self,
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        self.update_from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }

    fn update_from_arg_matches_mut(
        &mut self,
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        #![allow(deprecated)]
        if __clap_arg_matches.contains_id("module") {
            #[allow(non_snake_case)]
            let module = &mut self.module;
            *module = __clap_arg_matches.remove_one::<String>("module")
        }
        if __clap_arg_matches.contains_id("lib") {
            #[allow(non_snake_case)]
            let lib = &mut self.lib;
            *lib = __clap_arg_matches.remove_one::<String>("lib")
        }
        if __clap_arg_matches.contains_id("target_dir") {
            #[allow(non_snake_case)]
            let target_dir = &mut self.target_dir;
            *target_dir = __clap_arg_matches.remove_one::<String>("target_dir")
        }
        ::std::result::Result::Ok(())
    }
}
#[allow(dead_code, unreachable_code, unused_variables, unused_braces)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting
)]
#[deny(clippy::correctness)]
impl clap::Args for DocSettings {
    fn group_id() -> Option<clap::Id> {
        Some(clap::Id::from("DocSettings"))
    }

    fn augment_args<'b>(__clap_app: clap::Command) -> clap::Command {
        {
            let __clap_app =
                __clap_app.group(clap::ArgGroup::new("DocSettings").multiple(true).args({
                    let members: [clap::Id; 3usize] = [
                        clap::Id::from("module"),
                        clap::Id::from("lib"),
                        clap::Id::from("target_dir"),
                    ];
                    members
                }));
            let __clap_app = __clap_app.arg({
                #[allow(deprecated)]
                let arg = clap::Arg::new("module")
                    .value_name("MODULE")
                    .value_parser(clap::value_parser!(String))
                    .action(clap::ArgAction::Set);
                let arg = arg.short('m').long("module");
                let arg = arg;
                arg
            });
            let __clap_app = __clap_app.arg({
                #[allow(deprecated)]
                let arg = clap::Arg::new("lib")
                    .value_name("LIB")
                    .value_parser(clap::value_parser!(String))
                    .action(clap::ArgAction::Set);
                let arg = arg.short('l').long("lib");
                let arg = arg;
                arg
            });
            let __clap_app = __clap_app.arg({
                #[allow(deprecated)]
                let arg = clap::Arg::new("target_dir")
                    .value_name("TARGET_DIR")
                    .value_parser(clap::value_parser!(String))
                    .action(clap::ArgAction::Set);
                let arg = arg.short('t').long("target-dir");
                let arg = arg;
                arg
            });
            __clap_app
        }
    }

    fn augment_args_for_update<'b>(__clap_app: clap::Command) -> clap::Command {
        {
            let __clap_app =
                __clap_app.group(clap::ArgGroup::new("DocSettings").multiple(true).args({
                    let members: [clap::Id; 3usize] = [
                        clap::Id::from("module"),
                        clap::Id::from("lib"),
                        clap::Id::from("target_dir"),
                    ];
                    members
                }));
            let __clap_app = __clap_app.arg({
                #[allow(deprecated)]
                let arg = clap::Arg::new("module")
                    .value_name("MODULE")
                    .value_parser(clap::value_parser!(String))
                    .action(clap::ArgAction::Set);
                let arg = arg.short('m').long("module");
                let arg = arg.required(false);
                arg
            });
            let __clap_app = __clap_app.arg({
                #[allow(deprecated)]
                let arg = clap::Arg::new("lib")
                    .value_name("LIB")
                    .value_parser(clap::value_parser!(String))
                    .action(clap::ArgAction::Set);
                let arg = arg.short('l').long("lib");
                let arg = arg.required(false);
                arg
            });
            let __clap_app = __clap_app.arg({
                #[allow(deprecated)]
                let arg = clap::Arg::new("target_dir")
                    .value_name("TARGET_DIR")
                    .value_parser(clap::value_parser!(String))
                    .action(clap::ArgAction::Set);
                let arg = arg.short('t').long("target-dir");
                let arg = arg.required(false);
                arg
            });
            __clap_app
        }
    }
}
