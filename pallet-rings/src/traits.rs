use codec::MaxEncodedLen;
use frame_support::{weights::Weight, Parameter};
use xcm::latest::{AssetId, MultiLocation};

pub trait ChainList: Parameter + MaxEncodedLen {
    type Balance: Into<u128>;
    type ChainAssets: ChainAssetsList;
    type Call;

    fn get_location(&self) -> MultiLocation;

    fn get_main_asset(&self) -> Self::ChainAssets;

    fn weight_to_fee(&self, weight: &Weight) -> Self::Balance;

    fn xcm_fee(&self, transact_weight: &Weight) -> Self::Balance;

    fn base_xcm_weight(&self) -> Weight;

    #[cfg(feature = "runtime-benchmarks")]
    fn benchmark_mock() -> Self;
}

pub trait ChainAssetsList: Parameter + MaxEncodedLen {
    type Chains: ChainList;

    fn get_chain(&self) -> Self::Chains;

    fn get_asset_id(&self) -> AssetId;
}
