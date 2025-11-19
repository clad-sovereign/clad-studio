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
| `pallet-clad-token`        | Complete (MVP)  | FRAME pallet with roles, freeze/unfreeze, whitelist, ERC-3643-compatible hooks. Extensible for voting rights and repayment oracles. |
| `clad-mobile`              | In development (Q1 2026) | Kotlin Multiplatform native signer (iOS/Android) with biometric authentication and offline QR signing. Eliminates browser/extension dependency for officials. |

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

## Development

```bash
git clone https://github.com/clad-sovereign/clad-studio.git
cd clad-studio
cargo build --release
```

## Roadmap

| Phase                  | Timeline         | Milestones |
|------------------------|------------------|------------|
| Phase 1 – Core         | Nov 2025 – Jan 2026 | Pallet complete • Minimal mobile signer • Grant submissions (Web3 Foundation / Polkadot Treasury) |
| Phase 2 – Pilots       | Feb – Jun 2026   | 2–3 sovereign/SOE pilots ($10–100M range) • Full mobile admin dashboard |
| Phase 3 – Deployment   | H2 2026 onward   | White-label deployments • Central-bank oracle integrations • Multi-jurisdiction operations |

Contact: helloclad@wideas.tech
