use libzakopane::config::Config;
use libzakopane::snapshot::Snapshot;
use libzakopane::structs::CompareCliOptions;
use libzakopane::structs::ZakopaneError;

use clap::{App, Arg, ArgMatches};

const DEFAULT_POLICY_ARG_NAME: &'static str = "default-policy";
const CONFIG_FILE_ARG_NAME: &'static str = "config";
const OLD_SNAPSHOT_PATH_ARG_NAME: &'static str = "old-snapshot-path";
const NEW_SNAPSHOT_PATH_ARG_NAME: &'static str = "new-snapshot-path";

// Holds one instance of each struct necessary to operate.
struct OperationalData {
    config: Config,
    old_snapshot: Snapshot,
    new_snapshot: Snapshot,
}

// Reads parsed command-line arguments and returns the appropriate
// operational data. Can abort the program on error.
fn complete_initialization(matches: &ArgMatches) -> Result<OperationalData, ZakopaneError> {
    // The two snapshot paths are required, so these are safe to unwrap.
    let old_snapshot_path = matches.value_of(OLD_SNAPSHOT_PATH_ARG_NAME).unwrap();
    let new_snapshot_path = matches.value_of(NEW_SNAPSHOT_PATH_ARG_NAME).unwrap();
    let old_contents = libzakopane::helpers::ingest_file(old_snapshot_path)?;
    let new_contents = libzakopane::helpers::ingest_file(new_snapshot_path)?;

    let options = CompareCliOptions {
        config_path: matches.value_of(CONFIG_FILE_ARG_NAME),
        default_policy: matches.value_of(DEFAULT_POLICY_ARG_NAME),
    };

    Ok(OperationalData {
        config: Config::new(&options)?,
        old_snapshot: Snapshot::new(&old_contents)?,
        new_snapshot: Snapshot::new(&new_contents)?,
    })
}

// Begins parsing command-line arguments. Can abort the program on
// error.
fn initialize() -> Result<OperationalData, ZakopaneError> {
    let matches = App::new("zakopane")
        .version("0.2.0")
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
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("compare") {
        return complete_initialization(&matches);
    }
    Err(ZakopaneError::Unknown("not implemented".to_string()))
}

fn main() {
    let operational_data = match initialize() {
        Ok(data) => data,
        Err(error) => {
            eprintln!("{}", error.to_string());
            std::process::exit(1);
        }
    };

    let OperationalData {
        config,
        new_snapshot,
        old_snapshot,
    } = operational_data;
    assert!(config.rules() > 0);
    let violations = libzakopane::compare(&config, &old_snapshot, &new_snapshot);
    println!("{}", violations);
}
