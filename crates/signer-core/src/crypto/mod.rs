//! Cryptographic primitives for the Clad Sovereign signer.
//!
//! All modules in this tree are `no_std + alloc` compatible and mirror the
//! Kotlin oracle implementations in `clad-mobile/shared/`:
//! - [`sr25519`] ← `crypto/Signer.kt` (SR25519 path)
//! - [`ed25519`] ← `crypto/Signer.kt` (ED25519 path)
//! - [`blake2`]  ← `crypto/Hasher.kt`
//! - [`ss58`]    ← `crypto/Ss58.kt`
//!
//! `CryptoError` is a flat error enum exposed through the UniFFI boundary.

pub mod blake2;
pub mod ed25519;
pub mod sr25519;
pub mod ss58;

use thiserror::Error;

/// Errors produced by cryptographic operations exposed at the FFI boundary.
///
/// Flat enum — no nested `Result<Result<…>>` shapes per ADR-007.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CryptoError {
    #[error("invalid secret key bytes")]
    InvalidSecretKey,
    #[error("invalid public key bytes")]
    InvalidPublicKey,
    #[error("invalid signature bytes")]
    InvalidSignature,
    #[error("invalid SS58 address")]
    InvalidAddress,
    #[error("invalid SS58 prefix")]
    InvalidPrefix,
    #[error("invalid or unsupported metadata")]
    InvalidMetadata,
    #[error("unknown pallet")]
    UnknownPallet,
    #[error("unknown call")]
    UnknownCall,
}
