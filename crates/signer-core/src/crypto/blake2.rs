//! Blake2b hashing functions.
//!
//! Mirrors `Hasher.kt` — `blake2b256` and `blake2b128` — used by Substrate for:
//! - Signing payload hashing when payload length ≥ 256 bytes
//! - Transaction hash computation (extrinsic hash)
//! - Storage key hashing (`Blake2_128Concat` hasher)
//!
//! Uses the [`blake2`] crate which supports `no_std`.

use alloc::vec::Vec;

use blake2::digest::consts::{U16, U32, U64};
use blake2::{Blake2b, Digest};

/// Compute a 32-byte Blake2b-256 hash of `data`.
///
/// Used by Substrate for signing payloads ≥ 256 bytes and as the extrinsic hash.
pub fn blake2b_256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute a 16-byte Blake2b-128 hash of `data`.
///
/// Used by Substrate for `Blake2_128Concat` storage key hashing.
pub fn blake2b_128(data: &[u8]) -> Vec<u8> {
    let mut hasher = Blake2b::<U16>::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute a 64-byte Blake2b-512 hash of `data`.
pub fn blake2b_512(data: &[u8]) -> Vec<u8> {
    let mut hasher = Blake2b::<U64>::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute `blake2b_128(data) || data` — the `Blake2_128Concat` storage hasher.
///
/// Used for storage maps where the key must be recoverable from the hash.
pub fn blake2b_128_concat(data: &[u8]) -> Vec<u8> {
    let mut result = blake2b_128(data);
    result.extend_from_slice(data);
    result
}
