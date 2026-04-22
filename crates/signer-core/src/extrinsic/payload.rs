//! Signing payload construction.
//!
//! Mirrors `ExtrinsicBuilder.buildSigningPayload` in
//! `clad-mobile/shared/src/commonMain/kotlin/tech/wideas/clad/substrate/extrinsic/ExtrinsicBuilder.kt`.
//!
//! # Signing payload wire format
//!
//! ```text
//! call_data || extra || additional
//! ```
//!
//! If `len(payload) >= 256`, Blake2b-256-hash it before signing.
//! This matches `MAX_PAYLOAD_SIZE_FOR_RAW_SIGNING = 256` in `ExtrinsicBuilder.kt`.
//!
//! References:
//! - Substrate `sp_runtime::generic::UncheckedExtrinsic::signature_payload`

use alloc::vec::Vec;

use crate::crypto::blake2::blake2b_256;

use super::signed_extensions::{ChainInfo, SignedExtra};

/// Maximum payload size (bytes) before Blake2b-256 hashing is required.
///
/// Source: `ExtrinsicBuilder.MAX_PAYLOAD_SIZE_FOR_RAW_SIGNING = 256`.
pub const MAX_PAYLOAD_SIZE_FOR_RAW_SIGNING: usize = 256;

/// Build the signing payload for a call.
///
/// Concatenates `call_data || extra || additional` then, if the result is ≥ 256
/// bytes, replaces it with its Blake2b-256 hash (32 bytes).
pub fn build_signing_payload(call_data: &[u8], extra: &SignedExtra, chain: &ChainInfo) -> Vec<u8> {
    let mut payload = Vec::with_capacity(call_data.len() + 64 + 72);
    payload.extend_from_slice(call_data);
    payload.extend_from_slice(&extra.encode_extra());
    payload.extend_from_slice(&extra.encode_additional(chain));

    if payload.len() >= MAX_PAYLOAD_SIZE_FOR_RAW_SIGNING {
        blake2b_256(&payload)
    } else {
        payload
    }
}
