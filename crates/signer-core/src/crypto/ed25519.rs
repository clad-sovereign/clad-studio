//! ED25519 signing and verification.
//!
//! Wraps [`ed25519_zebra`] which is `no_std + alloc` compatible.
//!
//! # Wire format
//! - Signing key (seed): 32 bytes (RFC 8032 private seed)
//! - Verification key (public key): 32 bytes
//! - Signature: 64 bytes

use alloc::vec::Vec;

use ed25519_zebra::{SigningKey, VerificationKey};

use super::CryptoError;

/// Sign `message` with the ED25519 signing key encoded as a 32-byte seed.
///
/// The signature is deterministic (RFC 8032 §5.1).
pub fn sign(message: &[u8], seed: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if seed.len() != 32 {
        return Err(CryptoError::InvalidSecretKey);
    }
    let seed_arr: [u8; 32] = seed.try_into().map_err(|_| CryptoError::InvalidSecretKey)?;
    let sk = SigningKey::from(seed_arr);
    let sig: ed25519_zebra::Signature = sk.sign(message);
    Ok(<[u8; 64]>::from(sig).to_vec())
}

/// Verify a 64-byte ED25519 `signature` over `message` against `public_key`.
pub fn verify(message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    let Ok(pk_arr): Result<[u8; 32], _> = public_key.try_into() else {
        return false;
    };
    let Ok(vk) = VerificationKey::try_from(pk_arr) else {
        return false;
    };
    let Ok(sig_arr): Result<[u8; 64], _> = signature.try_into() else {
        return false;
    };
    let sig = ed25519_zebra::Signature::from(sig_arr);
    vk.verify(&sig, message).is_ok()
}

/// Derive the 32-byte ED25519 public key (verification key) from a 32-byte seed.
pub fn public_key_from_seed(seed: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if seed.len() != 32 {
        return Err(CryptoError::InvalidSecretKey);
    }
    let seed_arr: [u8; 32] = seed.try_into().map_err(|_| CryptoError::InvalidSecretKey)?;
    let sk = SigningKey::from(seed_arr);
    let vk = VerificationKey::from(&sk);
    Ok(<[u8; 32]>::from(vk).to_vec())
}
