use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'a> {
    Moan,
    MoanNamed(&'a str),
    Remembering(Value),
    RememberingNamed(&'a str, Value),
    Rend,
    Turn,
    Value(Value),
}
