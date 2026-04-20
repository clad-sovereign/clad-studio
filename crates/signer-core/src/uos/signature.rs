use crate::uos::constants::{
    CRYPTO_ECDSA, CRYPTO_ED25519, CRYPTO_SR25519, SIGNATURE_LENGTH_ECDSA,
    SIGNATURE_LENGTH_ED25519_SR25519,
};
use crate::uos::error::UosError;

/// Returns the expected signature length in bytes for a given crypto-type byte.
pub fn signature_length_for(crypto_type: u8) -> Result<usize, UosError> {
    match crypto_type {
        CRYPTO_ED25519 | CRYPTO_SR25519 => Ok(SIGNATURE_LENGTH_ED25519_SR25519),
        CRYPTO_ECDSA => Ok(SIGNATURE_LENGTH_ECDSA),
        _ => Err(UosError::UnknownCryptoType),
    }
}

/// UOS signature response from the signing device.
///
/// Binary layout:
/// ```text
/// ┌─────────────┬───────────────────┐
/// │ Crypto Type │ Signature         │
/// │ (1 byte)    │ (64 or 65 bytes)  │
/// └─────────────┴───────────────────┘
/// ```
///
/// Total: 65 bytes for Ed25519/Sr25519, 66 bytes for ECDSA.
#[derive(Debug, Clone, PartialEq)]
pub struct UosSignature {
    /// Cryptographic curve type (must match the corresponding request).
    pub crypto_type: u8,
    /// Raw signature bytes.
    pub signature: Vec<u8>,
}

impl UosSignature {
    /// Creates a new `UosSignature`, validating the signature length against
    /// the expected size for `crypto_type`.
    pub fn new(crypto_type: u8, signature: Vec<u8>) -> Result<Self, UosError> {
        let expected = signature_length_for(crypto_type)?;
        if signature.len() != expected {
            return Err(UosError::WrongSignatureLength);
        }
        Ok(Self { crypto_type, signature })
    }

    /// Encodes this signature to UOS binary format: `[crypto_type][signature…]`.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + self.signature.len());
        out.push(self.crypto_type);
        out.extend_from_slice(&self.signature);
        out
    }

    /// Decodes binary data into a `UosSignature`.
    pub fn decode(data: &[u8]) -> Result<Self, UosError> {
        if data.is_empty() {
            return Err(UosError::EmptyPayload);
        }
        let crypto_type = data[0];
        let expected = signature_length_for(crypto_type)?;
        let actual = data.len() - 1;
        if actual != expected {
            return Err(UosError::WrongSignatureLength);
        }
        Ok(Self { crypto_type, signature: data[1..].to_vec() })
    }

    /// Creates an Sr25519 signature.
    pub fn sr25519(signature: Vec<u8>) -> Result<Self, UosError> {
        Self::new(CRYPTO_SR25519, signature)
    }

    /// Creates an Ed25519 signature.
    pub fn ed25519(signature: Vec<u8>) -> Result<Self, UosError> {
        Self::new(CRYPTO_ED25519, signature)
    }

    /// Creates an ECDSA signature.
    pub fn ecdsa(signature: Vec<u8>) -> Result<Self, UosError> {
        Self::new(CRYPTO_ECDSA, signature)
    }
}
