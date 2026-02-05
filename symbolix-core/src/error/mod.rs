use std::io;
use std::num::ParseIntError;
use thiserror::Error;

use crate::error::io_error::IoError;
use crate::error::other_error::OtherError;
use crate::error::semantic_error::SemanticError;
use crate::error::syntax_error::SyntaxError;
use crate::error::type_error::TypeError;

pub mod io_error;
pub mod other_error;
pub mod semantic_error;
pub mod syntax_error;
pub mod type_error;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Io(IoError),
    Syntax(SyntaxError),
    Semantic(SemanticError),
    Type(TypeError),
    Other(OtherError),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse int error: {0}")]
    Parse(#[from] ParseIntError),

    #[error("Semantic error: {0}")]
    Semantic(#[from] SemanticError),

    #[error("Other error: {0}")]
    Msg(String),
}

impl Error {
    pub fn semantic_error(msg: &str) -> Self {
        Error::Semantic(SemanticError::new(String::from(msg)))
    }
}
