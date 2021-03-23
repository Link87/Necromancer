use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Moan,
    MoanNamed(String),
    Remembering(Value),
    RememberingNamed(String, Value),
    Rend,
    Turn,
    Value(Value),
}
