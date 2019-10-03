use libzakocmp::config::Config;
use libzakocmp::errors::ZakocmpError;
use libzakocmp::snapshot::Snapshot;

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

fn main() {
    let (config, older_snapshot, newer_snapshot) = match initialize() {
        Ok((c, o, n)) => (c, o, n),
        Err(error) => {
            eprintln!("{}", error.to_string());
            std::process::exit(1);
        }
    };
    assert!(config.rules() > 0);
    let violations = libzakocmp::enter(&config, &older_snapshot, &newer_snapshot);
    println!("{}", violations);
}
