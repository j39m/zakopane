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

// TODO(j39m): Figure out if this best goes here or in `main.rs`.
//use clap::Parser;

#[derive(clap::Parser)]
#[command(name = clap::crate_name!(), version = clap::crate_version!(), about = "take checksums")]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    Checksum(ChecksumArgs),
    Compare(CompareArgs),
}

#[derive(clap::Args)]
pub struct ChecksumArgs {
    #[arg(help = "target directory")]
    pub target: std::path::PathBuf,
    #[arg(short, help = "simultaneous checksum tasks cap", default_value_t = 8)]
    pub jmax: u32,
    #[arg(short, help = "output path")]
    pub output_path: std::path::PathBuf,
    #[arg(
        long,
        help = "byte threshold for which single-threaded checksumming is forced"
    )]
    pub big_file_bytes: Option<usize>,
}

#[derive(clap::Args)]
pub struct CompareArgs {
    #[arg()]
    pub old_snapshot: std::path::PathBuf,
    #[arg()]
    pub new_snapshot: std::path::PathBuf,
    #[arg(short)]
    pub config: Option<std::path::PathBuf>,
}
