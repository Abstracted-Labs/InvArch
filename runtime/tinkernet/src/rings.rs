use crate::{assets::CORE_ASSET_ID, Balance, Event, ParachainInfo, Runtime};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    parameter_types,
    traits::Get,
    weights::{
        Weight, WeightToFee, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
};
use pallet_rings::ParachainList;
use scale_info::TypeInfo;
use smallvec::smallvec;
use sp_runtime::Perbill;
use xcm::prelude::*;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum Parachains {
    Basilisk,
    TinkernetTest,
}

impl ParachainList for Parachains {
    type Balance = Balance;

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

    fn get_asset(&self) -> AssetId {
        match self {
            Self::Basilisk => AssetId::Concrete(MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralIndex(0u128)),
            }),

            Self::TinkernetTest => AssetId::Concrete(MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralIndex(CORE_ASSET_ID as u128)),
            }),
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
