//! Substrate extrinsic construction.
//!
//! Mirrors the Kotlin oracle files in
//! `clad-mobile/shared/src/commonMain/kotlin/tech/wideas/clad/substrate/extrinsic/`
//! and produces bit-identical SCALE-encoded call data and signed extrinsics.
//!
//! All modules are `no_std + alloc` compatible.
//!
//! # Module layout
//!
//! - [`era`]              — `Era::Immortal` / `Era::Mortal` SCALE encoding
//! - [`call`]             — `CladTokenCalls` + `MultisigCalls` call builders
//! - [`signed_extensions`] — Extra (era + nonce + tip) and Additional fields
//! - [`payload`]          — `build_signing_payload` + ≥ 256-byte Blake2b rule
//! - [`signed`]           — `build_signed_extrinsic`, `complete_with_signature`
//! - [`metadata`]         — hand-rolled call encoding; subxt-core deferred

pub mod call;
pub mod era;
pub mod metadata;
pub mod payload;
pub mod signed;
pub mod signed_extensions;

// Re-export the primary user-facing types.
pub use call::CallData;
pub use era::Era;
pub use signed::SignedExtrinsic;
pub use signed_extensions::{ChainInfo, SignedExtra};
