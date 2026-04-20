/// Protocol identifier — ASCII `'S'` for Substrate.
/// All UOS payloads must start with this byte.
pub const SUBSTRATE_ID: u8 = 0x53;

/// Edwards curve (EdDSA).
pub const CRYPTO_ED25519: u8 = 0x00;

/// Schnorrkel/Ristretto — default for Polkadot/Substrate.
pub const CRYPTO_SR25519: u8 = 0x01;

/// secp256k1 — Ethereum-compatible.
pub const CRYPTO_ECDSA: u8 = 0x02;

/// Sign a mortal (time-limited) transaction.
pub const CMD_SIGN_TX: u8 = 0x00;

/// Sign a raw hash (used when payload is too large to encode directly).
pub const CMD_SIGN_HASH: u8 = 0x01;

/// Sign an immortal (no-expiry) transaction.
pub const CMD_SIGN_IMMORTAL: u8 = 0x02;

/// Sign an arbitrary message.
pub const CMD_SIGN_MSG: u8 = 0x03;

/// Maximum bytes per QR frame.
pub const FRAME_SIZE: usize = 1024;

/// Account ID (public key) length in bytes.
pub const ACCOUNT_ID_LENGTH: usize = 32;

/// Signature length for Ed25519 and Sr25519.
pub const SIGNATURE_LENGTH_ED25519_SR25519: usize = 64;

/// Signature length for ECDSA (includes recovery byte).
pub const SIGNATURE_LENGTH_ECDSA: usize = 65;

/// Multi-part frame header size: 2 bytes index + 2 bytes count (big-endian).
pub const FRAME_HEADER_SIZE: usize = 4;

/// Minimum valid payload size: substrate ID (1) + crypto type (1) + action (1)
/// + account ID (32) = 35 bytes.
pub const MIN_PAYLOAD_SIZE: usize = 35;
