//! Minimal hand-rolled SCALE compact encoding helpers.
//!
//! SCALE compact encoding (little-endian, mode bits in the lowest 2 bits):
//!
//! | Mode bits | Value range           | Encoding        |
//! |-----------|----------------------|-----------------|
//! | `00`      | 0–63                 | `(v << 2) as u8` (1 byte) |
//! | `01`      | 64–16383             | `(v << 2 | 1) as u16 LE` (2 bytes) |
//! | `10`      | 16384–1073741823     | `(v << 2 | 2) as u32 LE` (4 bytes) |
//! | `11`      | 1073741824–bignum    | byte-length prefix + big-endian bytes |
//!
//! References:
//! - https://docs.substrate.io/reference/scale-codec/
//! - `parity-scale-codec` crate source (for reference only; we don't use it here
//!   to avoid pulling a dependency into this narrow helper)

use alloc::vec::Vec;

/// SCALE Compact<u64> encoding.
pub fn compact_u64(v: u64) -> Vec<u8> {
    compact_u128(v as u128)
}

/// SCALE Compact<u128> encoding.
pub fn compact_u128(v: u128) -> Vec<u8> {
    if v < 64 {
        vec![(v as u8) << 2]
    } else if v < 16384 {
        let encoded = ((v as u16) << 2) | 0b01;
        encoded.to_le_bytes().to_vec()
    } else if v < 1_073_741_824 {
        let encoded = ((v as u32) << 2) | 0b10;
        encoded.to_le_bytes().to_vec()
    } else {
        // Big-integer mode: find the minimum number of bytes needed.
        let mut bytes = Vec::new();
        let mut n = v;
        while n > 0 {
            bytes.push((n & 0xFF) as u8);
            n >>= 8;
        }
        let len_byte = (((bytes.len() - 4) as u8) << 2) | 0b11;
        let mut out = vec![len_byte];
        out.extend_from_slice(&bytes);
        out
    }
}

/// SCALE Compact<usize> (same encoding, used for Vec length prefixes).
pub fn compact_usize(v: usize) -> Vec<u8> {
    compact_u128(v as u128)
}
