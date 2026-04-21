//! Known-answer tests for SCALE-encoded call data.
//!
//! Reads `tests/corpora/extrinsic/call_data.json` and verifies that the Rust
//! builder produces byte-identical output to the expected vectors.
//!
//! The corpus was computed from audited pallet/call constants in `call.rs` and
//! the SCALE wire-format rules documented in `call.rs` module doc.  Pending
//! Kotlin-oracle cross-check (Phase 2b).

use signer_core::extrinsic::{call, metadata};

fn corpus_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpora/extrinsic/call_data.json")
}

/// Load and parse the call_data corpus, then verify each vector.
#[test]
fn call_data_corpus() {
    let raw = std::fs::read_to_string(corpus_path()).expect("call_data.json missing");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("parse call_data.json");

    let vectors = v["vectors"].as_array().expect("vectors array missing");
    assert!(!vectors.is_empty(), "corpus must have at least one vector");

    for vec in vectors {
        let call_name = vec["call"].as_str().unwrap();
        let args_v = &vec["args"];
        let expected_hex = vec["expected_bytes_hex"].as_str().unwrap();

        let account_hex = args_v["account_hex"].as_str().unwrap();
        let account = hex::decode(account_hex).expect("bad account_hex");
        assert_eq!(account.len(), 32, "AccountId must be 32 bytes");

        let args: Vec<Vec<u8>> = if let Some(amount) = args_v["amount"].as_u64() {
            // Pack amount as raw LE u128 (16 bytes) as expected by metadata::build_call_data.
            let amount_bytes = (amount as u128).to_le_bytes().to_vec();
            vec![account, amount_bytes]
        } else {
            vec![account]
        };

        let got = metadata::build_call_data("CladToken", call_name, &args)
            .unwrap_or_else(|e| panic!("build_call_data({call_name}) failed: {e:?}"));
        assert_eq!(hex::encode(&got), expected_hex, "call_data mismatch for call '{call_name}'");
    }
}

// ── Direct builder tests (complement to corpus) ───────────────────────────────

#[test]
fn mint_builder_matches_corpus_vector() {
    let alice =
        hex::decode("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").unwrap();
    let got = call::mint(&alice, 1);
    assert_eq!(
        hex::encode(&got),
        "080000d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d04"
    );
}

#[test]
fn freeze_builder_matches_corpus_vector() {
    let alice =
        hex::decode("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").unwrap();
    let got = call::freeze(&alice);
    assert_eq!(
        hex::encode(&got),
        "080200d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
    );
}

#[test]
fn sort_signatories_is_lexicographic() {
    let mut sigs: Vec<Vec<u8>> = vec![vec![0xFF; 32], vec![0x00; 32], vec![0x80; 32]];
    call::sort_signatories(&mut sigs);
    assert_eq!(sigs[0], vec![0x00u8; 32]);
    assert_eq!(sigs[2], vec![0xFFu8; 32]);
}
