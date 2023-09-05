use crate::{Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin};
use codec::{Decode, Encode};
use frame_support::dispatch::GetDispatchInfo;
use pallet_nft_origins::{ChainVerifier, Parachain};
use scale_info::TypeInfo;
use sp_runtime::{traits::Dispatchable, BoundedVec};
use sp_std::boxed::Box;
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

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
#[repr(u8)]
pub enum RegisteredCalls {
    VoteInMultisig {
        core_id: <Runtime as pallet_inv4::Config>::CoreId,
        call_hash: <Runtime as frame_system::Config>::Hash,
        aye: bool,
    } = 0,

    WithdrawVoteInMultisig {
        core_id: <Runtime as pallet_inv4::Config>::CoreId,
        call_hash: <Runtime as frame_system::Config>::Hash,
    } = 1,

    OperateMultisig {
        core_id: <Runtime as pallet_inv4::Config>::CoreId,
        metadata: Option<BoundedVec<u8, <Runtime as pallet_inv4::Config>::MaxMetadata>>,
        fee_asset: pallet_inv4::fee_handling::FeeAsset,
        call: Box<<Runtime as pallet_inv4::Config>::RuntimeCall>,
    } = 2,

    TestNftCall = 3,
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
            Self::VoteInMultisig {
                core_id,
                call_hash,
                aye,
            } => Ok(pallet_inv4::Pallet::<Runtime>::nft_vote_multisig(
                origin.into(),
                core_id,
                call_hash,
                aye,
            )?
            .into()),

            Self::WithdrawVoteInMultisig { core_id, call_hash } => {
                Ok(pallet_inv4::Pallet::<Runtime>::nft_withdraw_vote_multisig(
                    origin.into(),
                    core_id,
                    call_hash,
                )?
                .into())
            }

            Self::OperateMultisig {
                core_id,
                metadata,
                fee_asset,
                call,
            } => Ok(pallet_inv4::Pallet::<Runtime>::nft_operate_multisig(
                origin.into(),
                core_id,
                metadata,
                fee_asset,
                call,
            )?
            .into()),

            Self::TestNftCall => {
                Ok(pallet_nft_origins::Pallet::<Runtime>::test_nft_location(origin.into())?.into())
            }
        }
    }
}

impl GetDispatchInfo for RegisteredCalls {
    fn get_dispatch_info(&self) -> frame_support::dispatch::DispatchInfo {
        match self.clone() {
            Self::VoteInMultisig {
                core_id,
                call_hash,
                aye,
            } => pallet_inv4::pallet::Call::<Runtime>::nft_vote_multisig {
                core_id,
                call_hash,
                aye,
            }
            .get_dispatch_info(),

            Self::WithdrawVoteInMultisig { core_id, call_hash } => {
                pallet_inv4::pallet::Call::<Runtime>::nft_withdraw_vote_multisig {
                    core_id,
                    call_hash,
                }
                .get_dispatch_info()
            }

            Self::OperateMultisig {
                core_id,
                metadata,
                fee_asset,
                call,
            } => pallet_inv4::pallet::Call::<Runtime>::nft_operate_multisig {
                core_id,
                metadata,
                fee_asset,
                call,
            }
            .get_dispatch_info(),

            Self::TestNftCall => pallet_nft_origins::pallet::Call::<Runtime>::test_nft_location {}
                .get_dispatch_info(),
        }
    }
}
