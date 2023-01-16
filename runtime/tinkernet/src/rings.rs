use crate::{assets::CORE_ASSET_ID, Balance, Event, ParachainInfo, Runtime};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    parameter_types,
    traits::Get,
    weights::{
        Weight, WeightToFee, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
};
use pallet_rings::{ParachainAssetsList, ParachainList};
use scale_info::TypeInfo;
use smallvec::smallvec;
use sp_runtime::Perbill;
use xcm::prelude::*;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum Parachains {
    Basilisk,
    TinkernetTest,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum ParachainAssets {
    Basilisk(BasiliskAssets),
    TinkernetTest(TinkernetTestAssets),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum BasiliskAssets {
    BSX,
    TNKR,
    KSM,
    AUSD,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum TinkernetTestAssets {
    TNKR,
}

impl ParachainAssetsList for ParachainAssets {
    type Parachains = Parachains;

    fn get_parachain(&self) -> Self::Parachains {
        match self {
            Self::Basilisk(_) => Parachains::Basilisk,
            Self::TinkernetTest(_) => Parachains::TinkernetTest,
        }
    }

    fn get_asset_id(&self) -> AssetId {
        match self {
            Self::Basilisk(_) => AssetId::Concrete(MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralIndex(0u128)),
            }),

            Self::TinkernetTest(_) => AssetId::Concrete(MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralIndex(CORE_ASSET_ID as u128)),
            }),
        }
    }
}

impl ParachainList for Parachains {
    type Balance = Balance;
    type ParachainAssets = ParachainAssets;

    fn from_para_id(para_id: u32) -> Option<Self> {
        match para_id {
            2090 => Some(Self::Basilisk),
            2126 => Some(Self::TinkernetTest),

            _ => None,
        }
    }

    fn get_location(&self) -> MultiLocation {
        match self {
            Self::Basilisk => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2090)),
            },

            Self::TinkernetTest => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2126)),
            },
        }
    }

    fn get_main_asset(&self) -> Self::ParachainAssets {
        match self {
            Self::Basilisk => ParachainAssets::Basilisk(BasiliskAssets::BSX),
            Self::TinkernetTest => ParachainAssets::TinkernetTest(TinkernetTestAssets::TNKR),
        }
    }

    fn weight_to_fee(&self, weight: &Weight) -> Self::Balance {
        match self {
            Self::Basilisk => BasiliskWeightToFee::weight_to_fee(weight),

            Self::TinkernetTest => crate::WeightToFee::weight_to_fee(weight),
        }
    }
}

parameter_types! {
    pub ParaId: u32 = ParachainInfo::get().into();
}

impl pallet_rings::Config for Runtime {
    type Event = Event;
    type ParaId = ParaId;
    type Parachains = Parachains;
}

struct BasiliskWeightToFee;
impl WeightToFeePolynomial for BasiliskWeightToFee {
    type Balance = Balance;

    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        let p = 11 * 1_000_000_000_000;
        let q =
            Balance::from(200 * frame_support::weights::constants::WEIGHT_PER_MICROS.ref_time());
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}
