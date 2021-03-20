use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    cmd: StatementCmd,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementCmd {
    Animate,
    AnimateNamed(String),
    Banish,
    BanishNamed(String),
    Disturb,
    DisturbNamed(String),
    Forget,
    ForgetNamed(String),
    Invoke,
    InvokeNamed(String),
    Remember(Value),
    RememberNamed(String, Value),
    Say(Value),
    SayNamed(String, Value),
}

impl Statement {
    pub fn new(cmd: StatementCmd) -> Statement {
        Statement { cmd }
    }

    pub fn cmd<'a>(&'a self) -> &'a StatementCmd {
        &self.cmd
    }
}
