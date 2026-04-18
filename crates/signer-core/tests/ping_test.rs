//! Phase 0 liveness check — exercises the `ping()` function through the
//! normal Rust API surface. The FFI boundary is exercised separately by the
//! Kotlin JVM sample (`android/sample`) and the SwiftPM sample (`ios/`).

use signer_core::ping;

#[test]
fn ping_returns_expected_greeting() {
    assert_eq!(ping(), "pong from signer-core");
}
