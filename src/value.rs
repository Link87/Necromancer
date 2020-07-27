use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    String(String),
    Void,
}

impl PartialEq<&Value> for Value {
    fn eq(&self, other: &&Value) -> bool {
        self == *other
    }
}

impl PartialEq<Value> for &Value {
    fn eq(&self, other: &Value) -> bool {
        *self == other
    }
}

impl Display for Value {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            Value::Integer(i) => write!(fmt, "{}", i),
            Value::String(s) => write!(fmt, "{}", s),
            Value::Void => write!(fmt, "The Void."),
        }
    }
}
