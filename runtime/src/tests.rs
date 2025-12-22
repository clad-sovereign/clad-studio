//! Integration tests for multi-sig admin operations on pallet-clad-token.
//!
//! These tests verify that multi-sig accounts can successfully execute admin operations
//! on the CladToken pallet, demonstrating the N-of-M governance pattern for ministry
//! committees.
//!
//! # Test Categories
//!
//! 1. **Basic Multi-Sig Flow**: Address derivation, deposit reservation, proposal creation
//! 2. **Multi-Sig Approval Flow**: Complete approval flow demonstration
//! 3. **Edge Cases**: Duplicate approvals, non-signatory rejection, timepoint tracking
//! 4. **Threshold Variations**: 1-of-2, 2-of-3, 3-of-5 configurations
//! 5. **Integration**: Full ministry workflow simulation
//!
//! # Admin Operations
//!
//! All admin operations (mint, freeze, whitelist, etc.) go through multi-sig governance.
//! There is no sudo bypass - this matches the production configuration.
//! See ADR-004: docs/adr/004-production-runtime-configuration.md

use crate::*;
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use sp_core::blake2_256;
use sp_keyring::sr25519::Keyring as AccountKeyring;
use sp_runtime::{traits::Hash, BuildStorage};

/// Type alias for call hash used by pallet-multisig
type CallHash = [u8; 32];

/// Standard test account balance (100 trillion units, enough for deposits and fees)
const TEST_ACCOUNT_BALANCE: u128 = 100_000_000_000_000;

/// Derive a multi-sig account address from signatories and threshold.
///
/// This replicates the Substrate multi-sig address derivation:
/// `blake2_256("modlpy/utilisuba" ++ compact(len) ++ sorted(signatories) ++ threshold_u16_le)`
fn derive_multisig_account(mut signatories: Vec<AccountId>, threshold: u16) -> AccountId {
    // Sort signatories lexicographically (required by pallet-multisig)
    signatories.sort();

    // Build the preimage following Substrate's multi-sig derivation
    let mut preimage = b"modlpy/utilisuba".to_vec();
    // Append SCALE-compact encoded length
    codec::Compact(signatories.len() as u32).encode_to(&mut preimage);
    // Append sorted signatories
    for acc in &signatories {
        acc.encode_to(&mut preimage);
    }
    // Append threshold as u16 little-endian
    threshold.encode_to(&mut preimage);

    // Hash and convert to AccountId
    let hash = blake2_256(&preimage);
    AccountId::new(hash)
}

/// Get other signatories (excluding caller) in sorted order.
/// pallet-multisig requires other_signatories to be sorted.
fn sorted_other_signatories(all_signatories: &[AccountId], caller: &AccountId) -> Vec<AccountId> {
    let mut others: Vec<_> = all_signatories.iter().filter(|s| *s != caller).cloned().collect();
    others.sort();
    others
}

/// Build test externalities with a 2-of-3 multi-sig as admin.
///
/// This matches the production configuration where a multi-sig committee
/// controls admin operations. No sudo pallet is included.
///
/// Admin: 2-of-3 multi-sig (Alice, Bob, Charlie)
fn new_test_ext() -> sp_io::TestExternalities {
    let alice = AccountKeyring::Alice.to_account_id();
    let bob = AccountKeyring::Bob.to_account_id();
    let charlie = AccountKeyring::Charlie.to_account_id();

    // Derive the 2-of-3 multi-sig address
    let admin_multisig =
        derive_multisig_account(vec![alice.clone(), bob.clone(), charlie.clone()], 2);

    let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

    // Fund test accounts with enough balance for deposits and fees
    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![
            (alice, TEST_ACCOUNT_BALANCE),
            (bob, TEST_ACCOUNT_BALANCE),
            (charlie, TEST_ACCOUNT_BALANCE),
            (AccountKeyring::Dave.to_account_id(), TEST_ACCOUNT_BALANCE),
            (AccountKeyring::Eve.to_account_id(), TEST_ACCOUNT_BALANCE),
            (AccountKeyring::Ferdie.to_account_id(), TEST_ACCOUNT_BALANCE),
            // Fund the admin multi-sig for any deposits
            (admin_multisig.clone(), TEST_ACCOUNT_BALANCE),
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    // Configure CladToken with multi-sig admin
    pallet_clad_token::GenesisConfig::<Runtime> {
        admin: Some(admin_multisig),
        token_name: b"Test Sovereign Bond".to_vec(),
        token_symbol: b"TSB".to_vec(),
        decimals: 6,
        whitelisted_accounts: vec![],
        initial_balances: vec![],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

/// Build test externalities with a custom admin account.
///
/// Allows tests to specify a different admin configuration.
fn new_test_ext_with_admin(admin: AccountId) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

    // Build balances list, avoiding duplicates if admin is a well-known account
    let mut balances = vec![
        (AccountKeyring::Alice.to_account_id(), TEST_ACCOUNT_BALANCE),
        (AccountKeyring::Bob.to_account_id(), TEST_ACCOUNT_BALANCE),
        (AccountKeyring::Charlie.to_account_id(), TEST_ACCOUNT_BALANCE),
        (AccountKeyring::Dave.to_account_id(), TEST_ACCOUNT_BALANCE),
        (AccountKeyring::Eve.to_account_id(), TEST_ACCOUNT_BALANCE),
        (AccountKeyring::Ferdie.to_account_id(), TEST_ACCOUNT_BALANCE),
    ];
    // Only add admin if not already in the list
    if !balances.iter().any(|(acc, _)| *acc == admin) {
        balances.push((admin.clone(), TEST_ACCOUNT_BALANCE));
    }

    pallet_balances::GenesisConfig::<Runtime> { balances, dev_accounts: None }
        .assimilate_storage(&mut t)
        .unwrap();

    pallet_clad_token::GenesisConfig::<Runtime> {
        admin: Some(admin),
        token_name: b"Test Sovereign Bond".to_vec(),
        token_symbol: b"TSB".to_vec(),
        decimals: 6,
        whitelisted_accounts: vec![],
        initial_balances: vec![],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

/// Execute a 2-of-3 multi-sig admin call (proposal + approval).
///
/// Helper function that handles the full multi-sig flow:
/// 1. First signer proposes the call
/// 2. Second signer approves (threshold met, executes)
fn execute_2of3_multisig_call(call: RuntimeCall) {
    let alice = AccountKeyring::Alice.to_account_id();
    let bob = AccountKeyring::Bob.to_account_id();
    let charlie = AccountKeyring::Charlie.to_account_id();

    let signatories = vec![alice.clone(), bob.clone(), charlie.clone()];
    let multisig_account = derive_multisig_account(signatories.clone(), 2);
    let call_hash: CallHash = BlakeTwo256::hash_of(&call).into();

    // Step 1: Alice proposes
    assert_ok!(Multisig::as_multi(
        RuntimeOrigin::signed(alice.clone()),
        2,
        sorted_other_signatories(&signatories, &alice),
        None,
        Box::new(call.clone()),
        Weight::zero(),
    ));

    // Step 2: Bob approves (threshold met, executes)
    let timepoint = pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash)
        .expect("Multisig should exist")
        .when;

    assert_ok!(Multisig::as_multi(
        RuntimeOrigin::signed(bob.clone()),
        2,
        sorted_other_signatories(&signatories, &bob),
        Some(timepoint),
        Box::new(call),
        Weight::from_parts(10_000_000_000, 1_000_000),
    ));
}

// ============================================================================
// Basic Multi-Sig Flow Tests
// ============================================================================

/// Tests that multi-sig address derivation is deterministic.
///
/// The same set of signatories and threshold should always produce
/// the same multi-sig account address.
#[test]
fn multisig_address_derivation_is_deterministic() {
    let alice = AccountKeyring::Alice.to_account_id();
    let bob = AccountKeyring::Bob.to_account_id();
    let charlie = AccountKeyring::Charlie.to_account_id();

    let signatories = vec![alice.clone(), bob.clone(), charlie.clone()];
    let threshold = 2u16;

    // Derive twice - should get same result
    let addr1 = derive_multisig_account(signatories.clone(), threshold);
    let addr2 = derive_multisig_account(signatories.clone(), threshold);

    assert_eq!(addr1, addr2, "Multi-sig address derivation must be deterministic");

    // Order of signatories shouldn't matter (they get sorted internally)
    let reordered = vec![charlie, alice, bob];
    let addr3 = derive_multisig_account(reordered, threshold);
    assert_eq!(addr1, addr3, "Signatory order should not affect derived address");
}

/// Tests that different thresholds produce different multi-sig addresses.
#[test]
fn different_thresholds_produce_different_addresses() {
    let alice = AccountKeyring::Alice.to_account_id();
    let bob = AccountKeyring::Bob.to_account_id();
    let charlie = AccountKeyring::Charlie.to_account_id();

    let signatories = vec![alice, bob, charlie];

    let addr_2of3 = derive_multisig_account(signatories.clone(), 2);
    let addr_3of3 = derive_multisig_account(signatories, 3);

    assert_ne!(addr_2of3, addr_3of3, "Different thresholds must produce different addresses");
}

/// Tests that deposit is reserved when creating a multi-sig proposal.
#[test]
fn multisig_proposal_reserves_deposit() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();
        let charlie = AccountKeyring::Charlie.to_account_id();

        let signatories_for_call = vec![bob.clone(), charlie.clone()]; // Excludes caller (Alice)
        let threshold = 2u16;

        // Get Alice's initial balance
        let initial_balance = Balances::free_balance(&alice);

        // Create a dummy call to test deposit
        let call: RuntimeCall =
            pallet_clad_token::Call::add_to_whitelist { account: alice.clone() }.into();

        // Propose the multi-sig call (Alice is first signer)
        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(alice.clone()),
            threshold,
            signatories_for_call,
            None, // First call, no timepoint
            Box::new(call),
            Weight::zero(), // Max weight for storage
        ));

        // Verify deposit was reserved
        let final_balance = Balances::free_balance(&alice);
        let reserved = Balances::reserved_balance(&alice);

        // Deposit = DepositBase + (DepositFactor * threshold)
        // From runtime: DepositBase = 1 unit, DepositFactor = 0.1 unit
        let expected_deposit = DepositBase::get() + DepositFactor::get() * Balance::from(threshold);

        assert!(
            reserved >= expected_deposit,
            "Deposit should be reserved: expected at least {expected_deposit}, got {reserved}"
        );
        assert!(initial_balance > final_balance, "Free balance should decrease after proposal");
    });
}

// ============================================================================
// Multi-Sig Approval Flow Tests
// ============================================================================

/// Tests complete 2-of-3 multi-sig approval flow.
///
/// Demonstrates the full multi-sig flow:
/// 1. Alice proposes (threshold not met, call stored)
/// 2. Bob approves (threshold met, call executes as multi-sig account)
#[test]
fn multisig_2of3_approval_executes_call() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();
        let charlie = AccountKeyring::Charlie.to_account_id();

        let signatories = vec![alice.clone(), bob.clone(), charlie.clone()];
        let multisig_account = derive_multisig_account(signatories.clone(), 2);

        // Use System::remark as a simple call that always succeeds
        let call: RuntimeCall = frame_system::Call::remark { remark: vec![1, 2, 3] }.into();

        let call_hash: CallHash = BlakeTwo256::hash_of(&call).into();

        // Step 1: Alice proposes (stores call, creates multi-sig entry)
        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(alice.clone()),
            2,
            sorted_other_signatories(&signatories, &alice),
            None,
            Box::new(call.clone()),
            Weight::zero(),
        ));

        // Step 2: Bob approves (threshold met, call should execute)
        let timepoint = pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash)
            .expect("Multisig should exist")
            .when;

        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(bob.clone()),
            2,
            sorted_other_signatories(&signatories, &bob),
            Some(timepoint),
            Box::new(call),
            Weight::from_parts(10_000_000_000, 1_000_000),
        ));

        // Verify the multi-sig entry is removed (call executed)
        assert!(
            pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash).is_none(),
            "Multi-sig should be cleared after execution"
        );
    });
}

// ============================================================================
// Edge Case Tests
// ============================================================================

/// Tests that duplicate approval from the same signer is rejected.
#[test]
fn duplicate_approval_rejected() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();
        let charlie = AccountKeyring::Charlie.to_account_id();

        let signatories = vec![alice.clone(), bob.clone(), charlie.clone()];
        let multisig_account = derive_multisig_account(signatories.clone(), 2);

        let call: RuntimeCall = frame_system::Call::remark { remark: vec![1, 2, 3] }.into();
        let call_hash: CallHash = BlakeTwo256::hash_of(&call).into();

        // Alice proposes
        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(alice.clone()),
            2,
            sorted_other_signatories(&signatories, &alice),
            None,
            Box::new(call.clone()),
            Weight::zero(),
        ));

        let timepoint = pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash)
            .expect("Multisig should exist")
            .when;

        // Alice tries to approve again - should fail
        assert_noop!(
            Multisig::as_multi(
                RuntimeOrigin::signed(alice.clone()),
                2,
                sorted_other_signatories(&signatories, &alice),
                Some(timepoint),
                Box::new(call),
                Weight::zero(),
            ),
            pallet_multisig::Error::<Runtime>::AlreadyApproved
        );
    });
}

/// Tests that timepoint must match for subsequent approvals.
#[test]
fn wrong_timepoint_rejected() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();
        let charlie = AccountKeyring::Charlie.to_account_id();

        let signatories = vec![alice.clone(), bob.clone(), charlie.clone()];
        let multisig_account = derive_multisig_account(signatories.clone(), 2);

        let call: RuntimeCall = frame_system::Call::remark { remark: vec![1, 2, 3] }.into();
        let call_hash: CallHash = BlakeTwo256::hash_of(&call).into();

        // Alice proposes
        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(alice.clone()),
            2,
            sorted_other_signatories(&signatories, &alice),
            None,
            Box::new(call.clone()),
            Weight::zero(),
        ));

        let actual_timepoint =
            pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash)
                .expect("Multisig should exist")
                .when;

        // Create wrong timepoint (different block)
        let wrong_timepoint = pallet_multisig::Timepoint {
            height: actual_timepoint.height + 100,
            index: actual_timepoint.index,
        };

        // Bob tries with wrong timepoint
        assert_noop!(
            Multisig::as_multi(
                RuntimeOrigin::signed(bob.clone()),
                2,
                sorted_other_signatories(&signatories, &bob),
                Some(wrong_timepoint),
                Box::new(call),
                Weight::zero(),
            ),
            pallet_multisig::Error::<Runtime>::WrongTimepoint
        );
    });
}

/// Tests that a non-signatory cannot approve a multi-sig proposal.
///
/// When a non-signatory tries to approve, they cannot produce the correct
/// multi-sig account address (since address = hash(signatories, threshold)).
/// The result is `UnexpectedTimepoint` because no proposal exists at the
/// (incorrect) multi-sig address they derive.
#[test]
fn non_signatory_cannot_approve() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();
        let charlie = AccountKeyring::Charlie.to_account_id();
        let outsider = AccountKeyring::Ferdie.to_account_id(); // Not a signatory

        let signatories = vec![alice.clone(), bob.clone(), charlie.clone()];
        let multisig_account = derive_multisig_account(signatories.clone(), 2);

        let call: RuntimeCall = frame_system::Call::remark { remark: vec![1, 2, 3] }.into();
        let call_hash: CallHash = BlakeTwo256::hash_of(&call).into();

        // Alice proposes
        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(alice.clone()),
            2,
            sorted_other_signatories(&signatories, &alice),
            None,
            Box::new(call.clone()),
            Weight::zero(),
        ));

        let timepoint = pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash)
            .expect("Multisig should exist")
            .when;

        // Ferdie (outsider) tries to approve - should fail
        // Since Ferdie isn't in the original signatories, sorted_other_signatories returns
        // all 3 original signatories. This produces a different multi-sig address
        // (hash includes Ferdie + all 3 = 4 accounts), so no proposal exists there.
        assert_noop!(
            Multisig::as_multi(
                RuntimeOrigin::signed(outsider.clone()),
                2,
                sorted_other_signatories(&signatories, &outsider), // Returns all 3 since outsider isn't in list
                Some(timepoint),
                Box::new(call),
                Weight::zero(),
            ),
            pallet_multisig::Error::<Runtime>::UnexpectedTimepoint
        );
    });
}

// ============================================================================
// Threshold Variation Tests
// ============================================================================

/// Tests 1-of-2 multi-sig where a single approval is enough.
///
/// With threshold=1 and 2 signatories, the first approval executes immediately.
#[test]
fn threshold_1of2_executes_immediately() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();

        let call: RuntimeCall = frame_system::Call::remark { remark: vec![1, 2, 3] }.into();

        // For threshold=1 with other signatories, use as_multi_threshold_1
        let mut other = vec![bob];
        other.sort();
        assert_ok!(Multisig::as_multi_threshold_1(
            RuntimeOrigin::signed(alice),
            other,
            Box::new(call),
        ));
        // The call executes immediately (System::remark doesn't have observable effects to verify)
    });
}

/// Tests 2-of-3 multi-sig (standard ministry committee).
#[test]
fn threshold_2of3_standard_committee() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();
        let charlie = AccountKeyring::Charlie.to_account_id();

        let signatories = vec![alice.clone(), bob.clone(), charlie.clone()];
        let multisig_account = derive_multisig_account(signatories.clone(), 2);

        let call: RuntimeCall = frame_system::Call::remark { remark: vec![1, 2, 3] }.into();
        let call_hash: CallHash = BlakeTwo256::hash_of(&call).into();

        // Alice proposes
        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(alice.clone()),
            2,
            sorted_other_signatories(&signatories, &alice),
            None,
            Box::new(call.clone()),
            Weight::zero(),
        ));

        // Verify multi-sig entry exists (call not yet executed)
        assert!(
            pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash).is_some(),
            "Multisig should exist after first approval"
        );

        let timepoint = pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash)
            .expect("Multisig should exist")
            .when;

        // Bob approves (2 of 2 threshold met)
        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(bob.clone()),
            2,
            sorted_other_signatories(&signatories, &bob),
            Some(timepoint),
            Box::new(call),
            Weight::from_parts(10_000_000_000, 1_000_000),
        ));

        // Verify multi-sig entry is removed (call executed)
        assert!(
            pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash).is_none(),
            "Multisig should be cleared after execution"
        );
    });
}

/// Tests 3-of-5 multi-sig (larger committee).
#[test]
fn threshold_3of5_larger_committee() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();
        let charlie = AccountKeyring::Charlie.to_account_id();
        let dave = AccountKeyring::Dave.to_account_id();
        let eve = AccountKeyring::Eve.to_account_id();

        let signatories =
            vec![alice.clone(), bob.clone(), charlie.clone(), dave.clone(), eve.clone()];
        let multisig_account = derive_multisig_account(signatories.clone(), 3);

        let call: RuntimeCall = frame_system::Call::remark { remark: vec![1, 2, 3] }.into();
        let call_hash: CallHash = BlakeTwo256::hash_of(&call).into();

        // Alice proposes (1 of 3)
        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(alice.clone()),
            3,
            sorted_other_signatories(&signatories, &alice),
            None,
            Box::new(call.clone()),
            Weight::zero(),
        ));

        // Verify multi-sig entry exists
        assert!(
            pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash).is_some(),
            "Multisig should exist after first approval"
        );

        let timepoint = pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash)
            .expect("Multisig should exist")
            .when;

        // Bob approves (2 of 3)
        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(bob.clone()),
            3,
            sorted_other_signatories(&signatories, &bob),
            Some(timepoint),
            Box::new(call.clone()),
            Weight::zero(),
        ));

        // Still not executed (need 3 approvals)
        assert!(
            pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash).is_some(),
            "Multisig should still exist after 2 of 3 approvals"
        );

        // Charlie approves (3 of 3 threshold met)
        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(charlie.clone()),
            3,
            sorted_other_signatories(&signatories, &charlie),
            Some(timepoint),
            Box::new(call),
            Weight::from_parts(10_000_000_000, 1_000_000),
        ));

        // Verify multi-sig entry is removed (call executed)
        assert!(
            pallet_multisig::Multisigs::<Runtime>::get(&multisig_account, call_hash).is_none(),
            "Multisig should be cleared after execution"
        );
    });
}

// ============================================================================
// Integration Tests - Full Workflow Simulation
// ============================================================================
//
// These tests demonstrate complete workflows with multi-sig governance.
// All admin operations go through the 2-of-3 multi-sig - no sudo bypass.

/// Tests a complete ministry workflow: whitelist -> mint -> transfer -> freeze.
///
/// Simulates a real-world bond issuance scenario:
/// 1. Admin (multi-sig) whitelists treasury and investor accounts
/// 2. Admin mints bond tokens to treasury
/// 3. Treasury distributes tokens to investor
/// 4. Compliance issue triggers freeze
/// 5. Issue resolved, investor unfrozen
#[test]
fn integration_full_ministry_workflow() {
    new_test_ext().execute_with(|| {
        let treasury = AccountKeyring::Dave.to_account_id();
        let investor = AccountKeyring::Eve.to_account_id();

        // Step 1: Whitelist treasury via multi-sig
        execute_2of3_multisig_call(
            pallet_clad_token::Call::add_to_whitelist { account: treasury.clone() }.into(),
        );
        assert!(CladToken::whitelist(&treasury));

        // Whitelist investor via multi-sig
        execute_2of3_multisig_call(
            pallet_clad_token::Call::add_to_whitelist { account: investor.clone() }.into(),
        );
        assert!(CladToken::whitelist(&investor));

        // Step 2: Mint bond tokens to treasury via multi-sig
        let bond_amount = 100_000_000_000_000u128; // $100M with 6 decimals
        execute_2of3_multisig_call(
            pallet_clad_token::Call::mint { to: treasury.clone(), amount: bond_amount }.into(),
        );

        assert_eq!(CladToken::balance_of(&treasury), bond_amount);
        assert_eq!(CladToken::total_supply(), bond_amount);

        // Step 3: Treasury distributes to investor (regular transfer, not admin op)
        let investment_amount = 10_000_000_000_000u128; // $10M
        assert_ok!(CladToken::transfer(
            RuntimeOrigin::signed(treasury.clone()),
            investor.clone(),
            investment_amount,
        ));

        assert_eq!(CladToken::balance_of(&treasury), bond_amount - investment_amount);
        assert_eq!(CladToken::balance_of(&investor), investment_amount);

        // Step 4: Compliance issue - freeze investor via multi-sig
        execute_2of3_multisig_call(
            pallet_clad_token::Call::freeze { account: investor.clone() }.into(),
        );

        assert!(CladToken::is_frozen(&investor));

        // Investor cannot transfer while frozen
        assert_noop!(
            CladToken::transfer(
                RuntimeOrigin::signed(investor.clone()),
                treasury.clone(),
                1_000_000,
            ),
            pallet_clad_token::Error::<Runtime>::AccountFrozen
        );

        // Step 5: Issue resolved - unfreeze via multi-sig
        execute_2of3_multisig_call(
            pallet_clad_token::Call::unfreeze { account: investor.clone() }.into(),
        );

        // Investor can transfer again
        assert_ok!(CladToken::transfer(
            RuntimeOrigin::signed(investor.clone()),
            treasury.clone(),
            1_000_000,
        ));

        // Total supply unchanged throughout
        assert_eq!(CladToken::total_supply(), bond_amount);
    });
}

// ============================================================================
// Admin Rotation Tests (set_admin extrinsic)
// ============================================================================
//
// These tests verify the set_admin functionality that enables admin rotation
// without runtime upgrades. This is critical for production deployments where
// ministry committees have personnel changes.

/// Tests that storage-based admin can perform admin operations.
///
/// When admin is set in storage, that admin can perform admin operations directly.
#[test]
fn storage_admin_can_perform_admin_operations() {
    let new_admin = AccountKeyring::Ferdie.to_account_id();
    let investor = AccountKeyring::Dave.to_account_id();

    new_test_ext_with_admin(new_admin.clone()).execute_with(|| {
        // New admin can whitelist accounts directly
        assert_ok!(CladToken::add_to_whitelist(
            RuntimeOrigin::signed(new_admin.clone()),
            investor.clone(),
        ));
        assert!(CladToken::whitelist(&investor));

        // New admin can mint tokens directly
        assert_ok!(CladToken::mint(
            RuntimeOrigin::signed(new_admin.clone()),
            investor.clone(),
            1_000_000,
        ));
        assert_eq!(CladToken::balance_of(&investor), 1_000_000);

        // New admin can freeze accounts directly
        assert_ok!(CladToken::freeze(RuntimeOrigin::signed(new_admin.clone()), investor.clone(),));
        assert!(CladToken::is_frozen(&investor));

        // New admin can unfreeze accounts directly
        assert_ok!(CladToken::unfreeze(RuntimeOrigin::signed(new_admin), investor.clone(),));
        assert!(!CladToken::is_frozen(&investor));
    });
}

/// Tests admin rotation from one admin to another via multi-sig.
///
/// Simulates a ministry committee personnel change:
/// 1. Initial admin is 2-of-3 multi-sig
/// 2. Multi-sig votes to set new admin
/// 3. New admin performs operations
#[test]
fn admin_rotation_via_multisig_works() {
    new_test_ext().execute_with(|| {
        let new_admin = AccountKeyring::Ferdie.to_account_id();
        let test_account = AccountKeyring::Dave.to_account_id();

        // Step 1: Rotate admin via multi-sig
        execute_2of3_multisig_call(
            pallet_clad_token::Call::set_admin { new_admin: new_admin.clone() }.into(),
        );
        assert_eq!(CladToken::admin(), Some(new_admin.clone()));

        // Step 2: New admin can perform operations directly
        assert_ok!(CladToken::add_to_whitelist(
            RuntimeOrigin::signed(new_admin.clone()),
            test_account.clone(),
        ));
        assert!(CladToken::whitelist(&test_account));

        // Step 3: Old multi-sig can NO longer perform admin operations
        // (The multi-sig address is no longer the admin)
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();
        let charlie = AccountKeyring::Charlie.to_account_id();
        let old_multisig = derive_multisig_account(vec![alice, bob, charlie], 2);

        assert_noop!(
            CladToken::mint(RuntimeOrigin::signed(old_multisig), test_account.clone(), 1000),
            sp_runtime::DispatchError::BadOrigin
        );

        // New admin CAN mint
        assert_ok!(CladToken::mint(RuntimeOrigin::signed(new_admin), test_account, 1000,));
    });
}

/// Tests multi-sig admin rotation workflow.
///
/// This is the production scenario: a 2-of-3 multi-sig committee rotates
/// to a new 3-of-5 multi-sig committee using the set_admin extrinsic.
#[test]
fn multisig_admin_rotation_to_new_multisig() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();
        let charlie = AccountKeyring::Charlie.to_account_id();
        let dave = AccountKeyring::Dave.to_account_id();
        let eve = AccountKeyring::Eve.to_account_id();

        // Current admin: 2-of-3 (Alice, Bob, Charlie)
        let old_multisig_signatories = vec![alice.clone(), bob.clone(), charlie.clone()];
        let old_multisig = derive_multisig_account(old_multisig_signatories.clone(), 2);

        // New admin: 3-of-5 (Alice, Bob, Charlie, Dave, Eve)
        let new_multisig_signatories =
            vec![alice.clone(), bob.clone(), charlie.clone(), dave.clone(), eve.clone()];
        let new_multisig = derive_multisig_account(new_multisig_signatories, 3);

        // Fund the new multi-sig account
        assert_ok!(Balances::transfer_allow_death(
            RuntimeOrigin::signed(alice.clone()),
            new_multisig.clone().into(),
            TEST_ACCOUNT_BALANCE / 10,
        ));

        // Step 1: Verify current admin
        assert_eq!(CladToken::admin(), Some(old_multisig.clone()));

        // Step 2: Old multi-sig votes to rotate to new multi-sig
        execute_2of3_multisig_call(
            pallet_clad_token::Call::set_admin { new_admin: new_multisig.clone() }.into(),
        );

        // Step 3: Verify admin was rotated
        assert_eq!(CladToken::admin(), Some(new_multisig.clone()));

        // Step 4: New multi-sig should be auto-whitelisted
        assert!(CladToken::whitelist(&new_multisig));

        // Step 5: Old multi-sig remains whitelisted (can hold tokens)
        assert!(CladToken::whitelist(&old_multisig));
    });
}

/// Tests that non-admin accounts cannot perform admin operations.
#[test]
fn non_admin_cannot_perform_admin_operations() {
    new_test_ext().execute_with(|| {
        let non_admin = AccountKeyring::Ferdie.to_account_id();
        let test_account = AccountKeyring::Dave.to_account_id();

        // Non-admin cannot whitelist
        assert_noop!(
            CladToken::add_to_whitelist(
                RuntimeOrigin::signed(non_admin.clone()),
                test_account.clone()
            ),
            sp_runtime::DispatchError::BadOrigin
        );

        // Non-admin cannot mint
        assert_noop!(
            CladToken::mint(RuntimeOrigin::signed(non_admin.clone()), test_account.clone(), 1000),
            sp_runtime::DispatchError::BadOrigin
        );

        // Non-admin cannot freeze
        assert_noop!(
            CladToken::freeze(RuntimeOrigin::signed(non_admin.clone()), test_account.clone()),
            sp_runtime::DispatchError::BadOrigin
        );

        // Non-admin cannot set new admin
        assert_noop!(
            CladToken::set_admin(RuntimeOrigin::signed(non_admin), test_account),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

/// Tests that AdminChanged event includes correct old_admin value.
#[test]
fn admin_changed_event_tracks_history() {
    new_test_ext().execute_with(|| {
        let first_admin_new = AccountKeyring::Ferdie.to_account_id();
        let second_admin = AccountKeyring::Dave.to_account_id();

        // Get current admin (2-of-3 multi-sig)
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();
        let charlie = AccountKeyring::Charlie.to_account_id();
        let current_admin = derive_multisig_account(vec![alice, bob, charlie], 2);

        // First set_admin: multi-sig -> first_admin_new
        execute_2of3_multisig_call(
            pallet_clad_token::Call::set_admin { new_admin: first_admin_new.clone() }.into(),
        );

        // Check event
        System::assert_has_event(
            pallet_clad_token::Event::AdminChanged {
                old_admin: Some(current_admin),
                new_admin: first_admin_new.clone(),
            }
            .into(),
        );

        // Second set_admin: first_admin_new -> second_admin
        assert_ok!(CladToken::set_admin(
            RuntimeOrigin::signed(first_admin_new.clone()),
            second_admin.clone(),
        ));

        // Check event has old_admin = Some(first_admin_new)
        System::assert_has_event(
            pallet_clad_token::Event::AdminChanged {
                old_admin: Some(first_admin_new),
                new_admin: second_admin,
            }
            .into(),
        );
    });
}
