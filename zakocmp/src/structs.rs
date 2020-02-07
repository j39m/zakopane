// This module defines miscellaneous, non-specialized structs that can
// appear anywhere in the crate.

#[derive(Debug)]
pub enum ZakocmpError {
    // Propagates I/O errors (e.g. from reading actual files).
    Io(std::io::Error),
    // Describes problems with zakocmp configuration files.
    Config(String),
    // Describes problems with zakocmp snapshot files.
    Snapshot(String),
    // Describes invalid command-line invocations.
    CommandLine(String),
    // Describes unknown or unspecified errors.
    Unknown(String),
}

impl std::fmt::Display for ZakocmpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZakocmpError::Io(io_error) => write!(f, "{}", io_error.to_string()),
            ZakocmpError::Config(message)
            | ZakocmpError::Snapshot(message)
            | ZakocmpError::CommandLine(message)
            | ZakocmpError::Unknown(message) => write!(f, "{}", message),
        }
    }
}

#[derive(Debug)]
pub struct CliOptions<'a> {
    // Every zakocmp run requires two snapshots to compare.
    pub old_snapshot_path: &'a str,
    pub new_snapshot_path: &'a str,
    // A config file with policies is optional.
    pub config_path: Option<&'a str>,
    // A default policy on the command-line is optional.
    pub default_policy: Option<&'a str>,
}
