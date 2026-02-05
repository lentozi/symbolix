use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("semantic error: {msg}")]
pub struct SemanticError {
    pub msg: String,
}

impl SemanticError {
    pub fn new(msg: String) -> Self {
        SemanticError { msg }
    }
}
