//! Checks fixture availability and JSON shape, NOT tokenizer conformance.

use std::{fs, path::Path};

#[test]
fn pinned_upstream_fixture_inventory_is_readable() {
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/html5lib-tests/tokenizer");
    let mut paths: Vec<_> = fs::read_dir(&directory)
        .expect("missing fixtures: run git submodule update --init --recursive")
        .map(|entry| entry.expect("read fixture directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "test")
        })
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no upstream tokenizer fixtures found");

    let mut cases = 0;
    for path in &paths {
        let bytes = fs::read(path).expect("read upstream fixture");
        let document: serde_json::Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        // xmlViolation fixtures have a separate root and different semantics.
        let tests = document
            .get("tests")
            .or_else(|| document.get("xmlViolationTests"))
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("{}: missing test array", path.display()));
        assert!(!tests.is_empty(), "{}: empty fixture", path.display());
        for test in tests {
            assert!(
                test["description"].is_string(),
                "{}: missing description",
                path.display()
            );
            assert!(
                test["input"].is_string(),
                "{}: missing input",
                path.display()
            );
            assert!(
                test["output"].is_array(),
                "{}: missing expected output",
                path.display()
            );
        }
        cases += tests.len();
    }

    eprintln!(
        "Inventory only: {} files, {cases} fixture cases; 0 conformance cases executed.",
        paths.len()
    );
}
