use indoc::indoc;

// Returns `path` with the cargo test data directory prepended.
fn data_path(path: Option<&str>) -> std::path::PathBuf {
    let mut result = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    result.push("tests");
    result.push("checksum-test-data");
    if let Some(provided_path) = path {
        result.push(provided_path);
    }
    result
}

#[test]
fn checksum_some_data() {
    assert_eq!(
        libzakopane::checksum(data_path(None)),
        indoc!(
            r#"
            8ec39490ae7374067429174fd55867628145b9d20b4871a10aba36d24f3a5a33  ./random-data-00
            a9f70c6a2c2e3a3f7269e7d897f846454204a312ca62115fed676647b485bd54  ./random-data-01
            "#
        )
    );
}
