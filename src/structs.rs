#[derive(thiserror::Error, Debug)]
pub enum ZakopaneError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Snapshot(String),
    #[error("{0}")]
    CommandLine(String),
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
