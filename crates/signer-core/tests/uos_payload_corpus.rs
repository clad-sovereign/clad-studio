/// Known-answer tests for `UosPayload::encode()` and `UosPayload::decode()`.
///
/// Each JSON file in `tests/corpora/payload/` contains a fixed input and the
/// expected binary output as a hex string.  The Rust implementation must
/// reproduce that output exactly; any byte-level deviation is a compatibility
/// regression against the Kotlin reference implementation.
use signer_core::uos::payload::UosPayload;
use std::path::Path;

fn corpus_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpora/payload")
}

#[test]
fn payload_corpus_encode() {
    let dir = corpus_dir();
    let mut count = 0;

    for entry in std::fs::read_dir(&dir).expect("corpus/payload dir missing") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {path:?}: {e}"));
        let v: serde_json::Value =
            serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parsing {path:?}: {e}"));

        let inp = &v["input"];
        let crypto_type = inp["crypto_type"].as_u64().unwrap() as u8;
        let action = inp["action"].as_u64().unwrap() as u8;
        let account_id_hex = inp["account_id_hex"].as_str().unwrap();
        let inner_payload_hex = inp["inner_payload_hex"].as_str().unwrap();

        let account_id =
            hex::decode(account_id_hex).unwrap_or_else(|_| panic!("{path:?}: bad account_id_hex"));
        let payload_bytes = if inner_payload_hex.is_empty() {
            Vec::new()
        } else {
            hex::decode(inner_payload_hex)
                .unwrap_or_else(|_| panic!("{path:?}: bad inner_payload_hex"))
        };

        let uos = UosPayload::new(crypto_type, action, account_id, payload_bytes);
        let encoded = uos.encode().unwrap_or_else(|e| panic!("{path:?}: encode failed: {e}"));

        let expected_hex = v["expected_bytes_hex"].as_str().unwrap();
        let expected = hex::decode(expected_hex)
            .unwrap_or_else(|_| panic!("{path:?}: bad expected_bytes_hex"));

        assert_eq!(encoded, expected, "{path:?}: encode output mismatch");
        count += 1;
    }

    assert!(count > 0, "no payload corpus files found in {dir:?}");
}

#[test]
fn payload_corpus_decode_roundtrip() {
    let dir = corpus_dir();
    let mut count = 0;

    for entry in std::fs::read_dir(&dir).expect("corpus/payload dir missing") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();

        let expected_hex = v["expected_bytes_hex"].as_str().unwrap();
        let expected_bytes = hex::decode(expected_hex).unwrap();

        // decode(encode(payload)) must give back the original struct
        let decoded = UosPayload::decode(&expected_bytes)
            .unwrap_or_else(|e| panic!("{path:?}: decode failed: {e}"));
        let re_encoded =
            decoded.encode().unwrap_or_else(|e| panic!("{path:?}: re-encode failed: {e}"));

        assert_eq!(expected_bytes, re_encoded, "{path:?}: decode→encode round-trip failed");
        count += 1;
    }

    assert!(count > 0, "no payload corpus files found in {dir:?}");
}
