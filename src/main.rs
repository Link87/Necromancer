use std::process;

use clap::App;
use clap::Arg;
use clap::ArgGroup;
use clap::ValueHint;
use env_logger::Builder;
use log::LevelFilter;
use log::{error, info};
use necromancer::{Config, OutputMode};

pub enum LogLevel {
    Error,
    Verbose,
    Debug,
}

fn main() {
    // let args: Vec<String> = env::args().collect();

    let matches = App::new("Necromancer")
        // .version(clap::crate_version!())
        // .author(clap::crate_authors!())
        // .about(clap::crate_description!())
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
                .help("Stop after parsing the scroll and print syntax tree."),
        )
        .group(ArgGroup::new("mode").args(&["syntax_tree_mode"]))
        .arg(
            Arg::new("v")
                .short('v')
                .multiple_occurrences(true)
                .max_occurrences(2)
                .help("Hear the screams more clearly."),
        )
        .get_matches();

    let mut builder = Builder::from_default_env();
    match matches.occurrences_of("v") {
        0 => builder.filter_level(LevelFilter::Error),
        1 => builder.filter_level(LevelFilter::Info),
        2 => builder.filter_level(LevelFilter::Debug),
        _ => panic!("Invalid log level!"),
    };
    builder.init();

    let mut config = Config::new(matches.value_of("path").unwrap().to_string());
    if matches.is_present("syntax_tree_mode") {
        config.set_output_mode(OutputMode::SyntaxTree)
    }

    info!("Executing file {}", config.path());

    if let Err(e) = necromancer::run(config) {
        error!("Application error: {}", e);
        process::exit(1);
    }
}
