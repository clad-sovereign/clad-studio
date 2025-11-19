// Allow clippy warnings for test code (bool assertions and borrows are fine here)
#![allow(clippy::bool_assert_comparison, clippy::needless_borrows_for_generic_args)]

use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn genesis_config_works() {
    new_test_ext().execute_with(|| {
        // Check token metadata
        assert_eq!(CladToken::token_name(), b"Test Token".to_vec());
        assert_eq!(CladToken::token_symbol(), b"TST".to_vec());
        assert_eq!(CladToken::decimals(), 6);

        // Check admin is whitelisted
        assert_eq!(CladToken::whitelist(&1), true);

        // Check initial balances
        assert_eq!(CladToken::balance_of(&2), 1_000_000);
        assert_eq!(CladToken::balance_of(&3), 500_000);
        assert_eq!(CladToken::total_supply(), 1_500_000);

        // Check whitelisted accounts
        assert_eq!(CladToken::whitelist(&2), true);
        assert_eq!(CladToken::whitelist(&3), true);
    });
}

#[test]
fn mint_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Admin (account 1) can mint
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 5, 10_000));

        // Check balance and total supply updated
        assert_eq!(CladToken::balance_of(&5), 10_000);
        assert_eq!(CladToken::total_supply(), 1_510_000);

        // Check event emitted
        System::assert_last_event(Event::Minted { to: 5, amount: 10_000 }.into());
    });
}

#[test]
fn mint_fails_for_non_admin() {
    new_test_ext().execute_with(|| {
        // Non-admin (account 2) cannot mint
        assert_noop!(
            CladToken::mint(RuntimeOrigin::signed(2), 5, 10_000),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn transfer_works_for_whitelisted_accounts() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Account 2 -> Account 3 transfer (both whitelisted)
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(2), 3, 100_000));

        // Check balances updated
        assert_eq!(CladToken::balance_of(&2), 900_000);
        assert_eq!(CladToken::balance_of(&3), 600_000);

        // Check event emitted
        System::assert_last_event(Event::Transferred { from: 2, to: 3, amount: 100_000 }.into());
    });
}

#[test]
fn transfer_fails_when_sender_not_whitelisted() {
    new_test_ext().execute_with(|| {
        // Mint tokens to non-whitelisted account 5
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 5, 10_000));

        // Account 5 (not whitelisted) cannot transfer
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(5), 2, 5_000),
            Error::<Test>::NotWhitelisted
        );
    });
}

#[test]
fn transfer_fails_when_receiver_not_whitelisted() {
    new_test_ext().execute_with(|| {
        // Account 2 (whitelisted) cannot transfer to account 5 (not whitelisted)
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(2), 5, 5_000),
            Error::<Test>::NotWhitelisted
        );
    });
}

#[test]
fn transfer_fails_when_sender_frozen() {
    new_test_ext().execute_with(|| {
        // Freeze account 2
        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(1), 2));

        // Frozen account 2 cannot transfer
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(2), 3, 5_000),
            Error::<Test>::AccountFrozen
        );
    });
}

#[test]
fn transfer_fails_with_insufficient_balance() {
    new_test_ext().execute_with(|| {
        // Account 2 tries to transfer more than balance
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(2), 3, 2_000_000),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn freeze_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Admin freezes account 2
        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(1), 2));

        // Check account is frozen
        assert_eq!(CladToken::is_frozen(&2), true);

        // Check event emitted
        System::assert_last_event(Event::Frozen { account: 2 }.into());
    });
}

#[test]
fn freeze_fails_for_non_admin() {
    new_test_ext().execute_with(|| {
        // Non-admin cannot freeze
        assert_noop!(
            CladToken::freeze(RuntimeOrigin::signed(2), 3),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn unfreeze_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Admin freezes then unfreezes account 2
        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(1), 2));
        assert_eq!(CladToken::is_frozen(&2), true);

        assert_ok!(CladToken::unfreeze(RuntimeOrigin::signed(1), 2));
        assert_eq!(CladToken::is_frozen(&2), false);

        // Check event emitted
        System::assert_last_event(Event::Unfrozen { account: 2 }.into());

        // Account 2 can transfer again
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(2), 3, 10_000));
    });
}

#[test]
fn unfreeze_fails_for_non_admin() {
    new_test_ext().execute_with(|| {
        // Non-admin cannot unfreeze
        assert_noop!(
            CladToken::unfreeze(RuntimeOrigin::signed(2), 3),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn add_to_whitelist_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Admin adds account 5 to whitelist
        assert_ok!(CladToken::add_to_whitelist(RuntimeOrigin::signed(1), 5));

        // Check account is whitelisted
        assert_eq!(CladToken::whitelist(&5), true);

        // Check event emitted
        System::assert_last_event(Event::Whitelisted { account: 5 }.into());
    });
}

#[test]
fn add_to_whitelist_fails_for_non_admin() {
    new_test_ext().execute_with(|| {
        // Non-admin cannot whitelist
        assert_noop!(
            CladToken::add_to_whitelist(RuntimeOrigin::signed(2), 5),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn remove_from_whitelist_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Admin removes account 2 from whitelist
        assert_ok!(CladToken::remove_from_whitelist(RuntimeOrigin::signed(1), 2));

        // Check account is not whitelisted
        assert_eq!(CladToken::whitelist(&2), false);

        // Check event emitted
        System::assert_last_event(Event::RemovedFromWhitelist { account: 2 }.into());

        // Account 2 can no longer transfer
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(2), 3, 5_000),
            Error::<Test>::NotWhitelisted
        );
    });
}

#[test]
fn remove_from_whitelist_fails_for_non_admin() {
    new_test_ext().execute_with(|| {
        // Non-admin cannot remove from whitelist
        assert_noop!(
            CladToken::remove_from_whitelist(RuntimeOrigin::signed(2), 3),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn whitelisted_account_can_transfer_after_being_added() {
    new_test_ext().execute_with(|| {
        // Mint tokens to account 5 (not whitelisted yet)
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 5, 50_000));

        // Add accounts 5 and 6 to whitelist
        assert_ok!(CladToken::add_to_whitelist(RuntimeOrigin::signed(1), 5));
        assert_ok!(CladToken::add_to_whitelist(RuntimeOrigin::signed(1), 6));

        // Now account 5 can transfer to account 6
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(5), 6, 10_000));

        assert_eq!(CladToken::balance_of(&5), 40_000);
        assert_eq!(CladToken::balance_of(&6), 10_000);
    });
}

#[test]
fn account_can_receive_transfer_when_frozen() {
    new_test_ext().execute_with(|| {
        // Freeze account 3
        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(1), 3));

        // Account 2 can still send to frozen account 3
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(2), 3, 10_000));

        assert_eq!(CladToken::balance_of(&3), 510_000);
    });
}

#[test]
fn multiple_transfers_work_correctly() {
    new_test_ext().execute_with(|| {
        // Multiple transfers
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(2), 3, 100_000));
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(3), 2, 50_000));
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(2), 3, 25_000));

        // Final balances
        assert_eq!(CladToken::balance_of(&2), 925_000);
        assert_eq!(CladToken::balance_of(&3), 575_000);
        assert_eq!(CladToken::total_supply(), 1_500_000); // Total unchanged
    });
}

#[test]
fn minting_increases_total_supply() {
    new_test_ext().execute_with(|| {
        let initial_supply = CladToken::total_supply();

        // Mint multiple times
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 5, 100_000));
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 6, 200_000));

        assert_eq!(CladToken::total_supply(), initial_supply + 300_000);
    });
}
