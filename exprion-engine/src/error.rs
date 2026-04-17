use std::{error::Error, fmt};

#[derive(Debug)]
pub enum JitError {
    ArityMismatch { expected: usize, actual: usize },
    MissingArgument(String),
    UnknownArgument(String),
    DuplicateArgument(String),
    UnsupportedLogicalVariable(String),
    UnsupportedPowerExponent(String),
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
            JitError::MissingArgument(name) => write!(f, "missing argument `{name}`"),
            JitError::UnknownArgument(name) => write!(f, "unknown argument `{name}`"),
            JitError::DuplicateArgument(name) => write!(f, "duplicate argument `{name}`"),
            JitError::UnsupportedLogicalVariable(name) => {
                write!(f, "logical JIT does not support boolean variable `{name}`")
            }
            JitError::UnsupportedPowerExponent(message) => f.write_str(message),
            JitError::UnsupportedExpression(message)
            | JitError::UnsupportedVariable(message)
            | JitError::UnsupportedPlatform(message)
            | JitError::Codegen(message) => f.write_str(message),
        }
    }
}

impl Error for JitError {}
