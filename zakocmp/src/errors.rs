// This module defines a unified error type used across zakocmp.

use std::string::String;

#[derive(Debug)]
pub enum ZakocmpError {
    // Propagates I/O errors (e.g. from reading actual files).
    Io(std::io::Error),
    // Describes problems with zakocmp configuration files.
    Config(String),
    // Describes problems with zakocmp snapshot files.
    Snapshot(String),
    // Describes unknown or unspecified errors.
    Unknown(String),
}

impl std::fmt::Display for ZakocmpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZakocmpError::Io(io_error) => write!(f, "{}", io_error.to_string()),
            ZakocmpError::Config(message)
            | ZakocmpError::Snapshot(message)
            | ZakocmpError::Unknown(message) => write!(f, "{}", message),
        }
    }
}
