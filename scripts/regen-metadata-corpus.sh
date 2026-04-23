#!/usr/bin/env bash
# Regenerate the SCALE metadata corpus from a running clad-node --dev instance.
# Usage: ./scripts/regen-metadata-corpus.sh [rpc_url]
# Default RPC URL: http://127.0.0.1:9944
#
# Writes two files to crates/signer-core/tests/corpora/metadata/:
#   metadata_vN.scale  — raw SCALE-encoded Metadata VN blob (N = detected version)
#   metadata_vN.json   — sidecar: spec_version, transaction_version, captured_at, metadata_version
#
# Run after every runtime/ version bump that changes pallet or call indices.
set -euo pipefail

RPC_URL="${1:-http://127.0.0.1:9944}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORPUS_DIR="$SCRIPT_DIR/../crates/signer-core/tests/corpora/metadata"

mkdir -p "$CORPUS_DIR"

echo "Fetching metadata from $RPC_URL..."
META_RESPONSE=$(curl -sf -X POST \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"state_getMetadata","params":[],"id":1}' \
    "$RPC_URL")

META_HEX=$(echo "$META_RESPONSE" | jq -r '.result')
if [ "$META_HEX" = "null" ] || [ -z "$META_HEX" ]; then
    echo "ERROR: state_getMetadata returned null or empty result" >&2
    exit 1
fi

# Detect metadata version from the 5th byte (index 4) of the binary blob.
# Magic: 4 bytes "meta" (0x6d657461), then 1 byte version.
VERSION_BYTE=$(echo "${META_HEX#0x}" | cut -c9-10)
META_VERSION=$((16#$VERSION_BYTE))
echo "Detected metadata version: V${META_VERSION}"

SCALE_FILE="$CORPUS_DIR/metadata_v${META_VERSION}.scale"
JSON_FILE="$CORPUS_DIR/metadata_v${META_VERSION}.json"

# Remove "0x" prefix then hex-decode to binary
echo "${META_HEX#0x}" | xxd -r -p > "$SCALE_FILE"

echo "Fetching runtime version..."
VERSION_RESPONSE=$(curl -sf -X POST \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"state_getRuntimeVersion","params":[],"id":1}' \
    "$RPC_URL")

SPEC_VERSION=$(echo "$VERSION_RESPONSE" | jq '.result.specVersion')
TX_VERSION=$(echo "$VERSION_RESPONSE" | jq '.result.transactionVersion')
CAPTURED_AT="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

jq -n \
    --argjson metadata_version "$META_VERSION" \
    --argjson spec_version "$SPEC_VERSION" \
    --argjson transaction_version "$TX_VERSION" \
    --arg captured_at "$CAPTURED_AT" \
    '{"metadata_version": $metadata_version, "spec_version": $spec_version, "transaction_version": $transaction_version, "captured_at": $captured_at}' \
    > "$JSON_FILE"

SCALE_SIZE=$(wc -c < "$SCALE_FILE" | tr -d ' ')
SCALE_SHA=$(shasum -a 256 "$SCALE_FILE" | awk '{print $1}')

echo "Corpus written to $CORPUS_DIR"
echo "  metadata_version=V${META_VERSION}  spec_version=$SPEC_VERSION  transaction_version=$TX_VERSION"
echo "  size=${SCALE_SIZE} bytes  sha256=$SCALE_SHA"
