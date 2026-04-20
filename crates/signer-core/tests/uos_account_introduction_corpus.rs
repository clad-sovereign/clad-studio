/// Known-answer tests for `AccountIntroduction::to_uri()` and `from_uri()`.
///
/// JSON files live in `tests/corpora/account_introduction/`.
/// Each file records input struct fields and the expected URI string.
use signer_core::uos::account_introduction::AccountIntroduction;
use std::path::Path;

fn corpus_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpora/account_introduction")
}

#[test]
fn account_introduction_corpus_to_uri() {
    let dir = corpus_dir();
    let mut count = 0;

    for entry in std::fs::read_dir(&dir).expect("corpus/account_introduction dir missing") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();

        let inp = &v["input"];
        let address = inp["address"].as_str().unwrap().to_string();
        let genesis_hash = inp["genesis_hash"].as_str().map(str::to_string);
        let name = inp["name"].as_str().map(str::to_string);

        let ai = AccountIntroduction::new(address, genesis_hash, name);
        let uri = ai.to_uri();

        let expected_uri = v["expected_uri"].as_str().unwrap();
        assert_eq!(uri, expected_uri, "{path:?}: to_uri mismatch");

        count += 1;
    }

    assert!(count > 0, "no account_introduction corpus files found in {dir:?}");
}

#[test]
fn account_introduction_corpus_round_trip() {
    let dir = corpus_dir();

    for entry in std::fs::read_dir(&dir).expect("corpus/account_introduction dir missing") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();

        let expected_uri = v["expected_uri"].as_str().unwrap();

        // Parse the URI back into a struct.
        let parsed = AccountIntroduction::from_uri(expected_uri)
            .unwrap_or_else(|e| panic!("{path:?}: from_uri failed: {e}"));

        // Re-serialise and compare.
        let re_uri = parsed.to_uri();
        assert_eq!(re_uri, expected_uri, "{path:?}: from_uri → to_uri round-trip failed");
    }
}
