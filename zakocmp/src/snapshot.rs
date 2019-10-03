// This module implements the snapshot files used by zakopane.
// ATOW a snapshot file is pretty much the output of the ``sha256sum''
// command with three extra lines atop.

use std::collections::HashMap;
use std::result::Result;
use std::str::Lines;
use std::string::String;

use crate::errors::ZakocmpError;

// Defines the number of lines preceding the actual checksum content.
const HEADER_LINES: usize = 3;

// Defines the number of hex characters in a sha256sum.
const CHECKSUM_CHARS: usize = 64;

// Defines a valid zakopane snapshot header.
const SNAPSHOT_HEADER_FOR_TESTING: &'static str = r#"simple-zakopane.sh: 2019-07-27-090032
simple-zakopane.sh: /home/kalvin
# this line is typically empty but must be present
"#;

// Accepts a borrowed string representation of some zakopane
// checksums, prepends the standard zakopane snapshot header to the
// same, and returns the owned result.
pub fn snapshot_string_for_testing(checksums: &str) -> String {
    let mut snapshot: String = SNAPSHOT_HEADER_FOR_TESTING.to_string();
    snapshot.push_str(checksums);
    snapshot
}

// Defines a zakopane snapshot, which maps paths to checksums.
#[derive(Debug)]
pub struct Snapshot {
    contents: HashMap<String, String>,
}

// Borrows the string representation of a line in a zakopane snapshot
// and returns sliced str's in a tuple of (checksum, path).
fn parse_snapshot_line(line: &str) -> Result<(&str, &str), ZakocmpError> {
    let bad_line = ZakocmpError::Snapshot(format!("malformed snapshot line: ``{}''", line));
    // A snapshot line should consist of the checksum, two spaces, and a
    // non-empty pathname.
    if line.len() < CHECKSUM_CHARS + 3
        || !line.is_char_boundary(CHECKSUM_CHARS)
        || !line.is_char_boundary(CHECKSUM_CHARS + 1)
        || !line.is_char_boundary(CHECKSUM_CHARS + 2)
    {
        return Err(bad_line);
    }

    let (checksum, path_with_leading_space) = line.split_at(CHECKSUM_CHARS);
    if !path_with_leading_space.starts_with("  ") {
        return Err(bad_line);
    }
    Ok((checksum, &path_with_leading_space[2..]))
}

impl Snapshot {
    // Borrows the string representation of a zakopane snapshot and
    // returns the corresponding Snapshot struct.
    pub fn new(snapshot: &str) -> Result<Snapshot, ZakocmpError> {
        let mut lines: Lines = snapshot.lines();

        // A zakopane snapshot starts with three extra lines intended
        // for human readers. zakocmp doesn't care about this header.
        let mut header_drain: usize = HEADER_LINES;
        while header_drain > 0 {
            match lines.next() {
                Some(_) => (),
                None => {
                    return Err(ZakocmpError::Snapshot(
                        "truncated zakopane snapshot".to_string(),
                    ))
                }
            };
            header_drain -= 1;
        }

        // Ingests the rest of the snapshot representation.
        let mut contents: HashMap<String, String> = HashMap::new();
        for line in lines {
            let (checksum, path) = parse_snapshot_line(line)?;
            match contents.insert(path.to_string(), checksum.to_string()) {
                None => (),
                Some(old_checksum) => {
                    return Err(ZakocmpError::Snapshot(format!(
                        "path collision: {} (was already {}, is now {})",
                        path, old_checksum, checksum
                    )))
                }
            };
        }
        Ok(Snapshot { contents: contents })
    }

    // Passes the inner struct's iterator straight out.
    pub fn iter(&self) -> std::collections::hash_map::Iter<String, String> {
        self.contents.iter()
    }

    // Returns a reference to the checksum of the file (if present).
    pub fn get(&self, key: &str) -> std::option::Option<&String> {
        self.contents.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Consumes a ZakocmpError and borrows a string slice. Asserts that
    // the error is of the Snapshot variant and starts with the string
    // slice.
    fn assert_snapshot_error(error: ZakocmpError, prefix: &str) {
        match error {
            ZakocmpError::Snapshot(message) => assert!(message.starts_with(prefix)),
            _ => panic!("expected ZakocmpError::Snapshot"),
        };
    }

    #[test]
    fn snapshot_must_have_proper_header() {
        let snapshot = Snapshot::new(SNAPSHOT_HEADER_FOR_TESTING).unwrap();
        assert_eq!(snapshot.contents.len(), 0);

        let snapshot_without_proper_header = r#"simple-zakopane.sh: 2019-07-27-090032
simple-zakopane.sh: /home/kalvin
"#;
        assert_snapshot_error(
            Snapshot::new(snapshot_without_proper_header).unwrap_err(),
            "truncated",
        );
    }

    #[test]
    fn snapshot_checksum_is_hex() {
        let checksum_ok =
            "4e8401b759a877c0d215ba95bb75bd7d08318cbdc395b3fae9763337ee3614a5  ./hello/there.txt";
        let snapshot = Snapshot::new(&snapshot_string_for_testing(checksum_ok)).unwrap();
        assert_eq!(snapshot.contents.len(), 1);

        // Defines the literal string "malformed" that appears
        // repeatedly in the following tests.
        let malformed: &str = "malformed";

        // Oh no! This checksum dropped a character off the end.
        let checksum_short =
            "4e8401b759a877c0d215ba95bb75bd7d08318cbdc395b3fae9763337ee3614a  ./hello/there.txt";
        assert_snapshot_error(
            Snapshot::new(&snapshot_string_for_testing(checksum_short)).unwrap_err(),
            malformed,
        );

        // Oh no! This checksum line does not refer to a path.
        let checksum_without_path =
            "4e8401b759a877c0d215ba95bb75bd7d08318cbdc395b3fae9763337ee3614a5  ";
        assert_snapshot_error(
            Snapshot::new(&snapshot_string_for_testing(checksum_without_path)).unwrap_err(),
            malformed,
        );

        // Checksum lines may not be empty or too short.
        assert_snapshot_error(
            Snapshot::new(&snapshot_string_for_testing("\n")).unwrap_err(),
            malformed,
        );
        assert_snapshot_error(
            Snapshot::new(&snapshot_string_for_testing("Hello there!")).unwrap_err(),
            malformed,
        );
    }

    #[test]
    fn snapshot_paths_may_not_repeat() {
        let checksums =
            r#"4e8401b759a877c0d215ba95bb75bd7d08318cbdc395b3fae9763337ee3614a5  ./hello/there.txt
0000000000000000000000000000000000000000000000000000000000000000  ./hello/there.txt"#;
        // The point of this test is not to catch identical checksums
        // (which occur naturally if you ever accidentally duplicate
        // files), but to catch repeated paths (which should not be
        // possible in a well-formed snapshot file).
        assert_snapshot_error(
            Snapshot::new(&snapshot_string_for_testing(checksums)).unwrap_err(),
            "path collision",
        );
    }

    #[test]
    fn snapshot_get() {
        // Creates a snapshot describing two files, each with contrived
        // but valid-looking checksums.
        let snapshot = Snapshot::new(&snapshot_string_for_testing(
            r#"0000000000000000000000000000000000000000000000000000000000000001  ./hello/there.txt
0000000000000000000000000000000000000000000000000000000000000002  ./general/kenobi.txt
00000000000000000000000000000000000000000000000000000000000000ff  ./you/are.txt
00000000000000000000000000000000000000000000000000000000000001ff  ./a/bold-one.txt
"#,
        ))
        .unwrap();
        assert_eq!(
            snapshot.get("./hello/there.txt").unwrap(),
            "0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(
            snapshot.get("./general/kenobi.txt").unwrap(),
            "0000000000000000000000000000000000000000000000000000000000000002"
        );
        assert_eq!(
            snapshot.get("./you/are.txt").unwrap(),
            "00000000000000000000000000000000000000000000000000000000000000ff"
        );
        assert_eq!(
            snapshot.get("./a/bold-one.txt").unwrap(),
            "00000000000000000000000000000000000000000000000000000000000001ff"
        );

        assert!(snapshot.get("blah-blah-nonexistent-key").is_none());
        // Note that Snapshots don't perform any path comprehension - as
        // far as a Snapshot key is concerned, it's an arbitrary
        // sequence of bytes.
        assert!(snapshot.get("a/bold-one.txt").is_none());
    }
}
