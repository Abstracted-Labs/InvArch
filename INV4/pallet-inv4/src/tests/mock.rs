use crate::{fee_handling::*, *};
use codec::{Decode, Encode};
use core::convert::TryFrom;
use frame_support::{
    parameter_types,
    traits::{
        fungibles::Credit, ConstU128, ConstU32, ConstU64, Contains, Currency, EnsureOrigin,
        EnsureOriginWithArg, GenesisBuild,
    },
};
use frame_system::EnsureRoot;
use orml_asset_registry::AssetMetadata;
use pallet_balances::AccountData;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};
use sp_std::{convert::TryInto, vec};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

type AccountId = u32;
type BlockNumber = u64;

pub const EXISTENTIAL_DEPOSIT: Balance = 1_000_000_000;

pub const ALICE: AccountId = 0;
pub const BOB: AccountId = 1;
pub const CHARLIE: AccountId = 2;
pub const DAVE: AccountId = 3;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>, Config<T>},
        INV4: pallet::{Pallet, Call, Storage, Event<T>, Origin<T>},
        CoreAssets: orml_tokens2::{Pallet, Call, Storage, Event<T>},
        Tokens: orml_tokens::{Pallet, Call, Storage, Event<T>},
        AssetRegistry: orml_asset_registry::{Pallet, Call, Storage, Event<T>, Config<T>},
    }
);

pub struct TestBaseCallFilter;
impl Contains<RuntimeCall> for TestBaseCallFilter {
    fn contains(_c: &RuntimeCall) -> bool {
        true
    }
}

impl frame_system::Config for Test {
    type RuntimeOrigin = RuntimeOrigin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = TestBaseCallFilter;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    /// The type for recording an account's balance.
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type MaxHolds = ConstU32<1>;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type HoldIdentifier = [u8; 8];
}

const UNIT: u128 = 1000000000000;

orml_traits2::parameter_type_with_key! {
    pub CoreExistentialDeposits: |_currency_id: <Test as pallet::Config>::CoreId| -> Balance {
        1u128
    };
}

pub struct CoreDustRemovalWhitelist;
impl Contains<AccountId> for CoreDustRemovalWhitelist {
    fn contains(_: &AccountId) -> bool {
        true
    }
}

pub struct DisallowIfFrozen;
impl orml_traits2::currency::OnTransfer<AccountId, <Test as pallet::Config>::CoreId, Balance>
    for DisallowIfFrozen
{
    fn on_transfer(
        currency_id: <Test as pallet::Config>::CoreId,
        _from: &AccountId,
        _to: &AccountId,
        _amount: Balance,
    ) -> sp_runtime::DispatchResult {
        if let Some(true) = INV4::is_asset_frozen(currency_id) {
            Err(sp_runtime::DispatchError::Token(
                sp_runtime::TokenError::Frozen,
            ))
        } else {
            Ok(())
        }
    }
}

pub struct HandleNewMembers;
impl orml_traits2::Happened<(AccountId, <Test as pallet::Config>::CoreId)> for HandleNewMembers {
    fn happened((member, core_id): &(AccountId, <Test as pallet::Config>::CoreId)) {
        INV4::add_member(core_id, member)
    }
}

pub struct HandleRemovedMembers;
impl orml_traits2::Happened<(AccountId, <Test as pallet::Config>::CoreId)>
    for HandleRemovedMembers
{
    fn happened((member, core_id): &(AccountId, <Test as pallet::Config>::CoreId)) {
        INV4::remove_member(core_id, member)
    }
}

pub struct INV4TokenHooks;
impl orml_traits2::currency::MutationHooks<AccountId, <Test as pallet::Config>::CoreId, Balance>
    for INV4TokenHooks
{
    type PreTransfer = DisallowIfFrozen;
    type OnDust = ();
    type OnSlash = ();
    type PreDeposit = ();
    type PostDeposit = ();
    type PostTransfer = ();
    type OnNewTokenAccount = HandleNewMembers;
    type OnKilledTokenAccount = HandleRemovedMembers;
}

impl orml_tokens2::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type Amount = i128;
    type CurrencyId = <Test as pallet::Config>::CoreId;
    type WeightInfo = ();
    type ExistentialDeposits = CoreExistentialDeposits;
    type MaxLocks = ConstU32<0u32>;
    type MaxReserves = ConstU32<0u32>;
    type DustRemovalWhitelist = CoreDustRemovalWhitelist;
    type ReserveIdentifier = [u8; 8];
    type CurrencyHooks = INV4TokenHooks;
}

parameter_types! {
    pub const MaxMetadata: u32 = 10000;
    pub const MaxCallers: u32 = 10000;
    pub const CoreSeedBalance: Balance = 1000000u128;
    pub const CoreCreationFee: Balance = UNIT;
    pub const GenesisHash: <Test as frame_system::Config>::Hash = H256([
        212, 46, 150, 6, 169, 149, 223, 228, 51, 220, 121, 85, 220, 42, 112, 244, 149, 243, 80,
        243, 115, 218, 162, 0, 9, 138, 232, 68, 55, 129, 106, 210,
    ]);

    pub const KSMCoreCreationFee: Balance = UNIT;
}

pub type AssetId = u32;

pub const CORE_ASSET_ID: AssetId = 0;
pub const KSM_ASSET_ID: AssetId = 1;

parameter_types! {
    pub const NativeAssetId: AssetId = CORE_ASSET_ID;
    pub const RelayAssetId: AssetId = KSM_ASSET_ID;
    pub const ExistentialDeposit: u128 = 100000000000;
    pub const MaxLocks: u32 = 1;
    pub const MaxReserves: u32 = 1;
    pub const MaxCallSize: u32 = 50 * 1024;
}

pub struct AssetAuthority;
impl EnsureOriginWithArg<RuntimeOrigin, Option<u32>> for AssetAuthority {
    type Success = ();

    fn try_origin(
        origin: RuntimeOrigin,
        _asset_id: &Option<u32>,
    ) -> Result<Self::Success, RuntimeOrigin> {
        EnsureRoot::try_origin(origin)
    }
}

impl orml_asset_registry::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AuthorityOrigin = AssetAuthority;
    type AssetId = AssetId;
    type Balance = Balance;
    type AssetProcessor = orml_asset_registry::SequentialId<Test>;
    type CustomMetadata = ();
    type WeightInfo = ();
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
    fn contains(_: &AccountId) -> bool {
        true
    }
}

pub type Amount = i128;

orml_traits::parameter_type_with_key! {
      pub ExistentialDeposits: |currency_id: AssetId| -> Balance {
          if currency_id == &CORE_ASSET_ID {
              ExistentialDeposit::get()
          } else {
              orml_asset_registry::ExistentialDeposits::<Test>::get(currency_id)
          }
      };
}

impl orml_tokens::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = AssetId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type DustRemovalWhitelist = DustRemovalWhitelist;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type CurrencyHooks = ();
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo, Debug)]
pub struct FeeCharger;

impl MultisigFeeHandler<Test> for FeeCharger {
    type Pre = (
        // tip
        Balance,
        // who paid the fee
        AccountId,
        // imbalance resulting from withdrawing the fee
        (),
        // asset_id for the transaction payment
        Option<AssetId>,
    );

    fn pre_dispatch(
        fee_asset: &FeeAsset,
        who: &AccountId,
        _call: &RuntimeCall,
        _info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
        _len: usize,
    ) -> Result<Self::Pre, frame_support::unsigned::TransactionValidityError> {
        Ok((
            0u128,
            *who,
            (),
            match fee_asset {
                FeeAsset::TNKR => None,
                FeeAsset::KSM => Some(1u32),
            },
        ))
    }

    fn post_dispatch(
        _fee_asset: &FeeAsset,
        _pre: Option<Self::Pre>,
        _info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
        _post_info: &sp_runtime::traits::PostDispatchInfoOf<RuntimeCall>,
        _len: usize,
        _result: &sp_runtime::DispatchResult,
    ) -> Result<(), frame_support::unsigned::TransactionValidityError> {
        Ok(())
    }

    fn handle_creation_fee(
        _imbalance: FeeAssetNegativeImbalance<
            <Balances as Currency<AccountId>>::NegativeImbalance,
            Credit<AccountId, Tokens>,
        >,
    ) {
    }
}

impl pallet::Config for Test {
    type MaxMetadata = MaxMetadata;
    type CoreId = u32;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type RuntimeCall = RuntimeCall;
    type MaxCallers = MaxCallers;
    type MaxSubAssets = MaxCallers;
    type CoreSeedBalance = CoreSeedBalance;
    type AssetsProvider = CoreAssets;
    type RuntimeOrigin = RuntimeOrigin;
    type CoreCreationFee = CoreCreationFee;
    type FeeCharger = FeeCharger;
    type GenesisHash = GenesisHash;
    type WeightInfo = crate::weights::SubstrateWeight<Test>;

    type Tokens = Tokens;
    type KSMAssetId = RelayAssetId;
    type KSMCoreCreationFee = KSMCoreCreationFee;

    type MaxCallSize = MaxCallSize;
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

pub const INITIAL_BALANCE: Balance = 100000000000000000;

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (ALICE, INITIAL_BALANCE),
                (BOB, INITIAL_BALANCE),
                (CHARLIE, INITIAL_BALANCE),
                (
                    util::derive_core_account::<Test, u32, u32>(0u32),
                    INITIAL_BALANCE,
                ),
            ],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        orml_asset_registry::GenesisConfig::<Test> {
            assets: vec![
                (
                    0u32,
                    AssetMetadata {
                        decimals: 12,
                        name: vec![],
                        symbol: vec![],
                        existential_deposit: ExistentialDeposit::get(),
                        location: None,
                        additional: (),
                    }
                    .encode(),
                ),
                (
                    1u32,
                    AssetMetadata {
                        decimals: 12,
                        name: vec![],
                        symbol: vec![],
                        existential_deposit: ExistentialDeposit::get(),
                        location: None,
                        additional: (),
                    }
                    .encode(),
                ),
            ],
            last_asset_id: 1u32,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        orml_tokens::GenesisConfig::<Test> {
            balances: vec![
                (ALICE, KSM_ASSET_ID, INITIAL_BALANCE),
                (BOB, KSM_ASSET_ID, INITIAL_BALANCE),
                (CHARLIE, KSM_ASSET_ID, INITIAL_BALANCE),
            ],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(0));

        ext
    }
}
