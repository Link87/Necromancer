use std::fmt::{Display, Formatter, Result};
use std::iter::repeat_with;
use std::ops::{Add, Div, Neg};

use zalgo::{Generator, GeneratorArgs, ZalgoSize};

/// A value that an entity can remember.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    String(String),
    Boolean(bool),
    Evil(String),
    Void,
}

impl Value {
    /// Generate a corrupted value.
    fn corrupted() -> Value {
        let text: String = repeat_with(fastrand::alphanumeric)
            .take(fastrand::usize(7..=13))
            .collect();
        Value::Evil(Self::curse(&text))
    }

    /// Corrupt the value and make it evil.
    fn corrupt(self: &Value) -> Value {
        Value::Evil(Self::curse(&self.to_string()))
    }

    /// Curse the text with zalgo.
    #[inline]
    fn curse(text: &str) -> String {
        let mut generator = Generator::new();
        let mut out = String::new();
        let args = GeneratorArgs::new(true, true, true, ZalgoSize::Maxi);
        generator.gen(text, &mut out, &args);
        out
    }
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

    /// The `+` operator for the `Value` type.
    ///
    /// Performs type inference on a best-effort basis.
    /// Returns some™ value if addition cannot be performed or would overflow.
    fn add(self, other: &Value) -> Value {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                if let Some(sum) = i1.checked_add(*i2) {
                    Value::Integer(sum)
                } else {
                    Value::corrupted()
                }
            }
            (Value::String(s1), Value::String(s2)) => Value::String(s1 + s2),
            (Value::String(s), Value::Integer(i)) => Value::String(format!("{}{}", s, i)),
            (Value::String(s), Value::Boolean(b)) => Value::String(format!("{}{}", s, b)),
            (Value::Integer(i), Value::String(s)) => Value::String(format!("{}{}", i, s)),
            (Value::Boolean(b), Value::String(s)) => Value::String(format!("{}{}", b, s)),
            (Value::Evil(e), v) => Value::Evil(format!("{}{}", e, v.corrupt())),
            (v, Value::Evil(e)) => Value::Evil(format!("{}{}", e, v.corrupt())),
            (Value::Void, v) => Value::from(v),
            (v, Value::Void) => Value::from(v),
            _ => Value::corrupted(),
        }
    }
}

impl<'a, 'b> Div<&'b Value> for &'a Value {
    type Output = Value;

    /// The `/` operator for the `Value` type.
    ///
    /// Performs type inference on a best-effort basis.
    /// Returns some™ value if division cannot be performed or would overflow.
    fn div(self, other: &Value) -> Value {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                if let Some(div) = i1.checked_div(*i2) {
                    Value::Integer(div)
                } else {
                    Value::corrupted()
                }
            }
            (Value::Void, v) => Value::from(v),
            (v, Value::Void) => Value::from(v),
            _ => Value::corrupted(),
        }
    }
}

impl<'a> Neg for &'a Value {
    type Output = Value;

    /// The unary `-` operator for the `Value` type.
    ///
    /// Performs type inference on a best-effort basis.
    /// Returns some™ value if negation cannot be performed or would overflow.
    fn neg(self) -> Value {
        match self {
            Value::Integer(i) => {
                if let Some(neg) = i.checked_neg() {
                    Value::Integer(neg)
                } else {
                    Value::corrupted()
                }
            }
            Value::Void => Value::Void,
            _ => Value::corrupted(),
        }
    }
}

impl From<&Value> for Value {
    fn from(value: &Value) -> Self {
        match value {
            Value::Integer(i) => Value::Integer(*i),
            Value::String(s) => Value::String(String::from(s)),
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Evil(e) => Value::Evil(String::from(e)),
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
            Value::Evil(e) => write!(fmt, "{}", e),
            Value::Void => Ok(()),
        }
    }
}
