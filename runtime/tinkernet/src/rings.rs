use crate::{
    assets::CORE_ASSET_ID, xcm_config::BaseXcmWeight, Balance, ParachainInfo, Runtime, RuntimeCall,
    RuntimeEvent,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    parameter_types,
    traits::Get,
    weights::{
        Weight, WeightToFee, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
};
use pallet_rings::{ChainAssetsList, ChainList};
use scale_info::TypeInfo;
use smallvec::smallvec;
use sp_runtime::Perbill;
use xcm::prelude::*;

parameter_types! {
    pub ParaId: u32 = ParachainInfo::get().into();
    pub MaxWeightedLength: u32 = 100_000;
}

impl pallet_rings::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ParaId = ParaId;
    type Chains = Chains;
    type MaxWeightedLength = MaxWeightedLength;
    type WeightInfo = pallet_rings::weights::SubstrateWeight<Runtime>;
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum Chains {
    Basilisk,
    Kusama,
    Statemine,

    TinkernetTest,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum ChainAssets {
    Basilisk(BasiliskAssets),
    Kusama(KusamaAssets),
    Statemine(StatemineAssets),

    TinkernetTest(TinkernetTestAssets),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum BasiliskAssets {
    BSX,
    TNKR,
    KSM,
    USDT,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum KusamaAssets {
    KSM,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum StatemineAssets {
    KSM,
    BILLCOIN,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum TinkernetTestAssets {
    TNKR,
}

impl ChainAssetsList for ChainAssets {
    type Chains = Chains;

    fn get_chain(&self) -> Self::Chains {
        match self {
            Self::Basilisk(_) => Chains::Basilisk,
            Self::Kusama(_) => Chains::Kusama,
            Self::Statemine(_) => Chains::Statemine,

            Self::TinkernetTest(_) => Chains::TinkernetTest,
        }
    }

    fn get_asset_id(&self) -> AssetId {
        match self {
            Self::Basilisk(asset) => {
                use BasiliskAssets::*;

                match asset {
                    KSM => AssetId::Concrete(MultiLocation {
                        parents: 1,
                        interior: Junctions::Here,
                    }),
                    USDT => AssetId::Concrete(MultiLocation {
                        parents: 1,
                        interior: Junctions::X3(
                            Junction::Parachain(1000),
                            Junction::PalletInstance(50u8),
                            Junction::GeneralIndex(1984u128),
                        ),
                    }),
                    BSX => AssetId::Concrete(MultiLocation {
                        parents: 0,
                        interior: Junctions::X1(Junction::GeneralIndex(0u128)),
                    }),
                    TNKR => AssetId::Concrete(MultiLocation {
                        parents: 1,
                        interior: Junctions::X2(
                            Junction::Parachain(2125),
                            Junction::GeneralIndex(0u128),
                        ),
                    }),
                }
            }

            Self::Kusama(KusamaAssets::KSM) => AssetId::Concrete(MultiLocation {
                parents: 0,
                interior: Junctions::Here,
            }),

            Self::Statemine(asset) => {
                use StatemineAssets::*;

                match asset {
                    KSM => AssetId::Concrete(MultiLocation {
                        parents: 1,
                        interior: Junctions::Here,
                    }),
                    BILLCOIN => AssetId::Concrete(MultiLocation {
                        parents: 0,
                        interior: Junctions::X2(
                            Junction::PalletInstance(50u8),
                            Junction::GeneralIndex(223u128),
                        ),
                    }),
                }
            }

            Self::TinkernetTest(TinkernetTestAssets::TNKR) => AssetId::Concrete(MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralIndex(CORE_ASSET_ID as u128)),
            }),
        }
    }
}

impl ChainList for Chains {
    type Balance = Balance;
    type ChainAssets = ChainAssets;
    type Call = RuntimeCall;

    fn get_location(&self) -> MultiLocation {
        match self {
            Self::Basilisk => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2090)),
            },

            Self::Kusama => MultiLocation {
                parents: 1,
                interior: Junctions::Here,
            },

            Self::Statemine => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(1000)),
            },

            Self::TinkernetTest => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2126)),
            },
        }
    }

    fn get_main_asset(&self) -> Self::ChainAssets {
        match self {
            Self::Basilisk => ChainAssets::Basilisk(BasiliskAssets::BSX),
            Self::Kusama => ChainAssets::Kusama(KusamaAssets::KSM),
            Self::Statemine => ChainAssets::Statemine(StatemineAssets::KSM),

            Self::TinkernetTest => ChainAssets::TinkernetTest(TinkernetTestAssets::TNKR),
        }
    }

    fn weight_to_fee(&self, weight: &Weight) -> Self::Balance {
        match self {
            Self::Basilisk => BasiliskWeightToFee::weight_to_fee(weight),
            // TODO: Set correct parameters.
            Self::Kusama => crate::WeightToFee::weight_to_fee(weight),
            // TODO: Set correct parameters.
            Self::Statemine => crate::WeightToFee::weight_to_fee(weight),

            Self::TinkernetTest => crate::WeightToFee::weight_to_fee(weight),
        }
    }

    fn base_xcm_weight(&self) -> Weight {
        Weight::from_ref_time(match self {
            // Basilisk's BaseXcmWeight == 100_000_000
            Self::Basilisk => 100_000_000,
            // TODO: Set correct base xcm weight.
            Self::Kusama => 100_000_000,
            // TODO: Set correct base xcm weight.
            Self::Statemine => 100_000_000,

            Self::TinkernetTest => BaseXcmWeight::get(),
        })
    }

    fn xcm_fee(&self, transact_weight: &Weight) -> Result<Self::Balance, ()> {
        Ok(self.weight_to_fee(
            &transact_weight
                .checked_add(&self.base_xcm_weight())
                .ok_or(())?,
        ))
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn benchmark_mock() -> Self {
        Self::Kusama
    }
}

struct BasiliskWeightToFee;
impl WeightToFeePolynomial for BasiliskWeightToFee {
    type Balance = Balance;

    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        // 11 * Basilisk's CENTS
        let p = 11 * 1_000_000_000_000;
        let q =
            // 200 * polkadot-v0.9.29's frame_support::weights::constants::WEIGHT_PER_MICRO.ref_time()
            Balance::from(200u128 * 1_000_000u128);
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}
