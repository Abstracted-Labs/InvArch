use frame_support::Parameter;
use xcm::latest::{AssetId, MultiLocation};

pub trait ParachainList: Parameter {
    fn get_location(&self) -> MultiLocation;

    fn get_asset(&self) -> AssetId;

    fn get_weight_to_fee(&self) -> u128;
}
