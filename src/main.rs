use std::process;

use clap::{command, value_parser, Arg, ArgAction, ArgGroup, ValueHint};
use env_logger::Builder;
use log::{error, info, LevelFilter};

fn main() {
    // Parse command line arguments.
    let matches = command!()
        .arg(
            Arg::new("path")
                .value_name("PATH")
                .help("Where to find the Zombie Scroll.")
                .index(1)
                .value_hint(ValueHint::FilePath)
                .required(true),
        )
        .arg(
            Arg::new("syntax_tree_mode")
                .short('t')
                .long("tree")
                .action(ArgAction::SetTrue)
                .help("Stop after parsing the scroll and print the AST."),
        )
        .group(ArgGroup::new("mode").args(&["syntax_tree_mode"]))
        .arg(
            Arg::new("verbose")
                .short('v')
                .action(ArgAction::Count)
                .value_parser(value_parser!(u8).range(..=2))
                .help("Hear the screams from the underworld more clearly."),
        )
        .get_matches();

    // Initialize the logger. The log level depends on the number of -v flags in the CLI arguments.
    let mut builder = Builder::from_default_env();
    match matches.get_count("verbose") {
        0 => builder.filter_level(LevelFilter::Error),
        1 => builder.filter_level(LevelFilter::Info),
        2 => builder.filter_level(LevelFilter::Debug),
        _ => unreachable!("Invalid log level!"),
    };
    builder.init();

    let path = matches.get_one::<String>("path").unwrap();

    // If the -t flag is set, print the AST and exit.
    // Otherwise, perfom the necromancy ritual.
    if matches.get_flag("syntax_tree_mode") {
        info!("Printing AST for file {}", path);
        match necromancer::parse(path) {
            Ok(scroll) => {
                print!("{:#?}", scroll);
            }
            Err(e) => {
                error!("{}", e);
                process::exit(1);
            }
        }
    } else {
        info!("Executing file {}", path);
        if let Err(err) = necromancer::summon(path) {
            error!("{}", err);
            process::exit(1);
        }
    }
}
