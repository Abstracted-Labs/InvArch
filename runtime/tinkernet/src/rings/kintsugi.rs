use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::WeakBoundedVec;
use scale_info::TypeInfo;
use sp_std::vec;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct Kintsugi;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum KintsugiAssets {
    KINT,
    KBTC,
    KSM,
}

impl RingsChain for Kintsugi {
    type Assets = KintsugiAssets;

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation {
        use KintsugiAssets::*;
        match asset {
            KINT => MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralKey(WeakBoundedVec::force_from(
                    vec![0, 12],
                    None,
                ))),
            },
            KBTC => MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralKey(WeakBoundedVec::force_from(
                    vec![0, 11],
                    None,
                ))),
            },
            KSM => MultiLocation {
                parents: 1,
                interior: Junctions::Here,
            },
        }
    }

    fn get_location() -> MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::X1(Junction::Parachain(2092)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        KintsugiAssets::KINT
    }
}
