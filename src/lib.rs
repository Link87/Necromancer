extern crate nom;

use std::error::Error;
use std::fs;

pub mod entity;
pub mod parse;
pub mod task;

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

    let result = parse::parse(&code);
    println!("{:?}", result);

    Ok(())
}
