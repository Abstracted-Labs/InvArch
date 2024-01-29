use crate::common_types::AssetId;
use frame_support::parameter_types;

pub const VARCH_ASSET_ID: AssetId = 0;

parameter_types! {
    pub const NativeAssetId: AssetId = VARCH_ASSET_ID;
}
