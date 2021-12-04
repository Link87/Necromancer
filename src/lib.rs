use std::error::Error;
use std::fs;

use log::debug;

pub mod parse;
pub mod recipe;
// pub mod summon;
pub mod value;

// use summon::Scheduler;

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
        Config { path, output: OutputMode::Run }
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
    let code = fs::read_to_string(config.path())?;

    match (config.output_mode(), parse::parse(&code)) {
        (OutputMode::SyntaxTree, Ok(syntax_tree)) => {
            print!("{:#?}", &syntax_tree);
            Ok(())
        }
        (OutputMode::Run, Ok(syntax_tree)) => {
            debug!("{:?}", &syntax_tree);
            // Scheduler::new(syntax_tree).schedule();
            Ok(())
        }
        (_, Err(error)) => Err(Box::new(nom::error::Error::new(
            String::from(error.input),
            error.code,
        ))),
    }
}
