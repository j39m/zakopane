// This module implements the snapshot files used by zakopane.
// ATOW a snapshot file is pretty much the output of the ``sha256sum''
// command with three extra lines atop.

use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::result::Result;
use std::str::Lines;
use std::string::String;

// Defines the number of lines preceding the actual checksum content.
const HEADER_LINES: usize = 3;

// Defines the number of hex characters in a sha256sum.
const CHECKSUM_CHARS: usize = 64;

// Defines a zakopane snapshot, which maps paths to checksums.
pub struct Snapshot {
    contents: HashMap<String, String>,
}

// Borrows the string representation of a line in a zakopane snapshot
// and returns sliced str's in a tuple of (path, checksum).
fn parse_snapshot_line(line: &str) -> Result<(&str, &str), Error> {
    let bad_line: Error = Error::new(
        ErrorKind::InvalidData,
        format!("malformed snapshot line: ``{}''", line),
    );
    // A snapshot line should consist of the checksum, a space, and a
    // non-empty pathname.
    if line.len() < CHECKSUM_CHARS + 2
        || !line.is_char_boundary(CHECKSUM_CHARS)
        || !line.is_char_boundary(CHECKSUM_CHARS + 1)
    {
        return Err(bad_line);
    }

    let (checksum, path_with_leading_space) = line.split_at(CHECKSUM_CHARS);
    if !path_with_leading_space.starts_with(" ") {
        return Err(bad_line);
    }
    Ok((checksum, &path_with_leading_space[1..]))
}

impl Snapshot {
    // Borrows the string representation of a zakopane snapshot and
    // returns the corresponding Snapshot struct.
    pub fn new(snapshot: &str) -> Result<Snapshot, Error> {
        let mut lines: Lines = snapshot.lines();

        // A zakopane snapshot starts with three extra lines intended
        // for human readers. zakocmp doesn't care about this header.
        let mut header_drain: usize = HEADER_LINES;
        while header_drain > 0 {
            match lines.next() {
                Some(_) => (),
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "truncated zakopane snapshot",
                    ))
                }
            };
            header_drain -= 1;
        }

        // Ingests the rest of the snapshot representation.
        let mut contents: HashMap<String, String> = HashMap::new();
        for line in lines {
            let (path, checksum) = parse_snapshot_line(line)?;
            match contents.insert(path.to_string(), checksum.to_string()) {
                None => (),
                Some(old_checksum) => {
                    return Err(Error::new(
                        ErrorKind::AlreadyExists,
                        format!(
                            "path collision: {} (was already {}, is now {})",
                            path, old_checksum, checksum
                        ),
                    ))
                }
            };
        }
        Ok(Snapshot { contents: contents })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defines a valid zakopane snapshot header.
    const SNAPSHOT_HEADER_FOR_TESTING: &'static str = r#"simple-zakopane.sh: 2019-07-27-090032
simple-zakopane.sh: /home/kalvin
# this line is typically empty but must be present
"#;

    // Accepts a borrowed string representation of some zakopane
    // checksums, prepends the standard zakopane snapshot header to the
    // same, and returns the owned result.
    fn snapshot_string_for_testing(checksums: &str) -> String {
        let mut snapshot: String = SNAPSHOT_HEADER_FOR_TESTING.to_string();
        snapshot.push_str(checksums);
        snapshot
    }

    #[test]
    fn snapshot_must_have_proper_header() {
        let snapshot = Snapshot::new(SNAPSHOT_HEADER_FOR_TESTING).unwrap();
        assert!(snapshot.contents.len() == 0);

        let snapshot_without_proper_header = r#"simple-zakopane.sh: 2019-07-27-090032
simple-zakopane.sh: /home/kalvin
"#;
        assert!(!Snapshot::new(snapshot_without_proper_header).is_ok());
    }

    #[test]
    fn snapshot_checksum_is_hex() {
        let checksum_ok =
            "4e8401b759a877c0d215ba95bb75bd7d08318cbdc395b3fae9763337ee3614a5 ./hello/there.txt";
        let snapshot = Snapshot::new(&snapshot_string_for_testing(checksum_ok)).unwrap();
        assert!(snapshot.contents.len() == 1);

        // Oh no! This checksum dropped a character off the end.
        let checksum_short =
            "4e8401b759a877c0d215ba95bb75bd7d08318cbdc395b3fae9763337ee3614a ./hello/there.txt";
        assert!(!Snapshot::new(&snapshot_string_for_testing(checksum_short)).is_ok());

        // Oh no! This checksum line does not refer to a path.
        let checksum_without_path =
            "4e8401b759a877c0d215ba95bb75bd7d08318cbdc395b3fae9763337ee3614a5 ";
        assert!(!Snapshot::new(&snapshot_string_for_testing(checksum_without_path)).is_ok());

        // Checksum lines may not be empty or too short.
        assert!(!Snapshot::new(&snapshot_string_for_testing("\n")).is_ok());
        assert!(!Snapshot::new(&snapshot_string_for_testing("Hello there!")).is_ok());
    }

    #[test]
    fn snapshot_paths_may_not_repeat() {
        let checksums =
            "4e8401b759a877c0d215ba95bb75bd7d08318cbdc395b3fae9763337ee3614a5 ./hello/there.txt
        4e8401b759a877c0d215ba95bb75bd7d08318cbdc395b3fae9763337ee3614a5 ./hello/there.txt";
        assert!(!Snapshot::new(&snapshot_string_for_testing(checksums)).is_ok());
    }
}
