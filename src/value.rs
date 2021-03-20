use std::fmt::{Display, Formatter, Result};
use std::ops::{Add, Div, Neg};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    String(String),
    Boolean(bool),
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

impl<'a> Add<&'a Value> for Value {
    type Output = Value;

    fn add(self, other: &Value) -> Value {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 + i2),
            (Value::String(s1), Value::String(s2)) => Value::String(s1 + s2),
            (Value::Void, v) => Value::from(v),
            (v, Value::Void) => Value::from(v),
            _ => unimplemented!(),
        }
    }
}

impl<'a, 'b> Div<&'b Value> for &'a Value {
    type Output = Value;

    fn div(self, other: &Value) -> Value {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 / i2),
            (Value::Void, v) => Value::from(v),
            (v, Value::Void) => Value::from(v),
            _ => unimplemented!(),
        }
    }
}

impl<'a> Neg for &'a Value {
    type Output = Value;

    fn neg(self) -> Value {
        match self {
            Value::Integer(i) => Value::Integer(-i),
            Value::Void => Value::Void,
            _ => unimplemented!(),
        }
    }
}

impl From<&Value> for Value {
    fn from(value: &Value) -> Self {
        match value {
            Value::Integer(i) => Value::Integer(*i),
            Value::String(s) => Value::String(String::from(s)),
            Value::Boolean(b) => Value::Boolean(*b),
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

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
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
            Value::Boolean(b) => write!(fmt, "{}", b),
            Value::Void => write!(fmt, "The Void."),
        }
    }
}
