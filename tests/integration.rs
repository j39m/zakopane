// This integration test suite relies heavily on the string
// representation of the Violations struct, as that's the final
// user-visible output that the zakopane binary presents.

use indoc::indoc;

use libzakopane::config::Config;
use libzakopane::snapshot::snapshot_string_for_testing;
use libzakopane::snapshot::Snapshot;

#[test]
fn test_basic_default_immutability() {
    let options = libzakopane::config::test_support::options(None, Some("immutable"));
    let config: Config = Config::new(&options).unwrap();

    // Verifies that empty snapshots never turn up violations.
    let empty_older = Snapshot::new(&snapshot_string_for_testing("")).unwrap();
    let empty_newer = Snapshot::new(&snapshot_string_for_testing("")).unwrap();
    let empty_violations = libzakopane::enter(&config, &empty_older, &empty_newer);
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
    let disjoint_violations = libzakopane::enter(&config, &disjoint_older, &disjoint_newer);
    // From zakopane's point of view, ``./a/b/c'' was deleted and
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

    // Verifies that zakopane catches the two changed files interspersed
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
    let shifty_violations = libzakopane::enter(&config, &shifty_older, &shifty_newer);
    assert_eq!(
        shifty_violations.to_string(),
        indoc!(
            r#"
            ! ./a/b/changed
            ! ./i/j/changed
            "#
        )
    );

    // As an additional sanity check, verifies that zakopane snapshots
    // are not sensitive to the ordering of input files.
    let shifty_newer_shuffled = Snapshot::new(&snapshot_string_for_testing(indoc!(
        r#"
        0000000000000000000000000000000000000000000000000000000000000000  ./e/f/unchanged
        ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff  ./i/j/changed
        0000000000000000000000000000000000000000000000000000000000000000  ./k/l/unchanged
        ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff  ./a/b/changed
        0000000000000000000000000000000000000000000000000000000000000000  ./g/h/unchanged
        0000000000000000000000000000000000000000000000000000000000000000  ./c/d/unchanged
        "#
    )))
    .unwrap();
    let the_same_shifty_violations =
        libzakopane::enter(&config, &shifty_older, &shifty_newer_shuffled);
    assert_eq!(
        shifty_violations.to_string(),
        the_same_shifty_violations.to_string()
    );
}

#[test]
fn test_overlapping_prefixes() {
    let config_path =
        libzakopane::config::test_support::data_path("config-with-several-more-policies");
    let options =
        libzakopane::config::test_support::options(Some(config_path.to_str().unwrap()), None);
    let config = Config::new(&options).unwrap();

    let snapshot_older = Snapshot::new(&snapshot_string_for_testing(indoc!(
        r#"
        0000000000000000000000000000000000000000000000000000000000000000  ./Music/hello-there.mp3
        0000000000000000000000000000000000000000000000000000000000000000  ./Pictures/general-kenobi.gif
        0000000000000000000000000000000000000000000000000000000000000000  ./Pictures/2020/you-are.gif
        0000000000000000000000000000000000000000000000000000000000000000  ./Pictures/2020/a-bold-one.gif
        0000000000000000000000000000000000000000000000000000000000000000  ./Pictures/2019/something-immutable.jpg
        0000000000000000000000000000000000000000000000000000000000000000  ./Pictures/2019/something-supposedly-immutable.jpg
        0000000000000000000000000000000000000000000000000000000000000000  ./Documents/nodelete-1.txt
        0000000000000000000000000000000000000000000000000000000000000000  ./Documents/nodelete-2.txt
        "#
    )))
    .unwrap();
    let snapshot_newer = Snapshot::new(&snapshot_string_for_testing(indoc!(
        r#"
        ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff  ./Music/hello-there.mp3
        0000000000000000000000000000000000000000000000000000000000000000  ./Music/copy-of-hello-there.mp3
        ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff  ./Pictures/general-kenobi.gif
        0000000000000000000000000000000000000000000000000000000000000000  ./Pictures/copy-of-general-kenobi.gif
        0000000000000000000000000000000000000000000000000000000000000000  ./Pictures/2020/copy-of-you-are.gif
        ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff  ./Pictures/2020/a-bold-one.gif
        0000000000000000000000000000000000000000000000000000000000000000  ./Pictures/2019/something-immutable.jpg
        ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff  ./Pictures/2019/something-supposedly-immutable.jpg
        ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff  ./Documents/nodelete-2.txt
        "#
    )))
    .unwrap();

    let violations = libzakopane::enter(&config, &snapshot_older, &snapshot_newer);
    assert_eq!(
        violations.to_string(),
        indoc!(
            r#"
            - ./Documents/nodelete-1.txt
            + ./Music/copy-of-hello-there.mp3
            ! ./Music/hello-there.mp3
            ! ./Pictures/2019/something-supposedly-immutable.jpg
            + ./Pictures/copy-of-general-kenobi.gif
            ! ./Pictures/general-kenobi.gif
            "#
        )
    );
}
