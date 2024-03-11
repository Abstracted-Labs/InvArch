use super::RingsChain;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedSlice;
use scale_info::TypeInfo;
use xcm::latest::{Junction, Junctions, MultiLocation};

pub struct Moonriver;

#[allow(non_camel_case_types)]
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum MoonriverAssets {
    /// Moonriver main asset.
    MOVR,
    /// KSM.
    xcKSM,
    /// TNKR.
    xcTNKR,
    /// Tether USD on asset hub.
    xcUSDT,
    /// RMRK on asset hub.
    xcRMRK,
    /// Karura aSEED (aUSD).
    xcaSeed,
    /// Karura.
    xcKAR,
    /// Bifrost Voucher KSM.
    xcvKSM,
    /// Bifrost Voucher BNC.
    xcvBNC,
    /// Bifrost Voucher MOVR.
    xcvMOVR,
    /// Bifrost.
    xcBNC,
    /// Phala.
    xcPHA,
    /// Shiden.
    xcSDN,
    /// Crust Shadow Native Token.
    xcCSM,
    /// Integritee.
    xcTEER,
    /// Robonomics Native Token
    xcXRT,
    /// Calamari.
    xcKMA,
    /// Parallel Heiko.
    xcHKO,
    /// Picasso.
    xcPICA,
    /// Kintsugi Wrapped BTC.
    xcKBTC,
    /// Kintsugi Native Token.
    xcKINT,
    /// Crab Parachain Token.
    xcCRAB,
    /// Litmus.
    xcLIT,
    /// Mangata X Native Token.
    xcMGX,
    /// Turing Network.
    xcTUR,
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
            xcUSDT => MultiLocation {
                parents: 1,
                interior: Junctions::X3(
                    Junction::Parachain(1000),
                    Junction::PalletInstance(50),
                    Junction::GeneralIndex(1984),
                ),
            },
            xcRMRK => MultiLocation {
                parents: 1,
                interior: Junctions::X3(
                    Junction::Parachain(1000),
                    Junction::PalletInstance(50),
                    Junction::GeneralIndex(8),
                ),
            },
            xcaSeed => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2000),
                    Junction::from(BoundedSlice::truncate_from(&hex_literal::hex!("0081"))),
                ),
            },
            xcKAR => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2000),
                    Junction::from(BoundedSlice::truncate_from(&hex_literal::hex!("0080"))),
                ),
            },
            xcvKSM => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2001),
                    Junction::from(BoundedSlice::truncate_from(&hex_literal::hex!("0104"))),
                ),
            },
            xcvBNC => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2001),
                    Junction::from(BoundedSlice::truncate_from(&hex_literal::hex!("0101"))),
                ),
            },
            xcvMOVR => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2001),
                    Junction::from(BoundedSlice::truncate_from(&hex_literal::hex!("010a"))),
                ),
            },
            xcBNC => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2001),
                    Junction::from(BoundedSlice::truncate_from(&hex_literal::hex!("0001"))),
                ),
            },
            xcPHA => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2004)),
            },
            xcSDN => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2007)),
            },
            xcCSM => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2012)),
            },
            xcTEER => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2015),
                    Junction::from(BoundedSlice::truncate_from(&hex_literal::hex!("54454552"))),
                ),
            },
            xcXRT => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2048)),
            },
            xcKMA => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2084)),
            },
            xcHKO => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2085),
                    Junction::from(BoundedSlice::truncate_from(&hex_literal::hex!("484b4f"))),
                ),
            },
            xcPICA => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2087)),
            },
            xcKBTC => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2092),
                    Junction::from(BoundedSlice::truncate_from(&hex_literal::hex!("000b"))),
                ),
            },
            xcKINT => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junction::Parachain(2092),
                    Junction::from(BoundedSlice::truncate_from(&hex_literal::hex!("000c"))),
                ),
            },
            xcCRAB => MultiLocation {
                parents: 1,
                interior: Junctions::X2(Junction::Parachain(2105), Junction::PalletInstance(5)),
            },
            xcLIT => MultiLocation {
                parents: 1,
                interior: Junctions::X2(Junction::Parachain(2106), Junction::PalletInstance(10)),
            },
            xcMGX => MultiLocation {
                parents: 1,
                interior: Junctions::X2(
                    Junsction::Parachain(2110),
                    Junction::from(BoundedSlice::truncate_from(&hex_literal::hex!("00000000"))),
                ),
            },
            xcTUR => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2114)),
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
