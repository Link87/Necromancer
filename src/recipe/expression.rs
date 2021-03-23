use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Moan,
    MoanNamed(String),
    Remembering,
    RememberingNamed(String),
    Rend,
    Turn,
    Value(Value),
}
