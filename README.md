![Clad Signer Demo – Native iOS & Android](https://cdn.loom.com/sessions/thumbnails/dd334230db154f9891f46664ae02aec4-9e6c0699711bd8ff-full-play.gif#t=0.1)\
*[Watch 60-second demo – November 2025](https://www.loom.com/share/dd334230db154f9891f46664ae02aec4)*

# Clad Studio

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![CI](https://github.com/clad-sovereign/clad-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/clad-sovereign/clad-studio/actions)

**Open-source tokenization toolkit for sovereign and emerging-market real-world assets**
Polkadot / Substrate • Rust • Compliance-first • Mobile-native • Geopolitically neutral

Designed for finance ministries, debt-management offices, and state-owned enterprises issuing compliant tokenized debt or equity on fully controllable infrastructure.

Primary reference: Paraguay sovereign equity tokenization (2025).

## Components

| Component           | Repository | Status | Description |
|---------------------|------------|--------|-------------|
| `pallet-clad-token` | [clad-studio](https://github.com/clad-sovereign/clad-studio) | ✅ Functional | FRAME pallet with roles, freeze/unfreeze, whitelist, ERC-3643-compatible hooks. Production hardening in progress. |
| `clad-node`         | [clad-studio](https://github.com/clad-sovereign/clad-studio) | ✅ Functional | Substrate node with Aura consensus and Grandpa finality. Enables local multi-validator testnet. |
| `clad-signer`       | [clad-mobile](https://github.com/clad-sovereign/clad-mobile) | 🚧 In Development | Kotlin Multiplatform native signer (iOS/Android) with biometric authentication and offline QR signing. |

## Target Markets

Emerging-market sovereigns and state-owned enterprises in:
- Central Asia & Southeast Asia
- Sub-Saharan Africa
- Latin America & Caribbean

**Focus:** Finance ministries, debt-management offices, and central banks requiring compliant tokenization infrastructure for sovereign bonds and equity issuance.

**Pilot timeline:** H1-H2 2026

**Public reference:** Paraguay sovereign equity tokenization (announced Q3 2025)

## Strategic positioning

**Aligned with Polkadot SDK best practices for compliant RWA tokenization**
Clad Studio implements the compliance-ready module approach described in Polkadot's official [RWA tokenization guide](https://polkadot.com/blog/real-world-assets-rwa-tokenization-guide/):
- Regulatory compliance via built-in freeze, whitelist, and role-based access control
- Institutional-grade security through Polkadot's shared security model
- Cross-chain interoperability via XCM for DeFi integration and liquidity access

**Filling the sovereign debt gap**
While existing Polkadot RWA projects focus on real estate (Xcavate), commodities (TVVIN), energy credits (Energy Web), and private credit (Centrifuge), Clad Studio targets the unclaimed sovereign and emerging-market segment:
- Tokenized sovereign debt issuance for finance ministries and debt-management offices
- State-owned enterprise equity tokenization on self-hosted infrastructure
- Mobile-native signing infrastructure for officials who use iOS/Android as primary work devices

This positions Clad Studio to become the reference implementation for sovereign RWA tokenization in the Polkadot ecosystem — building on Paraguay's 2025 precedent while creating reusable, grant-funded public infrastructure.

## Quick Start

### Prerequisites

**macOS:**
```bash
brew install cmake pkg-config openssl git curl protobuf
rustup target add wasm32-unknown-unknown
```

**Linux (Debian/Ubuntu):**
```bash
sudo apt install build-essential git clang curl libssl-dev llvm libudev-dev \
  make protobuf-compiler pkg-config
rustup target add wasm32-unknown-unknown
```

### Build & Run

```bash
# Clone the repository
git clone https://github.com/clad-sovereign/clad-studio.git
cd clad-studio

# Build the node (takes ~5-10 minutes first time)
cargo build --release --locked

# Start the node
./target/release/clad-node --dev --tmp

# You should see:
# ✅ Genesis block initialized
# ✅ Block production (every 6 seconds)
# ✅ RPC server at ws://127.0.0.1:9944
```

### Available Commands

```bash
# Run tests
cargo test --locked

# Format code
cargo fmt

# Lint code
cargo clippy --locked -- -D warnings
```

**⚠️ External RPC Access (Testing Only)**
```bash
# WARNING: Only use for local testing on private networks
# DO NOT expose publicly without proper security configuration
./target/release/clad-node --dev --tmp --rpc-external --rpc-cors all
```

### Multi-Validator Local Testnet

#### Option 1: Quick 2-Node Setup (Fastest)

For rapid testing of basic consensus (Aura block production + Grandpa finality):

```bash
# Terminal 1 - Start Alice
./target/release/clad-node \
  --chain local \
  --alice \
  --tmp \
  --unsafe-force-node-key-generation \
  --port 30333 \
  --rpc-port 9944

# Terminal 2 - Start Bob
./target/release/clad-node \
  --chain local \
  --bob \
  --tmp \
  --unsafe-force-node-key-generation \
  --port 30334 \
  --rpc-port 9945
```

**Why `--unsafe-force-node-key-generation`?**
Auto-generates network keys for `--tmp` ephemeral testnets. Only for quick local testing—keys regenerate on restart, breaking peer connectivity.

#### Option 2: 3-Validator Network (Recommended for SDK Validation)

For comprehensive consensus testing with GRANDPA finalization (requires 2/3 validators):

```bash
# Terminal 1 - Alice (bootnode)
./target/release/clad-node \
  --alice --validator \
  --base-path /tmp/clad-alice \
  --chain local \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --port 30333 \
  --rpc-port 9944 \
  --rpc-methods=unsafe

# Terminal 2 - Bob
./target/release/clad-node \
  --bob --validator \
  --base-path /tmp/clad-bob \
  --chain local \
  --node-key 0000000000000000000000000000000000000000000000000000000000000002 \
  --port 30334 \
  --rpc-port 9945 \
  --rpc-methods=unsafe \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp

# Terminal 3 - Charlie
./target/release/clad-node \
  --charlie --validator \
  --base-path /tmp/clad-charlie \
  --chain local \
  --node-key 0000000000000000000000000000000000000000000000000000000000000003 \
  --port 30335 \
  --rpc-port 9946 \
  --rpc-methods=unsafe \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

**Verify consensus is working:**
```bash
# Check block production (should show "best: #N, finalized #N-2, peers: 2")
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://localhost:9944 | jq

# Expected output: {"peers": 2, "isSyncing": false}
```

**Why 3 validators?**
- GRANDPA requires 2/3 supermajority for finalization
- Tests realistic network partitioning scenarios
- Matches typical proof-of-authority testnet configurations
- Validates SDK upgrade compatibility (block production + finalization + peer discovery)

**Cleanup:**
```bash
pkill clad-node
rm -rf /tmp/clad-*
```

**Note:** The `--chain local` spec is for multi-validator testing only. Production sovereign chains require custom chain specifications with proper genesis configuration, validator session keys, and security hardening.

## Roadmap

| Phase                  | Timeline         | Milestones |
|------------------------|------------------|------------|
| Phase 1 – Foundation   | Nov 2025 – Feb 2026 | Pallet production hardening (benchmarking, weights, migrations) • Docker containerization • Production mobile signing infrastructure • Polkadot Open Source Grant execution |
| Phase 2 – Pilots       | Mar – Jun 2026   | 2–3 sovereign/SOE pilots ($10–100M range) • Full mobile admin dashboard • Security audit |
| Phase 3 – Deployment   | H2 2026 onward   | White-label deployments • Central-bank oracle integrations • Multi-jurisdiction operations |

Contact: helloclad@wideas.tech

> **Disclaimer**  
> Clad Sovereign is pre-pilot software. It is not yet intended for production use or real fund issuance. Use only on testnets or local chains.