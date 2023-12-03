#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Get;
use sp_std::convert::TryInto;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

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
    use xcm::{
        latest::{prelude::*, MultiAsset, Weight, WildMultiAsset},
        DoubleEncoded,
    };

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_inv4::Config + pallet_xcm::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Chains: ChainList;

        #[pallet::constant]
        type MaxXCMCallLength: Get<u32>;

        type MaintenanceOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn is_under_maintenance)]
    pub type ChainsUnderMaintenance<T: Config> =
        StorageMap<_, Blake2_128Concat, MultiLocation, bool>;

    #[pallet::error]
    pub enum Error<T> {
        SendingFailed,
        WeightTooHigh,
        FailedToCalculateXcmFee,
        FailedToReanchorAsset,
        FailedToInvertLocation,
        DifferentChains,
        ChainUnderMaintenance,
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

        AssetsBridged {
            origin_chain_asset: <<T as pallet::Config>::Chains as ChainList>::ChainAssets,
            amount: u128,
            from: <T as pallet_inv4::Config>::CoreId,
            to: <T as frame_system::Config>::AccountId,
        },

        ChainMaintenanceStatusChanged {
            chain: <T as Config>::Chains,
            under_maintenance: bool,
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        Result<INV4Origin<T>, <T as frame_system::Config>::RuntimeOrigin>:
            From<<T as frame_system::Config>::RuntimeOrigin>,

        <T as pallet_inv4::Config>::CoreId: Into<u32>,

        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
        T::AccountId: From<[u8; 32]>,
    {
        #[pallet::call_index(0)]
        #[pallet::weight((<T as Config>::WeightInfo::set_maintenance_status(), Pays::No))]
        pub fn set_maintenance_status(
            origin: OriginFor<T>,
            chain: <T as Config>::Chains,
            under_maintenance: bool,
        ) -> DispatchResult {
            T::MaintenanceOrigin::ensure_origin(origin)?;

            ChainsUnderMaintenance::<T>::insert(chain.get_location(), under_maintenance);

            Self::deposit_event(Event::<T>::ChainMaintenanceStatusChanged {
                chain,
                under_maintenance,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(
            <T as Config>::WeightInfo::send_call(call.len() as u32)
        )]
        pub fn send_call(
            origin: OriginFor<T>,
            destination: <T as pallet::Config>::Chains,
            weight: Weight,
            fee_asset: <<T as pallet::Config>::Chains as ChainList>::ChainAssets,
            fee: u128,
            call: BoundedVec<u8, T::MaxXCMCallLength>,
        ) -> DispatchResult {
            let core = ensure_multisig::<T, OriginFor<T>>(origin)?;
            let core_id = core.id.into();

            let dest = destination.get_location();

            ensure!(
                !Self::is_under_maintenance(dest).unwrap_or(false),
                Error::<T>::ChainUnderMaintenance
            );

            let descend_interior = Junction::Plurality {
                id: BodyId::Index(core_id),
                part: BodyPart::Voice,
            };

            let fee_asset_location = fee_asset.get_asset_location();

            let beneficiary: MultiLocation = MultiLocation {
                parents: 0,
                interior: Junctions::X3(
                    Junction::GlobalConsensus(<T as pallet_inv4::Config>::GLOBAL_NETWORK_ID),
                    Junction::Parachain(<T as pallet_inv4::Config>::PARA_ID),
                    descend_interior,
                ),
            };

            let fee_multiasset = MultiAsset {
                id: AssetId::Concrete(fee_asset_location),
                fun: Fungibility::Fungible(fee),
            };

            let message = Xcm(vec![
                Instruction::UniversalOrigin(Junction::GlobalConsensus(
                    <T as pallet_inv4::Config>::GLOBAL_NETWORK_ID,
                )),
                Instruction::DescendOrigin(descend_interior.into()),
                Instruction::WithdrawAsset(fee_multiasset.clone().into()),
                Instruction::BuyExecution {
                    fees: fee_multiasset,
                    weight_limit: WeightLimit::Unlimited,
                },
                Instruction::Transact {
                    origin_kind: OriginKind::Native,
                    require_weight_at_most: weight,
                    call: <DoubleEncoded<_> as From<Vec<u8>>>::from(call.clone().to_vec()),
                },
                Instruction::RefundSurplus,
                Instruction::DepositAsset {
                    assets: MultiAssetFilter::Wild(WildMultiAsset::All),
                    beneficiary,
                },
            ]);

            pallet_xcm::Pallet::<T>::send_xcm(Junctions::Here, dest, message)
                .map_err(|_| Error::<T>::SendingFailed)?;

            Self::deposit_event(Event::CallSent {
                sender: core.id,
                destination,
                call: call.to_vec(),
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::transfer_assets())]
        pub fn transfer_assets(
            origin: OriginFor<T>,
            asset: <<T as pallet::Config>::Chains as ChainList>::ChainAssets,
            amount: u128,
            to: <T as frame_system::Config>::AccountId,
            fee_asset: <<T as pallet::Config>::Chains as ChainList>::ChainAssets,
            fee: u128,
        ) -> DispatchResult {
            let core = ensure_multisig::<T, OriginFor<T>>(origin)?;
            let core_id = core.id.into();

            let chain = asset.get_chain();
            let dest = chain.get_location();

            ensure!(
                !Self::is_under_maintenance(dest).unwrap_or(false),
                Error::<T>::ChainUnderMaintenance
            );

            ensure!(chain == fee_asset.get_chain(), Error::<T>::DifferentChains);

            let descend_interior = Junction::Plurality {
                id: BodyId::Index(core_id),
                part: BodyPart::Voice,
            };

            let asset_location = asset.get_asset_location();

            let multi_asset = MultiAsset {
                id: AssetId::Concrete(asset_location),
                fun: Fungibility::Fungible(amount),
            };

            let beneficiary: MultiLocation = MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::AccountId32 {
                    network: None,
                    id: to.clone().into(),
                }),
            };

            let core_multilocation: MultiLocation = MultiLocation {
                parents: 0,
                interior: Junctions::X3(
                    Junction::GlobalConsensus(<T as pallet_inv4::Config>::GLOBAL_NETWORK_ID),
                    Junction::Parachain(<T as pallet_inv4::Config>::PARA_ID),
                    descend_interior,
                ),
            };

            let fee_multiasset = MultiAsset {
                id: AssetId::Concrete(fee_asset.get_asset_location()),
                fun: Fungibility::Fungible(fee),
            };

            let message = Xcm(vec![
                Instruction::UniversalOrigin(Junction::GlobalConsensus(
                    <T as pallet_inv4::Config>::GLOBAL_NETWORK_ID,
                )),
                Instruction::DescendOrigin(descend_interior.into()),
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
                    beneficiary: core_multilocation,
                },
            ]);

            pallet_xcm::Pallet::<T>::send_xcm(Junctions::Here, dest, message)
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

        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::bridge_assets())]
        pub fn bridge_assets(
            origin: OriginFor<T>,
            asset: <<T as pallet::Config>::Chains as ChainList>::ChainAssets,
            destination: <<<T as pallet::Config>::Chains as ChainList>::ChainAssets as ChainAssetsList>::Chains,
            fee: u128,
            amount: u128,
            to: Option<<T as frame_system::Config>::AccountId>,
        ) -> DispatchResult {
            let core = ensure_multisig::<T, OriginFor<T>>(origin)?;

            let core_id = core.id.into();
            let core_account = core.to_account_id();

            let from_chain = asset.get_chain();
            let from_chain_location = from_chain.get_location();

            let dest = destination.get_location();

            ensure!(
                !(Self::is_under_maintenance(from_chain_location).unwrap_or(false)
                    || Self::is_under_maintenance(dest).unwrap_or(false)),
                Error::<T>::ChainUnderMaintenance
            );

            let descend_interior = Junction::Plurality {
                id: BodyId::Index(core_id),
                part: BodyPart::Voice,
            };

            let asset_location = asset.get_asset_location();

            let inverted_destination = dest
                .reanchored(&from_chain_location, *from_chain_location.interior())
                .map(|inverted| {
                    if let (ml, Some(Junction::OnlyChild) | None) = inverted.split_last_interior() {
                        ml
                    } else {
                        inverted
                    }
                })
                .map_err(|_| Error::<T>::FailedToInvertLocation)?;

            let multiasset = MultiAsset {
                id: AssetId::Concrete(asset_location),
                fun: Fungibility::Fungible(amount),
            };

            let fee_multiasset = MultiAsset {
                id: AssetId::Concrete(asset_location),
                fun: Fungibility::Fungible(fee),
            };

            let reanchored_multiasset = multiasset
                .clone()
                .reanchored(&dest, *from_chain_location.interior())
                .map(|mut reanchored| {
                    if let AssetId::Concrete(ref mut m) = reanchored.id {
                        if let (ml, Some(Junction::OnlyChild) | None) = (*m).split_last_interior() {
                            *m = ml;
                        }
                    }
                    reanchored
                })
                .map_err(|_| Error::<T>::FailedToReanchorAsset)?;

            let core_multilocation: MultiLocation = MultiLocation {
                parents: 0,
                interior: Junctions::X3(
                    Junction::GlobalConsensus(<T as pallet_inv4::Config>::GLOBAL_NETWORK_ID),
                    Junction::Parachain(<T as pallet_inv4::Config>::PARA_ID),
                    descend_interior,
                ),
            };

            let beneficiary: MultiLocation = match to.clone() {
                Some(to_inner) => MultiLocation {
                    parents: 0,
                    interior: Junctions::X1(Junction::AccountId32 {
                        network: None,
                        id: to_inner.into(),
                    }),
                },
                None => core_multilocation,
            };

            let message = if asset_location.starts_with(&dest) {
                Xcm(vec![
                    Instruction::UniversalOrigin(Junction::GlobalConsensus(
                        <T as pallet_inv4::Config>::GLOBAL_NETWORK_ID,
                    )),
                    Instruction::DescendOrigin(descend_interior.into()),
                    WithdrawAsset(vec![fee_multiasset.clone(), multiasset.clone()].into()),
                    Instruction::BuyExecution {
                        fees: fee_multiasset,
                        weight_limit: WeightLimit::Unlimited,
                    },
                    InitiateReserveWithdraw {
                        assets: multiasset.into(),
                        reserve: inverted_destination,
                        xcm: Xcm(vec![
                            Instruction::BuyExecution {
                                fees: reanchored_multiasset,
                                weight_limit: WeightLimit::Unlimited,
                            },
                            Instruction::DepositAsset {
                                assets: All.into(),
                                beneficiary,
                            },
                            Instruction::RefundSurplus,
                            Instruction::DepositAsset {
                                assets: All.into(),
                                beneficiary,
                            },
                        ]),
                    },
                    Instruction::RefundSurplus,
                    Instruction::DepositAsset {
                        assets: All.into(),
                        beneficiary: core_multilocation,
                    },
                ])
            } else {
                Xcm(vec![
                    Instruction::UniversalOrigin(Junction::GlobalConsensus(
                        <T as pallet_inv4::Config>::GLOBAL_NETWORK_ID,
                    )),
                    Instruction::DescendOrigin(descend_interior.into()),
                    // Pay execution fees
                    Instruction::WithdrawAsset(fee_multiasset.clone().into()),
                    Instruction::BuyExecution {
                        fees: fee_multiasset,
                        weight_limit: WeightLimit::Unlimited,
                    },
                    // Actual reserve transfer instruction
                    Instruction::TransferReserveAsset {
                        assets: multiasset.into(),
                        dest: inverted_destination,
                        xcm: Xcm(vec![
                            Instruction::BuyExecution {
                                fees: reanchored_multiasset,
                                weight_limit: WeightLimit::Unlimited,
                            },
                            Instruction::DepositAsset {
                                assets: MultiAssetFilter::Wild(WildMultiAsset::All),
                                beneficiary,
                            },
                        ]),
                    },
                    // Refund unused fees
                    Instruction::RefundSurplus,
                    Instruction::DepositAsset {
                        assets: MultiAssetFilter::Wild(WildMultiAsset::All),
                        beneficiary: core_multilocation,
                    },
                ])
            };

            pallet_xcm::Pallet::<T>::send_xcm(Junctions::Here, from_chain_location, message)
                .map_err(|_| Error::<T>::SendingFailed)?;

            Self::deposit_event(Event::AssetsBridged {
                origin_chain_asset: asset,
                from: core.id,
                amount,
                to: to.unwrap_or(core_account),
            });

            Ok(())
        }
    }
}
