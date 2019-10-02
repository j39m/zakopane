// This module defines a container that expresses zakocmp policy
// violations - i.e. human-readable descriptions of notable
// discrepancies between zakocmp snapshots.

use crate::errors::ZakocmpError;

// Represents possible policy violations as caller-passable ints.
pub const ADDED: i32 = 0;
pub const DELETED: i32 = 1 << 0;
pub const MODIFIED: i32 = 1 << 1;

// Represents human-readable counterparts to the above. These are meant
// for printing etc. and so are not relevant to callers.
const REPR_ADDED: &'static str = "+";
const REPR_DELETED: &'static str = "-";
const REPR_MODIFIED: &'static str = "!";

// A single violation consists of the offending path (arbitrary bytes)
// and the kind of violation (i32 - as above).
pub struct Violations {
    data: std::vec::Vec<(String, i32)>,
}

impl Violations {
    pub fn new() -> Violations {
        Violations { data: vec![] }
    }

    // Accepts a path (arbitrary string of bytes) and a kind of policy
    // violation.
    pub fn add(&mut self, path: &str, kind: i32) -> std::result::Result<(), ZakocmpError> {
        match kind {
            ADDED | DELETED | MODIFIED => (),
            _ => return Err(ZakocmpError::Unknown(format!("bad kind: {}", kind))),
        };
        self.data.push((path.to_owned(), kind));
        Ok(())
    }
}

fn display_violation_type(kind: i32) -> &'static str {
    match kind {
        ADDED => REPR_ADDED,
        DELETED => REPR_DELETED,
        MODIFIED => REPR_MODIFIED,
        // This case is serious: the burden is on us to have weeded out
        // invalid violation kinds in previous calls to add().
        _ => panic!(format!("BUG: bad kind: {}", kind)),
    }
}

impl std::fmt::Display for Violations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sorted_violations = self.data.to_owned();
        sorted_violations.sort_unstable();
        for (path, kind) in sorted_violations.into_iter() {
            write!(f, "{} {}\n", display_violation_type(kind), path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn violations_add_fail() {
        let mut violations = Violations::new();
        let error = violations.add("some/path", 999).unwrap_err();
        match error {
            ZakocmpError::Unknown(message) => assert!(message.starts_with("bad")),
            _ => panic!("not a ZakocmpError::Unknown"),
        };
    }

    #[test]
    fn violations_add_okay() {
        let mut violations = Violations::new();
        assert!(violations
            .add(
                "arbitrary bytes are allowed so long as the type is valid",
                ADDED
            )
            .is_ok());
        // Empty strings are allowed, too.
        assert!(violations.add("", MODIFIED).is_ok());
        // Doubling up on strings is perfectly legitimate, too.
        assert!(violations.add("", MODIFIED).is_ok());
        assert!(violations.add("", ADDED).is_ok());
        assert!(violations.add("", DELETED).is_ok());
    }

    #[test]
    fn violations_display() {
        let mut violations = Violations::new();
        assert!(violations.add("jello there!", ADDED).is_ok());
        assert!(violations.add("iello there!", MODIFIED).is_ok());
        assert!(violations.add("hello there!", DELETED).is_ok());
        assert!(violations.add("a/path/of/some/sort", ADDED).is_ok());
        assert!(violations.add("b/path/of/some/sort", MODIFIED).is_ok());
        assert!(violations.add("z/path/of/some/sort", DELETED).is_ok());

        assert_eq!(
            format!("{}", violations),
            r#"+ a/path/of/some/sort
! b/path/of/some/sort
- hello there!
! iello there!
+ jello there!
- z/path/of/some/sort
"#
        );
    }
}
