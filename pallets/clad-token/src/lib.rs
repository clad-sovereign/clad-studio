#![cfg_attr(not(feature = "std"), no_std)]
// Allow deprecated weight constants for MVP (will be replaced by benchmarks post-grant)
#![allow(deprecated)]
#![allow(clippy::let_unit_value)]

use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, traits::EnsureOrigin};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Token name (e.g., "Sovereign Bond Token")
    #[pallet::storage]
    #[pallet::getter(fn token_name)]
    pub type TokenName<T> = StorageValue<_, BoundedVec<u8, ConstU32<64>>, ValueQuery>;

    /// Token symbol (e.g., "SBT")
    #[pallet::storage]
    #[pallet::getter(fn token_symbol)]
    pub type TokenSymbol<T> = StorageValue<_, BoundedVec<u8, ConstU32<16>>, ValueQuery>;

    /// Token decimals (e.g., 6 for USDC-style, 18 for ETH-style)
    #[pallet::storage]
    #[pallet::getter(fn decimals)]
    pub type Decimals<T> = StorageValue<_, u8, ValueQuery>;

    /// Total token supply
    #[pallet::storage]
    #[pallet::getter(fn total_supply)]
    pub type TotalSupply<T> = StorageValue<_, u128, ValueQuery>;

    /// Account balances
    #[pallet::storage]
    #[pallet::getter(fn balance_of)]
    pub type Balances<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u128, ValueQuery>;

    /// Frozen accounts (cannot send transfers)
    #[pallet::storage]
    #[pallet::getter(fn is_frozen)]
    pub type Frozen<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    /// Whitelisted accounts (can send/receive transfers)
    #[pallet::storage]
    #[pallet::getter(fn whitelist)]
    pub type Whitelist<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Tokens transferred from one account to another
        Transferred { from: T::AccountId, to: T::AccountId, amount: u128 },
        /// New tokens minted
        Minted { to: T::AccountId, amount: u128 },
        /// Account frozen (cannot send transfers)
        Frozen { account: T::AccountId },
        /// Account unfrozen
        Unfrozen { account: T::AccountId },
        /// Account added to whitelist
        Whitelisted { account: T::AccountId },
        /// Account removed from whitelist
        RemovedFromWhitelist { account: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        InsufficientBalance,
        NotWhitelisted,
        AccountFrozen,
        Overflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn mint(origin: OriginFor<T>, to: T::AccountId, amount: u128) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            TotalSupply::<T>::mutate(|supply| *supply += amount);
            Balances::<T>::mutate(to.clone(), |bal| *bal += amount);
            Self::deposit_event(Event::Minted { to, amount });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn transfer(origin: OriginFor<T>, to: T::AccountId, amount: u128) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(Whitelist::<T>::get(&sender), Error::<T>::NotWhitelisted);
            ensure!(Whitelist::<T>::get(&to), Error::<T>::NotWhitelisted);
            ensure!(!Frozen::<T>::get(&sender), Error::<T>::AccountFrozen);
            ensure!(Balances::<T>::get(&sender) >= amount, Error::<T>::InsufficientBalance);

            Balances::<T>::mutate(&sender, |bal| *bal -= amount);
            Balances::<T>::mutate(to.clone(), |bal| *bal += amount);
            Self::deposit_event(Event::Transferred { from: sender, to, amount });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn freeze(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            Frozen::<T>::insert(&account, true);
            Self::deposit_event(Event::Frozen { account });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn unfreeze(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            Frozen::<T>::remove(&account);
            Self::deposit_event(Event::Unfrozen { account });
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn add_to_whitelist(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            Whitelist::<T>::insert(&account, true);
            Self::deposit_event(Event::Whitelisted { account });
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
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

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// Initial admin account (will be whitelisted by default)
        pub admin: Option<T::AccountId>,
        /// Token name
        pub token_name: Vec<u8>,
        /// Token symbol
        pub token_symbol: Vec<u8>,
        /// Token decimals
        pub decimals: u8,
        /// Accounts to whitelist at genesis
        pub whitelisted_accounts: Vec<T::AccountId>,
        /// Initial token mints (account, amount)
        pub initial_balances: Vec<(T::AccountId, u128)>,
    }

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
