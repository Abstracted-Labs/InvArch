use codec::MaxEncodedLen;
use frame_support::Parameter;
use xcm::latest::MultiLocation;

pub trait ChainList: Parameter + MaxEncodedLen {
    type Balance: Into<u128>;
    type ChainAssets: ChainAssetsList;

    fn get_location(&self) -> MultiLocation;

    fn get_main_asset(&self) -> Self::ChainAssets;

    #[cfg(feature = "runtime-benchmarks")]
    fn benchmark_mock() -> Self;
}

pub trait ChainAssetsList: Parameter + MaxEncodedLen {
    type Chains: ChainList;

    fn get_chain(&self) -> Self::Chains;

    fn get_asset_location(&self) -> MultiLocation;
}
