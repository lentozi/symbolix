use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("failed to read file '{path}': {msg}")]
pub struct IoError {
    pub path: PathBuf,
    pub msg: String,
}
