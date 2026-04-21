//! Known-answer tests for the crypto module.
//!
//! SS58 and Blake2b vectors come from `tests/corpora/crypto/`. ED25519 is
//! tested as a sign→verify roundtrip (RFC 8032 deterministic; no OS RNG
//! needed). SR25519 sign+verify roundtrip lives in the library unit tests
//! (`src/crypto/sr25519.rs`) using `sign_deterministic`, keeping the
//! randomized `sign` path out of integration tests per the agreed design.
//! Byte-stable KAT vectors for both algorithms are in `crypto_kotlin_oracle.rs`
//! (ignored, pending Phase 2b Kotlin oracle extraction).

use signer_core::{blake2, ed25519, ss58};
use std::path::Path;

fn corpora_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpora/crypto")
}

// ── SS58 encode corpus ────────────────────────────────────────────────────────

#[test]
fn ss58_encode_corpus() {
    let path = corpora_dir().join("ss58_encode.json");
    let raw = std::fs::read_to_string(&path).expect("ss58_encode.json missing");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("parse ss58_encode.json");

    let pk_hex = v["input"]["public_key_hex"].as_str().unwrap();
    let prefix = v["input"]["prefix"].as_u64().unwrap() as u16;
    let expected_address = v["expected_address"].as_str().unwrap();

    let pk = hex::decode(pk_hex).expect("bad public_key_hex");
    let addr = ss58::encode(&pk, prefix).expect("ss58::encode failed");
    assert_eq!(addr, expected_address, "SS58 encode mismatch");
}

// ── SS58 decode corpus ────────────────────────────────────────────────────────

#[test]
fn ss58_decode_corpus() {
    let path = corpora_dir().join("ss58_decode.json");
    let raw = std::fs::read_to_string(&path).expect("ss58_decode.json missing");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("parse ss58_decode.json");

    let address = v["input"]["address"].as_str().unwrap();
    let expected_pk_hex = v["expected_public_key_hex"].as_str().unwrap();
    let expected_prefix = v["expected_prefix"].as_u64().unwrap() as u16;

    let (pk, prefix) = ss58::decode(address).expect("ss58::decode failed");
    let got_hex = hex::encode(&pk);
    assert_eq!(got_hex, expected_pk_hex, "SS58 decode pubkey mismatch");
    assert_eq!(prefix, expected_prefix, "SS58 decode prefix mismatch");
}

// ── SS58 roundtrip property test ──────────────────────────────────────────────

#[test]
fn ss58_encode_decode_roundtrip() {
    // Arbitrary 32-byte key; any prefix in 0..=16383.
    let key: [u8; 32] = *b"test key for ss58 roundtrip!!!!\0";
    for prefix in [0u16, 42, 1337] {
        let addr = ss58::encode(&key, prefix).expect("encode");
        let (decoded_key, decoded_prefix) = ss58::decode(&addr).expect("decode");
        assert_eq!(&decoded_key, &key, "key roundtrip failed for prefix {prefix}");
        assert_eq!(decoded_prefix, prefix, "prefix roundtrip failed for prefix {prefix}");
    }
}

// ── Blake2b-256 corpus ────────────────────────────────────────────────────────

#[test]
fn blake2b_256_corpus() {
    let path = corpora_dir().join("blake2b_256.json");
    let raw = std::fs::read_to_string(&path).expect("blake2b_256.json missing");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("parse blake2b_256.json");

    for vector in v["vectors"].as_array().expect("vectors array missing") {
        let input_hex = vector["input_hex"].as_str().unwrap();
        let expected_hex = vector["expected_hash_hex"].as_str().unwrap();

        let input = if input_hex.is_empty() { vec![] } else { hex::decode(input_hex).unwrap() };
        let got = blake2::blake2b_256(&input);
        assert_eq!(
            hex::encode(&got),
            expected_hex,
            "blake2b_256 mismatch for vector: {}",
            vector["description"].as_str().unwrap_or("?")
        );
    }
}

// ── ED25519 sign + verify roundtrip ──────────────────────────────────────────
// ED25519 (RFC 8032) is deterministic — no OS RNG needed.
// SR25519 roundtrip lives in library unit tests (src/crypto/sr25519.rs)
// using sign_deterministic, keeping the randomized sign path out of here.

#[test]
fn ed25519_sign_verify_roundtrip() {
    let seed = [0x37u8; 32];
    let message = b"clad-sovereign phase-2 ed25519 roundtrip";

    let sig = ed25519::sign(message, &seed).expect("ed25519::sign failed");
    assert_eq!(sig.len(), 64, "signature must be 64 bytes");

    let pubkey = ed25519::public_key_from_seed(&seed).expect("public_key_from_seed failed");
    assert_eq!(pubkey.len(), 32, "public key must be 32 bytes");

    assert!(
        ed25519::verify(message, &sig, &pubkey),
        "ed25519::verify failed for freshly signed message"
    );
    assert!(!ed25519::verify(b"wrong message", &sig, &pubkey), "verify must reject wrong message");
}
