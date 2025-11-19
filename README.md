# Clad Studio

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
