#!/usr/bin/env bash
# build-android.sh — Produce Android FFI artifacts AND a host-platform
# shared library the JVM sample test consumes via JNA.
#
# Phase 0 scope (per docs/migration/01-phases.md and the Phase 0 sprint
# plan): two outputs.
#
#   1. `signer-core-<sha>.aar` — a real Android AAR bundling Kotlin bindings
#      and per-ABI `.so` files. Produced by CI from Linux with the Android
#      NDK installed. On macOS dev machines without the NDK, we skip the
#      cross-compile step and emit a placeholder AAR containing only the
#      Kotlin sources; real device consumption arrives in Phase 3.
#
#   2. A host-platform `.dylib` / `.so` at
#      `build/android-host/<lib>signer_core.<ext>` so `./gradlew :sample:test`
#      can load the library via JNA and exercise `ping()` on the dev machine
#      or in Linux CI. This is what satisfies exit criterion #2 ("one
#      Android test calls into Rust and passes") locally, without an
#      emulator.
#
# Usage:
#   ./build-android.sh            # host .dylib/.so + Kotlin bindings + best-effort AAR
#   ANDROID_NDK_HOME=... ./build-android.sh   # forces full cross-compile

set -euo pipefail

HERE="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$HERE"

WORKSPACE_ROOT="$( cd "$HERE/../.." && pwd )"
CRATE_NAME="signer_core"              # matches [lib].name in Cargo.toml
LIB_FILE_PREFIX="lib${CRATE_NAME}"    # Unix shared-lib naming
GEN_DIR="$HERE/build/generated-kotlin"
HOST_LIB_DIR="$HERE/build/android-host"
AAR_STAGE_DIR="$HERE/build/aar-stage"
AAR_OUT_DIR="$HERE/build/android"

HOST_OS="$(uname -s)"
case "$HOST_OS" in
    Darwin) HOST_LIB_EXT="dylib"; HOST_TARGET="aarch64-apple-darwin" ;;
    Linux)  HOST_LIB_EXT="so";    HOST_TARGET="x86_64-unknown-linux-gnu" ;;
    *)      echo "ERROR: unsupported host OS: $HOST_OS" >&2; exit 1 ;;
esac

echo ">> clean previous build output"
rm -rf "$GEN_DIR" "$HOST_LIB_DIR" "$AAR_STAGE_DIR" "$AAR_OUT_DIR"
mkdir -p "$GEN_DIR" "$HOST_LIB_DIR" "$AAR_STAGE_DIR" "$AAR_OUT_DIR"

echo ">> generate Kotlin bindings from the UDL"
# The Kotlin file lands under `<package_name replacing . with />/signer_core.kt`
# per uniffi.toml ([bindings.kotlin].package_name = tech.wideas.clad.signer).
cargo run --locked --release \
    --manifest-path "$HERE/Cargo.toml" \
    --features uniffi-cli \
    --bin uniffi-bindgen -- \
    generate "$HERE/src/${CRATE_NAME}.udl" \
    --language kotlin \
    --out-dir "$GEN_DIR" \
    --no-format

KOTLIN_SRC="$GEN_DIR/tech/wideas/clad/signer/${CRATE_NAME}.kt"
[[ -f "$KOTLIN_SRC" ]] || { echo "ERROR: generated Kotlin missing at $KOTLIN_SRC" >&2; exit 1; }

echo ">> build host-platform cdylib for the JVM sample"
( cd "$WORKSPACE_ROOT" \
  && cargo build --locked --release -p signer-core --target "$HOST_TARGET" )

HOST_LIB_SRC="$WORKSPACE_ROOT/target/$HOST_TARGET/release/${LIB_FILE_PREFIX}.${HOST_LIB_EXT}"
[[ -f "$HOST_LIB_SRC" ]] || { echo "ERROR: host library missing at $HOST_LIB_SRC" >&2; exit 1; }
cp "$HOST_LIB_SRC" "$HOST_LIB_DIR/"
echo "   staged: $HOST_LIB_DIR/${LIB_FILE_PREFIX}.${HOST_LIB_EXT}"

echo ">> cross-compile per-ABI .so (if Android NDK is available)"
ANDROID_ABIS=(
    "aarch64-linux-android:arm64-v8a"
    "armv7-linux-androideabi:armeabi-v7a"
    "x86_64-linux-android:x86_64"
    "i686-linux-android:x86"
)
NDK_OK=0
if [[ -n "${ANDROID_NDK_HOME:-}" && -d "${ANDROID_NDK_HOME}" ]] \
   && command -v cargo-ndk >/dev/null 2>&1; then
    NDK_OK=1
fi

if [[ "$NDK_OK" -eq 1 ]]; then
    echo "   ANDROID_NDK_HOME=$ANDROID_NDK_HOME — running cross-compile"
    for entry in "${ANDROID_ABIS[@]}"; do
        RUST_TARGET="${entry%%:*}"
        ANDROID_ABI="${entry##*:}"
        rustup target add "$RUST_TARGET" >/dev/null
        JNI_LIBS_DIR="$AAR_STAGE_DIR/jni/$ANDROID_ABI"
        mkdir -p "$JNI_LIBS_DIR"
        ( cd "$WORKSPACE_ROOT" \
          && cargo ndk -t "$ANDROID_ABI" -o "$AAR_STAGE_DIR/jni" \
             build --locked --release -p signer-core )
    done
    AAR_HAS_NATIVE=1
else
    echo "   (skipped — set ANDROID_NDK_HOME and install cargo-ndk to enable)"
    AAR_HAS_NATIVE=0
fi

echo ">> stage AAR contents"
# Minimal AAR structure:
#   AndroidManifest.xml
#   classes.jar                 (Kotlin bindings compiled — skipped for PoC; sources only)
#   jni/<abi>/libsigner_core.so (if NDK present)
#   res/                        (empty directory required by AAR spec)
#   R.txt                       (empty)
cat > "$AAR_STAGE_DIR/AndroidManifest.xml" <<'MANIFEST'
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
          package="tech.wideas.clad.signer">
    <uses-sdk android:minSdkVersion="26" />
</manifest>
MANIFEST
mkdir -p "$AAR_STAGE_DIR/res"
: > "$AAR_STAGE_DIR/R.txt"

# Ship the Kotlin bindings as source in a conventionally-named directory.
# The Android Gradle Plugin does not compile sources from an AAR; Phase 3 will
# switch to a fully-compiled .aar. For Phase 0 this is sufficient because the
# JVM sample consumes the Kotlin binding directly from `$GEN_DIR`, not from
# the AAR.
mkdir -p "$AAR_STAGE_DIR/sources"
cp "$KOTLIN_SRC" "$AAR_STAGE_DIR/sources/"

COMMIT_SHA="$(git -C "$WORKSPACE_ROOT" rev-parse HEAD 2>/dev/null || echo 'unknown')"
BRANCH="$(git -C "$WORKSPACE_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')"
BUILT_AT="$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
RUST_VERSION="$(rustc --version)"
NDK_VERSION="${ANDROID_NDK_HOME:-unavailable}"

cat > "$AAR_STAGE_DIR/VERSION.txt" <<MANIFEST
commit=$COMMIT_SHA
branch=$BRANCH
built-at=$BUILT_AT
rust=$RUST_VERSION
uniffi=0.28.3
ndk=$NDK_VERSION
native_slices_included=$AAR_HAS_NATIVE
phase=0
MANIFEST

SHORT_SHA="${COMMIT_SHA:0:7}"
AAR_FILE="$AAR_OUT_DIR/signer-core-${SHORT_SHA}.aar"
( cd "$AAR_STAGE_DIR" && zip -qr "$AAR_FILE" . )
echo "   aar: $AAR_FILE"

echo ">> done"
echo "   host lib:   $HOST_LIB_DIR/${LIB_FILE_PREFIX}.${HOST_LIB_EXT}"
echo "   kotlin src: $KOTLIN_SRC"
echo "   aar:        $AAR_FILE"
