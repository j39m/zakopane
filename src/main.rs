use std::io::Write;

use libzakopane::config::Config;
use libzakopane::snapshot::Snapshot;
use libzakopane::structs::ZakopaneError;

fn generate_snapshot_header(
    path: &std::path::PathBuf,
    start_time: &chrono::DateTime<chrono::offset::Local>,
) -> String {
    let buffer: Vec<String> = vec![
        format!("zakopane: {}", start_time),
        format!("zakopane: {}", path.display()),
        String::new(),
        String::new(),
    ];

    buffer.join("\n")
}

fn do_checksum(args: libzakopane::structs::ChecksumArgs) {
    if !args.target.is_dir() {
        eprintln!("``{}'' is not a dir", args.target.display());
        return;
    }
    let start_time = chrono::offset::Local::now();
    println!("checksum ``{}'' at {start_time}", args.target.display(),);
    let mut output = std::fs::File::create(&args.output_path).unwrap();

    let header = generate_snapshot_header(&args.target, &start_time);
    let output_path = args.output_path.clone();
    let checksums = libzakopane::checksum(args);

    output.write_all(header.as_ref()).unwrap();
    output.write_all(checksums.as_ref()).unwrap();
    println!("wrote ``{}''", output_path.display());

    let end_time = chrono::offset::Local::now();
    println!(
        "finished at {end_time} ({}s elapsed)",
        (end_time - start_time).num_seconds()
    );
}

fn do_compare(args: libzakopane::structs::CompareArgs) {
    let config = Config::new(args.config).unwrap();
    let old_snapshot = Snapshot::new(
        libzakopane::helpers::ingest_file(args.old_snapshot)
            .unwrap()
            .as_str(),
    )
    .unwrap();
    let new_snapshot = Snapshot::new(
        libzakopane::helpers::ingest_file(args.new_snapshot)
            .unwrap()
            .as_str(),
    )
    .unwrap();
    let violations = libzakopane::compare(&config, &old_snapshot, &new_snapshot);
    println!("{}", violations);
}

fn main() {
    use clap::Parser;
    use libzakopane::structs::{Cli, Subcommand};
    let cli = Cli::parse();
    match cli.subcommand {
        Subcommand::Checksum(args) => do_checksum(args),
        Subcommand::Compare(args) => do_compare(args),
    }
}
