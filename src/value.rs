use std::fmt::{Display, Formatter, Result};
use std::iter::repeat_with;
use std::ops::{Add, Div, Neg};

use malachite::num::arithmetic::traits::CheckedDiv;
use malachite::Integer;
use zalgo::{Generator, GeneratorArgs, ZalgoSize};

/// A value that an entity can remember.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Value {
    Integer(Integer),
    String(String),
    Boolean(bool),
    Infernal(String),
    #[default]
    Void,
}

impl Value {
    /// Generate a corrupted value.
    fn corrupted() -> Value {
        let text: String = repeat_with(fastrand::alphanumeric)
            .take(fastrand::usize(7..=13))
            .collect();
        Value::Infernal(text)
    }

    /// Curse the text with zalgo.
    #[inline]
    fn curse(text: &str) -> String {
        let mut generator = Generator::new();
        let mut o̶̪̙͕̱͌̽͒́ủ̷͔̩͓t̷̩͋̓̾ = String::new();
        let args = GeneratorArgs::new(true, true, true, ZalgoSize::Maxi);
        generator.gen(text, &mut o̶̪̙͕̱͌̽͒́ủ̷͔̩͓t̷̩͋̓̾, &args);
        o̶̪̙͕̱͌̽͒́ủ̷͔̩͓t̷̩͋̓̾
    }
}

impl PartialEq<&Value> for Value {
    fn eq(&self, other: &&Value) -> bool {
        match (self, other) {
            (Value::Infernal(_), _) => false,
            (_, Value::Infernal(_)) => false,
            _ => self == *other,
        }
    }
}

impl PartialEq<Value> for &Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Infernal(_), _) => false,
            (_, Value::Infernal(_)) => false,
            _ => *self == other,
        }
    }
}

impl<'a> Add<&'a Value> for Value {
    type Output = Value;

    /// The `+` operator for the `Value` type.
    ///
    /// Performs type inference on a best-effort basis.
    /// Returns an infernal value if the resulting type is incomprehensible to humans.
    fn add(self, other: &Value) -> Value {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 + i2),
            (Value::String(s1), Value::String(s2)) => Value::String(s1 + s2),
            (Value::String(s), Value::Integer(i)) => Value::String(format!("{}{}", s, i)),
            (Value::String(s), Value::Boolean(b)) => Value::String(format!("{}{}", s, b)),
            (Value::Integer(i), Value::String(s)) => Value::String(format!("{}{}", i, s)),
            (Value::Boolean(b), Value::String(s)) => Value::String(format!("{}{}", b, s)),
            (Value::Infernal(e), v) => Value::Infernal(format!("{}{}", e, v)),
            (v, Value::Infernal(e)) => Value::Infernal(format!("{}{}", e, v)),
            (Value::Void, v) => Value::from(v),
            (v, Value::Void) => v,
            _ => Value::corrupted(),
        }
    }
}

impl<'a, 'b> Div<&'b Value> for &'a Value {
    type Output = Value;

    /// The `/` operator for the `Value` type.
    ///
    /// Performs type inference on a best-effort basis.
    /// Returns some™ value if division cannot be performed.
    fn div(self, other: &Value) -> Value {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => {
                if let Some(div) = i1.checked_div(i2) {
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
    /// Returns some™ value if negation cannot be performed.
    fn neg(self) -> Value {
        match self {
            Value::Integer(i) => Value::Integer(-i),
            Value::Void => Value::Void,
            _ => Value::corrupted(),
        }
    }
}

impl From<&Value> for Value {
    fn from(value: &Value) -> Self {
        match value {
            Value::Integer(i) => Value::Integer(i.clone()),
            Value::String(s) => Value::String(String::from(s)),
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Infernal(e) => Value::Infernal(String::from(e)),
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

impl From<Integer> for Value {
    fn from(value: Integer) -> Self {
        Value::Integer(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl Display for Value {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            Value::Integer(i) => write!(fmt, "{}", i),
            Value::String(s) => write!(fmt, "{}", s),
            Value::Boolean(b) => write!(fmt, "{}", b),
            Value::Infernal(i̸̭̩̫͇͇̤͛̀̔̋̇) => write!(fmt, "{}", Value::curse(i̸̭̩̫͇͇̤͛̀̔̋̇)),
            Value::Void => Ok(()),
        }
    }
}
