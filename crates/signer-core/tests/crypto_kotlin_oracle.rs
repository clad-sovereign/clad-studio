//! Kotlin-oracle byte-stable KAT tests for crypto primitives.
//!
//! These tests verify that the Rust crypto output is byte-identical to the
//! Kotlin oracle (`Signer.kt`, `Hasher.kt`, `Ss58.kt`).
//!
//! **Status**: All tests are `#[ignore]`d pending Phase 2b work:
//! - Kotlin oracle JSON extraction script (`UosCryptoCorpusExport.kt`)
//! - SR25519/ED25519 deterministic test-vector generation
//! - Corpus files in `tests/corpora/crypto/`
//!
//! See: `docs/restructure-roadmap.md` § Deferred / discovered work.

// ── SR25519 byte-stable KAT ───────────────────────────────────────────────────

#[test]
#[ignore = "Phase 2b: requires Kotlin oracle corpus (UosCryptoCorpusExport.kt)"]
fn sr25519_kotlin_oracle_kat() {
    // When implemented: read tests/corpora/crypto/sr25519_sign.json,
    // call sr25519::sign_deterministic (test-only) per vector, assert byte equality.
    todo!("Phase 2b")
}

#[test]
#[ignore = "Phase 2b: requires Kotlin oracle corpus"]
fn sr25519_verify_kotlin_oracle_kat() {
    // When implemented: read tests/corpora/crypto/sr25519_verify.json,
    // call sr25519::verify per vector, assert all return true.
    todo!("Phase 2b")
}

// ── ED25519 byte-stable KAT ───────────────────────────────────────────────────

#[test]
#[ignore = "Phase 2b: requires Kotlin oracle corpus (UosCryptoCorpusExport.kt)"]
fn ed25519_kotlin_oracle_kat() {
    // When implemented: read tests/corpora/crypto/ed25519_sign.json,
    // call ed25519::sign per vector (deterministic), assert byte equality.
    todo!("Phase 2b")
}

#[test]
#[ignore = "Phase 2b: requires Kotlin oracle corpus"]
fn ed25519_verify_kotlin_oracle_kat() {
    todo!("Phase 2b")
}

// ── Blake2b byte-stable KAT ───────────────────────────────────────────────────

#[test]
#[ignore = "Phase 2b: requires Kotlin oracle corpus (Hasher.kt output)"]
fn blake2b_256_kotlin_oracle_kat() {
    // When implemented: read tests/corpora/crypto/blake2b_256_oracle.json,
    // call blake2::blake2b_256 per vector, assert byte equality.
    todo!("Phase 2b")
}

#[test]
#[ignore = "Phase 2b: requires Kotlin oracle corpus (Hasher.kt output)"]
fn blake2b_512_kotlin_oracle_kat() {
    todo!("Phase 2b")
}
