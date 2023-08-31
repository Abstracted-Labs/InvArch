use crate::{Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::GetDispatchInfo;
use pallet_nft_origins::{ChainVerifier, Parachain};
use scale_info::TypeInfo;
use sp_runtime::traits::Dispatchable;
use xcm::latest::Junction;

impl pallet_nft_origins::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type RuntimeOrigin = RuntimeOrigin;

    type Chains = RegisteredChains;
    type RegisteredCalls = RegisteredCalls;
}

pub enum RegisteredChains {
    Moonriver,
    TinkernetTest,
}

impl ChainVerifier for RegisteredChains {
    fn get_chain_from_verifier(para_id: u32, verifier_part: Junction) -> Option<Parachain> {
        match (para_id, verifier_part) {
            // Moonriver
            (
                2023,
                Junction::AccountKey20 {
                    network: None,
                    key: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                },
            ) => Some(Parachain(2023)),

            (2126, Junction::GeneralIndex(69)) => Some(Parachain(2126)),

            // Fallback to storage for testing purposes.
            // In reality these won't change much, so storage won't be necessary.
            _ => {
                if let Some(Parachain(p)) =
                    pallet_nft_origins::RegisteredChains::<Runtime>::get(verifier_part)
                {
                    if p == para_id {
                        Some(Parachain(p))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum RegisteredCalls {
    VoteInCore {
        core_id: <Runtime as pallet_inv4::Config>::CoreId,
        proposal: <Runtime as frame_system::Config>::Hash,
        vote: bool,
    },
    TestNftCall,
}

impl Dispatchable for RegisteredCalls {
    type RuntimeOrigin = <RuntimeCall as Dispatchable>::RuntimeOrigin;
    type Config = <RuntimeCall as Dispatchable>::Config;
    type Info = <RuntimeCall as Dispatchable>::Info;
    type PostInfo = <RuntimeCall as Dispatchable>::PostInfo;

    fn dispatch(
        self,
        origin: Self::RuntimeOrigin,
    ) -> sp_runtime::DispatchResultWithInfo<Self::PostInfo> {
        match self {
            Self::VoteInCore { .. } => todo!(),
            Self::TestNftCall => {
                Ok(pallet_nft_origins::Pallet::<Runtime>::test_nft_location(origin.into())?.into())
            }
        }
    }
}

impl GetDispatchInfo for RegisteredCalls {
    fn get_dispatch_info(&self) -> frame_support::dispatch::DispatchInfo {
        match self {
            Self::VoteInCore { .. } => todo!(),
            Self::TestNftCall => pallet_nft_origins::pallet::Call::<Runtime>::test_nft_location {}
                .get_dispatch_info(),
        }
    }
}
