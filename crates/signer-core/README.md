# `crates/signer-core`

Shared protocol and crypto primitives for the Clad Sovereign signer, exposed
to Android (Kotlin) and iOS (Swift) via [UniFFI].

**Phase 0 status:** the public surface is exactly one function, `ping() -> String`.
This crate exists right now to prove the FFI cross-compile pipeline end-to-end
before any correctness-sensitive code is migrated. See:

- [ADR-007](../../docs/adr/007-rust-signer-core-via-uniffi.md) — why this crate exists.
- [docs/migration/01-phases.md](../../docs/migration/01-phases.md) — Phase 0 scope, Phase 1+ roadmap.
- [BUILD.md](./BUILD.md) — how to author UDL, regenerate bindings, and build `.aar` / `.xcframework`.

## Layout

```
crates/signer-core/
├── Cargo.toml
├── build.rs                 # UniFFI scaffolding generation
├── uniffi.toml              # package/module names for the generated bindings
├── src/
│   ├── lib.rs               # ping() implementation
│   ├── signer_core.udl      # FFI interface definition (authoritative)
│   └── bin/uniffi-bindgen.rs  # bindgen CLI wrapper
├── tests/
│   └── ping_test.rs         # Rust-side liveness check
├── build-ios.sh             # produces build/ios/SignerCore.xcframework
├── build-android.sh         # produces build/android-host/ + build/android/*.aar
├── BUILD.md                 # human-facing build + FFI guide
├── android/                 # Phase 0 Gradle JVM sample (throwaway)
└── ios/                     # Phase 0 SwiftPM sample (throwaway)
```

## Do not

- Add features to this crate that the UDL does not declare.
- Add new consumers (clad-mobile, clad-dashboard, etc.) in Phase 0. Consumer
  wiring starts in Phase 3, per the migration plan.
- Introduce non-`no_std + alloc`-compatible dependencies without revising
  ADR-007. The Phase 2 NFC card firmware reuses this crate.

[UniFFI]: https://mozilla.github.io/uniffi-rs/
