use crate::{
    common_types::CommonId, AccountId, Balance, Balances, CoreAssets, Runtime, RuntimeCall,
    RuntimeEvent, RuntimeOrigin,
};
use frame_support::{parameter_types, traits::AsEnsureOriginWithArg};
use frame_system::{EnsureNever, EnsureRoot, RawOrigin};

parameter_types! {
    pub const MaxMetadata: u32 = 10000;
    pub const MaxCallers: u32 = 10000;
    pub const CoreSeedBalance: Balance = 1000000u128;
}

impl pallet_inv4::Config for Runtime {
    type MaxMetadata = MaxMetadata;
    type CoreId = CommonId;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type RuntimeCall = RuntimeCall;
    type MaxCallers = MaxCallers;
    type MaxSubAssets = MaxCallers;
    type CoreSeedBalance = CoreSeedBalance;
    type AssetsProvider = CoreAssets;
    type RuntimeOrigin = RuntimeOrigin;
    type AssetFreezer = AssetFreezer;
}

parameter_types! {
    pub const AssetDeposit: u32 = 0;
    pub const AssetAccountDeposit: u32 = 0;
    pub const MetadataDepositBase: u32 = 0;
    pub const MetadataDepositPerByte: u32 = 0;
    pub const ApprovalDeposit: u32 = 0;
    pub const StringLimit: u32 = 100;
    pub const RemoveItemsList: u32 = 5;
}

impl pallet_assets::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = CommonId;
    type AssetIdParameter = CommonId;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureNever<AccountId>>;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = AssetAccountDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = ();
    type WeightInfo = ();
    type Extra = ();
    type RemoveItemsLimit = RemoveItemsList;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

pub struct AssetFreezer;
impl pallet_inv4::multisig::FreezeAsset<CommonId> for AssetFreezer {
    fn freeze_asset(asset_id: CommonId) -> frame_support::dispatch::DispatchResult {
        CoreAssets::freeze_asset(RawOrigin::Root.into(), asset_id)
    }

    fn thaw_asset(asset_id: CommonId) -> frame_support::dispatch::DispatchResult {
        CoreAssets::thaw_asset(RawOrigin::Root.into(), asset_id)
    }
}
