//! Benchmarking setup for pallet-clad-token

use super::*;

#[allow(unused)]
use crate::Pallet as CladToken;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn mint() {
        let recipient: T::AccountId = account("recipient", 0, 0);
        let amount: u128 = 1_000_000;
        let origin = T::AdminOrigin::try_successful_origin().expect("Admin origin");

        #[extrinsic_call]
        _(origin as T::RuntimeOrigin, recipient.clone(), amount);

        assert_eq!(Balances::<T>::get(&recipient), amount);
    }

    #[benchmark]
    fn transfer() {
        let caller: T::AccountId = whitelisted_caller();
        let recipient: T::AccountId = account("recipient", 0, 0);
        let amount: u128 = 1_000_000;

        // Setup: whitelist both accounts and give caller balance
        Whitelist::<T>::insert(&caller, true);
        Whitelist::<T>::insert(&recipient, true);
        Balances::<T>::insert(&caller, 10_000_000);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), recipient.clone(), amount);

        assert_eq!(Balances::<T>::get(&recipient), amount);
    }

    #[benchmark]
    fn freeze() {
        let account: T::AccountId = whitelisted_caller();
        let origin = T::AdminOrigin::try_successful_origin().expect("Admin origin");

        #[extrinsic_call]
        _(origin as T::RuntimeOrigin, account.clone());

        assert_eq!(Frozen::<T>::get(&account), true);
    }

    #[benchmark]
    fn unfreeze() {
        let account: T::AccountId = whitelisted_caller();
        Frozen::<T>::insert(&account, true);
        let origin = T::AdminOrigin::try_successful_origin().expect("Admin origin");

        #[extrinsic_call]
        _(origin as T::RuntimeOrigin, account.clone());

        assert_eq!(Frozen::<T>::get(&account), false);
    }

    #[benchmark]
    fn add_to_whitelist() {
        let account: T::AccountId = whitelisted_caller();
        let origin = T::AdminOrigin::try_successful_origin().expect("Admin origin");

        #[extrinsic_call]
        _(origin as T::RuntimeOrigin, account.clone());

        assert_eq!(Whitelist::<T>::get(&account), true);
    }

    #[benchmark]
    fn remove_from_whitelist() {
        let account: T::AccountId = whitelisted_caller();
        Whitelist::<T>::insert(&account, true);
        let origin = T::AdminOrigin::try_successful_origin().expect("Admin origin");

        #[extrinsic_call]
        _(origin as T::RuntimeOrigin, account.clone());

        assert_eq!(Whitelist::<T>::get(&account), false);
    }

    impl_benchmark_test_suite!(CladToken, crate::mock::new_test_ext(), crate::mock::Test);
}
