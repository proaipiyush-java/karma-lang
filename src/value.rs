use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    /// The bootstrap interpreter uses shared immutable backing storage so a
    /// borrow can be represented without copying the string bytes. This is an
    /// interpreter implementation detail; native Karma is free to lower String
    /// to a pointer/length/capacity representation later.
    String(Rc<str>),
    Unit,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{v}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
            Value::Unit => write!(f, "()"),
        }
    }
}
