//! Shared test harness for roundtrip-node integration tests.
//!
//! All helpers in this module panic on RPC error — this is test code, not production code.
//!
//! Used by:
//!   - `tests/extrinsic_signed.rs` (`signed_extrinsic_roundtrip_against_node`)
//!   - `tests/extrinsic_signing_payload.rs` (`build_signing_payload_large_payload_is_hashed`)
//!
//! Requires a running `clad-node --dev --rpc-port 9944` instance.
//! Set `CLAD_DEV_RPC_URL` to override the default endpoint.

use signer_core::extrinsic::signed_extensions::ChainInfo;

// ── Well-known dev-chain constants ────────────────────────────────────────────

/// SR25519 mini-secret (seed) for the `//Alice` development key.
///
/// Source: Substrate `sp_keyring::Sr25519Keyring::Alice.to_raw_vec()`.
/// This is publicly documented and not a secret.
const ALICE_SEED: [u8; 32] = [
    0xe5, 0xbe, 0x9a, 0x50, 0x92, 0xb8, 0x1b, 0xca, 0x64, 0xbe, 0x81, 0xd2, 0x12, 0xe7, 0xf2, 0xf9,
    0xeb, 0xa1, 0x83, 0xbb, 0x7a, 0x90, 0x95, 0x4f, 0x7b, 0x76, 0x36, 0x1f, 0x6e, 0xdb, 0x5c, 0x0a,
];

/// Hex-encoded `//Alice` AccountId used as the account identifier in RPC calls.
const ALICE_ACCOUNT_HEX: &str =
    "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

/// Well-known `//Bob` AccountId (sr25519 public key) for the dev chain.
///
/// Source: `sp_keyring::Sr25519Keyring::Bob.to_raw_vec()`.
const BOB_ACCOUNT_ID: [u8; 32] = [
    0x8e, 0xaf, 0x04, 0x15, 0x16, 0x87, 0x73, 0x63, 0x26, 0xc9, 0xfe, 0xa1, 0x7e, 0x25, 0xfc, 0x52,
    0x87, 0x61, 0x36, 0x93, 0xc9, 0x12, 0x90, 0x9c, 0xb2, 0x26, 0xaa, 0x47, 0x94, 0xf2, 0x6a, 0x48,
];

// ── Sr25519Keypair ────────────────────────────────────────────────────────────

/// A test-only keypair holding the seed and derived public key.
pub struct Sr25519Keypair {
    pub seed: [u8; 32],
    pub public_key: Vec<u8>,
}

impl Sr25519Keypair {
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        signer_core::crypto::sr25519::sign(message, &self.seed).expect("sr25519 sign failed")
    }
}

// ── RPC helpers ───────────────────────────────────────────────────────────────

/// RPC endpoint. Reads `CLAD_DEV_RPC_URL`, defaulting to `http://127.0.0.1:9944`.
pub fn rpc_url() -> String {
    std::env::var("CLAD_DEV_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:9944".to_string())
}

/// Blocking JSON-RPC POST. Panics on any network or JSON error.
pub fn rpc_request(method: &str, params: serde_json::Value) -> serde_json::Value {
    let client = reqwest::blocking::Client::new();
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    client
        .post(rpc_url())
        .json(&body)
        .send()
        .unwrap_or_else(|e| panic!("RPC request '{method}' failed: {e}"))
        .json::<serde_json::Value>()
        .unwrap_or_else(|e| panic!("RPC response '{method}' not valid JSON: {e}"))
}

// ── Chain info ────────────────────────────────────────────────────────────────

/// Fetch chain information needed for extrinsic construction.
///
/// Calls `chain_getBlockHash(0)` for genesis, then `state_getRuntimeVersion`
/// for spec/tx versions.  For immortal transactions the block_hash equals
/// the genesis hash.
pub fn chain_info() -> ChainInfo {
    let genesis_resp = rpc_request("chain_getBlockHash", serde_json::json!([0u32]));
    let genesis_hex =
        genesis_resp["result"].as_str().expect("chain_getBlockHash returned non-string");
    let genesis_hash =
        hex::decode(genesis_hex.trim_start_matches("0x")).expect("genesis hash is valid hex");
    assert_eq!(genesis_hash.len(), 32, "genesis hash must be 32 bytes");

    let version_resp = rpc_request("state_getRuntimeVersion", serde_json::json!([]));
    let result = &version_resp["result"];
    let spec_version = result["specVersion"].as_u64().expect("specVersion missing") as u32;
    let tx_version =
        result["transactionVersion"].as_u64().expect("transactionVersion missing") as u32;

    ChainInfo {
        block_hash: genesis_hash.clone(), // immortal: block_hash == genesis_hash
        genesis_hash,
        spec_version,
        tx_version,
    }
}

// ── Keypair helpers ───────────────────────────────────────────────────────────

/// Derive the `//Alice` sr25519 keypair from the well-known mini-secret seed.
pub fn alice_sr25519_keypair() -> Sr25519Keypair {
    let public_key = signer_core::crypto::sr25519::public_key_from_seed(&ALICE_SEED)
        .expect("Alice keypair derivation failed");
    Sr25519Keypair { seed: ALICE_SEED, public_key }
}

/// Return the well-known `//Bob` AccountId for the dev chain.
pub fn bob_account_id() -> [u8; 32] {
    BOB_ACCOUNT_ID
}

/// Fetch the current next nonce (account index) for the given 0x-hex account ID.
///
/// `system_accountNextIndex` requires an SS58 address; converts hex → SS58(42) internally.
pub fn account_nonce(account_hex: &str) -> u64 {
    let bytes = hex::decode(account_hex.trim_start_matches("0x")).expect("invalid account hex");
    let ss58 = signer_core::crypto::ss58::encode(&bytes, 42).expect("SS58 encode failed");
    let resp = rpc_request("system_accountNextIndex", serde_json::json!([ss58]));
    resp["result"].as_u64().expect("system_accountNextIndex returned non-u64")
}

// ── Submission ────────────────────────────────────────────────────────────────

/// Result of a watched extrinsic submission.
pub struct BlockInclusion {
    /// Extrinsic hash hex string as returned by `author_submitExtrinsic`.
    pub tx_hash_hex: String,
}

/// Submit a signed extrinsic and wait for it to be included in a block.
///
/// Polls `system_accountNextIndex` for Alice's account (the dev-chain signer)
/// every 500ms for up to 30s.  Panics on timeout.
pub fn submit_and_watch(extrinsic: &[u8]) -> BlockInclusion {
    let hex_encoded = format!("0x{}", hex::encode(extrinsic));

    let initial_nonce = account_nonce(ALICE_ACCOUNT_HEX);

    let resp = rpc_request("author_submitExtrinsic", serde_json::json!([hex_encoded]));
    if let Some(err) = resp.get("error") {
        panic!("author_submitExtrinsic returned RPC error: {err}");
    }
    let tx_hash_hex = resp["result"]
        .as_str()
        .expect("author_submitExtrinsic should return a string hash")
        .to_string();

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if std::time::Instant::now() > deadline {
            panic!(
                "extrinsic not included within 30s (tx_hash: {tx_hash_hex}, initial_nonce: {initial_nonce})"
            );
        }
        let nonce = account_nonce(ALICE_ACCOUNT_HEX);
        if nonce > initial_nonce {
            return BlockInclusion { tx_hash_hex };
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
