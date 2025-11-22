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

| Component                  | Status          | Description |
|----------------------------|-----------------|-------------|
| `pallet-clad-token`        | ✅ Complete (MVP)  | FRAME pallet with roles, freeze/unfreeze, whitelist, ERC-3643-compatible hooks. Extensible for voting rights and repayment oracles. |
| `clad-node`                | ✅ Complete (Milestone 2) | Substrate node with Aura consensus and Grandpa finality. Complete runtime integration with operational RPC endpoints. |
| `clad-mobile`              | 🚧 In Development | Kotlin Multiplatform native signer (iOS/Android) with biometric authentication and offline QR signing. Production delivery: Feb 2026. |

## Target jurisdictions (2026 pilots)
Indonesia • Kazakhstan • Nigeria • Egypt • Peru • Vietnam • Côte d'Ivoire • Uzbekistan • Rwanda • Paraguay follow-ons

## Strategic positioning

**Aligned with Polkadot SDK best practices for compliant RWA tokenization**
Clad Studio implements the compliance-ready module approach described in Polkadot's official [RWA tokenization guide](https://polkadot.com/blog/real-world-assets-rwa-tokenization-guide/):
- Regulatory compliance via built-in freeze, whitelist, and role-based access control
- Institutional-grade security through Polkadot's shared security model
- Cross-chain interoperability via XCM for DeFi integration and liquidity access

**Filling the sovereign debt gap**
While existing Polkadot RWA projects focus on real estate (Xcavate), commodities (TVVIN), energy credits (Energy Web), and private credit (Centrifuge), Clad Studio targets the unclaimed sovereign and emerging-market segment:
- Tokenized sovereign debt issuance for finance ministries and debt-management offices
- State-owned enterprise equity tokenization outside Western-controlled rails
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

To test consensus with multiple validators (Aura block production + Grandpa finality):

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

The `--tmp` flag creates temporary storage for blockchain data, but network keys require explicit generation. The `--unsafe-force-node-key-generation` flag automatically creates network keys when they don't exist.

**Important:** This flag is named "unsafe" because it regenerates keys on each restart, which breaks peer connectivity in real deployments. Only use for ephemeral test environments.

**For persistent local testnets:**
```bash
# Generate stable network keys (once)
./target/release/clad-node key generate-node-key --file /path/to/alice.key
./target/release/clad-node key generate-node-key --file /path/to/bob.key

# Start validators with persistent keys (data survives restarts)
./target/release/clad-node --chain local --alice --node-key-file /path/to/alice.key
./target/release/clad-node --chain local --bob --node-key-file /path/to/bob.key
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