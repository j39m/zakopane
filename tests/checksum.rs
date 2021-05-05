use indoc::indoc;

// Returns `path` with the cargo test data directory prepended.
fn data_path(path: &str) -> std::path::PathBuf {
    let mut result = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    result.push("tests");
    result.push("checksum-test-data");
    result.push(path);
    result
}

#[test]
fn basic_checksum() {
    assert_eq!(
        libzakopane::checksum(data_path("basic-test")),
        indoc!(
            r#"
            8ec39490ae7374067429174fd55867628145b9d20b4871a10aba36d24f3a5a33  ./random-data-00
            a9f70c6a2c2e3a3f7269e7d897f846454204a312ca62115fed676647b485bd54  ./random-data-01
            "#
        )
    );
}

// Verifies
// *    that `zakopane checksum` does not descend into hidden
//      directories but that
// *    it makes the sole exception for the case when the target path is
//      itself a hidden directory.
#[test]
fn checksum_hidden_target_directory() {
    assert_eq!(
        libzakopane::checksum(data_path("hidden-target-test/.hidden/")),
        indoc!(
            r#"
            4bd4b6cff60b4bd1f618ed8fa6bf20c86bc7bc297498b9d43612713cf756bbd8  ./random-data-00
            35190dff137f1cc1a08df389ab6d0ba091f20a12098e062780ce0b5ccd796129  ./random-data-01
            "#
        )
    );
}

// Verifies that `zakopane checksum` does not follow symlinks when
// producing snapshot data.
#[test]
fn checksum_skipping_symlinks() {
    assert_eq!(
        libzakopane::checksum(data_path("symlink-test")),
        indoc!(
            r#"
            e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  ./empty-file
            "#
        )
    );
}
