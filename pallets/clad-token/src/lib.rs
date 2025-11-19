#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    ensure,
    pallet_prelude::*,
    traits::EnsureOrigin,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_std::prelude::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn total_supply)]
    pub type TotalSupply<T> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn balance_of)]
    pub type Balances<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn is_frozen)]
    pub type Frozen<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn whitelist)]
    pub type Whitelist<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Transferred { from: T::AccountId, to: T::AccountId, amount: u128 },
        Minted { to: T::AccountId, amount: u128 },
        Frozen { account: T::AccountId },
        Unfrozen { account: T::AccountId },
        Whitelisted { account: T::AccountId },
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
    }
}