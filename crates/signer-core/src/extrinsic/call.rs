//! SCALE-encoded call data builders.
//!
//! Mirrors `CladTokenCalls.kt` and `MultisigCalls.kt` 1:1.
//!
//! # Wire format per call
//!
//! ```text
//! [pallet_index: u8] [call_index: u8] [params...]
//! ```
//!
//! Params are SCALE-encoded:
//! - `AccountId`: raw 32 bytes (no prefix — `writeAccountId` in Kotlin)
//! - `u128 amount`: SCALE Compact<u128>
//! - `u16 threshold`: little-endian u16
//! - `Vec<AccountId>`: Compact<len> followed by each AccountId (32 bytes each)
//! - `Option<Timepoint>`: 0x00 (None) or 0x01 + height(u32 LE) + index(u32 LE)
//! - `Vec<u8> callData`: Compact<len> followed by bytes
//! - `Weight`: refTime (Compact<u64>) + proofSize (Compact<u64>)
//!
//! # Pallet indices
//!
//! - `pallet-clad-token`: index **8** (source: `runtime/src/lib.rs` `construct_runtime!`)
//! - `pallet-multisig`:   index **7** (source: `MultisigPalletConfig.PALLET_INDEX`)

use alloc::vec::Vec;

use self::scale::{compact_u128, compact_u64, compact_usize};

// Re-export the helper module so tests can use it directly.
pub(crate) mod scale;

/// An opaque SCALE-encoded call blob.
pub type CallData = Vec<u8>;

// ── pallet-clad-token ─────────────────────────────────────────────────────────

/// Pallet index for `pallet-clad-token` in the Clad runtime.
pub const CLAD_TOKEN_PALLET: u8 = 8;

/// Call indices for `pallet-clad-token`.
pub mod clad_token_call {
    pub const MINT: u8 = 0;
    pub const TRANSFER: u8 = 1;
    pub const FREEZE: u8 = 2;
    pub const UNFREEZE: u8 = 3;
    pub const ADD_TO_WHITELIST: u8 = 4;
    pub const REMOVE_FROM_WHITELIST: u8 = 5;
    pub const SET_ADMIN: u8 = 6;
}

/// Build a `mint(to, amount)` call.
///
/// `to` must be exactly 32 bytes (AccountId).
/// `amount` is a SCALE Compact<u128>.
pub fn mint(to: &[u8], amount: u128) -> CallData {
    assert_eq!(to.len(), 32, "AccountId must be 32 bytes");
    let mut out = Vec::with_capacity(2 + 32 + 17);
    out.push(CLAD_TOKEN_PALLET);
    out.push(clad_token_call::MINT);
    out.push(0x00); // MultiAddress::Id prefix
    out.extend_from_slice(to);
    out.extend_from_slice(&compact_u128(amount));
    out
}

/// Build a `transfer(to, amount)` call.
pub fn transfer(to: &[u8], amount: u128) -> CallData {
    assert_eq!(to.len(), 32, "AccountId must be 32 bytes");
    let mut out = Vec::with_capacity(2 + 33 + 17);
    out.push(CLAD_TOKEN_PALLET);
    out.push(clad_token_call::TRANSFER);
    out.push(0x00); // MultiAddress::Id prefix
    out.extend_from_slice(to);
    out.extend_from_slice(&compact_u128(amount));
    out
}

/// Build a `freeze(account)` call.
pub fn freeze(account: &[u8]) -> CallData {
    assert_eq!(account.len(), 32, "AccountId must be 32 bytes");
    let mut out = Vec::with_capacity(2 + 33);
    out.push(CLAD_TOKEN_PALLET);
    out.push(clad_token_call::FREEZE);
    out.push(0x00); // MultiAddress::Id prefix
    out.extend_from_slice(account);
    out
}

/// Build an `unfreeze(account)` call.
pub fn unfreeze(account: &[u8]) -> CallData {
    assert_eq!(account.len(), 32, "AccountId must be 32 bytes");
    let mut out = Vec::with_capacity(2 + 33);
    out.push(CLAD_TOKEN_PALLET);
    out.push(clad_token_call::UNFREEZE);
    out.push(0x00); // MultiAddress::Id prefix
    out.extend_from_slice(account);
    out
}

/// Build an `add_to_whitelist(account)` call.
pub fn add_to_whitelist(account: &[u8]) -> CallData {
    assert_eq!(account.len(), 32, "AccountId must be 32 bytes");
    let mut out = Vec::with_capacity(2 + 33);
    out.push(CLAD_TOKEN_PALLET);
    out.push(clad_token_call::ADD_TO_WHITELIST);
    out.push(0x00); // MultiAddress::Id prefix
    out.extend_from_slice(account);
    out
}

/// Build a `remove_from_whitelist(account)` call.
pub fn remove_from_whitelist(account: &[u8]) -> CallData {
    assert_eq!(account.len(), 32, "AccountId must be 32 bytes");
    let mut out = Vec::with_capacity(2 + 33);
    out.push(CLAD_TOKEN_PALLET);
    out.push(clad_token_call::REMOVE_FROM_WHITELIST);
    out.push(0x00); // MultiAddress::Id prefix
    out.extend_from_slice(account);
    out
}

/// Build a `set_admin(new_admin)` call.
pub fn set_admin(new_admin: &[u8]) -> CallData {
    assert_eq!(new_admin.len(), 32, "AccountId must be 32 bytes");
    let mut out = Vec::with_capacity(2 + 33);
    out.push(CLAD_TOKEN_PALLET);
    out.push(clad_token_call::SET_ADMIN);
    out.push(0x00); // MultiAddress::Id prefix
    out.extend_from_slice(new_admin);
    out
}

// ── pallet-multisig ───────────────────────────────────────────────────────────

/// Pallet index for `pallet-multisig` in the Clad runtime.
pub const MULTISIG_PALLET: u8 = 7;

/// Call indices for `pallet-multisig`.
pub mod multisig_call {
    pub const AS_MULTI_THRESHOLD_1: u8 = 0;
    pub const AS_MULTI: u8 = 1;
    pub const APPROVE_AS_MULTI: u8 = 2;
    pub const CANCEL_AS_MULTI: u8 = 3;
}

/// Build an `as_multi(threshold, other_signatories, maybe_timepoint, call, max_weight)` call.
///
/// `other_signatories` must be sorted and must not contain the caller.
/// `call_data` is the SCALE-encoded inner call (length-prefixed as `Vec<u8>`).
/// `max_weight` is `(ref_time, proof_size)`.
pub fn as_multi(
    threshold: u16,
    other_signatories: &[&[u8]],
    maybe_timepoint: Option<(u32, u32)>,
    call_data: &[u8],
    max_weight: (u64, u64),
) -> CallData {
    let mut out = Vec::new();
    out.push(MULTISIG_PALLET);
    out.push(multisig_call::AS_MULTI);
    // threshold: u16 LE
    out.extend_from_slice(&threshold.to_le_bytes());
    // other_signatories: Vec<AccountId>
    encode_account_id_vec(&mut out, other_signatories);
    // maybe_timepoint: Option<(block_number: u32, index: u32)>
    encode_option_timepoint(&mut out, maybe_timepoint);
    // call: Box<Call> encoded as length-prefixed byte vector
    out.extend_from_slice(&compact_usize(call_data.len()));
    out.extend_from_slice(call_data);
    // max_weight: Weight { ref_time: Compact<u64>, proof_size: Compact<u64> }
    out.extend_from_slice(&compact_u64(max_weight.0));
    out.extend_from_slice(&compact_u64(max_weight.1));
    out
}

/// Build an `approve_as_multi(threshold, other_signatories, maybe_timepoint, call_hash, max_weight)` call.
///
/// `call_hash` must be exactly 32 bytes (Blake2-256 hash of the inner call).
pub fn approve_as_multi(
    threshold: u16,
    other_signatories: &[&[u8]],
    maybe_timepoint: Option<(u32, u32)>,
    call_hash: &[u8],
    max_weight: (u64, u64),
) -> CallData {
    assert_eq!(call_hash.len(), 32, "call_hash must be 32 bytes");
    let mut out = Vec::new();
    out.push(MULTISIG_PALLET);
    out.push(multisig_call::APPROVE_AS_MULTI);
    out.extend_from_slice(&threshold.to_le_bytes());
    encode_account_id_vec(&mut out, other_signatories);
    encode_option_timepoint(&mut out, maybe_timepoint);
    out.extend_from_slice(call_hash); // [u8; 32] — fixed, no length prefix
    out.extend_from_slice(&compact_u64(max_weight.0));
    out.extend_from_slice(&compact_u64(max_weight.1));
    out
}

/// Build a `cancel_as_multi(threshold, other_signatories, timepoint, call_hash)` call.
pub fn cancel_as_multi(
    threshold: u16,
    other_signatories: &[&[u8]],
    timepoint: (u32, u32),
    call_hash: &[u8],
) -> CallData {
    assert_eq!(call_hash.len(), 32, "call_hash must be 32 bytes");
    let mut out = Vec::new();
    out.push(MULTISIG_PALLET);
    out.push(multisig_call::CANCEL_AS_MULTI);
    out.extend_from_slice(&threshold.to_le_bytes());
    encode_account_id_vec(&mut out, other_signatories);
    // Timepoint is required (not Option) for cancel.
    encode_timepoint(&mut out, timepoint);
    out.extend_from_slice(call_hash);
    out
}

/// Sort a slice of 32-byte AccountIds lexicographically (raw bytes).
///
/// Mirrors `MultisigCalls.sortSignatories`.
pub fn sort_signatories(signatories: &mut [Vec<u8>]) {
    signatories.sort();
}

// ── SCALE helpers (private) ───────────────────────────────────────────────────

fn encode_account_id_vec(out: &mut Vec<u8>, accounts: &[&[u8]]) {
    out.extend_from_slice(&compact_usize(accounts.len()));
    for acc in accounts {
        assert_eq!(acc.len(), 32, "AccountId must be 32 bytes");
        out.extend_from_slice(acc);
    }
}

fn encode_option_timepoint(out: &mut Vec<u8>, tp: Option<(u32, u32)>) {
    match tp {
        None => out.push(0x00),
        Some((height, index)) => {
            out.push(0x01);
            encode_timepoint(out, (height, index));
        }
    }
}

fn encode_timepoint(out: &mut Vec<u8>, (height, index): (u32, u32)) {
    out.extend_from_slice(&height.to_le_bytes());
    out.extend_from_slice(&index.to_le_bytes());
}
