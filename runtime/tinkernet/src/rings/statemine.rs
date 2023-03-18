use super::RingsChain;
use crate::{Balance, Weight};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::weights::{
    WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
use scale_info::TypeInfo;
use smallvec::smallvec;
use sp_runtime::Perbill;
use xcm::latest::{AssetId, Junction, Junctions, MultiLocation};

pub struct Statemine;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum StatemineAssets {
    KSM,
    BILLCOIN,
}

impl RingsChain for Statemine {
    type Assets = StatemineAssets;

    fn get_asset_id(asset: &Self::Assets) -> AssetId {
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

    fn get_location() -> MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::X1(Junction::Parachain(1000)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        StatemineAssets::KSM
    }

    fn base_xcm_weight() -> Weight {
        // TODO: Set correct base xcm weight.
        Weight::from_parts(1_000_000_000, 64 * 1024)
    }
}

impl WeightToFeePolynomial for Statemine {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        // in Kusama, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
        // in Statemine, we map to 1/10 of that, or 1/100 CENT
        let p = 1_000_000_000_000 / 30 / 100;
        let q =
            100 * Balance::from(Weight::from_parts(1_000u64.saturating_mul(110_536), 0).ref_time());

        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}
