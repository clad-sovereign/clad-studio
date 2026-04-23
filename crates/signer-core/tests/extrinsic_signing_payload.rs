//! Tests for `build_signing_payload`.
//!
//! The signing payload is:
//!   `call_data || encoded_extra || encoded_additional`
//!
//! When the payload is ≥ 256 bytes, Blake2b-256 is applied before signing.
//! This behaviour is tested by `build_signing_payload_large_payload_is_hashed`.

use signer_core::extrinsic::{call, payload, signed_extensions};

/// Smoke test: immortal transfer payload is non-empty and does not panic.
#[test]
fn build_signing_payload_immortal_smoke() {
    let alice = [
        0xd4u8, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9, 0x9f,
        0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7, 0xa5, 0x6d,
        0xa2, 0x7d,
    ];
    let call_data = call::transfer(&alice, 1);

    let extra = signed_extensions::SignedExtra { era_period: 0, era_phase: 0, nonce: 0, tip: 0 };
    let chain = signed_extensions::ChainInfo {
        genesis_hash: vec![0u8; 32],
        block_hash: vec![0u8; 32],
        spec_version: 1,
        tx_version: 1,
    };

    let p = payload::build_signing_payload(&call_data, &extra, &chain);
    assert!(!p.is_empty(), "signing payload must not be empty");
}

/// When the payload is ≥ 256 bytes the function must return the 32-byte
/// Blake2b-256 hash of the full payload, not the raw bytes.
///
/// Gated behind `roundtrip-node` to group it with the live-node suite so CI
/// runs it in the same job; it does not actually require a running node.
#[cfg_attr(
    not(feature = "roundtrip-node"),
    ignore = "Phase 2b: enable with --features roundtrip-node (does not require a live node)"
)]
#[test]
fn build_signing_payload_large_payload_is_hashed() {
    use signer_core::crypto::blake2::blake2b_256;

    // Build a call_data that is large enough to push the full payload above 256
    // bytes.  A remark-style 220-byte inner data vector works well: the transfer
    // call itself is 36 bytes, and extra + additional add ~72 bytes, so 220 bytes
    // of padding easily exceeds the 256-byte threshold.
    let alice = [
        0xd4u8, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9, 0x9f,
        0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7, 0xa5, 0x6d,
        0xa2, 0x7d,
    ];

    // Start from a normal transfer call and append padding to exceed 256 bytes.
    let mut call_data = call::transfer(&alice, 1);
    call_data.extend(vec![0xffu8; 220]); // total call_data ≈ 256 bytes alone

    let extra = signed_extensions::SignedExtra { era_period: 0, era_phase: 0, nonce: 0, tip: 0 };
    let chain = signed_extensions::ChainInfo {
        genesis_hash: vec![0u8; 32],
        block_hash: vec![0u8; 32],
        spec_version: 1,
        tx_version: 1,
    };

    // Compute what the full payload would be before truncation.
    let mut full = Vec::new();
    full.extend_from_slice(&call_data);
    full.extend_from_slice(&extra.encode_extra());
    full.extend_from_slice(&extra.encode_additional(&chain));

    assert!(
        full.len() >= 256,
        "test setup error: payload is only {} bytes, need ≥ 256",
        full.len()
    );

    let result = payload::build_signing_payload(&call_data, &extra, &chain);

    // The function must return the 32-byte hash, not the raw payload.
    assert_eq!(result.len(), 32, "hashed payload must be 32 bytes (got {})", result.len());
    assert_eq!(
        result,
        blake2b_256(&full),
        "hashed payload must equal Blake2b-256 of the full payload"
    );
}
