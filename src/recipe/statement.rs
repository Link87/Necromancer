use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
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
