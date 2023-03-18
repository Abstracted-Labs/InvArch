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

pub struct Basilisk;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum BasiliskAssets {
    BSX,
    TNKR,
    KSM,
    USDT,
}

impl RingsChain for Basilisk {
    type Assets = BasiliskAssets;

    fn get_asset_id(asset: &Self::Assets) -> xcm::latest::AssetId {
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
                interior: Junctions::X2(Junction::Parachain(2125), Junction::GeneralIndex(0u128)),
            }),
        }
    }

    fn get_location() -> xcm::latest::MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::X1(Junction::Parachain(2090)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        BasiliskAssets::BSX
    }

    fn base_xcm_weight() -> Weight {
        Weight::from_ref_time(100_000_000)
    }
}

impl WeightToFeePolynomial for Basilisk {
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
