// #![warn(missing_docs)]
#![doc = include_str!("../README.md")]
use std::error::Error;
use std::fs;

use log::debug;

pub mod parse;
pub mod scroll;
pub mod necro;
pub mod value;

use necro::Necromancer;

pub struct Config {
    path: String,
    output: OutputMode,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
    Run,
    SyntaxTree,
}

impl Config {
    pub fn new(path: String) -> Config {
        Config {
            path,
            output: OutputMode::Run,
        }
    }

    pub fn path(&self) -> &str {
        &self.path[..]
    }

    pub fn output_mode(&self) -> OutputMode {
        self.output
    }

    pub fn set_output_mode(&mut self, mode: OutputMode) {
        self.output = mode;
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let code = Box::new(fs::read_to_string(config.path())?);
    let code: &'static str = Box::leak(code);

    match (config.output_mode(), parse::parse(&code)) {
        (OutputMode::SyntaxTree, Ok(scroll)) => {
            print!("{:#?}", &scroll);
            Ok(())
        }
        (OutputMode::Run, Ok(scroll)) => {
            debug!("{:?}", &scroll);
            Necromancer::unroll(scroll).initiate();
            Ok(())
        }
        (_, Err(error)) => Err(Box::new(nom::error::Error::new(
            String::from(error.input),
            error.code,
        ))),
    }
}
