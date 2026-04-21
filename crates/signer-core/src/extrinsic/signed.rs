//! Signed extrinsic construction.
//!
//! Mirrors `ExtrinsicBuilder.buildExtrinsicData` and `buildSignedExtrinsic` in
//! `clad-mobile/shared/src/commonMain/kotlin/tech/wideas/clad/substrate/extrinsic/ExtrinsicBuilder.kt`.
//!
//! # Signed extrinsic wire format (Substrate v4)
//!
//! ```text
//! Compact<length> || ExtrinsicData
//!
//! ExtrinsicData =
//!     version_byte(0x84)        -- signed(bit7) | version(4)
//!     || address                -- MultiAddress::Id: 0x00 || AccountId(32)
//!     || signature              -- MultiSignature::Sr25519: 0x01 || sig(64)
//!     || extra                  -- Era || Compact<Nonce> || Compact<Tip>
//!     || call_data              -- pallet_index || call_index || params
//! ```
//!
//! The extrinsic hash is Blake2b-256 of the complete length-prefixed bytes.

use alloc::vec::Vec;

use crate::crypto::blake2::blake2b_256;

use super::call::scale::compact_usize;
use super::signed_extensions::SignedExtra;

/// Version byte: signed(bit7=1) | format version 4.
const SIGNED_EXTRINSIC_VERSION: u8 = 0x84;

/// `MultiAddress::Id` enum variant — raw AccountId (32 bytes).
const MULTI_ADDRESS_ID: u8 = 0x00;

/// `MultiSignature::Sr25519` enum variant — 64-byte signature.
const MULTI_SIGNATURE_SR25519: u8 = 0x01;

/// A complete signed extrinsic ready for submission.
///
/// Maps to `SignedExtrinsic` in `ExtrinsicBuilder.kt`.
#[derive(Debug, Clone)]
pub struct SignedExtrinsic {
    /// Complete SCALE-encoded extrinsic (with compact length prefix).
    pub encoded: Vec<u8>,
    /// Blake2b-256 hash of `encoded` (for transaction tracking).
    pub hash: Vec<u8>,
}

/// Build a signed extrinsic from pre-computed parts.
///
/// Arguments:
/// - `call_data`: SCALE-encoded call (pallet + call index + params)
/// - `signer_public_key`: 32-byte SR25519 public key
/// - `signature`: 64-byte SR25519 signature over the signing payload
/// - `extra`: signed extension fields (era, nonce, tip)
pub fn build_signed_extrinsic(
    call_data: &[u8],
    signer_public_key: &[u8],
    signature: &[u8],
    extra: &SignedExtra,
) -> SignedExtrinsic {
    let body = build_extrinsic_body(call_data, signer_public_key, signature, extra);
    wrap_with_length(body)
}

/// Complete an unsigned extrinsic payload with an externally computed signature.
///
/// Identical to [`build_signed_extrinsic`] — separated to mirror the Kotlin API
/// (`completeWithSignature` vs `buildSignedExtrinsic`).
pub fn complete_with_signature(
    call_data: &[u8],
    signer_public_key: &[u8],
    signature: &[u8],
    extra: &SignedExtra,
) -> SignedExtrinsic {
    build_signed_extrinsic(call_data, signer_public_key, signature, extra)
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Encode the inner extrinsic body (without the compact-length prefix).
fn build_extrinsic_body(
    call_data: &[u8],
    signer_public_key: &[u8],
    signature: &[u8],
    extra: &SignedExtra,
) -> Vec<u8> {
    assert_eq!(signer_public_key.len(), 32, "public key must be 32 bytes");
    assert_eq!(signature.len(), 64, "signature must be 64 bytes");

    let mut out = Vec::with_capacity(1 + 33 + 65 + 64 + call_data.len());

    // version byte
    out.push(SIGNED_EXTRINSIC_VERSION);

    // address: MultiAddress::Id (0x00 + raw AccountId)
    out.push(MULTI_ADDRESS_ID);
    out.extend_from_slice(signer_public_key);

    // signature: MultiSignature::Sr25519 (0x01 + 64 bytes)
    out.push(MULTI_SIGNATURE_SR25519);
    out.extend_from_slice(signature);

    // extra (era, nonce, tip)
    out.extend_from_slice(&extra.encode_extra());

    // call data
    out.extend_from_slice(call_data);

    out
}

/// Prepend SCALE compact length and compute the extrinsic hash.
fn wrap_with_length(body: Vec<u8>) -> SignedExtrinsic {
    let mut encoded = compact_usize(body.len());
    encoded.extend_from_slice(&body);
    let hash = blake2b_256(&encoded);
    SignedExtrinsic { encoded, hash }
}
