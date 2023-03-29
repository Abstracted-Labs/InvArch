use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct Statemine;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum StatemineAssets {
    KSM,
    BILLCOIN,

    Custom(MultiLocation),
}

impl RingsChain for Statemine {
    type Assets = StatemineAssets;

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation {
        use StatemineAssets::*;
        match asset {
            KSM => MultiLocation {
                parents: 1,
                interior: Junctions::Here,
            },
            BILLCOIN => MultiLocation {
                parents: 0,
                interior: Junctions::X2(
                    Junction::PalletInstance(50u8),
                    Junction::GeneralIndex(223u128),
                ),
            },

            Custom(multilocation) => multilocation.clone(),
        }
    }

    fn get_location() -> MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::X1(Junction::Parachain(1000)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        StatemineAssets::KSM
    }
}
