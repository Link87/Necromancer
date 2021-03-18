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

impl From<&Value> for Value {
    fn from(value: &Value) -> Self {
        match value {
            Value::Integer(i) => Value::Integer(*i),
            Value::String(s) => Value::String(String::from(s)),
            Value::Void => Value::Void,
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(String::from(value))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl Default for Value {
    fn default() -> Value {
        Value::Void
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
