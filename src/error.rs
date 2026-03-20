//! Error types for the Louie framework.
//!
//! Provides a unified [`Error`] enum covering all failure modes in the
//! framework: I/O errors, protocol errors, action dispatch errors, and
//! layout constraint violations.

use std::fmt;

/// The unified error type for all Louie operations.
#[derive(Debug)]
pub enum Error {
    /// An I/O error from the terminal backend or file system.
    Io(std::io::Error),
    /// A JSON serialization or deserialization error.
    Json(serde_json::Error),
    /// An agent protocol error (malformed request, unknown type, etc.).
    Protocol(String),
    /// An action dispatch error (unknown action, validation failure, etc.).
    Action(String),
    /// A widget was not found by its agent ID.
    WidgetNotFound(String),
    /// A layout constraint error.
    Layout(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Json(e) => write!(f, "JSON error: {e}"),
            Error::Protocol(msg) => write!(f, "Protocol error: {msg}"),
            Error::Action(msg) => write!(f, "Action error: {msg}"),
            Error::WidgetNotFound(id) => write!(f, "Widget not found: {id}"),
            Error::Layout(msg) => write!(f, "Layout error: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

/// A specialized [`Result`] type for Louie operations.
pub type Result<T> = std::result::Result<T, Error>;
