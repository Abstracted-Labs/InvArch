// Copyright 2021-2022 InvArch Association.
// This file is part of InvArch.

// InvArch is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// InvArch is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with InvArch.  If not, see <http://www.gnu.org/licenses/>.

//! InvArch Chain Specifications and utilities for building them.
//!
//! Learn more about Substrate chain specifications at
//! https://docs.substrate.io/v3/runtime/chain-specs/

use cumulus_primitives_core::ParaId;

use invarch_runtime::{
    AccountId,
    AuraId,
    Signature,
    EXISTENTIAL_DEPOSIT,
    // EVMConfig,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
// use sp_consensus_aura::sr25519::AuthorityId as AuraId;
// use hex_literal::hex;
use serde::{Deserialize, Serialize};
use sp_core::{
    sr25519,
    Pair,
    Public,
    // H160, U256
};
use sp_runtime::traits::{IdentifyAccount, Verify};
// use std::{collections::BTreeMap, str::FromStr};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<invarch_runtime::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Helper function to generate a crypto pair from seed
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}
impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
    get_public_from_seed::<AuraId>(seed)
}

// TODO: Update to AccountId32::new([..])

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_public_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(keys: AuraId) -> invarch_runtime::SessionKeys {
    invarch_runtime::SessionKeys { aura: keys }
}

pub fn development_config() -> ChainSpec {
    // Give your base currency a unit name and decimal places
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 12u32.into());
    properties.insert("ss58Format".into(), 42u32.into());

    ChainSpec::from_genesis(
        // Name
        "InvArch Dev Net",
        // ID
        "invarch_dev_net",
        ChainType::Development,
        move || {
            testnet_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // initial collators.
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                1000.into(),
            )
        },
        // Bootnodes
        Vec::new(),
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        None,
        // Extensions
        Extensions {
            relay_chain: "rococo-local".into(), // TODO: You MUST set this to the correct network!
            para_id: 1000,
        },
    )
}

pub fn local_testnet_config() -> ChainSpec {
    // Give your base currency a unit name and decimal places
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 12u32.into());
    properties.insert("ss58Format".into(), 42u32.into());

    ChainSpec::from_genesis(
        // Name
        "InvArch Local Testnet",
        // ID
        "invarch_local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // initial collators.
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                1000.into(),
            )
        },
        // Bootnodes
        Vec::new(),
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        None,
        // Extensions
        Extensions {
            relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
            para_id: 1000,
        },
    )
}

pub fn tinkernet_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/tinker-spec-raw.json")[..])
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    root_key: AccountId,
    invulnerables: Vec<(AccountId, AuraId)>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> invarch_runtime::GenesisConfig {
    invarch_runtime::GenesisConfig {
        system: invarch_runtime::SystemConfig {
            // Add Wasm runtime to storage.
            code: invarch_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        balances: invarch_runtime::BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        parachain_info: invarch_runtime::ParachainInfoConfig { parachain_id: id },
        collator_selection: invarch_runtime::CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
            ..Default::default()
        },
        session: invarch_runtime::SessionConfig {
            keys: invulnerables
                .into_iter()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                 // account id
                        acc,                         // validator id
                        template_session_keys(aura), // session keys
                    )
                })
                .collect(),
        },
        // no need to pass anything to aura, in fact it will panic if we do. Session will take care
        // of this.
        aura: Default::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
        polkadot_xcm: invarch_runtime::PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
        },
        sudo: invarch_runtime::SudoConfig {
            // Assign network admin rights.
            key: Some(root_key),
        },
        // evm: EVMConfig {
        //     accounts: {
        //         let mut map = BTreeMap::new();
        //         map.insert(
        //             // H160 address of Alice dev account
        //             // Derived from SS58 (42 prefix) address
        //             // SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
        //             // hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
        //             // Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
        //             H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
        //                 .expect("internal H160 is valid; qed"),
        //             pallet_evm::GenesisAccount {
        //                 balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
        //                     .expect("internal U256 is valid; qed"),
        //                 code: Default::default(),
        //                 nonce: Default::default(),
        //                 storage: Default::default(),
        //             },
        //         );
        //         map.insert(
        //             // H160 address of CI test runner account
        //             H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
        //                 .expect("internal H160 is valid; qed"),
        //             pallet_evm::GenesisAccount {
        //                 balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
        //                     .expect("internal U256 is valid; qed"),
        //                 code: Default::default(),
        //                 nonce: Default::default(),
        //                 storage: Default::default(),
        //             },
        //         );
        //         map
        //     },
        // },
    }
}
