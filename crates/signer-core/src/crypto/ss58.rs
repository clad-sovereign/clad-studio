//! SS58 address encoding and decoding.
//!
//! Mirrors `Ss58.kt` — `encode(publicKey, networkPrefix)` / `decode(address)`.
//!
//! # SS58 wire format
//!
//! ```text
//! base58_encode( prefix_bytes || public_key(32) || checksum(2) )
//! ```
//!
//! where:
//! - `prefix_bytes` encodes the network prefix (u16) using the Substrate
//!   "canary" scheme:
//!   - prefix 0–63  → 1 byte: the prefix value itself
//!   - prefix 64–16383 → 2 bytes: the canary encoding described below
//! - `checksum` is the first 2 bytes of `blake2b_512("SS58PRE" || payload)`
//!
//! Substrate canary 2-byte prefix encoding (prefixes 64–16383):
//! ```text
//! raw = prefix
//! b0  = ((raw & 0xFC) >> 2) | 0x40   -- bits 7..2 of raw (shifted), OR'd with 0x40
//! b1  = (raw >> 8) | ((raw & 0x03) << 6)  -- high bits + low 2 bits
//! bytes = [b0, b1]
//! ```
//!
//! References:
//! - https://docs.substrate.io/reference/address-formats/
//! - https://github.com/paritytech/ss58-registry
//! - Substrate `sp_core::crypto::Ss58Codec` (polkadot-sdk)

use alloc::{string::String, vec::Vec};

use super::CryptoError;
use crate::crypto::blake2::blake2b_512;

/// The SS58 checksum prefix string.
const SS58_PREFIX: &[u8] = b"SS58PRE";

// ── Encoding ──────────────────────────────────────────────────────────────────

/// Encode a 32-byte `public_key` as an SS58 address with the given `prefix`.
///
/// `prefix` must be in 0–16383.  Use 42 for the generic Substrate network
/// (matching `NetworkPrefix.GENERIC_SUBSTRATE` and `NetworkPrefix.CLAD` in
/// `Ss58.kt` and `NetworkPrefix.kt`).
pub fn encode(public_key: &[u8], prefix: u16) -> Result<String, CryptoError> {
    if public_key.len() != 32 {
        return Err(CryptoError::InvalidPublicKey);
    }
    if prefix > 16383 {
        return Err(CryptoError::InvalidPrefix);
    }

    let prefix_bytes = encode_prefix(prefix);
    let mut payload = prefix_bytes;
    payload.extend_from_slice(public_key);

    let checksum = ss58_checksum(&payload);
    payload.extend_from_slice(&checksum[..2]);

    Ok(bs58::encode(payload).into_string())
}

/// Decode an SS58 `address` into `(public_key_32_bytes, prefix)`.
pub fn decode(address: &str) -> Result<(Vec<u8>, u16), CryptoError> {
    let raw = bs58::decode(address).into_vec().map_err(|_| CryptoError::InvalidAddress)?;

    if raw.len() < 35 {
        return Err(CryptoError::InvalidAddress);
    }

    let (prefix, offset) = decode_prefix(&raw)?;

    // payload without checksum
    let payload_end = raw.len() - 2;
    if payload_end < offset {
        return Err(CryptoError::InvalidAddress);
    }
    let public_key = raw[offset..payload_end].to_vec();
    if public_key.len() != 32 {
        return Err(CryptoError::InvalidAddress);
    }

    // verify checksum
    let expected_checksum = ss58_checksum(&raw[..payload_end]);
    if raw[payload_end..] != expected_checksum[..2] {
        return Err(CryptoError::InvalidAddress);
    }

    Ok((public_key, prefix))
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn encode_prefix(prefix: u16) -> Vec<u8> {
    if prefix < 64 {
        vec![prefix as u8]
    } else {
        // Canary 2-byte encoding for prefixes 64–16383.
        let b0 = (((prefix & 0xFC) >> 2) as u8) | 0x40;
        let b1 = ((prefix >> 8) as u8) | (((prefix & 0x03) << 6) as u8);
        vec![b0, b1]
    }
}

fn decode_prefix(raw: &[u8]) -> Result<(u16, usize), CryptoError> {
    if raw.is_empty() {
        return Err(CryptoError::InvalidAddress);
    }
    let b0 = raw[0];
    if b0 < 64 {
        Ok((b0 as u16, 1))
    } else if b0 < 128 {
        if raw.len() < 2 {
            return Err(CryptoError::InvalidAddress);
        }
        let b1 = raw[1];
        // Reverse of encode_prefix canary encoding:
        // b0 = ((raw & 0xFC) >> 2) | 0x40  →  raw_low = (b0 & 0x3F) << 2
        // b1 = (raw >> 8) | ((raw & 0x03) << 6)  →  raw_high = b1 & 0x3F, raw_bits01 = b1 >> 6
        let full = (((b0 & 0x3F) as u16) << 2) | ((b1 as u16) >> 6) | ((b1 as u16 & 0x3F) << 8);
        Ok((full, 2))
    } else {
        Err(CryptoError::InvalidPrefix)
    }
}

fn ss58_checksum(payload: &[u8]) -> Vec<u8> {
    let mut data = Vec::with_capacity(SS58_PREFIX.len() + payload.len());
    data.extend_from_slice(SS58_PREFIX);
    data.extend_from_slice(payload);
    blake2b_512(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Alice's well-known seed and SS58 address (generic Substrate prefix 42).
    ///
    /// Seed:    0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
    /// SS58:    5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
    ///
    /// Cross-checked against polkadot-js/apps and Substrate's own test suite.
    #[test]
    fn alice_ss58_roundtrip() {
        let pubkey =
            hex::decode("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
                .unwrap();
        let addr = encode(&pubkey, 42).unwrap();
        assert_eq!(addr, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");

        let (decoded_pk, decoded_prefix) = decode(&addr).unwrap();
        assert_eq!(decoded_pk, pubkey);
        assert_eq!(decoded_prefix, 42);
    }
}
