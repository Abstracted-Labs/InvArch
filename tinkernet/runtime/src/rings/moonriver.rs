use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct Moonriver;

#[allow(non_camel_case_types)]
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum MoonriverAssets {
    MOVR,
    xcKSM,
    xcTNKR,
    Erc20([u8; 20]),
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
            xcKSM => MultiLocation {
                parents: 1,
                interior: Junctions::Here,
            },
            xcTNKR => MultiLocation {
                parents: 1,
                interior: Junctions::X2(Junction::Parachain(2125), Junction::GeneralIndex(0)),
            },
            Erc20(address) => MultiLocation {
                parents: 0,
                interior: Junctions::X2(
                    Junction::PalletInstance(48),
                    Junction::AccountKey20 {
                        network: None,
                        key: *address,
                    },
                ),
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
