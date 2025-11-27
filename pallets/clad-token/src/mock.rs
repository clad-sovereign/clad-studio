//! Mock runtime for pallet-clad-token tests.
//!
//! This module provides a minimal mock runtime configuration for testing the
//! CladToken pallet in isolation. It sets up a test environment with:
//!
//! # Test Fixtures
//!
//! ## Accounts
//! - **Account 1**: Admin account with `AdminOrigin` privileges (can mint, freeze, whitelist)
//! - **Account 2**: Whitelisted user with 1,000,000 tokens initial balance
//! - **Account 3**: Whitelisted user with 500,000 tokens initial balance
//! - **Accounts 4+**: Not whitelisted, zero balance (use for testing non-whitelisted scenarios)
//!
//! ## Initial State (via `new_test_ext()`)
//! - Token name: "Test Token"
//! - Token symbol: "TST"
//! - Decimals: 6
//! - Total supply: 1,500,000 (sum of account 2 and 3 balances)
//! - Whitelisted accounts: 1 (admin), 2, 3
//! - Frozen accounts: none
//!
//! # Example Usage
//! ```ignore
//! #[test]
//! fn my_test() {
//!     new_test_ext().execute_with(|| {
//!         // Account 2 has 1_000_000 tokens and is whitelisted
//!         assert_eq!(CladToken::balance_of(&2), 1_000_000);
//!         // Account 1 is admin and can mint
//!         assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 5, 1000));
//!     });
//! }
//! ```

use crate as pallet_clad_token;
use frame_support::{
    derive_impl, parameter_types,
    traits::{ConstU32, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        CladToken: pallet_clad_token,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const AdminAccount: u64 = 1;
}

pub struct EnsureAdmin;
impl frame_support::traits::EnsureOrigin<RuntimeOrigin> for EnsureAdmin {
    type Success = u64;

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        match o.clone().into() {
            Ok(frame_system::RawOrigin::Signed(account)) if account == AdminAccount::get() => {
                Ok(account)
            }
            _ => Err(o),
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Ok(RuntimeOrigin::signed(AdminAccount::get()))
    }
}

impl pallet_clad_token::Config for Test {
    type AdminOrigin = EnsureAdmin;
    type WeightInfo = ();
}

/// Build genesis storage with standard test fixtures.
///
/// Creates a test environment with:
/// - Admin (account 1) whitelisted
/// - Accounts 2 and 3 whitelisted with initial balances
/// - Token metadata: "Test Token" / "TST" / 6 decimals
///
/// See module-level documentation for detailed fixture information.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

    pallet_clad_token::GenesisConfig::<Test> {
        admin: Some(AdminAccount::get()),
        token_name: b"Test Token".to_vec(),
        token_symbol: b"TST".to_vec(),
        decimals: 6,
        whitelisted_accounts: vec![2, 3],
        initial_balances: vec![(2, 1_000_000), (3, 500_000)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
