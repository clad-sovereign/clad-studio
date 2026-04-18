// `empty_line_after_doc_comments` is suppressed at crate level because the
// UniFFI 0.28 scaffolding generator (invoked below via `include_scaffolding!`)
// emits `///`-style doc comments followed by a blank line on top-level
// constants — a pattern newer clippy rejects but which we cannot modify
// without forking the generator. Remove this attribute once UniFFI is
// upgraded past the fix (tracked for Phase 1 cleanup).
#![allow(clippy::empty_line_after_doc_comments)]

//! # signer-core
//!
//! Shared protocol and crypto primitives for the Clad Sovereign signer.
//!
//! **Phase 0 status:** this crate currently exposes only a trivial `ping()`
//! function, used to prove the UniFFI cross-compile pipeline end-to-end on
//! Android and iOS. Real protocol and crypto code is scheduled to arrive in
//! Phases 1 and 2 (see `clad-studio/docs/migration/01-phases.md`). Do not
//! build consumers against this crate's API yet.
//!
//! See ADR-007 (`docs/adr/007-rust-signer-core-via-uniffi.md`) for the
//! architectural motivation.

// Pull in the UniFFI-generated scaffolding produced by `build.rs` from
// `src/signer_core.udl`. This macro emits the extern "C" ABI that the
// generated Kotlin and Swift bindings call into.
uniffi::include_scaffolding!("signer_core");

/// Returns a fixed greeting string.
///
/// Phase 0 liveness check: exercised by the Rust unit test in
/// `tests/ping_test.rs`, the Kotlin JVM sample in
/// `android/sample/src/test/kotlin/PingTest.kt`, and the Swift sample in
/// `ios/Tests/SignerCoreTests/PingTests.swift`.
pub fn ping() -> String {
    "pong from signer-core".to_string()
}
