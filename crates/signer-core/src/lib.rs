//! # signer-core
//!
//! Shared protocol and crypto primitives for the Clad Sovereign signer.
//!
//! **Phase 2 status:** the `crypto` and `extrinsic` modules are now available
//! and provide SR25519/ED25519 signing, Blake2b hashing, SS58 encoding, and
//! SCALE-encoded extrinsic construction.  The `uos` module (Phase 1) is
//! unchanged.
//!
//! See ADR-007 (`docs/adr/007-rust-signer-core-via-uniffi.md`) for the
//! architectural motivation.
//!
//! # no_std note
//!
//! The `crypto` and `extrinsic` modules are written using `alloc::` types
//! exclusively and are no_std-compatible in isolation (ADR-007 Phase-2 NFC
//! requirement).  The crate itself links std because UniFFI scaffolding
//! generates `std`-using code; a std-free firmware build would exclude the
//! UniFFI surface and depend on `signer-core` as a library crate directly.

// `alloc` is explicitly declared so the `crypto` and `extrinsic` modules can
// use `alloc::` paths, ensuring they stay compatible with no_std targets
// (e.g., future NFC firmware builds that won't include UniFFI).
extern crate alloc;

pub mod crypto;
pub mod extrinsic;
pub mod uos;

pub use crypto::CryptoError;
pub use crypto::{blake2, ed25519, sr25519, ss58};
pub use extrinsic::{CallData, ChainInfo, Era, SignedExtra, SignedExtrinsic};
pub use uos::account_introduction::AccountIntroduction;
pub use uos::error::UosError;
pub use uos::multipart::{FrameDecodeProgress, MultiPartQrDecoder, MultiPartQrEncoder};
pub use uos::payload::UosPayload;
pub use uos::signature::UosSignature;

uniffi::include_scaffolding!("signer_core");

// ── Phase 0 liveness check ───────────────────────────────────────────────────

/// Returns a fixed greeting string.
///
/// Phase 0 liveness check: exercised by the Rust unit test in
/// `tests/ping_test.rs`, the Kotlin JVM sample in
/// `android/sample/src/test/kotlin/PingTest.kt`, and the Swift sample in
/// `ios/Tests/SignerCoreTests/PingTests.swift`.
pub fn ping() -> String {
    "pong from signer-core".to_string()
}

// ── UOS namespace-level free functions ───────────────────────────────────────
// These are the entry points declared in `signer_core.udl`.  They bridge the
// UniFFI `dictionary` types (which use `Vec<u8>` for all byte fields) and the
// internal UOS types.

/// Encodes a [`UosPayload`] to its binary UOS representation.
pub fn encode_payload(payload: UosPayload) -> Result<Vec<u8>, UosError> {
    payload.encode()
}

/// Decodes a binary UOS payload into a [`UosPayload`].
pub fn decode_payload(data: Vec<u8>) -> Result<UosPayload, UosError> {
    UosPayload::decode(&data)
}

/// Encodes a [`UosSignature`] to its binary UOS representation.
pub fn encode_signature(sig: UosSignature) -> Result<Vec<u8>, UosError> {
    Ok(sig.encode())
}

/// Decodes a binary UOS signature into a [`UosSignature`].
pub fn decode_signature(data: Vec<u8>) -> Result<UosSignature, UosError> {
    UosSignature::decode(&data)
}

/// Serialises an [`AccountIntroduction`] to a `substrate:…` URI.
pub fn account_intro_to_uri(account: AccountIntroduction) -> String {
    account.to_uri()
}

/// Parses an [`AccountIntroduction`] from a `substrate:…` URI.
pub fn account_intro_from_uri(uri: String) -> Result<AccountIntroduction, UosError> {
    AccountIntroduction::from_uri(&uri)
}

// ── Phase 2: Crypto free functions ───────────────────────────────────────────

/// Encode a 32-byte public key as an SS58 address with the given prefix.
pub fn ss58_encode(public_key: Vec<u8>, prefix: u16) -> Result<String, CryptoError> {
    ss58::encode(&public_key, prefix)
}

/// Decode an SS58 address, returning the 32-byte public key.
pub fn ss58_decode(address: String) -> Result<Vec<u8>, CryptoError> {
    let (pk, _prefix) = ss58::decode(&address)?;
    Ok(pk)
}

/// Sign a payload with SR25519 using the provided 32-byte seed.
pub fn sr25519_sign(signing_payload: Vec<u8>, secret_key: Vec<u8>) -> Result<Vec<u8>, CryptoError> {
    sr25519::sign(&signing_payload, &secret_key)
}

/// Verify an SR25519 signature.
pub fn sr25519_verify(message: Vec<u8>, signature: Vec<u8>, public_key: Vec<u8>) -> bool {
    sr25519::verify(&message, &signature, &public_key)
}

/// Sign a payload with ED25519 using the provided 32-byte seed.
pub fn ed25519_sign(signing_payload: Vec<u8>, secret_key: Vec<u8>) -> Result<Vec<u8>, CryptoError> {
    ed25519::sign(&signing_payload, &secret_key)
}

/// Verify an ED25519 signature.
pub fn ed25519_verify(message: Vec<u8>, signature: Vec<u8>, public_key: Vec<u8>) -> bool {
    ed25519::verify(&message, &signature, &public_key)
}

/// Compute a 32-byte Blake2b-256 hash of `data`.
pub fn blake2b_256(data: Vec<u8>) -> Vec<u8> {
    blake2::blake2b_256(&data)
}

// ── Phase 2: Extrinsic free functions ────────────────────────────────────────

/// Build SCALE-encoded call data by pallet + call name with pre-encoded args.
pub fn build_call_data(
    pallet_name: String,
    call_name: String,
    args: Vec<Vec<u8>>,
) -> Result<Vec<u8>, CryptoError> {
    extrinsic::metadata::build_call_data(&pallet_name, &call_name, &args)
}

/// Build the signing payload for a call (applies Blake2b-256 if ≥ 256 bytes).
pub fn build_signing_payload(call_data: Vec<u8>, extra: SignedExtra, chain: ChainInfo) -> Vec<u8> {
    extrinsic::payload::build_signing_payload(&call_data, &extra, &chain)
}

/// Build a signed extrinsic from call data, public key, and signature.
pub fn build_signed_extrinsic(
    call_data: Vec<u8>,
    signer_public_key: Vec<u8>,
    signature: Vec<u8>,
    extra: SignedExtra,
) -> SignedExtrinsic {
    extrinsic::signed::build_signed_extrinsic(&call_data, &signer_public_key, &signature, &extra)
}

/// Complete an unsigned extrinsic payload with an externally produced signature.
pub fn complete_with_signature(
    call_data: Vec<u8>,
    signer_public_key: Vec<u8>,
    signature: Vec<u8>,
    extra: SignedExtra,
) -> SignedExtrinsic {
    extrinsic::signed::complete_with_signature(&call_data, &signer_public_key, &signature, &extra)
}

/// Compute the Blake2b-256 hash of call data (used as multisig call_hash).
pub fn compute_call_hash(call_data: Vec<u8>) -> Vec<u8> {
    blake2::blake2b_256(&call_data)
}
