use super::RingsChain;
use crate::{Balance, Weight};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::weights::{
    WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
use scale_info::TypeInfo;
use smallvec::smallvec;
use sp_runtime::Perbill;
use xcm::latest::{AssetId, Junctions, MultiLocation};

pub struct Kusama;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum KusamaAssets {
    KSM,
}

impl RingsChain for Kusama {
    type Assets = KusamaAssets;

    fn get_asset_id(asset: &Self::Assets) -> AssetId {
        use KusamaAssets::*;
        match asset {
            KSM => AssetId::Concrete(MultiLocation {
                parents: 0,
                interior: Junctions::Here,
            }),
        }
    }

    fn get_location() -> MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::Here,
        }
    }

    fn get_main_asset() -> Self::Assets {
        KusamaAssets::KSM
    }

    fn base_xcm_weight() -> Weight {
        Weight::from_parts(1_000_000_000, 64 * 1024)
    }
}

impl WeightToFeePolynomial for Kusama {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        let p = 1_000_000_000_000 / 30 / 100;
        let q =
            10 * Balance::from(Weight::from_parts(1_000u64.saturating_mul(106_013), 0).ref_time());
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}
