#!/usr/bin/env bash
# build-ios.sh — Produce a local .xcframework for the iOS sample consumer.
#
# Invoked both by developers (local smoke test of the FFI pipeline) and by
# the `signer-core.yml` GitHub Actions workflow. The two invocations must
# behave identically; any deviation is a reproducibility bug.
#
# Output:
#   crates/signer-core/build/ios/SignerCore.xcframework   (binary framework)
#   crates/signer-core/ios/Sources/SignerCore/SignerCore.swift (generated Swift)
#   crates/signer-core/ios/Sources/SignerCore/signer_coreFFI.h (C header)
#   crates/signer-core/ios/Sources/SignerCore/module.modulemap (umbrella mod)
#
# Phase 0 scope: aarch64-apple-ios (device) + aarch64-apple-ios-sim
# (Apple-silicon simulator). x86_64 simulator slice omitted because:
#   (a) Rust 1.88 std for x86_64-apple-ios-sim is tier-3 (no pre-built std);
#   (b) Phase 0 only requires one passing iOS test, and we run it on the
#       aarch64-apple-ios-sim slice (the dev machine is Apple-silicon).
# Phase 3 revisits if Intel-Mac simulator support becomes a requirement.

set -euo pipefail

# Resolve crate root regardless of where the script is invoked from.
HERE="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$HERE"

WORKSPACE_ROOT="$( cd "$HERE/../.." && pwd )"
CRATE_NAME="signer_core"            # matches [lib].name in Cargo.toml
FRAMEWORK_NAME="SignerCore"         # matches uniffi.toml [bindings.swift].module_name
BUILD_DIR="$HERE/build/ios"
GEN_DIR="$HERE/build/generated-swift"
SAMPLE_SOURCES="$HERE/ios/Sources/SignerCore"

# iOS slices we produce. Keyed by Rust target; value is the xcframework
# library path (under target/<triple>/release/) to feed `xcodebuild`.
TARGETS=(
    "aarch64-apple-ios"             # physical device (arm64)
    "aarch64-apple-ios-sim"         # Apple-silicon simulator (arm64)
)

echo ">> clean previous build output"
rm -rf "$BUILD_DIR" "$GEN_DIR"
mkdir -p "$BUILD_DIR" "$GEN_DIR" "$SAMPLE_SOURCES"

echo ">> ensure iOS targets installed"
for t in "${TARGETS[@]}"; do
    rustup target add "$t" >/dev/null
done

echo ">> cargo build (release, staticlib) per iOS target"
for t in "${TARGETS[@]}"; do
    echo "   - $t"
    ( cd "$WORKSPACE_ROOT" \
      && cargo build --locked --release -p signer-core --target "$t" )
done

echo ">> generate Swift bindings from the UDL"
# Bindgen reads the shared library's metadata to emit idiomatic Swift. We
# point it at any one of the built dylibs; they all encode the same UDL.
PROBE_LIB="$WORKSPACE_ROOT/target/aarch64-apple-ios/release/lib${CRATE_NAME}.a"
cargo run --locked --release \
    --manifest-path "$HERE/Cargo.toml" \
    --features uniffi-cli \
    --bin uniffi-bindgen -- \
    generate "$HERE/src/${CRATE_NAME}.udl" \
    --language swift \
    --out-dir "$GEN_DIR" \
    --no-format

# UniFFI emits files named after the Swift `module_name` from uniffi.toml:
# `<Module>.swift`, `<Module>FFI.h`, and `<Module>FFI.modulemap`.
# Swift Package consumption needs the header and modulemap to live inside
# the xcframework slice; the `.swift` file is compiled as a Package source.
HEADER="$GEN_DIR/${FRAMEWORK_NAME}FFI.h"
MODULEMAP="$GEN_DIR/${FRAMEWORK_NAME}FFI.modulemap"
SWIFT_SRC="$GEN_DIR/${FRAMEWORK_NAME}.swift"

for f in "$HEADER" "$MODULEMAP" "$SWIFT_SRC"; do
    [[ -f "$f" ]] || { echo "ERROR: expected generator output $f missing" >&2; exit 1; }
done

# xcframework expects the modulemap at a conventional filename.
cp "$MODULEMAP" "$BUILD_DIR/module.modulemap"

echo ">> assemble xcframework"
XCF_OUT="$BUILD_DIR/${FRAMEWORK_NAME}.xcframework"
rm -rf "$XCF_OUT"

# Each slice is a static library plus a headers dir containing the UniFFI
# C header and a module.modulemap renamed to the conventional path.
BUILD_ARGS=()
for t in "${TARGETS[@]}"; do
    SLICE_HEADERS="$BUILD_DIR/headers-$t"
    mkdir -p "$SLICE_HEADERS"
    cp "$HEADER" "$SLICE_HEADERS/"
    cp "$MODULEMAP" "$SLICE_HEADERS/module.modulemap"
    STATIC_LIB="$WORKSPACE_ROOT/target/$t/release/lib${CRATE_NAME}.a"
    [[ -f "$STATIC_LIB" ]] || { echo "ERROR: $STATIC_LIB missing" >&2; exit 1; }
    BUILD_ARGS+=(-library "$STATIC_LIB" -headers "$SLICE_HEADERS")
done

xcodebuild -create-xcframework "${BUILD_ARGS[@]}" -output "$XCF_OUT"

echo ">> stage Swift sources for the SwiftPM sample"
cp "$SWIFT_SRC" "$SAMPLE_SOURCES/${FRAMEWORK_NAME}.swift"

echo ">> write VERSION.txt manifest into the xcframework"
RUST_VERSION="$(rustc --version)"
UNIFFI_VERSION="$(cargo run --locked --release \
    --manifest-path "$HERE/Cargo.toml" \
    --features uniffi-cli \
    --bin uniffi-bindgen -- --version 2>/dev/null || echo 'uniffi-bindgen 0.28.3')"
XCODE_VERSION="$(xcodebuild -version | tr '\n' ' ')"
COMMIT_SHA="$(git -C "$WORKSPACE_ROOT" rev-parse HEAD 2>/dev/null || echo 'unknown')"
BRANCH="$(git -C "$WORKSPACE_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')"
BUILT_AT="$(date -u +'%Y-%m-%dT%H:%M:%SZ')"

cat > "$XCF_OUT/VERSION.txt" <<MANIFEST
commit=$COMMIT_SHA
branch=$BRANCH
built-at=$BUILT_AT
rust=$RUST_VERSION
uniffi=$UNIFFI_VERSION
xcode=$XCODE_VERSION
targets=${TARGETS[*]}
MANIFEST

echo ">> done"
echo "   framework: $XCF_OUT"
echo "   swift src: $SAMPLE_SOURCES/${FRAMEWORK_NAME}.swift"
