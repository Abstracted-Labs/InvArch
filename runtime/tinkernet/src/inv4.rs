use crate::{common_types::CommonId, Balance, Balances, Runtime, RuntimeCall, RuntimeEvent};
use frame_support::parameter_types;

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
}
