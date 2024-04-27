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
}

impl std::fmt::Display for ZakopaneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZakopaneError::Io(io_error) => write!(f, "{}", io_error.to_string()),
            ZakopaneError::Config(message)
            | ZakopaneError::Snapshot(message)
            | ZakopaneError::CommandLine(message) => write!(f, "{}", message),
        }
    }
}

#[derive(Debug)]
pub struct ChecksumCliOptions {
    pub path: std::path::PathBuf,
    pub output_path: std::path::PathBuf,
    pub start_time: chrono::DateTime<chrono::offset::Local>,
    pub max_tasks: usize,

    // User-defined value for what constitutes a "big file" for which
    // the checksum dispatcher will force single-threaded digest
    // calculation.
    pub big_file_bytes: Option<u64>,
}

impl ChecksumCliOptions {
    pub fn new(
        path: std::path::PathBuf,
        optional_output_path: Option<std::path::PathBuf>,
        max_tasks: usize,
        big_file_bytes: Option<u64>,
    ) -> Result<Self, ZakopaneError> {
        if max_tasks < 1 {
            return Err(ZakopaneError::CommandLine(format!(
                "invalid task cap: ``{}''",
                max_tasks
            )));
        }

        let start_time = chrono::offset::Local::now();
        let output_path = match optional_output_path {
            Some(path) => path,
            None => std::path::PathBuf::from(start_time.format("%Y-%m-%d-%H%M.txt").to_string()),
        };

        Ok(Self {
            path,
            output_path,
            start_time,
            max_tasks,
            big_file_bytes,
        })
    }
}
