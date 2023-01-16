use codec::MaxEncodedLen;
use frame_support::{weights::Weight, Parameter};
use xcm::latest::{AssetId, MultiLocation};

pub trait ParachainList: Parameter + MaxEncodedLen {
    type Balance: Into<u128>;

    fn from_para_id(para_id: u32) -> Option<Self>;

    fn get_location(&self) -> MultiLocation;

    fn get_asset(&self) -> AssetId;

    fn weight_to_fee(&self, weight: &Weight) -> Self::Balance;
}
