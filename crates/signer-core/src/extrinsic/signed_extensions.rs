//! Substrate signed extensions encoding.
//!
//! Mirrors `SignedExtensions.kt` from
//! `clad-mobile/shared/src/commonMain/kotlin/tech/wideas/clad/substrate/extrinsic/`.
//!
//! # Extra (serialized into the extrinsic after the signature)
//!
//! ```text
//! Era || Compact<Nonce> || Compact<Tip>
//! ```
//!
//! # Additional (signed but NOT serialized into the extrinsic)
//!
//! ```text
//! spec_version(u32 LE) || tx_version(u32 LE) || genesis_hash([u8;32]) || block_hash([u8;32])
//! ```
//!
//! The `block_hash` field is the genesis hash for immortal transactions and
//! the recent block hash for mortal transactions.

use alloc::vec::Vec;

use super::call::scale::{compact_u128, compact_u64};
use super::era::Era;

/// Chain information required for transaction construction and signing.
///
/// Maps to the Kotlin `ChainInfo` data class in `SignedExtensions.kt`.
#[derive(Debug, Clone)]
pub struct ChainInfo {
    /// Genesis hash of the chain (32 bytes).
    pub genesis_hash: Vec<u8>,
    /// Recent block hash used for mortal era signing (32 bytes).
    /// For immortal transactions this should equal `genesis_hash`.
    pub block_hash: Vec<u8>,
    /// Runtime spec version (from `state_getRuntimeVersion`).
    pub spec_version: u32,
    /// Transaction format version (from `state_getRuntimeVersion`).
    pub tx_version: u32,
}

/// Signed extension fields attached to an extrinsic.
///
/// Maps to the Kotlin `SignedExtensions` data class.
///
/// Field names are aligned with the UDL dictionary so UniFFI can map them
/// without a conversion layer.
#[derive(Debug, Clone)]
pub struct SignedExtra {
    /// Era period (number of blocks for which the transaction is valid).
    /// Set to 0 for an immortal transaction.
    /// For mortal transactions: a power of 2 in the range 4–65536.
    pub era_period: u64,
    /// Era phase (`current_block % era_period`).  Set to 0 for immortal.
    pub era_phase: u64,
    /// Account nonce (prevents replay).
    pub nonce: u64,
    /// Priority tip (usually 0).
    ///
    /// `u64` at the FFI boundary (UniFFI 0.28 does not support `u128`).
    /// SCALE-encoded as `Compact<u128>` by zero-extending at encoding time.
    pub tip: u64,
}

impl SignedExtra {
    /// Derive a [`Era`] value from the `era_period` / `era_phase` fields.
    fn era(&self) -> Era {
        if self.era_period == 0 {
            Era::Immortal
        } else {
            Era::Mortal { period: self.era_period, phase: self.era_phase }
        }
    }

    /// Encode the **extra** portion: `Era || Compact<Nonce> || Compact<Tip>`.
    ///
    /// This is appended to the extrinsic after the signature.
    pub fn encode_extra(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.era().encode());
        out.extend_from_slice(&compact_u64(self.nonce));
        out.extend_from_slice(&compact_u128(self.tip as u128));
        out
    }

    /// Encode the **additional** portion: `spec_version || tx_version || genesis_hash || block_hash`.
    ///
    /// This is signed but NOT included in the extrinsic wire format.
    /// `block_hash` is chosen based on the era (genesis for immortal, recent block for mortal).
    pub fn encode_additional(&self, chain: &ChainInfo) -> Vec<u8> {
        let era = self.era();
        let block_hash = era.block_hash_for_signing(&chain.genesis_hash, &chain.block_hash);
        let mut out = Vec::new();
        out.extend_from_slice(&chain.spec_version.to_le_bytes());
        out.extend_from_slice(&chain.tx_version.to_le_bytes());
        out.extend_from_slice(&chain.genesis_hash);
        out.extend_from_slice(block_hash);
        out
    }
}
