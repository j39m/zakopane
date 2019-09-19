// This module defines a unified error type used across zakocmp.

use std::string::String;

#[derive(Debug)]
pub enum ZakocmpError {
    // Propagates I/O errors (e.g. from reading actual files).
    Io(std::io::Error),
    // Describes problems with zakocmp configuration files.
    Config(String),
}
