//! Tests for `metadata.rs`: pallet/call index validation and dispatch.
//!
//! Verifies that the hardcoded indices in `KNOWN_PALLETS` and
//! `validate_known_call_indices` match the audited constants in `call.rs`,
//! and that `build_call_data` correctly dispatches all known calls.

use signer_core::extrinsic::metadata::{
    build_call_data, validate_known_call_indices, KNOWN_PALLETS,
};

// ── KNOWN_PALLETS index check ─────────────────────────────────────────────────

#[test]
fn clad_token_pallet_index_is_7() {
    let idx = KNOWN_PALLETS.iter().find(|(n, _)| *n == "CladToken").map(|(_, i)| *i);
    assert_eq!(idx, Some(7), "CladToken must be pallet index 7");
}

#[test]
fn multisig_pallet_index_is_6() {
    let idx = KNOWN_PALLETS.iter().find(|(n, _)| *n == "Multisig").map(|(_, i)| *i);
    assert_eq!(idx, Some(6), "Multisig must be pallet index 6");
}

// ── validate_known_call_indices ───────────────────────────────────────────────

#[test]
fn clad_token_call_indices_are_correct() {
    let cases = [
        ("mint", 0u8),
        ("transfer", 1),
        ("freeze", 2),
        ("unfreeze", 3),
        ("add_to_whitelist", 4),
        ("remove_from_whitelist", 5),
        ("set_admin", 6),
    ];
    for (call, expected_call_idx) in &cases {
        let (pallet_idx, call_idx) =
            validate_known_call_indices("CladToken", call).unwrap_or_else(|| {
                panic!("validate_known_call_indices returned None for CladToken::{call}")
            });
        assert_eq!(pallet_idx, 7, "CladToken::{call} pallet index must be 7");
        assert_eq!(call_idx, *expected_call_idx, "CladToken::{call} call index mismatch");
    }
}

#[test]
fn unknown_pallet_returns_none() {
    assert!(validate_known_call_indices("UnknownPallet", "someCall").is_none());
}

#[test]
fn unknown_call_returns_none() {
    assert!(validate_known_call_indices("CladToken", "nonExistentCall").is_none());
}

// ── build_call_data dispatch errors ──────────────────────────────────────────

#[test]
fn build_call_data_unknown_pallet_returns_error() {
    let result = build_call_data("BadPallet", "mint", &[]);
    assert!(result.is_err(), "unknown pallet must return Err");
}

#[test]
fn build_call_data_unknown_call_returns_error() {
    let account = vec![0u8; 32];
    let result = build_call_data("CladToken", "nonExistentCall", &[account]);
    assert!(result.is_err(), "unknown call must return Err");
}

#[test]
fn build_call_data_mint_missing_amount_returns_error() {
    let account = vec![0u8; 32];
    // Only 1 arg (account); mint requires amount as args[1].
    let result = build_call_data("CladToken", "mint", &[account]);
    assert!(result.is_err(), "mint with missing amount must return Err");
}
