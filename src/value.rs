#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    String(String),
    None,
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
