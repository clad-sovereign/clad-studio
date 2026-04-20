use crate::uos::constants::{
    ACCOUNT_ID_LENGTH, CMD_SIGN_IMMORTAL, CMD_SIGN_MSG, CMD_SIGN_TX, CRYPTO_SR25519,
    MIN_PAYLOAD_SIZE, SUBSTRATE_ID,
};
use crate::uos::error::UosError;

/// UOS (Universal Offline Signatures) payload for unsigned transactions.
///
/// Binary layout:
/// ```text
/// ┌────────┬─────────────┬────────┬──────────────┬─────────────┐
/// │ Byte 0 │ Byte 1      │ Byte 2 │ Bytes 3–34   │ Bytes 35+   │
/// ├────────┼─────────────┼────────┼──────────────┼─────────────┤
/// │ 0x53   │ Crypto Type │ Action │ Account ID   │ Payload     │
/// │ ('S')  │             │        │ (32 bytes)   │ (variable)  │
/// └────────┴─────────────┴────────┴──────────────┴─────────────┘
/// ```
///
/// Compatible with Polkadot Vault, polkadot-js, Nova Wallet, and Subwallet.
///
/// `account_id` must always be exactly 32 bytes; this is validated in
/// `encode()` and `decode()`.
#[derive(Debug, Clone, PartialEq)]
pub struct UosPayload {
    /// Cryptographic curve type (see `constants::CRYPTO_*`).
    pub crypto_type: u8,
    /// Action to perform (see `constants::CMD_*`).
    pub action: u8,
    /// 32-byte raw public key of the signing account (`Vec<u8>` for FFI compat;
    /// length validated at encode/decode boundaries).
    pub account_id: Vec<u8>,
    /// Variable-length signing payload (opaque bytes; SCALE-encoded extrinsic
    /// data in the sr25519 signing phases, arbitrary message bytes for
    /// `CMD_SIGN_MSG`).
    pub payload: Vec<u8>,
}

impl UosPayload {
    /// Creates a new `UosPayload`.
    pub fn new(crypto_type: u8, action: u8, account_id: Vec<u8>, payload: Vec<u8>) -> Self {
        Self { crypto_type, action, account_id, payload }
    }

    /// Encodes this payload to UOS binary format.
    ///
    /// Returns `Err(WrongAccountIdLength)` if `account_id` is not 32 bytes.
    pub fn encode(&self) -> Result<Vec<u8>, UosError> {
        if self.account_id.len() != ACCOUNT_ID_LENGTH {
            return Err(UosError::WrongAccountIdLength);
        }
        let mut out = Vec::with_capacity(MIN_PAYLOAD_SIZE + self.payload.len());
        out.push(SUBSTRATE_ID);
        out.push(self.crypto_type);
        out.push(self.action);
        out.extend_from_slice(&self.account_id);
        out.extend_from_slice(&self.payload);
        Ok(out)
    }

    /// Decodes binary data into a `UosPayload`.
    ///
    /// Returns `Err` if the data is too short or the substrate ID is wrong.
    pub fn decode(data: &[u8]) -> Result<Self, UosError> {
        if data.len() < MIN_PAYLOAD_SIZE {
            return Err(UosError::PayloadTooShort);
        }
        if data[0] != SUBSTRATE_ID {
            return Err(UosError::InvalidSubstrateId);
        }
        let crypto_type = data[1];
        let action = data[2];
        let account_id = data[3..35].to_vec();
        let payload = if data.len() > 35 { data[35..].to_vec() } else { Vec::new() };
        Ok(Self { crypto_type, action, account_id, payload })
    }

    /// Creates a payload for signing a mortal transaction with Sr25519.
    pub fn for_sign_tx(account_id: Vec<u8>, signing_payload: Vec<u8>) -> Self {
        Self::new(CRYPTO_SR25519, CMD_SIGN_TX, account_id, signing_payload)
    }

    /// Creates a payload for signing an immortal transaction with Sr25519.
    pub fn for_sign_immortal(account_id: Vec<u8>, signing_payload: Vec<u8>) -> Self {
        Self::new(CRYPTO_SR25519, CMD_SIGN_IMMORTAL, account_id, signing_payload)
    }

    /// Creates a payload for signing an arbitrary message with Sr25519.
    pub fn for_sign_message(account_id: Vec<u8>, message: Vec<u8>) -> Self {
        Self::new(CRYPTO_SR25519, CMD_SIGN_MSG, account_id, message)
    }
}
