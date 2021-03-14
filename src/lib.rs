extern crate either;
extern crate log;
extern crate nom;
extern crate tokio;

use log::debug;

use std::error::Error;
use std::fs;

pub mod entity;
pub mod parse;
pub mod schedule;
pub mod statement;
pub mod task;
pub mod value;

use schedule::Scheduler;

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
            Scheduler::new().schedule(&syntax_tree);
            Ok(())
        }
        Err(error) => Err(Box::new(nom::error::Error::new(
            String::from(error.input),
            error.code,
        ))),
    }
}
