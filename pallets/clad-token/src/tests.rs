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

// ============================================================================
// Edge Case Tests
// ============================================================================

/// Tests that minting zero tokens works correctly.
///
/// Zero-amount mints are intentionally allowed and emit events. This follows
/// ERC-20/ERC-3643 standards and enables:
/// - Triggering compliance hooks without actual token movement
/// - Complete audit trails for all admin actions
/// - Identity verification workflows (proving control of an account)
#[test]
fn mint_zero_amount_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let initial_supply = CladToken::total_supply();
        let initial_balance = CladToken::balance_of(&5);

        // Mint zero tokens
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 5, 0));

        // Supply and balance should be unchanged
        assert_eq!(CladToken::total_supply(), initial_supply);
        assert_eq!(CladToken::balance_of(&5), initial_balance);

        // Event should still be emitted
        System::assert_last_event(Event::Minted { to: 5, amount: 0 }.into());
    });
}

/// Tests that freezing an already frozen account succeeds idempotently.
/// This is valid behavior - re-freezing should not error.
#[test]
fn freeze_already_frozen_account_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Freeze account 2
        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(1), 2));
        assert_eq!(CladToken::is_frozen(&2), true);

        // Freeze again - should succeed
        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(1), 2));
        assert_eq!(CladToken::is_frozen(&2), true);

        // Event should be emitted for second freeze too
        System::assert_last_event(Event::Frozen { account: 2 }.into());
    });
}

/// Tests that unfreezing a non-frozen account succeeds idempotently.
/// This is valid behavior - unfreezing a non-frozen account should not error.
#[test]
fn unfreeze_non_frozen_account_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Account 2 is not frozen initially
        assert_eq!(CladToken::is_frozen(&2), false);

        // Unfreeze anyway - should succeed
        assert_ok!(CladToken::unfreeze(RuntimeOrigin::signed(1), 2));
        assert_eq!(CladToken::is_frozen(&2), false);

        // Event should be emitted
        System::assert_last_event(Event::Unfrozen { account: 2 }.into());
    });
}

/// Tests that whitelisting an already whitelisted account succeeds idempotently.
/// Re-whitelisting is valid and should not error.
#[test]
fn whitelist_already_whitelisted_account_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Account 2 is already whitelisted in genesis
        assert_eq!(CladToken::whitelist(&2), true);

        // Whitelist again - should succeed
        assert_ok!(CladToken::add_to_whitelist(RuntimeOrigin::signed(1), 2));
        assert_eq!(CladToken::whitelist(&2), true);

        // Event should be emitted
        System::assert_last_event(Event::Whitelisted { account: 2 }.into());
    });
}

/// Tests that removing a non-whitelisted account from whitelist succeeds idempotently.
/// Removing a non-whitelisted account should not error.
#[test]
fn remove_non_whitelisted_account_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Account 5 is not whitelisted
        assert_eq!(CladToken::whitelist(&5), false);

        // Remove anyway - should succeed
        assert_ok!(CladToken::remove_from_whitelist(RuntimeOrigin::signed(1), 5));
        assert_eq!(CladToken::whitelist(&5), false);

        // Event should be emitted
        System::assert_last_event(Event::RemovedFromWhitelist { account: 5 }.into());
    });
}

/// Tests that transferring zero tokens works correctly.
///
/// Zero-amount transfers are intentionally allowed and emit events. This follows
/// ERC-20/ERC-3643 standards and is useful for:
/// - Triggering transfer hooks in compliance systems
/// - Proving account ownership/control
/// - Maintaining complete audit trails
#[test]
fn transfer_zero_amount_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let initial_sender_balance = CladToken::balance_of(&2);
        let initial_receiver_balance = CladToken::balance_of(&3);

        // Transfer zero tokens
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(2), 3, 0));

        // Balances should be unchanged
        assert_eq!(CladToken::balance_of(&2), initial_sender_balance);
        assert_eq!(CladToken::balance_of(&3), initial_receiver_balance);

        // Event should be emitted
        System::assert_last_event(Event::Transferred { from: 2, to: 3, amount: 0 }.into());
    });
}

/// Tests that an account can transfer tokens to itself.
/// Self-transfers are valid and should work correctly.
#[test]
fn self_transfer_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let initial_balance = CladToken::balance_of(&2);

        // Transfer to self
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(2), 2, 100_000));

        // Balance should be unchanged (sent and received same amount)
        assert_eq!(CladToken::balance_of(&2), initial_balance);

        // Event should be emitted
        System::assert_last_event(Event::Transferred { from: 2, to: 2, amount: 100_000 }.into());
    });
}

/// Tests that self-transfer fails when the account is frozen.
/// Frozen accounts cannot send even to themselves.
#[test]
fn self_transfer_fails_when_frozen() {
    new_test_ext().execute_with(|| {
        // Freeze account 2
        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(1), 2));

        // Self-transfer should fail because account is frozen
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(2), 2, 100_000),
            Error::<Test>::AccountFrozen
        );
    });
}

/// Tests that transfer of exact balance works (transfers all tokens).
#[test]
fn transfer_exact_balance_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let exact_balance = CladToken::balance_of(&2);

        // Transfer exact balance
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(2), 3, exact_balance));

        // Sender should have zero balance
        assert_eq!(CladToken::balance_of(&2), 0);
        assert_eq!(CladToken::balance_of(&3), 500_000 + exact_balance);
    });
}

/// Tests that transfer fails when amount exceeds balance by just 1.
/// Ensures boundary condition is handled correctly.
#[test]
fn transfer_fails_when_amount_exceeds_balance_by_one() {
    new_test_ext().execute_with(|| {
        let balance = CladToken::balance_of(&2);

        // Try to transfer balance + 1
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(2), 3, balance + 1),
            Error::<Test>::InsufficientBalance
        );
    });
}

// ============================================================================
// Integration Tests - Multi-step Workflows
// ============================================================================

/// Tests a complete lifecycle: mint -> whitelist -> transfer -> freeze -> unfreeze.
/// Simulates a real-world token management scenario.
#[test]
fn integration_full_token_lifecycle() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Step 1: Mint tokens to a new account (account 10)
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 10, 500_000));
        assert_eq!(CladToken::balance_of(&10), 500_000);

        // Step 2: Whitelist the new account and a recipient
        assert_ok!(CladToken::add_to_whitelist(RuntimeOrigin::signed(1), 10));
        assert_ok!(CladToken::add_to_whitelist(RuntimeOrigin::signed(1), 11));

        // Step 3: Transfer from account 10 to account 11
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(10), 11, 200_000));
        assert_eq!(CladToken::balance_of(&10), 300_000);
        assert_eq!(CladToken::balance_of(&11), 200_000);

        // Step 4: Freeze account 10
        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(1), 10));
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(10), 11, 100_000),
            Error::<Test>::AccountFrozen
        );

        // Step 5: Unfreeze and transfer again
        assert_ok!(CladToken::unfreeze(RuntimeOrigin::signed(1), 10));
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(10), 11, 100_000));
        assert_eq!(CladToken::balance_of(&10), 200_000);
        assert_eq!(CladToken::balance_of(&11), 300_000);

        // Step 6: Remove from whitelist - transfers should fail
        assert_ok!(CladToken::remove_from_whitelist(RuntimeOrigin::signed(1), 10));
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(10), 11, 50_000),
            Error::<Test>::NotWhitelisted
        );
    });
}

/// Tests multiple concurrent transfers between multiple accounts.
/// Validates that the pallet handles complex multi-party scenarios correctly.
#[test]
fn integration_multi_party_transfers() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Setup: Create and whitelist accounts 10, 11, 12
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 10, 1_000_000));
        assert_ok!(CladToken::add_to_whitelist(RuntimeOrigin::signed(1), 10));
        assert_ok!(CladToken::add_to_whitelist(RuntimeOrigin::signed(1), 11));
        assert_ok!(CladToken::add_to_whitelist(RuntimeOrigin::signed(1), 12));

        // Transfers: 10 -> 11 -> 12 -> 10 (circular)
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(10), 11, 400_000));
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(11), 12, 300_000));
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(12), 10, 100_000));

        // Verify final balances
        assert_eq!(CladToken::balance_of(&10), 700_000); // 1_000_000 - 400_000 + 100_000
        assert_eq!(CladToken::balance_of(&11), 100_000); // 0 + 400_000 - 300_000
        assert_eq!(CladToken::balance_of(&12), 200_000); // 0 + 300_000 - 100_000

        // Total supply should remain unchanged
        let initial_supply = 1_500_000; // From genesis
        let minted = 1_000_000;
        assert_eq!(CladToken::total_supply(), initial_supply + minted);
    });
}

/// Tests admin operations in sequence to ensure state consistency.
#[test]
fn integration_admin_operations_sequence() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create new account
        let account = 20u64;

        // Whitelist -> Freeze -> Unfreeze -> Remove from whitelist
        assert_ok!(CladToken::add_to_whitelist(RuntimeOrigin::signed(1), account));
        assert_eq!(CladToken::whitelist(&account), true);
        assert_eq!(CladToken::is_frozen(&account), false);

        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(1), account));
        assert_eq!(CladToken::whitelist(&account), true);
        assert_eq!(CladToken::is_frozen(&account), true);

        assert_ok!(CladToken::unfreeze(RuntimeOrigin::signed(1), account));
        assert_eq!(CladToken::whitelist(&account), true);
        assert_eq!(CladToken::is_frozen(&account), false);

        assert_ok!(CladToken::remove_from_whitelist(RuntimeOrigin::signed(1), account));
        assert_eq!(CladToken::whitelist(&account), false);
        assert_eq!(CladToken::is_frozen(&account), false);
    });
}

/// Tests that frozen status and whitelist status are independent.
#[test]
fn frozen_and_whitelist_status_are_independent() {
    new_test_ext().execute_with(|| {
        // Account 2 is whitelisted but not frozen
        assert_eq!(CladToken::whitelist(&2), true);
        assert_eq!(CladToken::is_frozen(&2), false);

        // Freeze without affecting whitelist
        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(1), 2));
        assert_eq!(CladToken::whitelist(&2), true);
        assert_eq!(CladToken::is_frozen(&2), true);

        // Remove from whitelist without affecting frozen status
        assert_ok!(CladToken::remove_from_whitelist(RuntimeOrigin::signed(1), 2));
        assert_eq!(CladToken::whitelist(&2), false);
        assert_eq!(CladToken::is_frozen(&2), true);

        // Unfreeze without affecting whitelist
        assert_ok!(CladToken::unfreeze(RuntimeOrigin::signed(1), 2));
        assert_eq!(CladToken::whitelist(&2), false);
        assert_eq!(CladToken::is_frozen(&2), false);
    });
}

/// Tests that minting to an existing account adds to their balance.
#[test]
fn mint_to_existing_account_adds_balance() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let initial_balance = CladToken::balance_of(&2);

        // Mint additional tokens to account 2
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 2, 250_000));

        // Balance should be added, not replaced
        assert_eq!(CladToken::balance_of(&2), initial_balance + 250_000);
    });
}

/// Tests that the receiver cannot transfer when frozen (can only receive).
#[test]
fn frozen_account_can_receive_but_not_send() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Freeze account 3
        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(1), 3));

        // Account 3 can still receive
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(2), 3, 50_000));
        assert_eq!(CladToken::balance_of(&3), 550_000);

        // Account 3 cannot send
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(3), 2, 10_000),
            Error::<Test>::AccountFrozen
        );
    });
}

// ============================================================================
// Access Control Tests
// ============================================================================

/// Tests that all admin-only functions reject non-admin callers.
#[test]
fn all_admin_functions_reject_non_admin() {
    new_test_ext().execute_with(|| {
        // Non-admin account (2) tries all admin functions
        assert_noop!(
            CladToken::mint(RuntimeOrigin::signed(2), 5, 1000),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            CladToken::freeze(RuntimeOrigin::signed(2), 3),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            CladToken::unfreeze(RuntimeOrigin::signed(2), 3),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            CladToken::add_to_whitelist(RuntimeOrigin::signed(2), 5),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            CladToken::remove_from_whitelist(RuntimeOrigin::signed(2), 3),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

/// Tests that transfer is the only user-callable function (non-admin can call it).
#[test]
fn transfer_is_user_callable() {
    new_test_ext().execute_with(|| {
        // Non-admin account (2) can call transfer
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(2), 3, 1000));
    });
}

// ============================================================================
// Genesis Configuration Tests
// ============================================================================

/// Tests that genesis config properly initializes token metadata.
#[test]
fn genesis_config_sets_token_metadata() {
    new_test_ext().execute_with(|| {
        assert_eq!(CladToken::token_name(), b"Test Token".to_vec());
        assert_eq!(CladToken::token_symbol(), b"TST".to_vec());
        assert_eq!(CladToken::decimals(), 6);
    });
}

/// Tests that genesis config properly sets initial supply from balances.
#[test]
fn genesis_config_calculates_total_supply() {
    new_test_ext().execute_with(|| {
        // Genesis has (2, 1_000_000) and (3, 500_000)
        assert_eq!(CladToken::total_supply(), 1_500_000);
    });
}

/// Tests that admin is whitelisted by genesis config.
#[test]
fn genesis_config_whitelists_admin() {
    new_test_ext().execute_with(|| {
        // Admin (account 1) should be whitelisted
        assert_eq!(CladToken::whitelist(&1), true);
    });
}

/// Tests that accounts not in genesis config have default values.
#[test]
fn non_genesis_accounts_have_default_values() {
    new_test_ext().execute_with(|| {
        // Account 99 was never configured
        assert_eq!(CladToken::balance_of(&99), 0);
        assert_eq!(CladToken::whitelist(&99), false);
        assert_eq!(CladToken::is_frozen(&99), false);
    });
}

// ============================================================================
// Storage Query Tests
// ============================================================================

/// Tests that storage getters return correct values.
#[test]
fn storage_getters_work_correctly() {
    new_test_ext().execute_with(|| {
        // Test all getter functions
        assert_eq!(CladToken::total_supply(), 1_500_000);
        assert_eq!(CladToken::balance_of(&2), 1_000_000);
        assert_eq!(CladToken::balance_of(&3), 500_000);
        assert_eq!(CladToken::is_frozen(&2), false);
        assert_eq!(CladToken::whitelist(&2), true);
        assert_eq!(CladToken::token_name(), b"Test Token".to_vec());
        assert_eq!(CladToken::token_symbol(), b"TST".to_vec());
        assert_eq!(CladToken::decimals(), 6);
    });
}

/// Tests that balance updates are reflected immediately.
#[test]
fn balance_updates_reflect_immediately() {
    new_test_ext().execute_with(|| {
        let initial = CladToken::balance_of(&2);
        assert_ok!(CladToken::transfer(RuntimeOrigin::signed(2), 3, 100));
        assert_eq!(CladToken::balance_of(&2), initial - 100);
    });
}

// ============================================================================
// Overflow Protection Tests
// ============================================================================

/// Tests that mint fails when total supply would overflow.
/// This is critical for security - prevents infinite token creation.
#[test]
fn mint_fails_on_total_supply_overflow() {
    new_test_ext().execute_with(|| {
        // First mint a large amount close to u128::MAX
        // Account 5 starts with 0 balance
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 5, u128::MAX - 2_000_000));

        // Now try to mint more than remaining capacity
        // Total supply is now: 1_500_000 (genesis) + (u128::MAX - 2_000_000)
        // Which is u128::MAX - 500_000
        // Trying to mint 1_000_000 should overflow
        assert_noop!(
            CladToken::mint(RuntimeOrigin::signed(1), 6, 1_000_000),
            Error::<Test>::Overflow
        );
    });
}

/// Tests that mint fails when recipient balance would overflow.
/// Even if total supply has room, individual balance overflow must be prevented.
#[test]
fn mint_fails_on_balance_overflow() {
    new_test_ext().execute_with(|| {
        // Mint max to account 5
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 5, u128::MAX - 1_500_000));

        // Try to mint 1 more to the same account - balance would overflow
        assert_noop!(CladToken::mint(RuntimeOrigin::signed(1), 5, 1), Error::<Test>::Overflow);
    });
}

/// Tests that transfer fails when receiver balance would overflow.
///
/// This is a defensive check that should not happen in practice since:
/// 1. Total supply is capped and checked on mint
/// 2. No single account can have more than total supply
///
/// We test this by directly setting storage to simulate a theoretical edge case
/// (e.g., after a future migration or bug).
#[test]
fn transfer_fails_on_receiver_balance_overflow() {
    new_test_ext().execute_with(|| {
        // Directly set account 10's balance to near u128::MAX to simulate edge case
        // This bypasses mint's overflow check - simulating a theoretical scenario
        crate::Balances::<Test>::insert(10, u128::MAX - 100);
        crate::Whitelist::<Test>::insert(10, true);

        // Account 2 tries to transfer to account 10 - would overflow account 10's balance
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(2), 10, 1000),
            Error::<Test>::Overflow
        );
    });
}

/// Tests that multiple sequential mints to the same account work correctly.
/// Verifies accumulation behavior over many operations.
#[test]
fn multiple_sequential_mints_accumulate_correctly() {
    new_test_ext().execute_with(|| {
        let account = 50u64;
        let mint_amount = 100_000u128;
        let num_mints = 10;

        for i in 0..num_mints {
            assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), account, mint_amount));
            assert_eq!(CladToken::balance_of(&account), mint_amount * (i + 1));
        }

        // Final balance check
        assert_eq!(CladToken::balance_of(&account), mint_amount * num_mints);

        // Total supply should include all mints
        let initial_supply = 1_500_000u128; // From genesis
        assert_eq!(CladToken::total_supply(), initial_supply + (mint_amount * num_mints));
    });
}

// ============================================================================
// Additional Whitelist Tests
// ============================================================================

/// Tests that transfer fails when only the receiver is not whitelisted.
/// Explicitly tests the receiver-side whitelist check.
#[test]
fn transfer_fails_when_only_receiver_not_whitelisted() {
    new_test_ext().execute_with(|| {
        // Account 2 is whitelisted (from genesis)
        // Account 99 is NOT whitelisted
        assert_eq!(CladToken::whitelist(&2), true);
        assert_eq!(CladToken::whitelist(&99), false);

        // Transfer should fail due to receiver not being whitelisted
        assert_noop!(
            CladToken::transfer(RuntimeOrigin::signed(2), 99, 1000),
            Error::<Test>::NotWhitelisted
        );

        // Verify sender's balance is unchanged
        assert_eq!(CladToken::balance_of(&2), 1_000_000);
    });
}

// ============================================================================
// Set Admin Tests
// ============================================================================

/// Tests that set_admin works when called by admin.
#[test]
fn set_admin_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Admin (account 1) sets new admin (account 50)
        assert_ok!(CladToken::set_admin(RuntimeOrigin::signed(1), 50));

        // Verify admin was set in storage
        assert_eq!(CladToken::admin(), Some(50));

        // Verify new admin was auto-whitelisted
        assert_eq!(CladToken::whitelist(&50), true);

        // Check AdminChanged event was emitted (old_admin is 1 from genesis)
        System::assert_has_event(
            Event::AdminChanged { old_admin: Some(AdminAccount::get()), new_admin: 50 }.into(),
        );

        // Check Whitelisted event was emitted for new admin
        System::assert_last_event(Event::Whitelisted { account: 50 }.into());
    });
}

/// Tests that set_admin fails when called by non-admin.
#[test]
fn set_admin_fails_for_non_admin() {
    new_test_ext().execute_with(|| {
        // Non-admin (account 2) cannot set admin
        assert_noop!(
            CladToken::set_admin(RuntimeOrigin::signed(2), 50),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

/// Tests that set_admin correctly tracks old admin in event.
#[test]
fn set_admin_tracks_old_admin() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // First set_admin: 1 (genesis) -> 50
        assert_ok!(CladToken::set_admin(RuntimeOrigin::signed(1), 50));
        System::assert_has_event(
            Event::AdminChanged { old_admin: Some(AdminAccount::get()), new_admin: 50 }.into(),
        );

        // Clear events for next assertion
        System::reset_events();
        System::set_block_number(2);

        // Second set_admin: 50 -> 60 (called by original genesis admin, account 1)
        // Note: In the mock, account 1 is always admin via EnsureRoot simulation
        assert_ok!(CladToken::set_admin(RuntimeOrigin::signed(1), 60));
        System::assert_has_event(Event::AdminChanged { old_admin: Some(50), new_admin: 60 }.into());

        // Verify final state
        assert_eq!(CladToken::admin(), Some(60));
    });
}

/// Tests that set_admin auto-whitelists the new admin.
#[test]
fn set_admin_auto_whitelists_new_admin() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Account 99 is not whitelisted initially
        assert_eq!(CladToken::whitelist(&99), false);

        // Set account 99 as admin
        assert_ok!(CladToken::set_admin(RuntimeOrigin::signed(1), 99));

        // Account 99 should now be whitelisted
        assert_eq!(CladToken::whitelist(&99), true);
    });
}

/// Tests that setting admin to already-whitelisted account works.
/// The whitelist operation is idempotent.
#[test]
fn set_admin_to_whitelisted_account_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Account 2 is already whitelisted from genesis
        assert_eq!(CladToken::whitelist(&2), true);

        // Set account 2 as admin
        assert_ok!(CladToken::set_admin(RuntimeOrigin::signed(1), 2));

        // Admin should be set
        assert_eq!(CladToken::admin(), Some(2));

        // Account 2 should still be whitelisted
        assert_eq!(CladToken::whitelist(&2), true);
    });
}

/// Tests that setting admin to same account works (idempotent).
#[test]
fn set_admin_to_same_account_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // First set admin to 50
        assert_ok!(CladToken::set_admin(RuntimeOrigin::signed(1), 50));
        assert_eq!(CladToken::admin(), Some(50));

        System::reset_events();
        System::set_block_number(2);

        // Set admin to 50 again
        assert_ok!(CladToken::set_admin(RuntimeOrigin::signed(1), 50));
        assert_eq!(CladToken::admin(), Some(50));

        // Event should still be emitted with old_admin = Some(50)
        System::assert_has_event(Event::AdminChanged { old_admin: Some(50), new_admin: 50 }.into());
    });
}

/// Tests the complete admin rotation workflow.
/// This simulates a ministry committee personnel change scenario.
#[test]
fn integration_admin_rotation_workflow() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Step 1: Initial state - admin set from genesis (account 1)
        assert_eq!(CladToken::admin(), Some(AdminAccount::get()));

        // Step 2: Genesis admin (account 1) sets new admin (multi-sig placeholder: 100)
        assert_ok!(CladToken::set_admin(RuntimeOrigin::signed(1), 100));
        assert_eq!(CladToken::admin(), Some(100));
        assert_eq!(CladToken::whitelist(&100), true);

        // Step 3: Verify new admin can perform admin operations via root
        // (In mock, admin 1 can still call admin functions via EnsureRoot)
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(1), 200, 1_000_000));
        assert_eq!(CladToken::balance_of(&200), 1_000_000);

        // Step 4: Rotate to new admin (simulating committee change: 100 -> 101)
        System::reset_events();
        System::set_block_number(2);
        assert_ok!(CladToken::set_admin(RuntimeOrigin::signed(1), 101));
        assert_eq!(CladToken::admin(), Some(101));
        assert_eq!(CladToken::whitelist(&101), true);

        // Old admin (100) remains whitelisted (can still hold tokens)
        assert_eq!(CladToken::whitelist(&100), true);

        // Step 5: Verify events for audit trail
        System::assert_has_event(
            Event::AdminChanged { old_admin: Some(100), new_admin: 101 }.into(),
        );
    });
}

/// Tests that admin storage is set from genesis config.
#[test]
fn admin_storage_set_from_genesis() {
    new_test_ext().execute_with(|| {
        // Admin storage should be set from genesis config (account 1)
        assert_eq!(CladToken::admin(), Some(AdminAccount::get()));
    });
}
