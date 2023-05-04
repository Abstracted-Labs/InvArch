use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions, MultiLocation};

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

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation {
        use BasiliskAssets::*;
        match asset {
            KSM => MultiLocation {
                parents: 1,
                interior: Junctions::Here,
            },
            USDT => MultiLocation {
                parents: 1,
                interior: Junctions::X3(
                    Junction::Parachain(1000),
                    Junction::PalletInstance(50u8),
                    Junction::GeneralIndex(1984u128),
                ),
            },
            BSX => MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralIndex(0u128)),
            },
            TNKR => MultiLocation {
                parents: 1,
                interior: Junctions::X2(Junction::Parachain(2125), Junction::GeneralIndex(0u128)),
            },
        }
    }

    fn get_location() -> MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::X1(Junction::Parachain(2090)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        BasiliskAssets::BSX
    }
}
