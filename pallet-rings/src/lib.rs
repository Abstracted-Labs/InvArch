#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Get;
use sp_std::convert::TryInto;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod traits;
pub mod weights;

pub use pallet::*;
pub use traits::{ChainAssetsList, ChainList};
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::OriginFor;
    use pallet_inv4::origin::{ensure_multisig, INV4Origin};
    use sp_std::{vec, vec::Vec};
    use xcm::{latest::prelude::*, DoubleEncoded};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_inv4::Config + pallet_xcm::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Chains: ChainList;

        #[pallet::constant]
        type ParaId: Get<u32>;

        #[pallet::constant]
        type MaxWeightedLength: Get<u32>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        SendingFailed,
        WeightTooHigh,
        FailedToCalculateXcmFee,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        CallSent {
            sender: <T as pallet_inv4::Config>::CoreId,
            destination: <T as pallet::Config>::Chains,
            call: Vec<u8>,
        },

        AssetsTransferred {
            chain: <<<T as pallet::Config>::Chains as ChainList>::ChainAssets as ChainAssetsList>::Chains,
            asset: <<T as pallet::Config>::Chains as ChainList>::ChainAssets,
            amount: u128,
            from: <T as pallet_inv4::Config>::CoreId,
            to: <T as frame_system::Config>::AccountId,
        },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        Result<
            INV4Origin<
                T,
                <T as pallet_inv4::Config>::CoreId,
                <T as frame_system::Config>::AccountId,
            >,
            <T as frame_system::Config>::RuntimeOrigin,
        >: From<<T as frame_system::Config>::RuntimeOrigin>,

        <T as pallet_inv4::Config>::CoreId: Into<u32>,

        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
    {
        #[pallet::call_index(0)]
        #[pallet::weight(
            <T as Config>::WeightInfo::send_call(
                (call.len() as u32)
                    .min(T::MaxWeightedLength::get())
            )
        )]
        pub fn send_call(
            origin: OriginFor<T>,
            destination: <T as pallet::Config>::Chains,
            weight: Weight,
            call: Vec<u8>,
        ) -> DispatchResult {
            let core = ensure_multisig::<T, OriginFor<T>>(origin)?;
            let core_id = core.id.into();

            let interior = Junctions::X1(Junction::Plurality {
                id: BodyId::Index(core_id),
                part: BodyPart::Voice,
            });

            let dest = destination.get_location();
            let dest_asset = destination.get_main_asset().get_asset_id();

            let beneficiary: MultiLocation = MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(<T as pallet::Config>::ParaId::get()),
                    Junction::Plurality {
                        id: BodyId::Index(core_id),
                        part: BodyPart::Voice,
                    },
                ),
            };

            let xcm_fee = destination
                .xcm_fee(&weight.checked_mul(2).ok_or(Error::<T>::WeightTooHigh)?)
                .map_err(|_| Error::<T>::FailedToCalculateXcmFee)?;

            let fee_multiasset = MultiAsset {
                id: dest_asset,
                fun: Fungibility::Fungible(xcm_fee.into()),
            };

            let message = Xcm(vec![
                Instruction::WithdrawAsset(fee_multiasset.clone().into()),
                Instruction::BuyExecution {
                    fees: fee_multiasset,
                    weight_limit: WeightLimit::Unlimited,
                },
                Instruction::Transact {
                    origin_type: OriginKind::Native,
                    require_weight_at_most: weight
                        .checked_mul(2)
                        .ok_or(Error::<T>::WeightTooHigh)?
                        .ref_time(),
                    call: <DoubleEncoded<_> as From<Vec<u8>>>::from(call.clone()),
                },
                Instruction::RefundSurplus,
                Instruction::DepositAsset {
                    assets: MultiAssetFilter::Wild(WildMultiAsset::All),
                    max_assets: 1,
                    beneficiary,
                },
            ]);

            pallet_xcm::Pallet::<T>::send_xcm(interior, dest, message)
                .map_err(|_| Error::<T>::SendingFailed)?;

            Self::deposit_event(Event::CallSent {
                sender: core.id,
                destination,
                call,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::transfer_assets())]
        pub fn transfer_assets(
            origin: OriginFor<T>,
            asset: <<T as pallet::Config>::Chains as ChainList>::ChainAssets,
            amount: u128,
            to: <T as frame_system::Config>::AccountId,
        ) -> DispatchResult {
            let core = ensure_multisig::<T, OriginFor<T>>(origin)?;
            let core_id = core.id.into();

            let interior = Junctions::X1(Junction::Plurality {
                id: BodyId::Index(core_id),
                part: BodyPart::Voice,
            });

            let chain = asset.get_chain();

            let dest = chain.get_location();

            let asset_id = asset.get_asset_id();

            let multi_asset = MultiAsset {
                id: asset_id,
                fun: Fungibility::Fungible(amount),
            };

            let beneficiary: MultiLocation = MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::AccountId32 {
                    network: NetworkId::Any,
                    id: to.clone().into(),
                }),
            };

            let core_multilocation: MultiLocation = MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(<T as pallet::Config>::ParaId::get()),
                    Junction::Plurality {
                        id: BodyId::Index(core_id),
                        part: BodyPart::Voice,
                    },
                ),
            };

            let xcm_fee = chain
                .xcm_fee(&Weight::zero())
                .map_err(|_| Error::<T>::FailedToCalculateXcmFee)?;

            let fee_multiasset = MultiAsset {
                id: chain.get_main_asset().get_asset_id(),
                fun: Fungibility::Fungible(xcm_fee.into()),
            };

            let message = Xcm(vec![
                // Pay execution fees
                Instruction::WithdrawAsset(fee_multiasset.clone().into()),
                Instruction::BuyExecution {
                    fees: fee_multiasset,
                    weight_limit: WeightLimit::Unlimited,
                },
                // Actual transfer instruction
                Instruction::TransferAsset {
                    assets: multi_asset.into(),
                    beneficiary,
                },
                // Refund unused fees
                Instruction::RefundSurplus,
                Instruction::DepositAsset {
                    assets: MultiAssetFilter::Wild(WildMultiAsset::All),
                    max_assets: 1,
                    beneficiary: core_multilocation,
                },
            ]);

            pallet_xcm::Pallet::<T>::send_xcm(interior, dest, message)
                .map_err(|_| Error::<T>::SendingFailed)?;

            Self::deposit_event(Event::AssetsTransferred {
                chain,
                asset,
                amount,
                from: core.id,
                to,
            });

            Ok(())
        }
    }
}
