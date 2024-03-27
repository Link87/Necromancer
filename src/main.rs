use std::process;

use clap::command;
use clap::value_parser;
use clap::Arg;
use clap::ArgAction;
use clap::ArgGroup;
use clap::ValueHint;
use env_logger::Builder;
use log::LevelFilter;
use log::{error, info};
use necromancer::{Config, OutputMode};


fn main() {
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

    let mut builder = Builder::from_default_env();
    match matches.get_count("verbose") {
        0 => builder.filter_level(LevelFilter::Error),
        1 => builder.filter_level(LevelFilter::Info),
        2 => builder.filter_level(LevelFilter::Debug),
        _ => unreachable!("Invalid log level!"),
    };
    builder.init();

    let mut config = Config::new(matches.get_one::<String>("path").unwrap());
    if matches.get_flag("syntax_tree_mode") {
        config.set_output_mode(OutputMode::SyntaxTree)
    }

    info!("Executing file {}", config.path());

    if let Err(e) = necromancer::run(config) {
        error!("Application error: {}", e);
        process::exit(1);
    }
}
