use codec::MaxEncodedLen;
use frame_support::{weights::Weight, Parameter};
use xcm::latest::{AssetId, MultiLocation};

pub trait ParachainList: Parameter + MaxEncodedLen {
    type Balance: Into<u128>;
    type ParachainAssets: ParachainAssetsList;

    fn from_para_id(para_id: u32) -> Option<Self>;

    fn get_location(&self) -> MultiLocation;

    fn get_main_asset(&self) -> Self::ParachainAssets;

    fn weight_to_fee(&self, weight: &Weight) -> Self::Balance;
}

pub trait ParachainAssetsList: Parameter + MaxEncodedLen {
    type Parachains: ParachainList;

    fn get_parachain(&self) -> Self::Parachains;

    fn get_asset_id(&self) -> AssetId;
}
