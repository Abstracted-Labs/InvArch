use crate::{Runtime, RuntimeEvent};
use pallet_nft_origins::{Chain, ChainVerifier};
use xcm::latest::Junction;

pub enum RegisteredChains {
    Moonriver,
    TinkernetTest,
}

impl ChainVerifier for RegisteredChains {
    fn get_chain_from_verifier(para_id: u32, verifier_part: Junction) -> Option<Chain> {
        match (para_id, verifier_part) {
            // Moonriver
            (
                2023,
                Junction::AccountKey20 {
                    network: None,
                    key: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                },
            ) => Some(Chain::Parachain(2023)),

            (2126, Junction::GeneralIndex(69)) => Some(Chain::Parachain(2126)),

            _ => None,
        }
    }
}

impl pallet_nft_origins::Config for Runtime {
    type Chains = RegisteredChains;

    type RuntimeEvent = RuntimeEvent;
}
