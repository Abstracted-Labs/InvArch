use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::WeakBoundedVec;
use scale_info::TypeInfo;
use sp_std::vec;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct Bifrost;

#[allow(non_camel_case_types)]
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum BifrostAssets {
    BNC,
    KSM,
    vKSM,
    USDT,
}

impl RingsChain for Bifrost {
    type Assets = BifrostAssets;

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation {
        use BifrostAssets::*;
        match asset {
            BNC => MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralKey(WeakBoundedVec::force_from(
                    vec![0, 1],
                    None,
                ))),
            },
            KSM => MultiLocation {
                parents: 1,
                interior: Junctions::Here,
            },
            vKSM => MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralKey(WeakBoundedVec::force_from(
                    vec![1, 4],
                    None,
                ))),
            },
            USDT => MultiLocation {
                parents: 1,
                interior: Junctions::X3(
                    Junction::Parachain(1000),
                    Junction::PalletInstance(50),
                    Junction::GeneralIndex(1984),
                ),
            },
        }
    }

    fn get_location() -> MultiLocation {
        MultiLocation {
            parents: 1,
            interior: Junctions::X1(Junction::Parachain(2030)),
        }
    }

    fn get_main_asset() -> Self::Assets {
        BifrostAssets::BNC
    }
}
