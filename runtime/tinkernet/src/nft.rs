use crate::{
    common_types::CommonId,
    constants::currency::{MILLIUNIT, UNIT},
    AccountId, Balance, Balances, Runtime, RuntimeEvent,
};
use frame_support::{parameter_types, traits::AsEnsureOriginWithArg};

use frame_system::{EnsureRoot, EnsureSigned};

parameter_types! {
      pub const CollectionDeposit: Balance = UNIT;
      pub const ItemDeposit: Balance = UNIT / 100;
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
    type Locker = ();
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
