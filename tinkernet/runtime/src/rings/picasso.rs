use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedSlice;
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct Picasso;

#[allow(non_camel_case_types)]
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum PicassoAssets {
    PICA,
    USDT,
    kUSD,
    KSM,
}

impl RingsChain for Picasso {
    type Assets = PicassoAssets;

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation {
        use PicassoAssets::*;
        match asset {
            PICA => MultiLocation {
                parents: 0,
                interior: Junctions::Here,
            },

            USDT => MultiLocation {
                parents: 1,
                interior: Junctions::X3(
                    Junction::Parachain(1000),
                    Junction::PalletInstance(50),
                    Junction::GeneralIndex(1984),
                ),
            },

            kUSD => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2000),
                    Junction::from(BoundedSlice::truncate_from(&[
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 129,
                    ])),
                ),
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
            interior: Junctions::X1(Junction::Parachain(2087)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        PicassoAssets::PICA
    }
}
