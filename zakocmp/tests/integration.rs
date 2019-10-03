// This integration test suite relies heavily on the string
// representation of the Violations struct, as that's the final
// user-visible output that the zakocmp binary presents.
extern crate indoc;

use indoc::indoc;
use libzakocmp::config::Config;
use libzakocmp::snapshot::snapshot_string_for_testing;
use libzakocmp::snapshot::Snapshot;

#[test]
fn test_basic_default_immutability() {
    let config: Config = Config::new(r#"default-policy: immutable"#).unwrap();

    // Verifies that empty snapshots never turn up violations.
    let empty_older = Snapshot::new(&snapshot_string_for_testing("")).unwrap();
    let empty_newer = Snapshot::new(&snapshot_string_for_testing("")).unwrap();
    let empty_violations = libzakocmp::enter(&config, &empty_older, &empty_newer);
    assert_eq!(empty_violations.to_string(), "");

    // Verifies that disjoint snapshots also violate this policy.
    let disjoint_older = Snapshot::new(&snapshot_string_for_testing(
        "0000000000000000000000000000000000000000000000000000000000000000  ./a/b/c",
    ))
    .unwrap();
    let disjoint_newer = Snapshot::new(&snapshot_string_for_testing(
        "0000000000000000000000000000000000000000000000000000000000000000  ./x/y/z",
    ))
    .unwrap();
    let disjoint_violations = libzakocmp::enter(&config, &disjoint_older, &disjoint_newer);
    // From zakocmp's point of view, ``./a/b/c'' was deleted and
    // ``./x/y/z'' was added.
    assert_eq!(
        disjoint_violations.to_string(),
        indoc!(
            r#"
            - ./a/b/c
            + ./x/y/z
            "#
        )
    );

    // Verifies that zakocmp catches the two changed files interspersed
    // with others that did not.
    let shifty_older = Snapshot::new(&snapshot_string_for_testing(indoc!(
        r#"
        0000000000000000000000000000000000000000000000000000000000000000  ./a/b/changed
        0000000000000000000000000000000000000000000000000000000000000000  ./c/d/unchanged
        0000000000000000000000000000000000000000000000000000000000000000  ./e/f/unchanged
        0000000000000000000000000000000000000000000000000000000000000000  ./g/h/unchanged
        0000000000000000000000000000000000000000000000000000000000000000  ./i/j/changed
        0000000000000000000000000000000000000000000000000000000000000000  ./k/l/unchanged
        "#
    )))
    .unwrap();
    let shifty_newer = Snapshot::new(&snapshot_string_for_testing(indoc!(
        r#"
        ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff  ./a/b/changed
        0000000000000000000000000000000000000000000000000000000000000000  ./c/d/unchanged
        0000000000000000000000000000000000000000000000000000000000000000  ./e/f/unchanged
        0000000000000000000000000000000000000000000000000000000000000000  ./g/h/unchanged
        ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff  ./i/j/changed
        0000000000000000000000000000000000000000000000000000000000000000  ./k/l/unchanged
        "#
    )))
    .unwrap();
    let shifty_violations = libzakocmp::enter(&config, &shifty_older, &shifty_newer);
    assert_eq!(
        shifty_violations.to_string(),
        indoc!(
            r#"
            ! ./a/b/changed
            ! ./i/j/changed
            "#
        )
    );
}
