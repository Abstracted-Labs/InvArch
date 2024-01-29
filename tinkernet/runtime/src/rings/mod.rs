use crate::{AccountId, Balance, Runtime, RuntimeEvent};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::parameter_types;
use frame_system::EnsureRoot;
use pallet_rings::{ChainAssetsList, ChainList};
use scale_info::TypeInfo;
use xcm::prelude::*;

mod basilisk;
use basilisk::Basilisk;
mod picasso;
use picasso::Picasso;
mod asset_hub;
use asset_hub::AssetHub;

parameter_types! {
    pub MaxXCMCallLength: u32 = 100_000;
}

impl pallet_rings::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Chains = Chains;
    type MaxXCMCallLength = MaxXCMCallLength;
    type MaintenanceOrigin = EnsureRoot<AccountId>;
    type WeightInfo = pallet_rings::weights::SubstrateWeight<Runtime>;
}

pub trait RingsChain {
    type Assets;

    fn get_asset_location(asset: &Self::Assets) -> MultiLocation;
    fn get_location() -> MultiLocation;
    fn get_main_asset() -> Self::Assets;
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum Chains {
    Basilisk,
    Picasso,
    AssetHub,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum ChainAssets {
    Basilisk(<Basilisk as RingsChain>::Assets),
    Picasso(<Picasso as RingsChain>::Assets),
    AssetHub(<AssetHub as RingsChain>::Assets),
}

impl ChainAssetsList for ChainAssets {
    type Chains = Chains;

    fn get_chain(&self) -> Self::Chains {
        match self {
            Self::Basilisk(_) => Chains::Basilisk,
            Self::Picasso(_) => Chains::Picasso,
            Self::AssetHub(_) => Chains::AssetHub,
        }
    }

    fn get_asset_location(&self) -> MultiLocation {
        match self {
            Self::Basilisk(asset) => Basilisk::get_asset_location(asset),
            Self::Picasso(asset) => Picasso::get_asset_location(asset),
            Self::AssetHub(asset) => AssetHub::get_asset_location(asset),
        }
    }
}

impl ChainList for Chains {
    type Balance = Balance;
    type ChainAssets = ChainAssets;

    fn get_location(&self) -> MultiLocation {
        match self {
            Self::Basilisk => Basilisk::get_location(),
            Self::Picasso => Picasso::get_location(),
            Self::AssetHub => AssetHub::get_location(),
        }
    }

    fn get_main_asset(&self) -> Self::ChainAssets {
        match self {
            Self::Basilisk => ChainAssets::Basilisk(Basilisk::get_main_asset()),
            Self::Picasso => ChainAssets::Picasso(Picasso::get_main_asset()),
            Self::AssetHub => ChainAssets::AssetHub(AssetHub::get_main_asset()),
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn benchmark_mock() -> Self {
        Self::Basilisk
    }
}
