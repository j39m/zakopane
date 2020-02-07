use libzakocmp::config::Config;
use libzakocmp::snapshot::Snapshot;
use libzakocmp::structs::CliOptions;
use libzakocmp::structs::ZakocmpError;

const USAGE_STRING: &'static str = "usage: zakocmp <config> <snapshot_older> <snapshot_newer>";

// Holds one instance of each struct necessary to operate.
struct OperationalData {
    config: Config,
    old_snapshot: Snapshot,
    new_snapshot: Snapshot,
}

// Collects all arguments, reads all file contents, and coerces them
// into the appropriate types.
fn initialize() -> Result<OperationalData, ZakocmpError> {
    let args: std::vec::Vec<std::string::String> = std::env::args().collect();
    if args.len() != 4 {
        return Err(ZakocmpError::Unknown(USAGE_STRING.to_string()));
    }

    let options = CliOptions {
        old_snapshot_path: &args[2],
        new_snapshot_path: &args[3],
        config_path: Some(&args[1]),
        default_policy: None,
    };

    let old_contents = libzakocmp::helpers::ingest_file(&args[2])?;
    let new_contents = libzakocmp::helpers::ingest_file(&args[3])?;

    Ok(OperationalData {
        config: Config::new(&options)?,
        old_snapshot: Snapshot::new(&old_contents)?,
        new_snapshot: Snapshot::new(&new_contents)?,
    })
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
    let violations = libzakocmp::enter(&config, &old_snapshot, &new_snapshot);
    println!("{}", violations);
}
