/// Errors produced by UOS encode/decode operations.
///
/// Exposed over the FFI as a sealed error enum (UniFFI `[Error] enum`).
/// UniFFI 0.28's flat `[Error] enum` requires no associated data on variants;
/// contextual information is carried by the `Display` message and readable on
/// Kotlin/Swift via the exception's `.message` / `localizedDescription`.
///
/// Phase 3 may migrate to `[Error] interface` (richer per-variant fields) once
/// the UniFFI upgrade lands.
#[derive(Debug, thiserror::Error)]
pub enum UosError {
    /// The first byte was not `0x53` (ASCII `'S'`).
    #[error("invalid substrate ID: expected 0x53")]
    InvalidSubstrateId,

    /// The binary data is shorter than the minimum required size (35 bytes).
    #[error("payload too short: need at least 35 bytes")]
    PayloadTooShort,

    /// The account ID slice was not exactly 32 bytes.
    #[error("wrong account ID length: expected 32 bytes")]
    WrongAccountIdLength,

    /// The crypto-type byte is not one of 0x00 / 0x01 / 0x02.
    #[error("unknown crypto type")]
    UnknownCryptoType,

    /// The signature bytes do not match the expected length for the crypto type.
    #[error("wrong signature length for the given crypto type")]
    WrongSignatureLength,

    /// A multi-part frame header could not be parsed (e.g. count is zero,
    /// or the frame index is out of range).
    #[error("malformed frame header")]
    MalformedFrameHeader,

    /// A new frame arrived whose total-count disagrees with a previously seen
    /// value — indicates the user is scanning a different QR sequence.
    #[error("frame count mismatch")]
    FrameCountMismatch,

    /// An empty byte slice was passed where non-empty data is required.
    #[error("empty payload")]
    EmptyPayload,

    /// An account-introduction URI could not be parsed.
    #[error("invalid URI")]
    InvalidUri,
}
