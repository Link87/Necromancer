use std::io::{self, Write};

use crate::value::Value;

#[derive(Debug)]
pub struct Statement {
    cmd: StatementCmd,
}

#[derive(Debug, PartialEq)]
pub enum StatementCmd {
    Say(Value),
}

impl Statement {
    pub fn new(cmd: StatementCmd) -> Statement {
        Statement { cmd }
    }

    pub fn cmd<'a>(&'a self) -> &'a StatementCmd {
        &self.cmd
    }

    pub fn execute(&mut self) {
        match &self.cmd {
            StatementCmd::Say(arg) => {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                handle.write_all((format!("{}", arg)).as_bytes());
            }
        }
    }
}
