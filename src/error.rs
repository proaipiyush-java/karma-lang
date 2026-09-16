use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KarmaError {
    pub phase: &'static str,
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl KarmaError {
    pub fn new(
        phase: &'static str,
        message: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            phase,
            message: message.into(),
            line,
            column,
        }
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::new("runtime", message, 0, 0)
    }
}

impl fmt::Display for KarmaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{} error: {}", self.phase, self.message)
        } else {
            write!(
                f,
                "{} error at {}:{}: {}",
                self.phase, self.line, self.column, self.message
            )
        }
    }
}

impl std::error::Error for KarmaError {}
