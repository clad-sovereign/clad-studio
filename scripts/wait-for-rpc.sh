#!/usr/bin/env bash
# Poll the Substrate JSON-RPC endpoint until it responds, then exit 0.
# Usage: ./scripts/wait-for-rpc.sh [rpc_url]
# Default RPC URL: http://127.0.0.1:9944
set -euo pipefail

RPC_URL="${1:-http://127.0.0.1:9944}"
MAX_WAIT=60

echo "Waiting for RPC at $RPC_URL (up to ${MAX_WAIT}s)..."
for i in $(seq 1 $MAX_WAIT); do
    if curl -sf -X POST \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"system_chain","params":[],"id":1}' \
        "$RPC_URL" > /dev/null 2>&1; then
        echo "RPC ready after ${i}s"
        exit 0
    fi
    sleep 1
done

echo "ERROR: RPC at $RPC_URL not ready after ${MAX_WAIT}s" >&2
exit 1
