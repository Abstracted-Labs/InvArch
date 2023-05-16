use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::WeakBoundedVec;
use scale_info::TypeInfo;
use sp_std::vec;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct Basilisk;

#[allow(non_camel_case_types)]
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum BasiliskAssets {
    BSX,
    TNKR,
    KSM,
    USDT,
    DAI,
    USDCet,
    XRT,
    aUSD,
    wETH,
    wBTC,
    wUSDT,
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
            DAI => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2000),
                    Junction::GeneralKey(WeakBoundedVec::force_from(
                        vec![
                            2, 75, 182, 175, 181, 250, 43, 7, 165, 209, 196, 153, 225, 195, 221,
                            181, 161, 94, 112, 154, 113,
                        ],
                        None,
                    )),
                ),
            },
            USDCet => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2000),
                    Junction::GeneralKey(WeakBoundedVec::force_from(
                        vec![
                            2, 31, 58, 16, 88, 122, 32, 17, 78, 162, 91, 161, 179, 136, 238, 45,
                            212, 163, 55, 206, 39,
                        ],
                        None,
                    )),
                ),
            },
            XRT => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2048)),
            },
            aUSD => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2000),
                    Junction::GeneralKey(WeakBoundedVec::force_from(vec![0, 129], None)),
                ),
            },
            wETH => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2000),
                    Junction::GeneralKey(WeakBoundedVec::force_from(
                        vec![
                            2, 236, 224, 204, 56, 2, 30, 115, 75, 239, 29, 93, 160, 113, 176, 39,
                            172, 47, 113, 24, 31,
                        ],
                        None,
                    )),
                ),
            },
            wBTC => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2000),
                    Junction::GeneralKey(WeakBoundedVec::force_from(
                        vec![
                            2, 102, 41, 28, 125, 136, 210, 237, 154, 112, 129, 71, 186, 228, 224,
                            129, 74, 118, 112, 94, 47,
                        ],
                        None,
                    )),
                ),
            },
            wUSDT => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2000),
                    Junction::GeneralKey(WeakBoundedVec::force_from(
                        vec![
                            2, 84, 225, 131, 229, 51, 253, 60, 110, 114, 222, 187, 45, 28, 171, 69,
                            29, 1, 127, 175, 114,
                        ],
                        None,
                    )),
                ),
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
