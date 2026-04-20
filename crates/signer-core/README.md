# `crates/signer-core`

Shared protocol and crypto primitives for the Clad Sovereign signer, exposed
to Android (Kotlin) and iOS (Swift) via [UniFFI].

**Phase 1 status:** the `uos::*` module is now available, providing a
byte-for-byte Rust port of the UOS (Universal Offline Signatures) protocol
— the same protocol previously implemented only in Kotlin inside
`clad-mobile/shared/`.  Sr25519 signing primitives and SCALE extrinsic
construction are still pending (Phase 2).  Neither mobile app consumes the Rust
path yet — that wiring lands in Phase 3 behind a feature flag.  See:

- [ADR-007](../../docs/adr/007-rust-signer-core-via-uniffi.md) — why this crate exists.
- [docs/migration/01-phases.md](../../docs/migration/01-phases.md) — Phase 0 scope, Phase 1+ roadmap.
- [BUILD.md](./BUILD.md) — how to author UDL, regenerate bindings, and build `.aar` / `.xcframework`.

## Layout

```
crates/signer-core/
├── Cargo.toml
├── build.rs                 # UniFFI scaffolding generation
├── src/
│   ├── lib.rs               # ping() + UOS free functions
│   ├── signer_core.udl      # FFI interface definition (authoritative)
│   ├── bin/uniffi-bindgen.rs
│   └── uos/                 # Phase 1: Universal Offline Signatures
│       ├── mod.rs
│       ├── constants.rs
│       ├── error.rs
│       ├── payload.rs
│       ├── signature.rs
│       ├── multipart.rs
│       └── account_introduction.rs
├── tests/
│   ├── ping_test.rs
│   ├── uos_payload_corpus.rs
│   ├── uos_signature_corpus.rs
│   ├── uos_multipart_corpus.rs
│   ├── uos_account_introduction_corpus.rs
│   ├── uos_property_tests.rs
│   └── corpora/             # golden JSON vectors
│       ├── README.md
│       ├── payload/
│       ├── signature/
│       ├── multipart/
│       └── account_introduction/
├── build-ios.sh
├── build-android.sh
├── BUILD.md
├── android/
└── ios/
```

## Do not

- Add features to this crate that the UDL does not declare.
- Add new consumers (clad-mobile, clad-dashboard, etc.) before Phase 3.
  Consumer wiring starts in Phase 3 behind a feature flag.
- Introduce non-`no_std + alloc`-compatible dependencies without revising
  ADR-007. The Phase 2 NFC card firmware reuses this crate.

[UniFFI]: https://mozilla.github.io/uniffi-rs/
