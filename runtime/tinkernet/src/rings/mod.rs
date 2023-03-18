use crate::{
    assets::CORE_ASSET_ID, xcm_config::BaseXcmWeight, Balance, ParachainInfo, Runtime, RuntimeCall,
    RuntimeEvent,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    parameter_types,
    traits::Get,
    weights::{Weight, WeightToFee},
};
use pallet_rings::{ChainAssetsList, ChainList};
use scale_info::TypeInfo;
use xcm::prelude::*;

mod basilisk;
mod kusama;
use basilisk::Basilisk;
use kusama::Kusama;
mod statemine;
use statemine::Statemine;

parameter_types! {
    pub ParaId: u32 = ParachainInfo::get().into();
    pub MaxWeightedLength: u32 = 100_000;
}

impl pallet_rings::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ParaId = ParaId;
    type Chains = Chains;
    type MaxWeightedLength = MaxWeightedLength;
    type WeightInfo = pallet_rings::weights::SubstrateWeight<Runtime>;
}

pub trait RingsChain: WeightToFee {
    type Assets;

    fn get_asset_id(asset: &Self::Assets) -> AssetId;
    fn get_location() -> MultiLocation;
    fn get_main_asset() -> Self::Assets;
    fn base_xcm_weight() -> Weight;
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum Chains {
    Basilisk,
    Kusama,
    Statemine,

    TinkernetTest,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum ChainAssets {
    Basilisk(<Basilisk as RingsChain>::Assets),
    Kusama(<Kusama as RingsChain>::Assets),
    Statemine(<Statemine as RingsChain>::Assets),

    TinkernetTest(TinkernetTestAssets),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum TinkernetTestAssets {
    TNKR,
}

impl ChainAssetsList for ChainAssets {
    type Chains = Chains;

    fn get_chain(&self) -> Self::Chains {
        match self {
            Self::Basilisk(_) => Chains::Basilisk,
            Self::Kusama(_) => Chains::Kusama,
            Self::Statemine(_) => Chains::Statemine,

            Self::TinkernetTest(_) => Chains::TinkernetTest,
        }
    }

    fn get_asset_id(&self) -> AssetId {
        match self {
            Self::Basilisk(asset) => Basilisk::get_asset_id(asset),
            Self::Kusama(asset) => Kusama::get_asset_id(asset),

            Self::Statemine(asset) => Statemine::get_asset_id(asset),

            Self::TinkernetTest(TinkernetTestAssets::TNKR) => AssetId::Concrete(MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::GeneralIndex(CORE_ASSET_ID as u128)),
            }),
        }
    }
}

impl ChainList for Chains {
    type Balance = Balance;
    type ChainAssets = ChainAssets;
    type Call = RuntimeCall;

    fn get_location(&self) -> MultiLocation {
        match self {
            Self::Basilisk => Basilisk::get_location(),
            Self::Kusama => Kusama::get_location(),
            Self::Statemine => Statemine::get_location(),

            Self::TinkernetTest => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2126)),
            },
        }
    }

    fn get_main_asset(&self) -> Self::ChainAssets {
        match self {
            Self::Basilisk => ChainAssets::Basilisk(Basilisk::get_main_asset()),
            Self::Kusama => ChainAssets::Kusama(Kusama::get_main_asset()),
            Self::Statemine => ChainAssets::Statemine(Statemine::get_main_asset()),

            Self::TinkernetTest => ChainAssets::TinkernetTest(TinkernetTestAssets::TNKR),
        }
    }

    fn weight_to_fee(&self, weight: &Weight) -> Self::Balance {
        match self {
            Self::Basilisk => Basilisk::weight_to_fee(weight),
            Self::Kusama => Kusama::weight_to_fee(weight),
            Self::Statemine => Statemine::weight_to_fee(weight),

            Self::TinkernetTest => crate::WeightToFee::weight_to_fee(weight),
        }
    }

    fn base_xcm_weight(&self) -> Weight {
        match self {
            Self::Basilisk => Basilisk::base_xcm_weight(),
            Self::Kusama => Kusama::base_xcm_weight(),
            Self::Statemine => Statemine::base_xcm_weight(),

            Self::TinkernetTest => Weight::from_ref_time(BaseXcmWeight::get()),
        }
    }

    fn xcm_fee(&self, transact_weight: &Weight) -> Result<Self::Balance, ()> {
        Ok(self.weight_to_fee(
            &transact_weight
                .checked_add(&self.base_xcm_weight())
                .ok_or(())?,
        ))
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn benchmark_mock() -> Self {
        Self::Kusama
    }
}
