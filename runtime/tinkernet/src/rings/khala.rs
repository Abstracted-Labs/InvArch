use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct Khala;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum KhalaAssets {
    PHA,
}

impl RingsChain for Khala {
    type Assets = KhalaAssets;

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation {
        use KhalaAssets::*;
        match asset {
            PHA => MultiLocation {
                parents: 0,
                interior: Junctions::Here,
            },
        }
    }

    fn get_location() -> MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::X1(Junction::Parachain(2004)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        KhalaAssets::PHA
    }
}
