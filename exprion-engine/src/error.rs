use std::{error::Error, fmt};

#[derive(Debug)]
pub enum JitError {
    ArityMismatch { expected: usize, actual: usize },
    UnsupportedExpression(String),
    UnsupportedVariable(String),
    UnsupportedPlatform(String),
    Codegen(String),
}

impl fmt::Display for JitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JitError::ArityMismatch { expected, actual } => {
                write!(f, "expected {expected} arguments, got {actual}")
            }
            JitError::UnsupportedExpression(message)
            | JitError::UnsupportedVariable(message)
            | JitError::UnsupportedPlatform(message)
            | JitError::Codegen(message) => f.write_str(message),
        }
    }
}

impl Error for JitError {}
