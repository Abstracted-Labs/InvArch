use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct Shiden;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum ShidenAssets {
    SDN,
}

impl RingsChain for Shiden {
    type Assets = ShidenAssets;

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation {
        use ShidenAssets::*;
        match asset {
            SDN => MultiLocation {
                parents: 0,
                interior: Junctions::Here,
            },
        }
    }

    fn get_location() -> MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::X1(Junction::Parachain(2007)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        ShidenAssets::SDN
    }
}
