use crate::error::io_error::IoError;
use crate::error::other_error::OtherError;
use crate::error::semantic_error::SemanticError;
use crate::error::syntax_error::SyntaxError;
use crate::error::type_error::TypeError;

pub mod io_error;
pub mod syntax_error;
pub mod semantic_error;
pub mod type_error;
pub mod other_error;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Io(IoError),
    Syntax(SyntaxError),
    Semantic(SemanticError),
    Type(TypeError),
    Other(OtherError),
}

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
}