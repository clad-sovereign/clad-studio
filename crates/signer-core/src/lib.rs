// `empty_line_after_doc_comments` is suppressed at crate level because the
// UniFFI 0.28 scaffolding generator (invoked below via `include_scaffolding!`)
// emits `///`-style doc comments followed by a blank line on top-level
// constants — a pattern newer clippy rejects but which we cannot modify
// without forking the generator. Tracked for removal after UniFFI upgrade
// (deferred to Phase 3 prep per the restructure roadmap).
#![allow(clippy::empty_line_after_doc_comments)]

//! # signer-core
//!
//! Shared protocol and crypto primitives for the Clad Sovereign signer.
//!
//! **Phase 1 status:** the `uos` module is now available and provides a
//! byte-for-byte Rust port of the UOS (Universal Offline Signatures) protocol
//! from `clad-mobile/shared/`.  Sr25519 signing primitives and SCALE extrinsic
//! construction land in Phase 2.  Neither mobile app consumes this Rust path
//! until Phase 3 (feature-flag wiring).
//!
//! See ADR-007 (`docs/adr/007-rust-signer-core-via-uniffi.md`) for the
//! architectural motivation.

pub mod uos;

pub use uos::account_introduction::AccountIntroduction;
pub use uos::error::UosError;
pub use uos::multipart::{FrameDecodeProgress, MultiPartQrDecoder, MultiPartQrEncoder};
pub use uos::payload::UosPayload;
pub use uos::signature::UosSignature;

uniffi::include_scaffolding!("signer_core");

// ── Phase 0 liveness check ───────────────────────────────────────────────────

/// Returns a fixed greeting string.
///
/// Phase 0 liveness check: exercised by the Rust unit test in
/// `tests/ping_test.rs`, the Kotlin JVM sample in
/// `android/sample/src/test/kotlin/PingTest.kt`, and the Swift sample in
/// `ios/Tests/SignerCoreTests/PingTests.swift`.
pub fn ping() -> String {
    "pong from signer-core".to_string()
}

// ── UOS namespace-level free functions ───────────────────────────────────────
// These are the entry points declared in `signer_core.udl`.  They bridge the
// UniFFI `dictionary` types (which use `Vec<u8>` for all byte fields) and the
// internal UOS types.

/// Encodes a [`UosPayload`] to its binary UOS representation.
pub fn encode_payload(payload: UosPayload) -> Result<Vec<u8>, UosError> {
    payload.encode()
}

/// Decodes a binary UOS payload into a [`UosPayload`].
pub fn decode_payload(data: Vec<u8>) -> Result<UosPayload, UosError> {
    UosPayload::decode(&data)
}

/// Encodes a [`UosSignature`] to its binary UOS representation.
pub fn encode_signature(sig: UosSignature) -> Result<Vec<u8>, UosError> {
    Ok(sig.encode())
}

/// Decodes a binary UOS signature into a [`UosSignature`].
pub fn decode_signature(data: Vec<u8>) -> Result<UosSignature, UosError> {
    UosSignature::decode(&data)
}

/// Serialises an [`AccountIntroduction`] to a `substrate:…` URI.
pub fn account_intro_to_uri(account: AccountIntroduction) -> String {
    account.to_uri()
}

/// Parses an [`AccountIntroduction`] from a `substrate:…` URI.
pub fn account_intro_from_uri(uri: String) -> Result<AccountIntroduction, UosError> {
    AccountIntroduction::from_uri(&uri)
}
