pub mod config;
pub mod helpers;
pub mod snapshot;
pub mod structs;
pub mod violations;

use config::Config;
use snapshot::Snapshot;
use violations::Violations;

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

// The main entry point of the zakocmp executable.
// Accepts a well-formed Config, older Snapshot, and newer Snapshot.
// Returns a Violation struct.
pub fn enter(config: &Config, older_snapshot: &Snapshot, newer_snapshot: &Snapshot) -> Violations {
    let mut violations = Violations::new();
    check_modifications_and_deletions(&config, &older_snapshot, &newer_snapshot, &mut violations);
    check_additions(&config, &older_snapshot, &newer_snapshot, &mut violations);

    violations
}
