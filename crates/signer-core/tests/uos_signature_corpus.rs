/// Known-answer tests for `UosSignature::encode()` and `UosSignature::decode()`.
///
/// JSON files live in `tests/corpora/signature/`.
use signer_core::uos::signature::UosSignature;
use std::path::Path;

fn corpus_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpora/signature")
}

#[test]
fn signature_corpus_encode() {
    let dir = corpus_dir();
    let mut count = 0;

    for entry in std::fs::read_dir(&dir).expect("corpus/signature dir missing") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();

        let inp = &v["input"];
        let crypto_type = inp["crypto_type"].as_u64().unwrap() as u8;
        let sig_hex = inp["signature_hex"].as_str().unwrap();
        let signature =
            hex::decode(sig_hex).unwrap_or_else(|_| panic!("{path:?}: bad signature_hex"));

        let uos = UosSignature::new(crypto_type, signature)
            .unwrap_or_else(|e| panic!("{path:?}: construct failed: {e}"));
        let encoded = uos.encode();

        let expected_hex = v["expected_bytes_hex"].as_str().unwrap();
        let expected = hex::decode(expected_hex)
            .unwrap_or_else(|_| panic!("{path:?}: bad expected_bytes_hex"));

        assert_eq!(encoded, expected, "{path:?}: encode mismatch");
        count += 1;
    }

    assert!(count > 0, "no signature corpus files found in {dir:?}");
}

#[test]
fn signature_corpus_decode_roundtrip() {
    let dir = corpus_dir();

    for entry in std::fs::read_dir(&dir).expect("corpus/signature dir missing") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();

        let expected_hex = v["expected_bytes_hex"].as_str().unwrap();
        let expected_bytes = hex::decode(expected_hex).unwrap();

        let decoded = UosSignature::decode(&expected_bytes)
            .unwrap_or_else(|e| panic!("{path:?}: decode failed: {e}"));
        let re_encoded = decoded.encode();

        assert_eq!(expected_bytes, re_encoded, "{path:?}: decode→encode round-trip failed");
    }
}
