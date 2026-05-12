use std::fmt;
use std::io;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Yaml(String),
    Json(serde_json::Error),
    Parse(String),

}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "I/O error: {e}"),
            AppError::Yaml(s) => write!(f, "YAML error: {s}"),
            AppError::Json(e) => write!(f, "JSON error: {e}"),
            AppError::Parse(s) => write!(f, "Parse error: {s}"),

        }
    }
}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e)
    }
}
