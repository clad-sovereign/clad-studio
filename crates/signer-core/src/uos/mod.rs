//! # `uos` — Universal Offline Signatures protocol
//!
//! A byte-for-byte Rust port of the UOS protocol currently implemented in
//! Kotlin inside `clad-mobile/shared/`.  Byte-level parity against the Kotlin
//! reference implementation is verified by the corpus tests under
//! `crates/signer-core/tests/`.
//!
//! ## Modules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`constants`] | Protocol magic bytes, crypto-type flags, frame sizes |
//! | [`error`] | [`UosError`] sealed enum |
//! | [`payload`] | [`UosPayload`] — unsigned transaction wrapper |
//! | [`signature`] | [`UosSignature`] — signed-response wrapper |
//! | [`multipart`] | [`MultiPartQrEncoder`] / [`MultiPartQrDecoder`] |
//! | [`account_introduction`] | [`AccountIntroduction`] URI codec |
//!
//! ## Status
//!
//! Phase 1 delivers the outer UOS wrapper only.  The inner SCALE-encoded
//! signing payload is treated as opaque bytes here; sr25519 signing primitives
//! and extrinsic construction land in Phase 2.  Neither mobile app consumes
//! this Rust path until Phase 3.
//!
//! See `clad-studio/docs/migration/01-phases.md` for phase entry/exit
//! criteria, and ADR-007 for the architectural rationale.

pub mod account_introduction;
pub mod constants;
pub mod error;
pub mod multipart;
pub mod payload;
pub mod signature;

pub use account_introduction::AccountIntroduction;
pub use error::UosError;
pub use multipart::{FrameDecodeProgress, MultiPartQrDecoder, MultiPartQrEncoder};
pub use payload::UosPayload;
pub use signature::UosSignature;
