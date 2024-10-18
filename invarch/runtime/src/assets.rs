use crate::{
    balances,
    common_types::AssetId,
    staking::MaxNameLength,
    xcm_config::{
        AccountIdToMultiLocation, BaseXcmWeight, CurrencyIdConvert, MaxInstructions,
        ParachainMinFee, SelfLocation, UniversalLocation, XcmConfig,
    },
    AccountId, Amount, Balance, Balances, BlockNumber, Runtime, RuntimeCall, RuntimeEvent, Tokens,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{parameter_types, traits::Everything};
use frame_system::EnsureRoot;
use orml_currencies::BasicCurrencyAdapter;
use orml_traits2::{location::AbsoluteReserveProvider, parameter_type_with_key};
use scale_info::TypeInfo;

use xcm_builder::FixedWeightBounds;

pub const VARCH_ASSET_ID: AssetId = 0;

parameter_types! {
    pub const NativeAssetId: AssetId = VARCH_ASSET_ID;
}

#[derive(TypeInfo, Encode, Decode, Clone, Eq, PartialEq, Debug, MaxEncodedLen)]
pub struct CustomMetadata {
    pub fee_per_second: u128,
}

impl orml_asset_registry::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AuthorityOrigin = EnsureRoot<AccountId>;
    type AssetId = AssetId;
    type Balance = Balance;
    type AssetProcessor = orml_asset_registry::SequentialId<Runtime>;
    type StringLimit = MaxNameLength;
    type CustomMetadata = CustomMetadata;
    type WeightInfo = ();
}

impl orml_currencies::Config for Runtime {
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = NativeAssetId;
    type WeightInfo = ();
}

parameter_type_with_key! {
    pub Eds: |currency_id: AssetId| -> Balance {
        if let Some(metadata) = orml_asset_registry::Pallet::<Runtime>::metadata::<AssetId>(*currency_id) {
            metadata.existential_deposit
        } else {
            // Asset does not exist - not supported
            Balance::MAX
        }
    };
}

impl orml_tokens2::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = AssetId;
    // Ideally we would use AssetRegistry but having multiple instances of orml pallets is causing
    // issues related to traits.
    type ExistentialDeposits = Eds;
    type MaxLocks = balances::MaxFreezes;
    type DustRemovalWhitelist = ();
    type MaxReserves = balances::MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type CurrencyHooks = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxAssetsForTransfer: usize = 50;
}

impl orml_xtokens::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type CurrencyId = AssetId;
    type CurrencyIdConvert = CurrencyIdConvert;
    type SelfLocation = SelfLocation;
    type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
    type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
    type BaseXcmWeight = BaseXcmWeight;
    type MaxAssetsForTransfer = MaxAssetsForTransfer;
    type MinXcmFee = ParachainMinFee;
    type ReserveProvider = AbsoluteReserveProvider;
    type UniversalLocation = UniversalLocation;
    type AccountIdToLocation = AccountIdToMultiLocation;
    type LocationsFilter = Everything;
    type RateLimiter = ();
    type RateLimiterId = ();
}
