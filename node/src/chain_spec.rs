use clad_runtime::{AccountId, RuntimeGenesisConfig, Signature, WASM_BINARY};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
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

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::builder(wasm_binary, Default::default())
        .with_name("Clad Studio Development")
        .with_id("clad_dev")
        .with_chain_type(ChainType::Development)
        .with_genesis_config_patch(testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            vec![
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_account_id_from_seed::<sr25519::Public>("Charlie"),
                get_account_id_from_seed::<sr25519::Public>("Dave"),
                get_account_id_from_seed::<sr25519::Public>("Eve"),
            ],
            true,
        ))
        .build())
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::builder(wasm_binary, Default::default())
        .with_name("Clad Studio Local Testnet")
        .with_id("clad_local")
        .with_chain_type(ChainType::Local)
        .with_genesis_config_patch(testnet_genesis(
            vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            vec![
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_account_id_from_seed::<sr25519::Public>("Charlie"),
                get_account_id_from_seed::<sr25519::Public>("Dave"),
                get_account_id_from_seed::<sr25519::Public>("Eve"),
                get_account_id_from_seed::<sr25519::Public>("Ferdie"),
            ],
            true,
        ))
        .build())
}

fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> serde_json::Value {
    const ENDOWMENT: u128 = 1_000_000_000_000_000;

    serde_json::json!({
        "balances": {
            "balances": endowed_accounts.iter().cloned().map(|k| (k, ENDOWMENT)).collect::<Vec<_>>(),
        },
        "aura": {
            "authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>(),
        },
        "grandpa": {
            "authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1u64)).collect::<Vec<_>>(),
        },
        "sudo": {
            "key": Some(root_key.clone()),
        },
        "cladToken": {
            "admin": root_key,
            "whitelisted_accounts": endowed_accounts,
        },
    })
}
