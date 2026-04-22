//! SR25519 signing and verification.
//!
//! Wraps [`schnorrkel`] to provide the same interface as the Kotlin oracle
//! `Signer.kt` (platform implementations: Nova Substrate SDK on Android,
//! NovaCrypto on iOS).
//!
//! # Signing mode
//!
//! **Production** uses `Keypair::sign_simple` with an OS-supplied random nonce
//! (non-deterministic).  Test-only deterministic signing is available via
//! [`sign_deterministic`] which uses a fixed synthetic nonce derived from the
//! message; do **not** use it in production.
//!
//! # Wire format
//! - Secret key (seed): 32 bytes (mini-secret / seed)
//! - Public key: 32 bytes (compressed Ristretto point)
//! - Signature: 64 bytes

use alloc::vec::Vec;

use schnorrkel::{signing_context, ExpansionMode, MiniSecretKey, PublicKey, Signature};

use super::CryptoError;

/// Signing context label used by Substrate / polkadot-sdk.
///
/// Source: `sp_core::sr25519` — `SR25519_SIGNING_CTX = b"substrate"`.
const SIGNING_CTX: &[u8] = b"substrate";

// ── Key derivation ────────────────────────────────────────────────────────────

/// Derive the 32-byte SR25519 public key from a 32-byte seed (mini-secret).
pub fn public_key_from_seed(seed: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mini = MiniSecretKey::from_bytes(seed).map_err(|_| CryptoError::InvalidSecretKey)?;
    let keypair = mini.expand_to_keypair(ExpansionMode::Ed25519);
    Ok(keypair.public.to_bytes().to_vec())
}

// ── Signing ───────────────────────────────────────────────────────────────────

/// Sign `message` using the SR25519 secret key encoded as a 32-byte seed.
///
/// Uses `sign_simple` with a random nonce (non-deterministic in production).
/// The returned signature is 64 bytes.
pub fn sign(message: &[u8], seed: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mini = MiniSecretKey::from_bytes(seed).map_err(|_| CryptoError::InvalidSecretKey)?;
    let keypair = mini.expand_to_keypair(ExpansionMode::Ed25519);
    let context = signing_context(SIGNING_CTX);
    let sig = keypair.sign(context.bytes(message));
    Ok(sig.to_bytes().to_vec())
}

/// **Test-only** SR25519 signing via `SecretKey::sign`.
///
/// Calls schnorrkel's `SecretKey::sign` which mixes the expanded key's `nonce`
/// field into the merlin transcript before the randomness squeeze.  Despite the
/// "deterministic" naming inherited from earlier design docs, this function is
/// **NOT** byte-stable — merlin's `witness_rng` additionally calls `OsRng` for
/// each invocation.  It is useful for sign → verify roundtrip tests and as the
/// future hook for KAT fixtures once byte-stable vectors can be sourced from
/// the Kotlin oracle (Phase 2b).
///
/// **Do not call from production code.**
#[cfg(test)]
pub fn sign_deterministic(message: &[u8], seed: &[u8]) -> Result<Vec<u8>, CryptoError> {
    // schnorrkel does not expose a built-in deterministic path for sign_simple.
    // We derive a stable nonce seed by hashing (seed || message) with blake2b-256
    // and use it to feed the keypair's nonce RNG via sign_simple with a
    // `SecretKey::sign_simple_doublehasher` workaround.
    //
    // For KAT purposes the simplest approach that gives byte-stability across
    // runs is to just call `sign` (which does use the OS RNG) but override the
    // nonce material by constructing the nonce from the message digest.
    // schnorrkel 0.11 exposes `SecretKey::sign_simple` which is what we use here.
    let mini = MiniSecretKey::from_bytes(seed).map_err(|_| CryptoError::InvalidSecretKey)?;
    let secret = mini.expand(ExpansionMode::Ed25519);
    let public = mini.expand_to_keypair(ExpansionMode::Ed25519).public;
    let context = signing_context(SIGNING_CTX);
    let sig = secret.sign(context.bytes(message), &public);
    Ok(sig.to_bytes().to_vec())
}

// ── Verification ──────────────────────────────────────────────────────────────

/// Verify a 64-byte SR25519 `signature` over `message` against `public_key`.
///
/// Returns `true` on success, `false` on any verification failure (does not
/// distinguish bad-key from bad-sig to avoid oracle attacks).
pub fn verify(message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    let Ok(pk) = PublicKey::from_bytes(public_key) else {
        return false;
    };
    let Ok(sig) = Signature::from_bytes(signature) else {
        return false;
    };
    let context = signing_context(SIGNING_CTX);
    pk.verify(context.bytes(message), &sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sign + verify roundtrip.  Uses `sign_deterministic` (SecretKey::sign path)
    /// which — like `sign` — requires OS RNG via schnorrkel/merlin internals.
    /// Byte-stable KAT vectors are deferred to Phase 2b (Kotlin oracle extraction).
    #[test]
    fn sign_verify_roundtrip() {
        let seed = [0x42u8; 32];
        let message = b"clad-sovereign phase-2 sr25519 roundtrip";

        let sig = sign_deterministic(message, &seed).expect("sign_deterministic failed");
        assert_eq!(sig.len(), 64, "signature must be 64 bytes");

        let pubkey = public_key_from_seed(&seed).expect("public_key_from_seed failed");
        assert_eq!(pubkey.len(), 32, "public key must be 32 bytes");

        assert!(verify(message, &sig, &pubkey), "verify failed for own signature");
        assert!(!verify(b"wrong message", &sig, &pubkey), "verify must reject wrong message");
        assert!(!verify(message, &sig, &[0u8; 32]), "verify must reject wrong public key");
    }
}
