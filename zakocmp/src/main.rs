mod config;
mod errors;
mod snapshot;
mod violations;

use config::Config;
use errors::ZakocmpError;
use snapshot::Snapshot;
use violations::Violations;

use std::io::Read;
use std::result::Result;
use std::string::String;

const USAGE_STRING: &'static str = "usage: zakocmp <config> <snapshot_older> <snapshot_newer>";

// Ingests the contents of a file.
fn slurp_contents(path: &str) -> Result<String, ZakocmpError> {
    let mut file = std::fs::File::open(std::path::Path::new(path)).map_err(ZakocmpError::Io)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(ZakocmpError::Io)?;
    Ok(contents)
}

// Collects all arguments, reads all file contents, and coerces them
// into the appropriate types. Returns tuple of the Config, the older
// Snapshot, and the newer Snapshot (i.e. in the order given on the
// command line).
fn initialize() -> Result<(Config, Snapshot, Snapshot), ZakocmpError> {
    let args: std::vec::Vec<std::string::String> = std::env::args().collect();
    if args.len() != 4 {
        return Err(ZakocmpError::Unknown(USAGE_STRING.to_string()));
    }

    let config_contents = slurp_contents(&args[1])?;
    let older_contents = slurp_contents(&args[2])?;
    let newer_contents = slurp_contents(&args[3])?;

    let config = Config::new(&config_contents)?;
    let older_snapshot = Snapshot::new(&older_contents)?;
    let newer_snapshot = Snapshot::new(&newer_contents)?;

    Ok((config, older_snapshot, newer_snapshot))
}

// Compares the older snapshot against the newer snapshot, accruing
// violations where discrepancies are detected per policy.
fn check_modifications_and_deletions(
    config: &Config,
    older_snapshot: &Snapshot,
    newer_snapshot: &Snapshot,
    violations: &mut Violations,
) {
    for (path, checksum) in older_snapshot.iter() {
        let (_rule_repr, policy) = config.match_policy(path);
        if policy == config::POLICY_IGNORE {
            continue;
        }

        match newer_snapshot.get(path) {
            Some(newer_checksum) => {
                if (policy & config::POLICY_NOMODIFY) != 0 && checksum != newer_checksum {
                    violations.add(path, violations::MODIFIED).unwrap();
                }
            }
            None => {
                if (policy & config::POLICY_NODELETE) != 0 {
                    violations.add(path, violations::DELETED).unwrap();
                }
            }
        }
    }
}

// Compares the newer snapshot against the older snapshot, accruing
// violations where discrepancies are detected per policy.
fn check_additions(
    config: &Config,
    older_snapshot: &Snapshot,
    newer_snapshot: &Snapshot,
    violations: &mut Violations,
) {
    for (path, _checksum) in newer_snapshot.iter() {
        let (_rule_repr, policy) = config.match_policy(path);
        if policy == config::POLICY_IGNORE {
            continue;
        }

        match older_snapshot.get(path) {
            Some(_older_checksum) => (),
            None => {
                if (policy & config::POLICY_NOADD) != 0 {
                    violations.add(path, violations::ADDED).unwrap();
                }
            }
        }
    }
}

fn main() {
    let (config, older_snapshot, newer_snapshot) = match initialize() {
        Ok((c, o, n)) => (c, o, n),
        Err(error) => {
            eprintln!("{}", error.to_string());
            std::process::exit(1);
        }
    };
    assert!(config.rules() > 0);

    let mut violations = Violations::new();
    check_modifications_and_deletions(&config, &older_snapshot, &newer_snapshot, &mut violations);
    check_additions(&config, &older_snapshot, &newer_snapshot, &mut violations);

    println!("{}", violations);
}
