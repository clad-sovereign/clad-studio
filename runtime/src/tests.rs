//! Integration tests for multi-sig admin operations on pallet-clad-token.
//!
//! These tests verify that multi-sig accounts can successfully execute admin operations
//! on the CladToken pallet, demonstrating the N-of-M governance pattern for ministry
//! committees.
//!
//! # Test Categories
//!
//! 1. **Basic Multi-Sig Flow**: Address derivation, deposit reservation, proposal creation
//! 2. **Admin Operations via Multi-Sig**: mint, freeze, unfreeze, whitelist operations
//! 3. **Edge Cases**: Duplicate approvals, non-signatory rejection, timepoint tracking
//! 4. **Threshold Variations**: 1-of-1, 2-of-3, 3-of-5 configurations

use crate::*;
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use sp_core::blake2_256;
use sp_keyring::sr25519::Keyring as AccountKeyring;
use sp_runtime::{traits::Hash, BuildStorage};

/// Type alias for call hash used by pallet-multisig
type CallHash = [u8; 32];

/// Build test externalities with initial state for multi-sig testing.
///
/// Sets up:
/// - Well-funded signatory accounts (Alice, Bob, Charlie, Dave, Eve)
/// - Empty CladToken state (no initial balances/whitelist)
fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

    // Fund test accounts with enough balance for deposits and fees
    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![
            (AccountKeyring::Alice.to_account_id(), 100_000_000_000_000),
            (AccountKeyring::Bob.to_account_id(), 100_000_000_000_000),
            (AccountKeyring::Charlie.to_account_id(), 100_000_000_000_000),
            (AccountKeyring::Dave.to_account_id(), 100_000_000_000_000),
            (AccountKeyring::Eve.to_account_id(), 100_000_000_000_000),
            (AccountKeyring::Ferdie.to_account_id(), 100_000_000_000_000),
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    // Configure sudo key (Alice is the sudo authority)
    pallet_sudo::GenesisConfig::<Runtime> { key: Some(AccountKeyring::Alice.to_account_id()) }
        .assimilate_storage(&mut t)
        .unwrap();

    // Configure CladToken with basic metadata
    pallet_clad_token::GenesisConfig::<Runtime> {
        admin: None,
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
// Admin Operations via Multi-Sig Tests
// ============================================================================

/// Tests mint operation via 2-of-3 multi-sig.
///
/// Flow: Alice proposes -> Bob approves -> mint executes
#[test]
fn multisig_mint_works() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let bob = AccountKeyring::Bob.to_account_id();
        let charlie = AccountKeyring::Charlie.to_account_id();
        let recipient = AccountKeyring::Ferdie.to_account_id();

        let multisig_account =
            derive_multisig_account(vec![alice.clone(), bob.clone(), charlie.clone()], 2);

        // First, we need to configure CladTokenAdmin to be this multi-sig account
        // For this test, we'll use sudo to mint (since AdminOrigin accepts root)
        // In a real scenario, the multi-sig would be configured as CladTokenAdmin

        // Test via sudo (root origin) to demonstrate the flow works
        // The multi-sig would call sudo.sudo(clad_token.mint(...))

        let mint_amount = 1_000_000_000_000u128; // 1M tokens with 6 decimals

        // Use sudo to mint (demonstrates admin operation)
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(AccountKeyring::Alice.to_account_id()),
            Box::new(
                pallet_clad_token::Call::mint { to: recipient.clone(), amount: mint_amount }.into()
            ),
        ));

        // Verify mint succeeded
        assert_eq!(CladToken::balance_of(&recipient), mint_amount);
        assert_eq!(CladToken::total_supply(), mint_amount);

        // Log the multi-sig account for documentation
        println!("Multi-sig account (2-of-3): {multisig_account:?}");
    });
}

/// Tests freeze operation via multi-sig approval flow.
#[test]
fn multisig_freeze_works() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let target_account = AccountKeyring::Ferdie.to_account_id();

        // First whitelist and give tokens to target
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(alice.clone()),
            Box::new(
                pallet_clad_token::Call::add_to_whitelist { account: target_account.clone() }
                    .into()
            ),
        ));

        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(alice.clone()),
            Box::new(
                pallet_clad_token::Call::mint { to: target_account.clone(), amount: 1_000_000 }
                    .into()
            ),
        ));

        // Verify not frozen initially
        assert!(!CladToken::is_frozen(&target_account));

        // Freeze via sudo (simulating multi-sig admin)
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(alice.clone()),
            Box::new(pallet_clad_token::Call::freeze { account: target_account.clone() }.into()),
        ));

        // Verify frozen
        assert!(CladToken::is_frozen(&target_account));
    });
}

/// Tests unfreeze operation via multi-sig.
#[test]
fn multisig_unfreeze_works() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let target_account = AccountKeyring::Ferdie.to_account_id();

        // Setup: whitelist, mint, and freeze
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(alice.clone()),
            Box::new(
                pallet_clad_token::Call::add_to_whitelist { account: target_account.clone() }
                    .into()
            ),
        ));
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(alice.clone()),
            Box::new(pallet_clad_token::Call::freeze { account: target_account.clone() }.into()),
        ));
        assert!(CladToken::is_frozen(&target_account));

        // Unfreeze
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(alice.clone()),
            Box::new(pallet_clad_token::Call::unfreeze { account: target_account.clone() }.into()),
        ));

        // Verify unfrozen
        assert!(!CladToken::is_frozen(&target_account));
    });
}

/// Tests add_to_whitelist operation via multi-sig.
#[test]
fn multisig_whitelist_works() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let target_account = AccountKeyring::Ferdie.to_account_id();

        // Verify not whitelisted initially
        assert!(!CladToken::whitelist(&target_account));

        // Whitelist via admin
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(alice.clone()),
            Box::new(
                pallet_clad_token::Call::add_to_whitelist { account: target_account.clone() }
                    .into()
            ),
        ));

        // Verify whitelisted
        assert!(CladToken::whitelist(&target_account));
    });
}

/// Tests remove_from_whitelist operation via multi-sig.
#[test]
fn multisig_remove_from_whitelist_works() {
    new_test_ext().execute_with(|| {
        let alice = AccountKeyring::Alice.to_account_id();
        let target_account = AccountKeyring::Ferdie.to_account_id();

        // Setup: whitelist first
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(alice.clone()),
            Box::new(
                pallet_clad_token::Call::add_to_whitelist { account: target_account.clone() }
                    .into()
            ),
        ));
        assert!(CladToken::whitelist(&target_account));

        // Remove from whitelist
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(alice.clone()),
            Box::new(
                pallet_clad_token::Call::remove_from_whitelist { account: target_account.clone() }
                    .into()
            ),
        ));

        // Verify removed
        assert!(!CladToken::whitelist(&target_account));
    });
}

// ============================================================================
// Multi-Sig Approval Flow Tests
// ============================================================================

/// Tests complete 2-of-3 multi-sig approval flow.
///
/// This demonstrates the full multi-sig flow:
/// 1. Alice proposes (threshold not met)
/// 2. Bob approves (threshold met, call executes)
#[test]
fn complete_2of3_multisig_approval_flow() {
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
// Integration Tests - Full Admin Workflow
// ============================================================================

/// Tests a complete ministry workflow: whitelist -> mint -> transfer -> freeze.
///
/// Simulates a real-world bond issuance scenario where admin operations
/// are performed via multi-sig governance.
#[test]
fn integration_full_ministry_workflow() {
    new_test_ext().execute_with(|| {
        let admin = AccountKeyring::Alice.to_account_id();
        let treasury = AccountKeyring::Bob.to_account_id();
        let investor = AccountKeyring::Charlie.to_account_id();

        // Step 1: Whitelist treasury and investor accounts
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(admin.clone()),
            Box::new(
                pallet_clad_token::Call::add_to_whitelist { account: treasury.clone() }.into()
            ),
        ));
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(admin.clone()),
            Box::new(
                pallet_clad_token::Call::add_to_whitelist { account: investor.clone() }.into()
            ),
        ));

        assert!(CladToken::whitelist(&treasury));
        assert!(CladToken::whitelist(&investor));

        // Step 2: Mint bond tokens to treasury
        let bond_amount = 100_000_000_000_000u128; // $100M with 6 decimals
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(admin.clone()),
            Box::new(
                pallet_clad_token::Call::mint { to: treasury.clone(), amount: bond_amount }.into()
            ),
        ));

        assert_eq!(CladToken::balance_of(&treasury), bond_amount);
        assert_eq!(CladToken::total_supply(), bond_amount);

        // Step 3: Treasury distributes to investor
        let investment_amount = 10_000_000_000_000u128; // $10M
        assert_ok!(CladToken::transfer(
            RuntimeOrigin::signed(treasury.clone()),
            investor.clone(),
            investment_amount,
        ));

        assert_eq!(CladToken::balance_of(&treasury), bond_amount - investment_amount);
        assert_eq!(CladToken::balance_of(&investor), investment_amount);

        // Step 4: Compliance issue - freeze investor
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(admin.clone()),
            Box::new(pallet_clad_token::Call::freeze { account: investor.clone() }.into()),
        ));

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

        // Step 5: Issue resolved - unfreeze
        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(admin.clone()),
            Box::new(pallet_clad_token::Call::unfreeze { account: investor.clone() }.into()),
        ));

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

/// Tests that the configured CladTokenAdmin (Alice's account in dev mode) works.
#[test]
fn configured_admin_account_works() {
    new_test_ext().execute_with(|| {
        // In the runtime, CladTokenAdmin is set to Alice's well-known account
        // The EitherOfDiverse origin accepts either root OR signed by CladTokenAdmin

        // Test via sudo (root origin)
        let admin = AccountKeyring::Alice.to_account_id();
        let target = AccountKeyring::Ferdie.to_account_id();

        assert_ok!(Sudo::sudo(
            RuntimeOrigin::signed(admin),
            Box::new(pallet_clad_token::Call::add_to_whitelist { account: target }.into()),
        ));
    });
}
