use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct Moonriver;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum MoonriverAssets {
    MOVR,
}

impl RingsChain for Moonriver {
    type Assets = MoonriverAssets;

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation {
        use MoonriverAssets::*;
        match asset {
            MOVR => MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::PalletInstance(3)),
            },
        }
    }

    fn get_location() -> MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::X1(Junction::Parachain(2023)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        MoonriverAssets::MOVR
    }
}
