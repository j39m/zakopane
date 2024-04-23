// This module defines a container that expresses zakopane policy
// violations - i.e. human-readable descriptions of notable
// discrepancies between zakopane snapshots.

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
enum ViolationType {
    Added,
    Deleted,
    Modified,
}

impl std::fmt::Display for ViolationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &ViolationType::Added => write!(f, "+")?,
            &ViolationType::Deleted => write!(f, "-")?,
            &ViolationType::Modified => write!(f, "!")?,
        };
        Ok(())
    }
}

pub struct Violations {
    data: std::vec::Vec<(String, ViolationType)>,
}

impl Violations {
    pub fn new() -> Violations {
        Violations { data: vec![] }
    }

    pub fn added(&mut self, path: &str) {
        self.data.push((path.to_owned(), ViolationType::Added));
    }
    pub fn deleted(&mut self, path: &str) {
        self.data.push((path.to_owned(), ViolationType::Deleted));
    }
    pub fn modified(&mut self, path: &str) {
        self.data.push((path.to_owned(), ViolationType::Modified));
    }
}

impl std::fmt::Display for Violations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sorted_violations = self.data.to_owned();
        sorted_violations.sort_unstable();
        for (path, kind) in sorted_violations.into_iter() {
            write!(f, "{kind} {path}\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn violations_display() {
        let mut violations = Violations::new();
        violations.added("jello there!");
        violations.modified("iello there!");
        violations.deleted("hello there!");
        violations.added("a/path/of/some/sort");
        violations.modified("b/path/of/some/sort");
        violations.deleted("z/path/of/some/sort");

        assert_eq!(
            format!("{}", violations),
            indoc!(
                r#"
               + a/path/of/some/sort
               ! b/path/of/some/sort
               - hello there!
               ! iello there!
               + jello there!
               - z/path/of/some/sort
            "#
            )
        );
    }
}
