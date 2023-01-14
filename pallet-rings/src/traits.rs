use frame_support::{weights::Weight, Parameter};
use xcm::latest::{AssetId, MultiLocation};

pub trait ParachainList: Parameter {
    type Balance: Into<u128>;

    fn get_location(&self) -> MultiLocation;

    fn get_asset(&self) -> AssetId;

    fn weight_to_fee(&self, weight: &Weight) -> Self::Balance;
}
