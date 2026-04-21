//! Substrate transaction era encoding.
//!
//! Mirrors `Era.kt` in `clad-mobile/shared/src/commonMain/kotlin/tech/wideas/clad/substrate/scale/`.
//!
//! # Era SCALE wire format
//!
//! - **Immortal**: single byte `0x00`
//! - **Mortal(period, phase)**: 2 bytes (little-endian u16)
//!   - `encoded_u16 = (log2(period) - 1).clamp(1, 15) | ((phase / quantize_factor) << 4)`
//!   - `quantize_factor = max(1, period >> 12)`
//!
//! Decode is the inverse:
//!   - `period = 2 << (encoded_u16 & 0x0F)`
//!   - `phase  = (encoded_u16 >> 4) * quantize_factor`
//!
//! References:
//! - `sp_runtime::generic::Era` in polkadot-sdk (source of truth)
//! - Substrate extrinsic format docs

use alloc::vec::Vec;

/// Transaction validity period.
///
/// For government-focused users the default is `Immortal` (valid until
/// included or nonce advances), matching `Era.Immortal` in `Era.kt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Era {
    /// Transaction is valid forever.
    ///
    /// SCALE: single byte `0x00`.
    /// Block hash used in signing payload: genesis hash.
    Immortal,
    /// Transaction is only valid within a specific block range.
    ///
    /// SCALE: 2-byte little-endian u16 (see module doc).
    /// Block hash used in signing payload: the recent block hash.
    Mortal {
        /// Number of blocks for which this transaction is valid.
        /// Must be a power of 2 in the range 4–65536.
        period: u64,
        /// Phase within the period (derived from block number % period).
        phase: u64,
    },
}

impl Era {
    /// SCALE-encode the era to bytes.
    ///
    /// Matches `sp_runtime::generic::Era::encode` in polkadot-sdk.
    ///
    /// Immortal → `[0x00]`
    /// Mortal   → 2-byte little-endian u16
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Era::Immortal => vec![0x00],
            Era::Mortal { period, phase } => {
                // `quantize_factor` reduces phase precision for very large periods.
                // For period ≤ 4096 it is 1 (no quantisation).
                let quantize_factor = (period >> 12).max(1);
                // Low 4 bits: log2(period) - 1, clamped to 1..=15.
                let period_bits = (period.trailing_zeros() - 1).clamp(1, 15) as u16;
                // High 12 bits: quantised phase.
                let phase_bits = ((phase / quantize_factor) << 4) as u16;
                let encoded: u16 = period_bits | phase_bits;
                encoded.to_le_bytes().to_vec()
            }
        }
    }

    /// Decode an era from a SCALE byte slice (1 or 2 bytes at the front).
    ///
    /// Matches `sp_runtime::generic::Era::decode` in polkadot-sdk.
    ///
    /// Returns `(era, bytes_consumed)`.
    pub fn decode(data: &[u8]) -> Option<(Self, usize)> {
        let b0 = *data.first()?;
        if b0 == 0 {
            Some((Era::Immortal, 1))
        } else {
            let b1 = *data.get(1)?;
            let encoded = b0 as u64 | ((b1 as u64) << 8);
            // `period = 2 << (encoded & 0x0F)` — inverse of `period.trailing_zeros() - 1`.
            let period = 2u64 << (encoded & 0x0F);
            let quantize_factor = (period >> 12).max(1);
            let phase = (encoded >> 4) * quantize_factor;
            Some((Era::Mortal { period, phase }, 2))
        }
    }

    /// Return the block hash to include in the signing payload.
    ///
    /// - Immortal → genesis hash (matches `Era.Immortal.getBlockHashForSigning`)
    /// - Mortal   → the recent block hash
    pub fn block_hash_for_signing<'a>(
        &self,
        genesis_hash: &'a [u8],
        block_hash: &'a [u8],
    ) -> &'a [u8] {
        match self {
            Era::Immortal => genesis_hash,
            Era::Mortal { .. } => block_hash,
        }
    }

    /// Construct a mortal era from a current block number.
    ///
    /// `period_blocks` is rounded to the nearest power of 2 and clamped to 4–65536.
    pub fn mortal_from_block(current_block: u64, period_blocks: u64) -> Self {
        let period = next_power_of_two(period_blocks.clamp(4, 65536));
        let quantize_factor = (period >> 12).max(1);
        let phase = (current_block % period) / quantize_factor * quantize_factor;
        Era::Mortal { period, phase }
    }
}

/// Round `v` up to the next power of two, clamped to 65536.
fn next_power_of_two(v: u64) -> u64 {
    if v <= 1 {
        return 1;
    }
    let p = (v - 1).leading_zeros();
    let result = 1u64 << (64 - p);
    result.min(65536)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn immortal_encodes_as_single_zero() {
        assert_eq!(Era::Immortal.encode(), vec![0x00]);
    }

    #[test]
    fn immortal_roundtrip() {
        let (era, consumed) = Era::decode(&[0x00]).unwrap();
        assert_eq!(era, Era::Immortal);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn mortal_roundtrip() {
        // period=64, phase=32 — cross-checked against polkadot-js and sp_runtime::generic::Era.
        // Expected encoding: encoded_u16 = (6-1) | (32 << 4) = 5 | 512 = 517 = [0x05, 0x02]
        let era = Era::Mortal { period: 64, phase: 32 };
        let encoded = era.encode();
        assert_eq!(encoded, vec![0x05, 0x02], "encoding must match Substrate wire format");
        let (decoded, consumed) = Era::decode(&encoded).unwrap();
        assert_eq!(consumed, 2);
        if let Era::Mortal { period, phase } = decoded {
            assert_eq!(period, 64);
            assert_eq!(phase, 32);
        } else {
            panic!("expected Mortal");
        }
    }

    #[test]
    fn mortal_from_block_roundtrip() {
        let era = Era::mortal_from_block(100, 64);
        let encoded = era.encode();
        assert_eq!(encoded.len(), 2);
        let (decoded, _) = Era::decode(&encoded).unwrap();
        assert_eq!(era, decoded);
    }
}
