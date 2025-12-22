use clad_runtime::{AccountId, Signature, WASM_BINARY};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

pub type ChainSpec = sc_service::GenericChainSpec<Option<()>>;

pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{seed}"), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Development chain specification.
///
/// Two validators (Alice + Bob) for realistic consensus testing.
/// Admin is a 2-of-3 multi-sig (Alice, Bob, Charlie) - no sudo, no bypasses.
///
/// See ADR-004: docs/adr/004-production-runtime-configuration.md
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // 2-of-3 multi-sig admin: Alice, Bob, Charlie
    // Derived address: 5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7
    // Use Polkadot.js Apps → Developer → Utilities → Create multisig to verify
    let admin_multisig = account_id_from_ss58("5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7")
        .expect("Valid SS58 address");

    Ok(ChainSpec::builder(wasm_binary, Default::default())
        .with_name("Clad Studio Development")
        .with_id("clad_dev")
        .with_chain_type(ChainType::Development)
        .with_genesis_config_patch(testnet_genesis(
            // Two validators for consensus
            vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
            // Admin: 2-of-3 multi-sig (Alice, Bob, Charlie)
            admin_multisig.clone(),
            // Endowed accounts (including multi-sig for deposits)
            vec![
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_account_id_from_seed::<sr25519::Public>("Charlie"),
                get_account_id_from_seed::<sr25519::Public>("Dave"),
                get_account_id_from_seed::<sr25519::Public>("Eve"),
                get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                admin_multisig,
            ],
        ))
        .build())
}

/// Parse an SS58 address string into an AccountId.
fn account_id_from_ss58(address: &str) -> Result<AccountId, String> {
    use sp_core::crypto::Ss58Codec;
    AccountId::from_ss58check(address).map_err(|e| format!("Invalid SS58 address: {e:?}"))
}

/// Configure testnet genesis state.
///
/// # Parameters
/// - `initial_authorities`: Validator set for Aura (block production) and Grandpa (finality)
/// - `admin`: Multi-sig account with admin privileges for pallet-clad-token
/// - `endowed_accounts`: Accounts pre-funded with native balance
fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    admin: AccountId,
    endowed_accounts: Vec<AccountId>,
) -> serde_json::Value {
    // Native token endowment: 1,000,000 tokens with 18 decimals (10^18 smallest units)
    // Each test account receives 1M tokens for development/testing
    const ENDOWMENT: u128 = 1_000_000 * 10u128.pow(18);

    serde_json::json!({
        "balances": {
            "balances": endowed_accounts.iter().cloned().map(|k| (k, ENDOWMENT)).collect::<Vec<_>>(),
        },
        "aura": {
            "authorities": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
        },
        "grandpa": {
            "authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1u64)).collect::<Vec<_>>(),
        },
        "cladToken": {
            "admin": admin,
            "tokenName": b"Clad Token".to_vec(),
            "tokenSymbol": b"CLAD".to_vec(),
            "decimals": 6u8,
            "whitelistedAccounts": endowed_accounts,
            "initialBalances": [],
        },
    })
}
