use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_ERROR_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone)]
pub struct ErrorExt {
    error_id: u32,
    error_kind: ErrorKind,
    error_message: String,
    is_fatal: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    Io,
    Lexical,
    Parse,
    Semantic,
    Syntax,
    Type,
    Other,
    Unknown,
}

impl ErrorExt {
    pub fn new(error_kind: ErrorKind, error_message: String, is_fatal: bool) -> Self {
        let error_id = NEXT_ERROR_ID.fetch_add(1, Ordering::SeqCst);
        ErrorExt {
            error_id,
            error_kind,
            error_message,
            is_fatal,
        }
    }

    pub fn error_id(&self) -> u32 {
        self.error_id
    }

    pub fn error_kind(&self) -> ErrorKind {
        self.error_kind
    }

    pub fn error_message(&self) -> &str {
        &self.error_message
    }

    pub fn is_fatal(&self) -> bool {
        self.is_fatal
    }

    pub fn semantic_error(error_message: &str, is_fatal: bool) -> Self {
        ErrorExt::new(ErrorKind::Semantic, String::from(error_message), is_fatal)
    }

    pub fn lexical_error(error_message: &str, is_fatal: bool) -> Self {
        ErrorExt::new(ErrorKind::Lexical, String::from(error_message), is_fatal)
    }
}
