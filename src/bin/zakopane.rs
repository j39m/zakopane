use std::io::Write;

use libzakopane::config::Config;
use libzakopane::snapshot::Snapshot;
use libzakopane::structs::ChecksumCliOptions;
use libzakopane::structs::CompareCliOptions;
use libzakopane::structs::ZakopaneError;

use clap::{value_t, App, Arg, ArgMatches};

const DEFAULT_POLICY_ARG_NAME: &'static str = "default-policy";
const CONFIG_FILE_ARG_NAME: &'static str = "config";
const OLD_SNAPSHOT_PATH_ARG_NAME: &'static str = "old-snapshot-path";
const NEW_SNAPSHOT_PATH_ARG_NAME: &'static str = "new-snapshot-path";

// Holds one instance of each struct necessary to execute the `compare`
// subcommand.
struct CompareData {
    config: Config,
    old_snapshot: Snapshot,
    new_snapshot: Snapshot,
}

enum SubcommandData {
    Compare(CompareData),
    Checksum(ChecksumCliOptions),
}

// Reads parsed command-line arguments and returns the appropriate
// operational data. Can abort the program on error.
fn compare_data_from(matches: &ArgMatches) -> Result<SubcommandData, ZakopaneError> {
    // The two snapshot paths are required, so these are safe to unwrap.
    let old_snapshot_path = matches.value_of(OLD_SNAPSHOT_PATH_ARG_NAME).unwrap();
    let new_snapshot_path = matches.value_of(NEW_SNAPSHOT_PATH_ARG_NAME).unwrap();
    let old_contents = libzakopane::helpers::ingest_file(old_snapshot_path)?;
    let new_contents = libzakopane::helpers::ingest_file(new_snapshot_path)?;

    let options = CompareCliOptions {
        config_path: matches.value_of(CONFIG_FILE_ARG_NAME),
        default_policy: matches.value_of(DEFAULT_POLICY_ARG_NAME),
    };

    Ok(SubcommandData::Compare(CompareData {
        config: Config::new(&options)?,
        old_snapshot: Snapshot::new(&old_contents)?,
        new_snapshot: Snapshot::new(&new_contents)?,
    }))
}

// Begins parsing command-line arguments. Can abort the program on
// error.
fn initialize() -> Result<SubcommandData, ZakopaneError> {
    let matches = App::new("zakopane")
        .version("0.3.1")
        .author("j39m")
        .about("checksums directories")
        .subcommand(
            App::new("compare")
                .about("compares two zakopane snapshots")
                .arg(
                    Arg::with_name(CONFIG_FILE_ARG_NAME)
                        .short("c")
                        .long("config")
                        .value_name("FILE")
                        .help("specifies a zakopane config")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name(DEFAULT_POLICY_ARG_NAME)
                        .short("d")
                        .long("default-policy")
                        .value_name("POLICY_TOKENS")
                        .help("specifies an explicit default policy")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name(OLD_SNAPSHOT_PATH_ARG_NAME)
                        .help("path to older snapshot")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::with_name(NEW_SNAPSHOT_PATH_ARG_NAME)
                        .help("path to newer snapshot")
                        .index(2)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("checksum")
                .about("produces checksums for a directory")
                .arg(
                    Arg::with_name("target-path")
                        .help("directory to checksum")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::with_name("max-tasks")
                        .short("j")
                        .takes_value(true)
                        .help("maximum number of simultaneous checksum tasks")
                        .default_value("8"),
                )
                .arg(
                    Arg::with_name("big-file-bytes")
                        .long("single-threaded-checksum-byte-threshold")
                        .takes_value(true)
                        .help(
                            "file size in bytes for which checksumming should be single-threaded",
                        ),
                ),
        )
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("compare") {
        return compare_data_from(&matches);
    }
    if let Some(ref matches) = matches.subcommand_matches("checksum") {
        let big_file_bytes = if matches.is_present("big-file-bytes") {
            Some(clap::value_t!(matches, "big-file-bytes", u64).unwrap_or_else(|e| e.exit()))
        } else {
            None
        };

        let options = ChecksumCliOptions::new(
            std::path::PathBuf::from(matches.value_of("target-path").unwrap()),
            clap::value_t!(matches, "max-tasks", usize).unwrap_or_else(|e| e.exit()),
            big_file_bytes,
        )?;
        return Ok(SubcommandData::Checksum(options));
    }
    panic!("BUG: unhandled subcommand");
}

fn do_compare(data: CompareData) {
    let CompareData {
        config,
        new_snapshot,
        old_snapshot,
    } = data;
    assert!(config.rules() > 0);
    let violations = libzakopane::compare(&config, &old_snapshot, &new_snapshot);
    println!("{}", violations);
}

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

fn do_checksum(options: ChecksumCliOptions) {
    if !options.path.is_dir() {
        eprintln!("``{}'' is not a dir", options.path.display());
        return;
    }
    println!("checksum ``{}'' at {}", options.path.display(), options.start_time);
    let output_basename = format!("{}.txt", options.start_time.format("%Y-%m-%d-%H%M"));
    let mut output_file = std::fs::File::create(&output_basename).unwrap();

    let header = generate_snapshot_header(&options.path, &options.start_time);
    let start_time = options.start_time;
    let checksums = libzakopane::checksum(options);

    output_file.write_all(header.as_ref()).unwrap();
    output_file.write_all(checksums.as_ref()).unwrap();
    println!("wrote ``{}''", output_basename);

    let end_time: chrono::DateTime<chrono::offset::Local> = chrono::offset::Local::now();
    println!(
        "finished at {} ({}s elapsed)",
        end_time,
        (end_time - start_time).num_seconds()
    );
}

fn main() {
    let subcommand = match initialize() {
        Ok(data) => data,
        Err(error) => {
            eprintln!("{}", error.to_string());
            std::process::exit(1);
        }
    };
    match subcommand {
        SubcommandData::Compare(compare_data) => return do_compare(compare_data),
        SubcommandData::Checksum(options) => return do_checksum(options),
    }
}
