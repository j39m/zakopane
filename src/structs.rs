// This module defines miscellaneous, non-specialized structs that can
// appear anywhere in the crate.

#[derive(Debug)]
pub enum ZakopaneError {
    // Propagates I/O errors (e.g. from reading actual files).
    Io(std::io::Error),
    // Describes problems with zakopane configuration files.
    Config(String),
    // Describes problems with zakopane snapshot files.
    Snapshot(String),
    // Describes invalid command-line invocations.
    CommandLine(String),
    // Describes unknown or unspecified errors.
    Unknown(String),
}

impl std::fmt::Display for ZakopaneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZakopaneError::Io(io_error) => write!(f, "{}", io_error.to_string()),
            ZakopaneError::Config(message)
            | ZakopaneError::Snapshot(message)
            | ZakopaneError::CommandLine(message)
            | ZakopaneError::Unknown(message) => write!(f, "{}", message),
        }
    }
}

#[derive(Debug)]
// Snapshot files are ingested early on and not stored here.
pub struct CompareCliOptions<'a> {
    // A config file with policies is optional.
    pub config_path: Option<&'a str>,
    // A default policy on the command-line is optional.
    pub default_policy: Option<&'a str>,
}
