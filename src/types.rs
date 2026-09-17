use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParamMode {
    Owned,
    Borrowed,
}

impl fmt::Display for ParamMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamMode::Owned => write!(f, "owned"),
            ParamMode::Borrowed => write!(f, "borrow"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Bool,
    String,
    Unit,
}

impl Type {
    /// Copy values may be duplicated implicitly because their representation is
    /// small and has no unique resource ownership.
    pub const fn is_copy(&self) -> bool {
        matches!(self, Type::Int | Type::Bool | Type::Unit)
    }

    /// Owned values participate in Karma's move rules.
    /// String is the first owned value in v0.3; files, sockets, buffers and
    /// user-defined resource types will follow the same model later.
    pub const fn is_owned(&self) -> bool {
        !self.is_copy()
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::String => write!(f, "String"),
            Type::Unit => write!(f, "Unit"),
        }
    }
}
