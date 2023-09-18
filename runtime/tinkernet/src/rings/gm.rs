use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct GM;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum GMAssets {
    FREN,
    GM,
    GN,
}

impl RingsChain for GM {
    type Assets = GMAssets;

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation {
        use GMAssets::*;
        match asset {
            FREN => MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralIndex(0u128)),
            },
            GM => MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralIndex(1u128)),
            },
            GN => MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralIndex(2u128)),
            },
        }
    }

    fn get_location() -> MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::X1(Junction::Parachain(2123)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        GMAssets::FREN
    }
}
