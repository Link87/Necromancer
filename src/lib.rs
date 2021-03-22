use log::debug;

use std::error::Error;
use std::fs;

pub mod recipe;
pub mod parse;
pub mod summoner;
pub mod value;

use summoner::Scheduler;

pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("Please provide a file name.");
        }

        let filename = args[1].clone();

        Ok(Config { filename })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let code = fs::read_to_string(config.filename)?;

    match parse::parse(&code) {
        Ok(syntax_tree) => {
            debug!("{:?}", &syntax_tree);
            Scheduler::new(syntax_tree).schedule();
            Ok(())
        }
        Err(error) => Err(Box::new(nom::error::Error::new(
            String::from(error.input),
            error.code,
        ))),
    }
}
