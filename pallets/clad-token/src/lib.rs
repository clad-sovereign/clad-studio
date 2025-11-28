//! # Clad Token Pallet
//!
//! A compliance-first security token pallet implementing ERC-3643 (T-REX) design patterns
//! for sovereign and emerging-market real-world asset (RWA) tokenization.
//!
//! ## Overview
//!
//! This pallet provides the core infrastructure for tokenizing sovereign debt instruments,
//! government bonds, and state-owned enterprise equity. It is designed specifically for
//! finance ministries, debt management offices, and central banks requiring regulatory
//! compliance with full control over their tokenization infrastructure.
//!
//! ### ERC-3643 Compliance
//!
//! The pallet follows the [ERC-3643 (T-REX)](https://erc3643.org/) standard for permissioned
//! security tokens, which requires:
//!
//! - **Identity verification**: Only whitelisted (KYC-verified) accounts can hold or transfer tokens
//! - **Transfer restrictions**: Transfers are blocked if sender or receiver is not whitelisted
//! - **Freeze capability**: Accounts can be frozen for compliance, sanctions, or legal reasons
//! - **Centralized admin control**: A designated authority (ministry, regulator) controls token operations
//!
//! ### Target Use Cases
//!
//! 1. **Sovereign Bond Tokenization**: Finance ministries can issue tokenized government bonds
//!    accessible to domestic and international investors without traditional custodian chains.
//!
//! 2. **Treasury Bills**: Short-term debt instruments with automatic maturity tracking
//!    (planned feature: auto-repayment oracles).
//!
//! 3. **State-Owned Enterprise Equity**: Partial privatization via tokenized equity shares
//!    with voting rights (planned feature: on-chain governance).
//!
//! ### Architecture Decisions
//!
//! - **`u128` balances**: Supports values up to 340 undecillion, sufficient for any sovereign
//!   debt instrument (even Zimbabwe's historical hyperinflation).
//!
//! - **`BoundedVec` for metadata**: Token name (64 bytes max) and symbol (16 bytes max) are
//!   bounded to prevent storage bloat attacks.
//!
//! - **Separate whitelist and freeze**: An account can be whitelisted but frozen—this allows
//!   temporary suspension without losing KYC status.
//!
//! - **Admin-only minting**: No permissionless minting; all token creation requires explicit
//!   ministry/regulator approval.
//!
//! ## Quick Start
//!
//! ### Typical Workflow
//!
//! ```text
//! 1. Admin whitelists investor accounts (KYC approval)
//! 2. Admin mints tokens to treasury/issuer account
//! 3. Treasury transfers tokens to whitelisted investors
//! 4. Investors can transfer among themselves (if both whitelisted)
//! 5. Admin can freeze accounts for compliance issues
//! ```
//!
//! ### Integration Example
//!
//! ```ignore
//! // In your runtime configuration:
//! impl pallet_clad_token::Config for Runtime {
//!     type AdminOrigin = EnsureRoot<AccountId>;  // Or custom multi-sig origin
//!     type WeightInfo = pallet_clad_token::weights::SubstrateWeight<Runtime>;
//! }
//! ```
//!
//! ## Storage Layout
//!
//! | Storage Item | Type | Purpose |
//! |--------------|------|---------|
//! | `TokenName` | `BoundedVec<u8, 64>` | Human-readable token name |
//! | `TokenSymbol` | `BoundedVec<u8, 16>` | Trading symbol (e.g., "KZT-BOND-2025") |
//! | `Decimals` | `u8` | Decimal precision (typically 6 or 18) |
//! | `TotalSupply` | `u128` | Total tokens in circulation |
//! | `Balances` | `Map<AccountId, u128>` | Per-account token balances |
//! | `Frozen` | `Map<AccountId, bool>` | Frozen account flags |
//! | `Whitelist` | `Map<AccountId, bool>` | KYC-approved account flags |
//!
//! ## Dispatchable Functions
//!
//! | Extrinsic | Permission | Description |
//! |-----------|------------|-------------|
//! | [`mint`](pallet::Pallet::mint) | Admin | Create new tokens |
//! | [`transfer`](pallet::Pallet::transfer) | Signed | Transfer tokens between accounts |
//! | [`freeze`](pallet::Pallet::freeze) | Admin | Freeze an account |
//! | [`unfreeze`](pallet::Pallet::unfreeze) | Admin | Unfreeze an account |
//! | [`add_to_whitelist`](pallet::Pallet::add_to_whitelist) | Admin | Approve account for transfers |
//! | [`remove_from_whitelist`](pallet::Pallet::remove_from_whitelist) | Admin | Revoke transfer approval |
//!
//! ## License
//!
//! Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::let_unit_value)]
#![warn(missing_docs)]

use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, traits::EnsureOrigin};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_std::prelude::*;

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod migrations;
pub mod weights;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

/// The main pallet module containing configuration, storage, events, errors, and dispatchables.
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Configuration trait for the Clad Token pallet.
    ///
    /// This trait defines the types and constants required to integrate this pallet
    /// into a Substrate runtime. Implementors must provide an admin origin for
    /// privileged operations and weight information for transaction fee calculation.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use frame_support::traits::EnsureOrigin;
    /// use frame_system::EnsureRoot;
    ///
    /// impl pallet_clad_token::Config for Runtime {
    ///     // Only sudo/root can perform admin operations
    ///     type AdminOrigin = EnsureRoot<AccountId>;
    ///     // Use benchmark-derived weights
    ///     type WeightInfo = pallet_clad_token::weights::SubstrateWeight<Runtime>;
    /// }
    /// ```
    ///
    /// # Security Considerations
    ///
    /// The `AdminOrigin` controls all privileged operations including minting,
    /// freezing, and whitelisting. For production deployments, consider:
    ///
    /// - Using a multi-signature origin (e.g., 3-of-5 ministry officials)
    /// - Implementing a council/governance origin for democratic oversight
    /// - Adding time-locks for large minting operations
    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
        /// Origin that can perform administrative operations.
        ///
        /// This origin is authorized to:
        /// - Mint new tokens ([`Pallet::mint`])
        /// - Freeze/unfreeze accounts ([`Pallet::freeze`], [`Pallet::unfreeze`])
        /// - Manage whitelist ([`Pallet::add_to_whitelist`], [`Pallet::remove_from_whitelist`])
        ///
        /// # Typical Configurations
        ///
        /// | Use Case | Origin Type | Example |
        /// |----------|-------------|---------|
        /// | Development | `EnsureRoot` | Sudo account has full control |
        /// | Ministry | `EnsureSigned(MinistryAccount)` | Single designated official |
        /// | Multi-sig | `EnsureProportionAtLeast<3, 5, ...>` | 3-of-5 officials must approve |
        /// | Council | `pallet_collective::EnsureMembers<...>` | Democratic governance |
        ///
        /// # Security Warning
        ///
        /// Never use `EnsureNone` or an overly permissive origin in production.
        /// Unauthorized minting would destroy the token's value and credibility.
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Weight information for extrinsics in this pallet.
        ///
        /// Weights determine transaction fees and block space allocation.
        /// Use benchmark-derived weights for accurate fee estimation.
        ///
        /// # Options
        ///
        /// - [`weights::SubstrateWeight<T>`](weights::SubstrateWeight): Benchmark-derived weights
        /// - `()`: Zero weights (testing only, not for production)
        ///
        /// # Generating Weights
        ///
        /// ```bash
        /// # Build with benchmarking feature
        /// cargo build --features runtime-benchmarks --release
        ///
        /// # Run benchmarks (requires node binary)
        /// frame-omni-bencher v1 benchmark pallet \
        ///   --runtime target/release/wbuild/clad-runtime/clad_runtime.compact.compressed.wasm \
        ///   --pallet "pallet_clad_token" \
        ///   --extrinsic "" \
        ///   --output ./pallets/clad-token/src/weights.rs
        /// ```
        type WeightInfo: WeightInfo;
    }

    /// The pallet struct, used as a marker for the pallet in `construct_runtime!`.
    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    // ═══════════════════════════════════════════════════════════════════════════
    // STORAGE ITEMS - Token Metadata
    // ═══════════════════════════════════════════════════════════════════════════

    /// Human-readable name of the token.
    ///
    /// This is the full name displayed in wallets, block explorers, and official documents.
    /// For sovereign bonds, include the issuing country and maturity year.
    ///
    /// # Format
    ///
    /// - **Maximum length**: 64 bytes (UTF-8 encoded)
    /// - **Recommended format**: `"[Country] [Instrument Type] [Year]"`
    ///
    /// # Examples
    ///
    /// | Token Name | Use Case |
    /// |------------|----------|
    /// | `"Kazakhstan Sovereign Bond 2030"` | 5-year government bond |
    /// | `"Malaysia Sukuk Token 2027"` | Islamic finance instrument |
    /// | `"Indonesia T-Bill Q4-2025"` | Short-term treasury bill |
    /// | `"KazMunayGas Equity Token"` | State-owned enterprise shares |
    ///
    /// # Storage
    ///
    /// - **Type**: `StorageValue` (single global value)
    /// - **Default**: Empty vector (must be set via genesis or migration)
    /// - **Mutability**: Set once at genesis; no extrinsic to change
    ///
    /// # Querying
    ///
    /// ```ignore
    /// // Via RPC (JavaScript)
    /// const name = await api.query.cladToken.tokenName();
    /// console.log(name.toUtf8()); // "Kazakhstan Sovereign Bond 2030"
    ///
    /// // Via getter function (Rust)
    /// let name: Vec<u8> = Pallet::<T>::token_name().to_vec();
    /// ```
    #[pallet::storage]
    #[pallet::getter(fn token_name)]
    pub type TokenName<T> = StorageValue<_, BoundedVec<u8, ConstU32<64>>, ValueQuery>;

    /// Trading symbol for the token.
    ///
    /// A short identifier used on exchanges, in mobile apps, and for quick reference.
    /// Similar to stock ticker symbols (e.g., AAPL, MSFT).
    ///
    /// # Format
    ///
    /// - **Maximum length**: 16 bytes (UTF-8 encoded)
    /// - **Recommended format**: `[ISO-3166]`-`[TYPE]`-`[YEAR]` or custom short code
    ///
    /// # Examples
    ///
    /// | Symbol | Meaning |
    /// |--------|---------|
    /// | `"KZT-BOND-2030"` | Kazakhstan bond maturing 2030 |
    /// | `"MYS-SUKUK-27"` | Malaysia sukuk maturing 2027 |
    /// | `"IDR-TBILL-Q4"` | Indonesia Q4 treasury bill |
    /// | `"KMG-EQ"` | KazMunayGas equity |
    ///
    /// # Storage
    ///
    /// - **Type**: `StorageValue` (single global value)
    /// - **Default**: Empty vector
    /// - **Mutability**: Set once at genesis
    #[pallet::storage]
    #[pallet::getter(fn token_symbol)]
    pub type TokenSymbol<T> = StorageValue<_, BoundedVec<u8, ConstU32<16>>, ValueQuery>;

    /// Number of decimal places for token amounts.
    ///
    /// Determines how raw `u128` values are displayed to users. For example,
    /// with `decimals = 6`, a raw value of `1_000_000` displays as `1.000000`.
    ///
    /// # Common Values
    ///
    /// | Decimals | Display | Use Case |
    /// |----------|---------|----------|
    /// | `0` | `1000000` → `1000000` | Whole units only (rare) |
    /// | `2` | `1000000` → `10000.00` | Traditional currency display |
    /// | `6` | `1000000` → `1.000000` | USDC/USDT style (recommended for bonds) |
    /// | `18` | `1000000` → `0.000000000001` | Ethereum-native compatibility |
    ///
    /// # Recommendation
    ///
    /// Use **6 decimals** for sovereign bonds. This provides sufficient precision
    /// for fractional ownership while keeping numbers manageable. Matches USDC/USDT
    /// conventions familiar to institutional investors.
    ///
    /// # Formula
    ///
    /// ```text
    /// display_value = raw_value / 10^decimals
    /// raw_value = display_value * 10^decimals
    /// ```
    ///
    /// # Storage
    ///
    /// - **Type**: `StorageValue<u8>` (single byte, 0-255)
    /// - **Default**: `0` (must be set via genesis)
    /// - **Mutability**: Set once at genesis
    #[pallet::storage]
    #[pallet::getter(fn decimals)]
    pub type Decimals<T> = StorageValue<_, u8, ValueQuery>;

    // ═══════════════════════════════════════════════════════════════════════════
    // STORAGE ITEMS - Supply & Balances
    // ═══════════════════════════════════════════════════════════════════════════

    /// Total number of tokens in circulation.
    ///
    /// This value increases when [`mint`](Pallet::mint) is called and represents
    /// the sum of all account balances. For sovereign bonds, this typically
    /// equals the total issuance amount of the debt instrument.
    ///
    /// # Invariant
    ///
    /// ```text
    /// TotalSupply == Σ Balances[account] for all accounts
    /// ```
    ///
    /// This invariant is maintained by the pallet and should never be violated.
    ///
    /// # Example Values
    ///
    /// | Bond Issue | Decimals | TotalSupply (raw) | Display Value |
    /// |------------|----------|-------------------|---------------|
    /// | $100M bond | 6 | `100_000_000_000_000` | 100,000,000.000000 |
    /// | $1B bond | 6 | `1_000_000_000_000_000` | 1,000,000,000.000000 |
    /// | 500M KZT bond | 2 | `50_000_000_000` | 500,000,000.00 |
    ///
    /// # Storage
    ///
    /// - **Type**: `StorageValue<u128>` (max ~340 undecillion)
    /// - **Default**: `0`
    /// - **Mutability**: Modified by [`mint`](Pallet::mint)
    ///
    /// # Querying
    ///
    /// ```ignore
    /// // Via RPC (JavaScript)
    /// const supply = await api.query.cladToken.totalSupply();
    /// const decimals = await api.query.cladToken.decimals();
    /// const displaySupply = supply.toBigInt() / BigInt(10 ** decimals.toNumber());
    /// ```
    #[pallet::storage]
    #[pallet::getter(fn total_supply)]
    pub type TotalSupply<T> = StorageValue<_, u128, ValueQuery>;

    /// Token balance for each account.
    ///
    /// Maps account IDs to their token holdings. Accounts not in this map
    /// have a balance of zero (via `ValueQuery` default).
    ///
    /// # Access Patterns
    ///
    /// | Operation | Method |
    /// |-----------|--------|
    /// | Read balance | `Balances::<T>::get(&account)` |
    /// | Set balance | `Balances::<T>::insert(&account, amount)` |
    /// | Remove (set to 0) | `Balances::<T>::remove(&account)` |
    /// | Check exists | `Balances::<T>::contains_key(&account)` |
    ///
    /// # Storage
    ///
    /// - **Type**: `StorageMap<AccountId, u128>`
    /// - **Hasher**: `Blake2_128Concat` (secure, key-recoverable)
    /// - **Default**: `0` for missing keys
    ///
    /// # Security Note
    ///
    /// Balance modifications should only occur through:
    /// - [`mint`](Pallet::mint): Admin creates new tokens
    /// - [`transfer`](Pallet::transfer): User transfers tokens
    /// - Genesis configuration: Initial distribution
    ///
    /// Direct storage manipulation outside these paths breaks the `TotalSupply` invariant.
    ///
    /// # Querying
    ///
    /// ```ignore
    /// // Via RPC (JavaScript)
    /// const balance = await api.query.cladToken.balances(accountId);
    ///
    /// // Via getter (Rust)
    /// let balance: u128 = Pallet::<T>::balance_of(&account);
    /// ```
    #[pallet::storage]
    #[pallet::getter(fn balance_of)]
    pub type Balances<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u128, ValueQuery>;

    // ═══════════════════════════════════════════════════════════════════════════
    // STORAGE ITEMS - Compliance Controls
    // ═══════════════════════════════════════════════════════════════════════════

    /// Accounts that are frozen and cannot send transfers.
    ///
    /// Frozen accounts can still **receive** tokens but cannot **send** them.
    /// This allows compliance officers to halt suspicious activity while
    /// preserving the account's ability to receive court-ordered returns.
    ///
    /// # Use Cases
    ///
    /// | Scenario | Action |
    /// |----------|--------|
    /// | Suspected fraud | Freeze account pending investigation |
    /// | Sanctions compliance | Freeze accounts matching OFAC/UN lists |
    /// | Legal dispute | Freeze until court order received |
    /// | Account recovery | Freeze to prevent further unauthorized transfers |
    ///
    /// # Relationship with Whitelist
    ///
    /// An account can be both **whitelisted** (KYC approved) and **frozen**:
    ///
    /// | Whitelisted | Frozen | Can Send | Can Receive |
    /// |-------------|--------|----------|-------------|
    /// | ✓ | ✗ | ✓ | ✓ |
    /// | ✓ | ✓ | ✗ | ✓ (if sender whitelisted) |
    /// | ✗ | ✗ | ✗ | ✗ |
    /// | ✗ | ✓ | ✗ | ✗ |
    ///
    /// # Storage
    ///
    /// - **Type**: `StorageMap<AccountId, bool>`
    /// - **Hasher**: `Blake2_128Concat`
    /// - **Default**: `false` (not frozen)
    /// - **Mutability**: Modified by [`freeze`](Pallet::freeze) / [`unfreeze`](Pallet::unfreeze)
    ///
    /// # Implementation Note
    ///
    /// We store `true` for frozen accounts and use `remove()` to unfreeze,
    /// which is more storage-efficient than storing `false` for all unfrozen accounts.
    #[pallet::storage]
    #[pallet::getter(fn is_frozen)]
    pub type Frozen<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    /// Accounts approved to participate in token transfers.
    ///
    /// The whitelist implements the KYC (Know Your Customer) requirement of ERC-3643.
    /// Both sender and receiver must be whitelisted for a transfer to succeed.
    ///
    /// # ERC-3643 Compliance
    ///
    /// Per the T-REX standard, security tokens must verify investor eligibility:
    ///
    /// > "Transfers SHALL be restricted to verified investors who have been
    /// > validated by an authorized identity registry."
    ///
    /// The whitelist serves as this identity registry in a simplified form.
    ///
    /// # Typical Workflow
    ///
    /// ```text
    /// 1. Investor submits KYC documents off-chain
    /// 2. Compliance officer verifies identity
    /// 3. Admin calls add_to_whitelist(investor)
    /// 4. Investor can now receive/send tokens
    /// ```
    ///
    /// # Storage
    ///
    /// - **Type**: `StorageMap<AccountId, bool>`
    /// - **Hasher**: `Blake2_128Concat`
    /// - **Default**: `false` (not whitelisted)
    /// - **Mutability**: Modified by [`add_to_whitelist`](Pallet::add_to_whitelist) /
    ///   [`remove_from_whitelist`](Pallet::remove_from_whitelist)
    ///
    /// # Security Note
    ///
    /// Removing an account from the whitelist does **not** confiscate their tokens.
    /// They retain their balance but cannot transfer it. To fully remove an investor,
    /// first transfer their tokens to a treasury account, then remove from whitelist.
    ///
    /// # Querying
    ///
    /// ```ignore
    /// // Check if account is whitelisted (JavaScript)
    /// const isWhitelisted = await api.query.cladToken.whitelist(accountId);
    ///
    /// // Rust getter
    /// let is_whitelisted: bool = Pallet::<T>::whitelist(&account);
    /// ```
    #[pallet::storage]
    #[pallet::getter(fn whitelist)]
    pub type Whitelist<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    // ═══════════════════════════════════════════════════════════════════════════
    // EVENTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Events emitted by this pallet.
    ///
    /// Events provide an audit trail for off-chain systems (block explorers, mobile apps,
    /// compliance dashboards) to track token operations. Each event is stored in the
    /// block's event log and can be queried via RPC.
    ///
    /// # Indexing for Off-Chain Systems
    ///
    /// Events are the primary mechanism for off-chain systems to track token activity.
    /// Subscribe to events via WebSocket or poll recent blocks:
    ///
    /// ```text
    /// // JavaScript: Subscribe to all CladToken events
    /// api.query.system.events((events) => {
    ///     events.forEach((record) => {
    ///         if (record.event.section === 'cladToken') {
    ///             console.log(record.event.method, record.event.data);
    ///         }
    ///     });
    /// });
    /// ```
    ///
    /// # Event Categories
    ///
    /// | Category | Events | Use Case |
    /// |----------|--------|----------|
    /// | Transfer | `Transferred`, `Minted` | Balance tracking, portfolio updates |
    /// | Compliance | `Frozen`, `Unfrozen` | Risk monitoring, alerts |
    /// | Access | `Whitelisted`, `RemovedFromWhitelist` | KYC status tracking |
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Tokens were transferred between accounts.
        ///
        /// This event is emitted by [`Pallet::transfer`] when tokens move between
        /// whitelisted, non-frozen accounts.
        ///
        /// # Fields
        ///
        /// - `from`: The sender's account ID (tokens debited)
        /// - `to`: The receiver's account ID (tokens credited)
        /// - `amount`: Number of tokens transferred (raw value, apply decimals for display)
        ///
        /// # Indexing Notes
        ///
        /// - Index by `from` to track outgoing transfers
        /// - Index by `to` to track incoming transfers
        /// - Sum `amount` values to calculate volume metrics
        ///
        /// # Example Event Data
        ///
        /// ```ignore
        /// // Block explorer display
        /// {
        ///     "event": "Transferred",
        ///     "from": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        ///     "to": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        ///     "amount": "1000000000000"  // 1,000,000 tokens with 6 decimals
        /// }
        /// ```
        Transferred {
            /// Account that sent the tokens.
            from: T::AccountId,
            /// Account that received the tokens.
            to: T::AccountId,
            /// Amount of tokens transferred (raw u128 value).
            amount: u128,
        },

        /// New tokens were created and credited to an account.
        ///
        /// This event is emitted by [`Pallet::mint`] when an admin creates new tokens.
        /// The total supply increases by `amount`.
        ///
        /// # Fields
        ///
        /// - `to`: The account receiving newly minted tokens
        /// - `amount`: Number of tokens created (raw value)
        ///
        /// # Compliance Significance
        ///
        /// Minting events represent new token issuance and should be:
        /// - Audited for authorized issuance
        /// - Matched against official bond issuance documents
        /// - Tracked for total supply reconciliation
        ///
        /// # Example
        ///
        /// ```ignore
        /// // Ministry mints $100M bond tokens (6 decimals)
        /// Minted {
        ///     to: ministry_treasury_account,
        ///     amount: 100_000_000_000_000  // 100M * 10^6
        /// }
        /// ```
        Minted {
            /// Account that received the minted tokens.
            to: T::AccountId,
            /// Amount of tokens minted (raw u128 value).
            amount: u128,
        },

        /// An account was frozen and can no longer send transfers.
        ///
        /// This event is emitted by [`Pallet::freeze`] when an admin restricts
        /// an account's ability to transfer tokens.
        ///
        /// # Fields
        ///
        /// - `account`: The account that was frozen
        ///
        /// # Compliance Significance
        ///
        /// Freeze events indicate:
        /// - Regulatory action (sanctions, court order)
        /// - Risk mitigation (suspected fraud)
        /// - Operational control (preventing unauthorized transfers)
        ///
        /// Off-chain systems should trigger alerts when freeze events occur.
        Frozen {
            /// Account that was frozen.
            account: T::AccountId,
        },

        /// A previously frozen account was unfrozen.
        ///
        /// This event is emitted by [`Pallet::unfreeze`] when an admin restores
        /// an account's ability to transfer tokens.
        ///
        /// # Fields
        ///
        /// - `account`: The account that was unfrozen
        Unfrozen {
            /// Account that was unfrozen.
            account: T::AccountId,
        },

        /// An account was added to the whitelist (KYC approved).
        ///
        /// This event is emitted by [`Pallet::add_to_whitelist`] when an admin
        /// approves an account for token transfers.
        ///
        /// # Fields
        ///
        /// - `account`: The newly whitelisted account
        ///
        /// # Workflow Context
        ///
        /// This typically follows successful KYC verification:
        /// 1. Investor submits identity documents off-chain
        /// 2. Compliance team verifies identity
        /// 3. Admin adds account to whitelist
        /// 4. This event is emitted
        /// 5. Investor can now receive/send tokens
        Whitelisted {
            /// Account that was added to the whitelist.
            account: T::AccountId,
        },

        /// An account was removed from the whitelist.
        ///
        /// This event is emitted by [`Pallet::remove_from_whitelist`] when an admin
        /// revokes an account's transfer privileges.
        ///
        /// # Fields
        ///
        /// - `account`: The account removed from whitelist
        ///
        /// # Important Note
        ///
        /// Removing from whitelist does NOT confiscate tokens. The account retains
        /// its balance but cannot transfer it. For full offboarding, transfer tokens
        /// to a treasury account first.
        RemovedFromWhitelist {
            /// Account that was removed from the whitelist.
            account: T::AccountId,
        },
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ERRORS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Errors that can occur when interacting with this pallet.
    ///
    /// Errors are returned when an extrinsic cannot complete successfully.
    /// They provide information about why the operation failed, allowing
    /// callers to handle failures appropriately.
    ///
    /// # Error Handling in Clients
    ///
    /// ```text
    /// // JavaScript: Check for specific errors
    /// try {
    ///     await api.tx.cladToken.transfer(to, amount).signAndSend(sender);
    /// } catch (error) {
    ///     if (error.message.includes('NotWhitelisted')) {
    ///         console.log('Recipient needs KYC approval first');
    ///     } else if (error.message.includes('InsufficientBalance')) {
    ///         console.log('Not enough tokens in account');
    ///     }
    /// }
    /// ```
    #[pallet::error]
    pub enum Error<T> {
        /// The sender does not have enough tokens to complete the transfer.
        ///
        /// # Triggered By
        ///
        /// - [`Pallet::transfer`] when `amount > sender_balance`
        ///
        /// # Resolution
        ///
        /// 1. Check current balance: `api.query.cladToken.balances(account)`
        /// 2. Reduce transfer amount or acquire more tokens
        /// 3. Account for decimals when calculating amounts
        ///
        /// # Example
        ///
        /// ```text
        /// Account balance: 1,000,000 (with 6 decimals = 1.0 tokens)
        /// Transfer amount: 2,000,000 (2.0 tokens)
        /// Result: InsufficientBalance error
        /// ```
        InsufficientBalance,

        /// The sender or receiver is not on the whitelist.
        ///
        /// # Triggered By
        ///
        /// - [`Pallet::transfer`] when sender is not whitelisted
        /// - [`Pallet::transfer`] when receiver is not whitelisted
        ///
        /// # Resolution
        ///
        /// 1. Verify both accounts are whitelisted:
        ///    - `api.query.cladToken.whitelist(sender)`
        ///    - `api.query.cladToken.whitelist(receiver)`
        /// 2. Contact admin to whitelist non-approved accounts
        /// 3. Complete KYC process before requesting whitelist
        ///
        /// # ERC-3643 Context
        ///
        /// This error enforces the identity verification requirement of compliant
        /// security tokens. Both parties must be verified investors.
        NotWhitelisted,

        /// The sender's account is frozen and cannot initiate transfers.
        ///
        /// # Triggered By
        ///
        /// - [`Pallet::transfer`] when sender is frozen
        ///
        /// # Resolution
        ///
        /// 1. Check freeze status: `api.query.cladToken.frozen(account)`
        /// 2. Contact admin to understand why account was frozen
        /// 3. Resolve underlying compliance issue
        /// 4. Request unfreeze via admin
        ///
        /// # Note
        ///
        /// Frozen accounts can still **receive** tokens. Only outgoing transfers
        /// are blocked. This allows court-ordered asset returns while preventing
        /// the frozen party from moving their holdings.
        AccountFrozen,

        /// Arithmetic overflow would occur (balance or supply exceeds u128 max).
        ///
        /// # Triggered By
        ///
        /// - [`Pallet::mint`] when `total_supply + amount > u128::MAX`
        /// - [`Pallet::mint`] when `recipient_balance + amount > u128::MAX`
        /// - [`Pallet::transfer`] when `recipient_balance + amount > u128::MAX`
        ///
        /// # Resolution
        ///
        /// This error is extremely rare in practice (u128 max is ~340 undecillion).
        /// If encountered:
        ///
        /// 1. Review minting amounts for errors (extra zeros?)
        /// 2. Check for bugs in amount calculation logic
        /// 3. Consider using smaller denominations (more decimals)
        ///
        /// # Technical Note
        ///
        /// The pallet uses `checked_add()` to detect overflow before modifying
        /// storage, ensuring no partial state changes occur on overflow.
        Overflow,
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // DISPATCHABLE FUNCTIONS (EXTRINSICS)
    // ═══════════════════════════════════════════════════════════════════════════

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Mint new tokens and credit them to an account.
        ///
        /// Creates `amount` new tokens and adds them to the `to` account's balance.
        /// This increases the total supply by `amount`.
        ///
        /// # Permissions
        ///
        /// **Admin only** - Requires [`Config::AdminOrigin`].
        ///
        /// # Parameters
        ///
        /// | Parameter | Type | Description |
        /// |-----------|------|-------------|
        /// | `origin` | `OriginFor<T>` | Must satisfy `AdminOrigin` |
        /// | `to` | `T::AccountId` | Recipient account for new tokens |
        /// | `amount` | `u128` | Number of tokens to create (raw value) |
        ///
        /// # Events
        ///
        /// - [`Event::Minted`] on success
        ///
        /// # Errors
        ///
        /// - [`Error::Overflow`] if `total_supply + amount > u128::MAX`
        /// - [`Error::Overflow`] if `recipient_balance + amount > u128::MAX`
        /// - `BadOrigin` if caller is not admin
        ///
        /// # Use Cases
        ///
        /// 1. **Initial bond issuance**: Ministry mints total bond value to treasury
        /// 2. **Supplemental issuance**: Additional tokens for reopened bond series
        /// 3. **Error correction**: Minting to compensate for system errors (rare)
        ///
        /// # Example
        ///
        /// ```ignore
        /// // Mint $100M bond tokens (6 decimals) to treasury account
        /// // Raw amount = 100,000,000 * 10^6 = 100_000_000_000_000
        /// CladToken::mint(
        ///     RawOrigin::Root.into(),
        ///     treasury_account,
        ///     100_000_000_000_000
        /// )?;
        /// ```
        ///
        /// # Security Considerations
        ///
        /// - Minting is irreversible; there is no "burn" function
        /// - Verify `amount` calculations carefully (account for decimals)
        /// - Consider multi-sig admin for production deployments
        /// - Log all minting operations for audit trail
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::mint())]
        pub fn mint(origin: OriginFor<T>, to: T::AccountId, amount: u128) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            // Check for overflow in total supply
            let new_supply =
                TotalSupply::<T>::get().checked_add(amount).ok_or(Error::<T>::Overflow)?;

            // Check for overflow in recipient balance
            let new_balance =
                Balances::<T>::get(&to).checked_add(amount).ok_or(Error::<T>::Overflow)?;

            // Apply changes only after all checks pass
            TotalSupply::<T>::put(new_supply);
            Balances::<T>::insert(&to, new_balance);
            Self::deposit_event(Event::Minted { to, amount });
            Ok(())
        }

        /// Transfer tokens from the caller to another account.
        ///
        /// Moves `amount` tokens from the caller's account to the `to` account.
        /// Both accounts must be whitelisted, and the caller must not be frozen.
        ///
        /// # Permissions
        ///
        /// **Signed** - Any account can call, but compliance checks apply.
        ///
        /// # Parameters
        ///
        /// | Parameter | Type | Description |
        /// |-----------|------|-------------|
        /// | `origin` | `OriginFor<T>` | Signed origin (the sender) |
        /// | `to` | `T::AccountId` | Recipient account |
        /// | `amount` | `u128` | Number of tokens to transfer (raw value) |
        ///
        /// # Pre-conditions
        ///
        /// All of the following must be true:
        /// - Sender is whitelisted (KYC approved)
        /// - Receiver is whitelisted (KYC approved)
        /// - Sender is not frozen
        /// - Sender has sufficient balance (`balance >= amount`)
        ///
        /// # Events
        ///
        /// - [`Event::Transferred`] on success
        ///
        /// # Errors
        ///
        /// - [`Error::NotWhitelisted`] if sender or receiver not on whitelist
        /// - [`Error::AccountFrozen`] if sender is frozen
        /// - [`Error::InsufficientBalance`] if sender has less than `amount`
        /// - [`Error::Overflow`] if receiver balance would overflow (extremely rare)
        ///
        /// # Use Cases
        ///
        /// 1. **Primary distribution**: Treasury transfers to institutional investors
        /// 2. **Secondary trading**: Investors trade tokens among themselves
        /// 3. **Settlement**: Off-chain OTC trades settled on-chain
        ///
        /// # Example
        ///
        /// ```ignore
        /// // Transfer 1,000 tokens (6 decimals) to another investor
        /// // Raw amount = 1,000 * 10^6 = 1_000_000_000
        /// CladToken::transfer(
        ///     RuntimeOrigin::signed(sender_account),
        ///     receiver_account,
        ///     1_000_000_000
        /// )?;
        /// ```
        ///
        /// # Self-Transfer
        ///
        /// Transferring to yourself (`sender == to`) is allowed and emits a
        /// `Transferred` event, but does not modify balances. This can be used
        /// for accounting purposes or to verify account status.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::transfer())]
        pub fn transfer(origin: OriginFor<T>, to: T::AccountId, amount: u128) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(Whitelist::<T>::get(&sender), Error::<T>::NotWhitelisted);
            ensure!(Whitelist::<T>::get(&to), Error::<T>::NotWhitelisted);
            ensure!(!Frozen::<T>::get(&sender), Error::<T>::AccountFrozen);

            let sender_balance = Balances::<T>::get(&sender);
            ensure!(sender_balance >= amount, Error::<T>::InsufficientBalance);

            // Handle self-transfer: no overflow check needed, balance unchanged
            if sender == to {
                Self::deposit_event(Event::Transferred { from: sender, to, amount });
                return Ok(());
            }

            // Check for overflow in receiver balance (defensive - should not happen with capped supply)
            let new_receiver_balance =
                Balances::<T>::get(&to).checked_add(amount).ok_or(Error::<T>::Overflow)?;

            // Apply changes only after all checks pass
            Balances::<T>::insert(&sender, sender_balance - amount);
            Balances::<T>::insert(&to, new_receiver_balance);
            Self::deposit_event(Event::Transferred { from: sender, to, amount });
            Ok(())
        }

        /// Freeze an account, preventing it from sending transfers.
        ///
        /// Frozen accounts retain their balance and can still receive tokens,
        /// but cannot initiate outgoing transfers until unfrozen.
        ///
        /// # Permissions
        ///
        /// **Admin only** - Requires [`Config::AdminOrigin`].
        ///
        /// # Parameters
        ///
        /// | Parameter | Type | Description |
        /// |-----------|------|-------------|
        /// | `origin` | `OriginFor<T>` | Must satisfy `AdminOrigin` |
        /// | `account` | `T::AccountId` | Account to freeze |
        ///
        /// # Events
        ///
        /// - [`Event::Frozen`] on success
        ///
        /// # Errors
        ///
        /// - `BadOrigin` if caller is not admin
        ///
        /// # Use Cases
        ///
        /// 1. **Sanctions compliance**: Freeze accounts matching sanctions lists
        /// 2. **Fraud prevention**: Halt transfers during investigation
        /// 3. **Legal hold**: Preserve assets per court order
        /// 4. **Account recovery**: Prevent unauthorized transfers after key compromise
        ///
        /// # Example
        ///
        /// ```ignore
        /// // Freeze a suspicious account pending investigation
        /// CladToken::freeze(RawOrigin::Root.into(), suspicious_account)?;
        /// ```
        ///
        /// # Idempotency
        ///
        /// Freezing an already-frozen account is a no-op (succeeds without error).
        /// This simplifies batch operations and retry logic.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::freeze())]
        pub fn freeze(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            Frozen::<T>::insert(&account, true);
            Self::deposit_event(Event::Frozen { account });
            Ok(())
        }

        /// Unfreeze an account, restoring its ability to send transfers.
        ///
        /// Removes the freeze flag from an account, allowing it to resume
        /// normal transfer operations (assuming it remains whitelisted).
        ///
        /// # Permissions
        ///
        /// **Admin only** - Requires [`Config::AdminOrigin`].
        ///
        /// # Parameters
        ///
        /// | Parameter | Type | Description |
        /// |-----------|------|-------------|
        /// | `origin` | `OriginFor<T>` | Must satisfy `AdminOrigin` |
        /// | `account` | `T::AccountId` | Account to unfreeze |
        ///
        /// # Events
        ///
        /// - [`Event::Unfrozen`] on success
        ///
        /// # Errors
        ///
        /// - `BadOrigin` if caller is not admin
        ///
        /// # Use Cases
        ///
        /// 1. **Investigation cleared**: Restore access after compliance review
        /// 2. **Sanctions delisted**: Account no longer on restricted lists
        /// 3. **Legal release**: Court order lifted
        ///
        /// # Example
        ///
        /// ```ignore
        /// // Unfreeze account after compliance review
        /// CladToken::unfreeze(RawOrigin::Root.into(), cleared_account)?;
        /// ```
        ///
        /// # Idempotency
        ///
        /// Unfreezing a non-frozen account is a no-op (succeeds without error).
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::unfreeze())]
        pub fn unfreeze(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            Frozen::<T>::remove(&account);
            Self::deposit_event(Event::Unfrozen { account });
            Ok(())
        }

        /// Add an account to the whitelist, allowing it to participate in transfers.
        ///
        /// Whitelisting represents KYC (Know Your Customer) approval. Only whitelisted
        /// accounts can send or receive tokens, enforcing the identity verification
        /// requirement of ERC-3643 compliant security tokens.
        ///
        /// # Permissions
        ///
        /// **Admin only** - Requires [`Config::AdminOrigin`].
        ///
        /// # Parameters
        ///
        /// | Parameter | Type | Description |
        /// |-----------|------|-------------|
        /// | `origin` | `OriginFor<T>` | Must satisfy `AdminOrigin` |
        /// | `account` | `T::AccountId` | Account to whitelist |
        ///
        /// # Events
        ///
        /// - [`Event::Whitelisted`] on success
        ///
        /// # Errors
        ///
        /// - `BadOrigin` if caller is not admin
        ///
        /// # Use Cases
        ///
        /// 1. **KYC approval**: Approve investor after identity verification
        /// 2. **Institutional onboarding**: Add new institutional investors
        /// 3. **Treasury setup**: Whitelist ministry/issuer accounts
        ///
        /// # Typical Workflow
        ///
        /// ```text
        /// 1. Investor submits KYC documents via off-chain process
        /// 2. Compliance team verifies identity and eligibility
        /// 3. Admin adds investor to whitelist
        /// 4. Investor can now receive tokens from treasury
        /// 5. Investor can trade with other whitelisted accounts
        /// ```
        ///
        /// # Example
        ///
        /// ```ignore
        /// // Whitelist a new institutional investor
        /// CladToken::add_to_whitelist(RawOrigin::Root.into(), investor_account)?;
        /// ```
        ///
        /// # Idempotency
        ///
        /// Whitelisting an already-whitelisted account is a no-op.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::add_to_whitelist())]
        pub fn add_to_whitelist(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            Whitelist::<T>::insert(&account, true);
            Self::deposit_event(Event::Whitelisted { account });
            Ok(())
        }

        /// Remove an account from the whitelist, preventing it from participating in transfers.
        ///
        /// The account will no longer be able to send or receive tokens. However,
        /// any existing balance is preserved—tokens are not confiscated.
        ///
        /// # Permissions
        ///
        /// **Admin only** - Requires [`Config::AdminOrigin`].
        ///
        /// # Parameters
        ///
        /// | Parameter | Type | Description |
        /// |-----------|------|-------------|
        /// | `origin` | `OriginFor<T>` | Must satisfy `AdminOrigin` |
        /// | `account` | `T::AccountId` | Account to remove from whitelist |
        ///
        /// # Events
        ///
        /// - [`Event::RemovedFromWhitelist`] on success
        ///
        /// # Errors
        ///
        /// - `BadOrigin` if caller is not admin
        ///
        /// # Use Cases
        ///
        /// 1. **KYC expiration**: Remove investors with expired verification
        /// 2. **Voluntary exit**: Investor requests removal from platform
        /// 3. **Compliance failure**: Investor no longer meets eligibility criteria
        ///
        /// # Important: Token Preservation
        ///
        /// Removing from whitelist does **NOT** confiscate tokens. The account
        /// retains its balance but cannot move it. For full offboarding:
        ///
        /// ```text
        /// 1. Coordinate with investor to transfer tokens to treasury
        /// 2. Remove account from whitelist
        /// 3. Process any fiat redemption off-chain
        /// ```
        ///
        /// # Example
        ///
        /// ```ignore
        /// // Remove investor with expired KYC
        /// CladToken::remove_from_whitelist(RawOrigin::Root.into(), expired_investor)?;
        /// ```
        ///
        /// # Idempotency
        ///
        /// Removing a non-whitelisted account is a no-op.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::remove_from_whitelist())]
        pub fn remove_from_whitelist(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            Whitelist::<T>::remove(&account);
            Self::deposit_event(Event::RemovedFromWhitelist { account });
            Ok(())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // GENESIS CONFIGURATION
    // ═══════════════════════════════════════════════════════════════════════════

    /// Genesis configuration for the Clad Token pallet.
    ///
    /// This struct defines the initial state of the token when the chain launches.
    /// It is typically configured in the chain spec file (`chain_spec.rs`) and
    /// applied during chain genesis.
    ///
    /// # Overview
    ///
    /// The genesis configuration allows you to:
    /// - Set token metadata (name, symbol, decimals)
    /// - Designate an admin account
    /// - Pre-whitelist accounts for transfers
    /// - Distribute initial token balances
    ///
    /// # Example Configuration (Rust)
    ///
    /// ```ignore
    /// // In chain_spec.rs
    /// use pallet_clad_token::GenesisConfig as CladTokenConfig;
    ///
    /// fn testnet_genesis() -> RuntimeGenesisConfig {
    ///     RuntimeGenesisConfig {
    ///         clad_token: CladTokenConfig {
    ///             admin: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
    ///             token_name: b"Kazakhstan Sovereign Bond 2030".to_vec(),
    ///             token_symbol: b"KZT-BOND-2030".to_vec(),
    ///             decimals: 6,
    ///             whitelisted_accounts: vec![
    ///                 get_account_id_from_seed::<sr25519::Public>("Alice"),
    ///                 get_account_id_from_seed::<sr25519::Public>("Bob"),
    ///             ],
    ///             initial_balances: vec![
    ///                 // Mint $100M to treasury (Alice)
    ///                 (get_account_id_from_seed::<sr25519::Public>("Alice"), 100_000_000_000_000),
    ///             ],
    ///         },
    ///         // ... other pallets
    ///     }
    /// }
    /// ```
    ///
    /// # Example Configuration (JSON Chain Spec)
    ///
    /// ```json
    /// {
    ///   "cladToken": {
    ///     "admin": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ///     "tokenName": "0x4b617a616b687374616e20536f7665726569676e20426f6e642032303330",
    ///     "tokenSymbol": "0x4b5a542d424f4e442d32303330",
    ///     "decimals": 6,
    ///     "whitelistedAccounts": [
    ///       "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ///       "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
    ///     ],
    ///     "initialBalances": [
    ///       ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 100000000000000]
    ///     ]
    ///   }
    /// }
    /// ```
    ///
    /// # Validation
    ///
    /// The genesis build will **panic** if:
    /// - `token_name` exceeds 64 bytes
    /// - `token_symbol` exceeds 16 bytes
    ///
    /// Always verify your configuration in a test environment before mainnet deployment.
    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// Optional admin account to be auto-whitelisted at genesis.
        ///
        /// If provided, this account will be added to the whitelist automatically,
        /// enabling it to receive tokens immediately. This is typically the
        /// ministry treasury or primary issuer account.
        ///
        /// # Note
        ///
        /// This does NOT grant admin privileges for extrinsics—that is controlled
        /// by [`Config::AdminOrigin`]. This only auto-whitelists the account.
        pub admin: Option<T::AccountId>,

        /// Human-readable token name.
        ///
        /// Must be 64 bytes or fewer (UTF-8 encoded).
        ///
        /// # Examples
        /// - `b"Kazakhstan Sovereign Bond 2030".to_vec()`
        /// - `b"Malaysia Sukuk Token 2027".to_vec()`
        pub token_name: Vec<u8>,

        /// Token trading symbol.
        ///
        /// Must be 16 bytes or fewer (UTF-8 encoded).
        ///
        /// # Examples
        /// - `b"KZT-BOND-2030".to_vec()`
        /// - `b"MYS-SUKUK-27".to_vec()`
        pub token_symbol: Vec<u8>,

        /// Number of decimal places for display purposes.
        ///
        /// Common values:
        /// - `6`: USDC/USDT style (recommended for bonds)
        /// - `18`: Ethereum-native compatibility
        /// - `2`: Traditional currency display
        pub decimals: u8,

        /// Accounts to whitelist at genesis.
        ///
        /// These accounts will be able to send/receive tokens immediately
        /// after chain launch. Typically includes:
        /// - Treasury/issuer accounts
        /// - Initial institutional investors
        /// - Market makers
        ///
        /// # Note
        ///
        /// The admin account (if provided) is automatically whitelisted
        /// and does not need to be included here.
        pub whitelisted_accounts: Vec<T::AccountId>,

        /// Initial token distribution as (account, amount) pairs.
        ///
        /// These balances are minted at genesis. The total supply is
        /// calculated as the sum of all amounts.
        ///
        /// # Amount Calculation
        ///
        /// Remember to account for decimals:
        /// - For $100M with 6 decimals: `100_000_000 * 10^6 = 100_000_000_000_000`
        /// - For $1B with 6 decimals: `1_000_000_000 * 10^6 = 1_000_000_000_000_000`
        ///
        /// # Important
        ///
        /// Accounts in this list are NOT automatically whitelisted.
        /// Make sure to also add them to `whitelisted_accounts` or
        /// specify an `admin` if the recipient should be able to transfer tokens.
        pub initial_balances: Vec<(T::AccountId, u128)>,
    }

    /// Genesis build implementation.
    ///
    /// This runs once when the chain is initialized, populating storage
    /// with the values from [`GenesisConfig`].
    ///
    /// # Initialization Order
    ///
    /// 1. Set token metadata (name, symbol, decimals)
    /// 2. Whitelist admin account (if provided)
    /// 3. Whitelist additional accounts
    /// 4. Mint initial balances
    /// 5. Calculate and set total supply
    ///
    /// # Panics
    ///
    /// - If `token_name` exceeds 64 bytes
    /// - If `token_symbol` exceeds 16 bytes
    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Set token metadata
            let name: BoundedVec<u8, ConstU32<64>> =
                self.token_name.clone().try_into().expect("Token name too long (max 64 bytes)");
            TokenName::<T>::put(name);

            let symbol: BoundedVec<u8, ConstU32<16>> =
                self.token_symbol.clone().try_into().expect("Token symbol too long (max 16 bytes)");
            TokenSymbol::<T>::put(symbol);

            Decimals::<T>::put(self.decimals);

            // Whitelist admin if provided
            if let Some(ref admin) = self.admin {
                Whitelist::<T>::insert(admin, true);
            }

            // Whitelist specified accounts
            for account in &self.whitelisted_accounts {
                Whitelist::<T>::insert(account, true);
            }

            // Mint initial balances
            let mut total: u128 = 0;
            for (account, amount) in &self.initial_balances {
                Balances::<T>::insert(account, amount);
                total = total.saturating_add(*amount);
            }
            TotalSupply::<T>::put(total);
        }
    }
}
