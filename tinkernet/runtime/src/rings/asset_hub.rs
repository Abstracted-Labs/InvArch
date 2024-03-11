use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct AssetHub;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum AssetHubAssets {
    KSM,
    Local(u32),
}

impl RingsChain for AssetHub {
    type Assets = AssetHubAssets;

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation {
        use AssetHubAssets::*;
        match asset {
            KSM => MultiLocation {
                parents: 1,
                interior: Junctions::Here,
            },
            Local(asset_id) => MultiLocation {
                parents: 0,
                interior: Junctions::X2(Junction::PalletKey(50), Junction::GeneralIndex(*asset_id)),
            },
        }
    }

    fn get_location() -> MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::X1(Junction::Parachain(1000)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        AssetHubAssets::KSM
    }
}
