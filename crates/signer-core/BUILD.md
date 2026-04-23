# signer-core — Build and FFI Guide

This document covers: how to author UDL, how to regenerate bindings, and how
to produce `.aar` (Android) and `.xcframework` (iOS) artifacts both locally
and in CI. Required reading before editing anything under `crates/signer-core/`.

Related documents:

- [ADR-007](../../docs/adr/007-rust-signer-core-via-uniffi.md) — architectural
  motivation for the UniFFI boundary.
- [docs/migration/01-phases.md](../../docs/migration/01-phases.md) — phase
  entry/exit criteria; Phase 0 established this pipeline.
- [UniFFI user guide](https://mozilla.github.io/uniffi-rs/) — canonical upstream docs.

---

## Toolchain versions (pinned)

| Tool | Version | Where pinned |
|------|---------|--------------|
| Rust | 1.88.0 | `clad-studio/rust-toolchain.toml` |
| UniFFI | 0.29.5 | `crates/signer-core/Cargo.toml` |
| Xcode | 16.4 | `.github/workflows/signer-core.yml` (macos-15 runner) |
| Gradle | 8.14.3 | `crates/signer-core/android/gradle/wrapper/gradle-wrapper.properties` |
| Android Gradle Plugin | 8.7.0 | `crates/signer-core/android/settings.gradle.kts` |
| Android SDK | compileSdk=35, minSdk=26, targetSdk=35 | `crates/signer-core/android/sample/build.gradle.kts` |
| JDK | 17 | `crates/signer-core/android/sample/build.gradle.kts` (kotlin jvmToolchain) |
| Kotlin | 2.0.21 | `crates/signer-core/android/settings.gradle.kts` |
| JNA | 5.14.0 | `crates/signer-core/android/sample/build.gradle.kts` |
| cargo-ndk | 3.5.x (CI only) | `.github/workflows/signer-core.yml` |
| Android NDK | r26d (26.3.11579264) | `.github/workflows/signer-core.yml` |

To run the Android instrumented test locally you need cargo-ndk and the
Android NDK — `build-android.sh` cross-compiles the per-ABI `.so` files into
`build/aar-stage/jni/`, which Gradle picks up via `jniLibs.srcDirs`. Boot an
emulator (API 30 x86_64) and run `./gradlew :sample:connectedAndroidTest`.

---

## Authoring the FFI surface

The authoritative FFI interface lives in [`src/signer_core.udl`](./src/signer_core.udl).
Any change to the FFI must flow:

1. Edit `src/signer_core.udl` to declare the new type/function.
2. Implement the matching Rust in `src/lib.rs` (signatures must agree).
3. `cargo build -p signer-core --locked` — the `build.rs` regenerates
   scaffolding and the compiler verifies Rust matches UDL.
4. Regenerate Swift/Kotlin bindings:
    - `./build-ios.sh` — produces xcframework + Swift source.
    - `./build-android.sh` — produces host dylib + Kotlin source (+ AAR if NDK available).
5. Run the three tests: `cargo test -p signer-core`, the iOS `xcodebuild
   test`, and the Android `./gradlew :sample:test` from `android/`.

Rust-side logic never imports UniFFI type conversions manually; the
`uniffi::include_scaffolding!` macro in `lib.rs` wires that up.

---

## Build locally

### Prerequisites (macOS dev machine)

```
rustup show                      # 1.88.0 auto-installed via rust-toolchain.toml
rustup target add aarch64-apple-ios aarch64-apple-ios-sim \
                  aarch64-apple-darwin
xcodebuild -version              # Xcode 16.4
java --version                   # OpenJDK 17+ (23 also works)
```

No separate `uniffi-bindgen` install: it is built on demand as a binary in
this crate (see `src/bin/uniffi-bindgen.rs`).

### Rust unit tests (fastest loop)

```
cd clad-studio
cargo test -p signer-core --locked
```

### iOS: build xcframework and run test

```
cd clad-studio/crates/signer-core
./build-ios.sh
cd ios
xcodebuild test \
  -scheme SignerCoreSample \
  -destination 'platform=iOS Simulator,name=iPhone 16 Pro'
```

Expected result: `Test Suite 'All tests' passed` + `** TEST SUCCEEDED **`.

### Android: build per-ABI .so files and run instrumented test

`build-android.sh` cross-compiles all four Android ABIs and stages the `.so`
files in `build/aar-stage/jni/<abi>/`. Gradle's `jniLibs.srcDirs` wires that
directory into the APK so the instrumented test APK on the emulator can load
`libsigner_core.so` via JNA.

**Prerequisites**: Android NDK r26d installed, `cargo-ndk` on `$PATH`, and a
running API-30 x86_64 emulator (AVD or hardware device with USB debugging).

```
cd clad-studio/crates/signer-core
export ANDROID_NDK_HOME=/path/to/ndk/26.3.11579264
./build-android.sh                                  # cross-compile all ABIs

cd android
# Optionally verify the test APK builds before starting the emulator:
./gradlew :sample:assembleDebugAndroidTest --no-daemon

# Then boot an emulator and run the instrumented tests:
./gradlew :sample:connectedAndroidTest --no-daemon
```

Expected result:

```
PingInstrumentedTest > pingReturnsExpectedGreeting PASSED
PingInstrumentedTest > blake2b256ProducesThirtyTwoBytesForFixedInput PASSED
PingInstrumentedTest > blake2b256IsDeterministic PASSED
PingInstrumentedTest > blake2b256DifferentInputsProduceDifferentDigests PASSED
BUILD SUCCESSFUL
```

The instrumented tests run inside the emulator's ART JVM against the real
per-ABI `.so`, confirming the full Rust → UniFFI → Kotlin → JNA → `libsigner_core.so`
stack works on Android — not just on the dev host.

### Android: full cross-compile

Required for instrumented tests and AAR production. `build-android.sh` also
accepts `ANDROID_NDK_HOME` to enable cross-compilation:

```
export ANDROID_NDK_HOME=/path/to/ndk/26.3.11579264
cargo install cargo-ndk --version 3.5.4
rustup target add aarch64-linux-android armv7-linux-androideabi \
                  x86_64-linux-android i686-linux-android
./build-android.sh     # detects ANDROID_NDK_HOME and cross-compiles all ABIs
```

Output:
- `build/android/signer-core-<sha>.aar` — distributable AAR with per-ABI `.so` files under `jni/<abi>/`.
- `build/aar-stage/jni/<abi>/libsigner_core.so` — also available here; the sample module's `jniLibs.srcDirs` points to this directory so Gradle packages the `.so` into the instrumented-test APK without a separate copy step.

---

## Artifact versioning

Every artifact produced by CI (and locally by `build-ios.sh` /
`build-android.sh`) embeds a `VERSION.txt` manifest:

```
commit=<full sha>
branch=<ref>
built-at=<iso-8601 utc>
rust=<rustc --version>
uniffi=<uniffi version>
ndk=<ndk path>           (Android only)
xcode=<xcodebuild -version>   (iOS only)
```

- Filenames include the 7-character short SHA (`signer-core-abcd123.aar`,
  `SignerCore-abcd123.xcframework.zip`).
- No GitHub Releases, no package registry — CI uploads via
  `actions/upload-artifact`. Pull artifacts via the GitHub Actions run URL.
- Retention: 14 days for feature branches, 90 days for `main`.
- Phase 3 revisits this when real mobile apps consume signer-core.

---

## CI pipeline

See [`.github/workflows/signer-core.yml`](../../.github/workflows/signer-core.yml).

Jobs (each path-filtered to `crates/signer-core/**`):

| Job | Runner | Role |
|-----|--------|------|
| `lint-test` | ubuntu-latest | `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` |
| `ios-xcframework` | macos-15 | Build staticlibs + xcframework + Swift test |
| `android-aar` | ubuntu-latest | Cross-compile per-ABI `.so` + assemble AAR + instrumented test on API-30 emulator |

All three must pass before a `crates/signer-core` change merges.

Local ↔ CI parity is preserved by making the workflow invoke the exact same
`build-ios.sh` and `build-android.sh` scripts developers use. Any deviation
is a reproducibility bug; fix the script, not the workflow.

---

## Troubleshooting

**Instrumented test fails with `UnsatisfiedLinkError` on emulator.**
The per-ABI `.so` files must be staged at `build/aar-stage/jni/<abi>/` before
assembling the APK. Run `./build-android.sh` (with `ANDROID_NDK_HOME` set) first.
If the emulator ABI is `x86_64` but only `arm64-v8a` `.so` files are present,
ensure the NDK cross-compiled all four targets successfully.

**`xcodebuild test` fails with "No such module 'SignerCore'".**
The SwiftPM sample reads the xcframework from `../build/ios/`. Run
`./build-ios.sh` before `xcodebuild test`. If the framework is stale after a
UDL change, delete `build/` and rerun `build-ios.sh`.

**Rust 1.88 can't find std for `x86_64-apple-ios-sim`.**
That's a tier-3 target without a pre-built std on 1.88. It's intentionally
excluded from `rust-toolchain.toml`. Revisit if/when Intel-Mac simulator
developers need to run iOS tests.

---

## Module: `uos`

Added in Phase 1.  A byte-for-byte Rust port of the UOS (Universal Offline
Signatures) protocol previously implemented only in Kotlin.

### Public API (via `lib.rs` re-exports)

| Type / function | Description |
|-----------------|-------------|
| `UosPayload` | Unsigned-transaction wrapper; `encode() -> Result<Vec<u8>, UosError>`, `decode(&[u8]) -> Result<Self, UosError>` |
| `UosSignature` | Signature response wrapper; same encode/decode pattern |
| `MultiPartQrEncoder` | Splits a payload into ≤1024-byte QR frames |
| `MultiPartQrDecoder` | Stateful reassembler; `add_frame(Vec<u8>) -> Result<FrameDecodeProgress, UosError>` |
| `AccountIntroduction` | `substrate:` URI codec; `to_uri() -> String`, `from_uri(&str) -> Result<Self, UosError>` |
| `UosError` | Sealed flat enum; all error paths in the module |
| namespace fns | `encode_payload`, `decode_payload`, `encode_signature`, `decode_signature`, `account_intro_to_uri`, `account_intro_from_uri` |

All types are declared in `src/signer_core.udl` and re-exported from `lib.rs`.

### Corpus test workflow

The corpus tests (`tests/uos_*_corpus.rs`) read golden JSON files from
`tests/corpora/` and assert byte-level parity with the Kotlin reference.

**Re-generate corpus from Kotlin oracle:**

```bash
cd clad-mobile
./gradlew :shared:jvmTest --tests "tech.wideas.clad.uos.UosCorpusExport"
```

**Run Rust corpus + property tests:**

```bash
cd clad-studio
cargo test -p signer-core --locked
cargo test -p signer-core --locked --test uos_property_tests
```

CI does **not** regenerate corpus files (that would defeat the oracle purpose).
See `tests/corpora/README.md` for the full workflow.

### Notes

- The inner SCALE-encoded signing payload is treated as **opaque bytes** in
  Phase 1.  `sr25519`, `ed25519`, and `ecdsa` signing primitives land in
  Phase 2.
- `account_introduction` URL encoding matches Java's `URLEncoder.encode`
  (`application/x-www-form-urlencoded`): spaces → `+`, unreserved set includes
  `*`.  This differs from RFC 3986; see `tests/corpora/README.md` for details.
- UniFFI upgrade resolved: bumped to 0.29.5 in Sprint A1 (Phase 3).
  `#![allow(clippy::empty_line_after_doc_comments)]` removed — the 0.29.x
  scaffolding generator no longer emits the offending pattern.
