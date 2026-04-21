//! Known-answer tests for `build_signing_payload`.
//!
//! **Status**: Stubs — pending Phase 2b.
//!
//! The signing payload is the concatenation of:
//!   `call_data || encoded_extra || encoded_additional`
//! Blake2b-256 is applied when the payload is ≥ 256 bytes.
//!
//! Full KAT vectors require a live `clad-node --dev` instance to capture the
//! genesis hash and spec/tx versions; those are deferred to Phase 2b along
//! with the roundtrip-node feature.
//!
//! See: `docs/restructure-roadmap.md` § Deferred / discovered work.

use signer_core::extrinsic::{call, payload, signed_extensions};

/// Smoke test: payload for a trivial immortal transfer does not panic and is
/// non-empty. Does not assert exact bytes (those require node corpus).
#[test]
fn build_signing_payload_immortal_smoke() {
    let alice = [
        0xd4u8, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9, 0x9f,
        0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7, 0xa5, 0x6d,
        0xa2, 0x7d,
    ];
    let call_data = call::transfer(&alice, 1);

    let extra = signed_extensions::SignedExtra {
        era_period: 0, // Immortal
        era_phase: 0,
        nonce: 0,
        tip: 0,
    };
    let chain = signed_extensions::ChainInfo {
        genesis_hash: vec![0u8; 32],
        block_hash: vec![0u8; 32],
        spec_version: 1,
        tx_version: 1,
    };

    let p = payload::build_signing_payload(&call_data, &extra, &chain);
    assert!(!p.is_empty(), "signing payload must not be empty");
}

#[test]
#[ignore = "Phase 2b: requires live clad-node corpus (genesis_hash, spec_version, tx_version)"]
fn build_signing_payload_kotlin_oracle_kat() {
    todo!("Phase 2b")
}

#[test]
#[ignore = "Phase 2b: requires large-payload Blake2b truncation corpus"]
fn build_signing_payload_large_payload_is_hashed() {
    todo!("Phase 2b")
}
