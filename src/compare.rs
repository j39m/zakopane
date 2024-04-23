use crate::config;
use crate::snapshot;
use crate::violations;

// Compares the older snapshot against the newer snapshot, accruing
// violations where discrepancies are detected per policy.
fn check_modifications_and_deletions(
    config: &config::Config,
    older_snapshot: &snapshot::Snapshot,
    newer_snapshot: &snapshot::Snapshot,
    violations: &mut violations::Violations,
) {
    for (path, checksum) in older_snapshot.iter() {
        let (_rule_repr, policy) = config.match_policy(path);
        if policy == config::POLICY_IGNORE {
            continue;
        }

        match newer_snapshot.get(path) {
            Some(newer_checksum) => {
                if (policy & config::POLICY_NOMODIFY) != 0 && checksum != newer_checksum {
                    violations.modified(path);
                }
            }
            None => {
                if (policy & config::POLICY_NODELETE) != 0 {
                    violations.deleted(path);
                }
            }
        }
    }
}

// Compares the newer snapshot against the older snapshot, accruing
// violations where discrepancies are detected per policy.
fn check_additions(
    config: &config::Config,
    older_snapshot: &snapshot::Snapshot,
    newer_snapshot: &snapshot::Snapshot,
    violations: &mut violations::Violations,
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
                    violations.added(path);
                }
            }
        }
    }
}

pub fn compare(
    config: &config::Config,
    older_snapshot: &snapshot::Snapshot,
    newer_snapshot: &snapshot::Snapshot,
) -> violations::Violations {
    let mut violations = violations::Violations::new();
    check_modifications_and_deletions(&config, &older_snapshot, &newer_snapshot, &mut violations);
    check_additions(&config, &older_snapshot, &newer_snapshot, &mut violations);

    violations
}
