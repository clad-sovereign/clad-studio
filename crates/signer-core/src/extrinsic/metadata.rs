//! Metadata-aware call encoding.
//!
//! # Current status — hand-rolled SCALE only (subxt-core deferred beyond restructure)
//!
//! `subxt-core 0.38` + `subxt-metadata 0.38` have transitive `std`-only paths that
//! block reliable `no_std + alloc` compilation without forking.  The integration is
//! **deferred beyond the restructure to the post-Phase-8 NFC firmware track** for
//! the following reasons:
//!
//! 1. `subxt-core 0.38` depends on `subxt-metadata 0.38`, which in turn depends on
//!    `scale-decode`, `scale-encode`, and `frame-metadata` with transitive `std`-only
//!    paths that make reliable `no_std + alloc` compilation non-trivial without forking.
//! 2. `pallet-clad-token` call indices 0–6 for pallet 8 are manually audited constants
//!    that are stable through all restructure phases.  Hand-rolled encoding with
//!    test-time cross-check is sufficient through Phase 3.
//! 3. `crates/server` (Phase 5) uses full `subxt` where `std` is fine; no `no_std`
//!    forcing function exists until post-Phase-8 NFC firmware work.
//!
//! Drift is caught by the metadata-corpus drift-detect test
//! (`tests/extrinsic_metadata_drift.rs`) and by the `roundtrip-node` CI job.

use crate::crypto::CryptoError;
use alloc::vec::Vec;

use super::call::{
    add_to_whitelist, freeze, mint, remove_from_whitelist, set_admin, transfer, unfreeze, CallData,
    CLAD_TOKEN_PALLET,
};

/// Known pallet names and their fixed indices in the Clad runtime.
///
/// Source: `runtime/src/lib.rs` `construct_runtime!` (verified against metadata_v14.scale corpus).
///
/// These are the indices that would be resolved dynamically by subxt-core once
/// that integration lands.  They are audited constants for now.
pub const KNOWN_PALLETS: &[(&str, u8)] = &[("CladToken", CLAD_TOKEN_PALLET), ("Multisig", 6)];

/// Build call data given a pallet name, call name, and raw argument bytes.
///
/// This is a thin dispatch layer over the typed builders in `call.rs`.
/// Only `CladToken` pallet calls are supported; all others return
/// [`CryptoError::UnknownPallet`] or [`CryptoError::UnknownCall`].
///
/// The `args` slice must contain SCALE-pre-encoded arguments in the order
/// expected by the call. Specifically:
///
/// | call            | args[0]              | args[1] (optional) |
/// |-----------------|----------------------|--------------------|
/// | `mint`          | AccountId (32 bytes) | raw LE u128 (16 bytes) |
/// | `transfer`      | AccountId (32 bytes) | raw LE u128 (16 bytes) |
/// | `freeze`        | AccountId (32 bytes) | — |
/// | `unfreeze`      | AccountId (32 bytes) | — |
/// | `add_to_whitelist`    | AccountId (32 bytes) | — |
/// | `remove_from_whitelist` | AccountId (32 bytes) | — |
/// | `set_admin`     | AccountId (32 bytes) | — |
///
/// For `mint` and `transfer`, `args[1]` is a raw little-endian u128 (16 bytes).
pub fn build_call_data(
    pallet_name: &str,
    call_name: &str,
    args: &[Vec<u8>],
) -> Result<CallData, CryptoError> {
    match pallet_name {
        "CladToken" => build_clad_token_call(call_name, args),
        _ => Err(CryptoError::UnknownPallet),
    }
}

fn build_clad_token_call(call_name: &str, args: &[Vec<u8>]) -> Result<CallData, CryptoError> {
    match call_name {
        "mint" | "transfer" => {
            let account = args.first().ok_or(CryptoError::UnknownCall)?;
            let amount_bytes = args.get(1).ok_or(CryptoError::UnknownCall)?;
            if amount_bytes.len() != 16 {
                return Err(CryptoError::UnknownCall);
            }
            let amount = u128::from_le_bytes(
                amount_bytes.as_slice().try_into().map_err(|_| CryptoError::UnknownCall)?,
            );
            if call_name == "mint" {
                Ok(mint(account, amount))
            } else {
                Ok(transfer(account, amount))
            }
        }
        "freeze" => {
            let account = args.first().ok_or(CryptoError::UnknownCall)?;
            Ok(freeze(account))
        }
        "unfreeze" => {
            let account = args.first().ok_or(CryptoError::UnknownCall)?;
            Ok(unfreeze(account))
        }
        "add_to_whitelist" => {
            let account = args.first().ok_or(CryptoError::UnknownCall)?;
            Ok(add_to_whitelist(account))
        }
        "remove_from_whitelist" => {
            let account = args.first().ok_or(CryptoError::UnknownCall)?;
            Ok(remove_from_whitelist(account))
        }
        "set_admin" => {
            let account = args.first().ok_or(CryptoError::UnknownCall)?;
            Ok(set_admin(account))
        }
        _ => Err(CryptoError::UnknownCall),
    }
}

/// Validate that the hardcoded pallet/call indices match the expected values.
///
/// This is a compile-time / unit-test cross-check.  Once subxt-core lands, this
/// function should additionally cross-check against the live metadata blob.
pub fn validate_known_call_indices(pallet_name: &str, call_name: &str) -> Option<(u8, u8)> {
    let pallet_idx =
        KNOWN_PALLETS.iter().find(|(name, _)| *name == pallet_name).map(|(_, idx)| *idx)?;

    let call_idx = match (pallet_name, call_name) {
        ("CladToken", "mint") => 0,
        ("CladToken", "transfer") => 1,
        ("CladToken", "freeze") => 2,
        ("CladToken", "unfreeze") => 3,
        ("CladToken", "add_to_whitelist") => 4,
        ("CladToken", "remove_from_whitelist") => 5,
        ("CladToken", "set_admin") => 6,
        _ => return None,
    };

    Some((pallet_idx, call_idx))
}
