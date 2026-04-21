//! Known-answer tests for `build_signed_extrinsic` and `complete_with_signature`.
//!
//! **Status**: Stubs — pending Phase 2b.
//!
//! Full KAT vectors require a live `clad-node --dev` instance to record:
//!   - genesis hash, spec/tx version, nonce
//!   - A reference SR25519 signing key and the resulting on-chain extrinsic hash
//!
//! Those are captured by the `roundtrip-node` feature and deferred to Phase 2b.
//!
//! See: `docs/restructure-roadmap.md` § Deferred / discovered work.

#[test]
#[ignore = "Phase 2b: requires live clad-node corpus for exact extrinsic bytes"]
fn signed_extrinsic_kotlin_oracle_kat() {
    todo!("Phase 2b")
}

#[test]
#[ignore = "Phase 2b: requires live clad-node corpus"]
fn complete_with_signature_kotlin_oracle_kat() {
    todo!("Phase 2b")
}

#[test]
#[ignore = "Phase 2b: roundtrip-node feature — submit to dev node and verify acceptance"]
fn signed_extrinsic_roundtrip_against_node() {
    todo!("Phase 2b")
}
