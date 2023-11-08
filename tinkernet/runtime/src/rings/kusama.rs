use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use xcm::latest::{Junctions, MultiLocation};

pub struct Kusama;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum KusamaAssets {
    KSM,
}

impl RingsChain for Kusama {
    type Assets = KusamaAssets;

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation {
        use KusamaAssets::*;
        match asset {
            KSM => MultiLocation {
                parents: 0,
                interior: Junctions::Here,
            },
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
}
