//! Roundtrip test for `build_signed_extrinsic` against a live `clad-node --dev`.
//!
//! The `signed_extrinsic_roundtrip_against_node` test requires a running node and
//! is gated behind the `roundtrip-node` Cargo feature.  The CI job `roundtrip-node`
//! in `.github/workflows/signer-core.yml` boots the node and enables this feature.
//!
//! Run locally:
//! ```bash
//! ./target/release/clad-node --dev --tmp --rpc-port 9944 --rpc-cors all &
//! ./scripts/wait-for-rpc.sh
//! cargo test -p signer-core --features roundtrip-node --locked -- --test-threads=1
//! ```

mod common;

/// Submit a `CladToken::transfer` from Alice to Bob against a live dev node and
/// verify the extrinsic is included in a block within 30s.
///
/// The transfer is expected to fail on-chain (Alice has no CladToken balance)
/// but the extrinsic IS included: Substrate includes all signed extrinsics with
/// valid signatures and nonces, even when the pallet dispatch returns an error.
/// Inclusion (nonce increment) is the proof that our signing implementation
/// produces well-formed extrinsics the runtime accepts.
#[cfg_attr(
    not(feature = "roundtrip-node"),
    ignore = "Phase 2b: requires clad-node --dev (enable with --features roundtrip-node)"
)]
#[test]
fn signed_extrinsic_roundtrip_against_node() {
    use signer_core::extrinsic::{call, payload, signed, signed_extensions};

    let chain = common::chain_info();
    let alice = common::alice_sr25519_keypair();
    let bob = common::bob_account_id();

    // Build a CladToken::transfer call: Alice → Bob, amount = 1.
    let call_data = call::transfer(&bob, 1);

    // Fetch Alice's current on-chain nonce.
    let nonce =
        common::account_nonce("0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");

    let extra = signed_extensions::SignedExtra {
        era_period: 0, // immortal
        era_phase: 0,
        nonce,
        tip: 0,
    };

    // Build the signing payload and sign it.
    let signing_payload = payload::build_signing_payload(&call_data, &extra, &chain);
    let signature = alice.sign(&signing_payload);

    // Assemble the signed extrinsic.
    let tx = signed::build_signed_extrinsic(&call_data, &alice.public_key, &signature, &extra);

    // Submit and wait for inclusion.
    let inclusion = common::submit_and_watch(&tx.encoded);

    // The tx hash returned by the node must equal Blake2b-256 of the submitted bytes.
    let expected_hash = format!("0x{}", hex::encode(&tx.hash));
    assert_eq!(
        inclusion.tx_hash_hex, expected_hash,
        "tx hash mismatch: node returned {}, expected {}",
        inclusion.tx_hash_hex, expected_hash
    );
}
