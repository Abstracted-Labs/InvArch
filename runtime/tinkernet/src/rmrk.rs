use crate::{
    common_types::CommonId,
    constants::currency::{MILLIUNIT, UNIT},
    AccountId, Balance, Balances, Runtime, RuntimeEvent,
};
use frame_support::{parameter_types, traits::AsEnsureOriginWithArg};

use frame_system::{EnsureRoot, EnsureSigned};
#[cfg(feature = "runtime-benchmarks")]
use pallet_rmrk_core::RmrkBenchmark;

parameter_types! {
      pub const ResourceSymbolLimit: u32 = 10;
      pub const PartsLimit: u32 = 25;
      pub const MaxPriorities: u32 = 25;
      pub const CollectionSymbolLimit: u32 = 100;
      pub const MaxResourcesOnMint: u32 = 100;
      pub const NestingBudget: u32 = 20;
}

impl pallet_rmrk_core::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ProtocolOrigin = frame_system::EnsureRoot<AccountId>;
    type ResourceSymbolLimit = ResourceSymbolLimit;
    type PartsLimit = PartsLimit;
    type MaxPriorities = MaxPriorities;
    type CollectionSymbolLimit = CollectionSymbolLimit;
    type MaxResourcesOnMint = MaxResourcesOnMint;
    type NestingBudget = NestingBudget;
    type WeightInfo = pallet_rmrk_core::weights::SubstrateWeight<Runtime>;
    #[cfg(feature = "runtime-benchmarks")]
    type Helper = RmrkBenchmark;
    type TransferHooks = ();
}

parameter_types! {
      pub const MaxPropertiesPerTheme: u32 = 100;
      pub const MaxCollectionsEquippablePerPart: u32 = 100;
}

impl pallet_rmrk_equip::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxPropertiesPerTheme = MaxPropertiesPerTheme;
    type MaxCollectionsEquippablePerPart = MaxCollectionsEquippablePerPart;
    type WeightInfo = pallet_rmrk_equip::weights::SubstrateWeight<Runtime>;
    #[cfg(feature = "runtime-benchmarks")]
    type Helper = RmrkBenchmark;
}

parameter_types! {
      pub const MinimumOfferAmount: Balance = UNIT / 10_000;
}

impl pallet_rmrk_market::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ProtocolOrigin = frame_system::EnsureRoot<AccountId>;
    type Currency = Balances;
    type MinimumOfferAmount = MinimumOfferAmount;
    type WeightInfo = pallet_rmrk_market::weights::SubstrateWeight<Runtime>;
    #[cfg(feature = "runtime-benchmarks")]
    type Helper = RmrkBenchmark;
}

parameter_types! {
      pub const CollectionDeposit: Balance = 0; //10 * MILLIUNIT;
      pub const ItemDeposit: Balance = 0; //UNIT;
      pub const KeyLimit: u32 = 32;
      pub const ValueLimit: u32 = 256;
      pub const UniquesMetadataDepositBase: Balance = 10 * MILLIUNIT;
      pub const AttributeDepositBase: Balance = 10 * MILLIUNIT;
      pub const DepositPerByte: Balance = MILLIUNIT;
      pub const UniquesStringLimit: u32 = 128;
}

impl pallet_uniques::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = CommonId;
    type ItemId = CommonId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
    type Locker = pallet_rmrk_core::Pallet<Runtime>;
    type CollectionDeposit = CollectionDeposit;
    type ItemDeposit = ItemDeposit;
    type MetadataDepositBase = UniquesMetadataDepositBase;
    type AttributeDepositBase = AttributeDepositBase;
    type DepositPerByte = DepositPerByte;
    type StringLimit = UniquesStringLimit;
    type KeyLimit = KeyLimit;
    type ValueLimit = ValueLimit;
    type WeightInfo = ();
}
