use crate::{
    traits::{ChainAssetsList, ChainList},
    *,
};
use codec::{Decode, Encode, MaxEncodedLen};
use core::convert::TryFrom;
use frame_support::{
    parameter_types,
    traits::{
        fungibles::Credit, ConstU128, ConstU32, ConstU64, Contains, Currency, EnsureOrigin,
        EnsureOriginWithArg, Everything, GenesisBuild, Nothing,
    },
};
use frame_system::EnsureRoot;
use orml_asset_registry::AssetMetadata;
use pallet_balances::AccountData;
use pallet_inv4::fee_handling::*;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, AccountId32};
pub use sp_std::{cell::RefCell, fmt::Debug};
use sp_std::{convert::TryInto, vec};
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, CurrencyAdapter as XcmCurrencyAdapter, FixedRateOfFungible,
    FixedWeightBounds, IsConcrete, SignedAccountId32AsNative, SignedToAccountId32,
    SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::XcmExecutor;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

type AccountId = AccountId32;
type BlockNumber = u64;

pub const EXISTENTIAL_DEPOSIT: Balance = 1_000_000_000;

pub const ALICE: AccountId = AccountId32::new([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);
pub const BOB: AccountId = AccountId32::new([
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);
pub const CHARLIE: AccountId = AccountId32::new([
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>, Config<T>},
        INV4: pallet_inv4::{Pallet, Call, Storage, Event<T>, Origin<T>},
        CoreAssets: orml_tokens2::{Pallet, Call, Storage, Event<T>},
        Rings: pallet::{Pallet, Call, Storage, Event<T>},
        XcmPallet: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin, Config},
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

thread_local! {
      pub static SENT_XCM: RefCell<Vec<(MultiLocation, Xcm<()>)>> = RefCell::new(Vec::new());
}

/// Sender that never returns error, always sends
pub struct TestSendXcm;
impl SendXcm for TestSendXcm {
    type Ticket = (MultiLocation, Xcm<()>);
    fn validate(
        dest: &mut Option<MultiLocation>,
        msg: &mut Option<Xcm<()>>,
    ) -> SendResult<(MultiLocation, Xcm<()>)> {
        let pair = (dest.take().unwrap(), msg.take().unwrap());
        Ok((pair, MultiAssets::new()))
    }
    fn deliver(pair: (MultiLocation, Xcm<()>)) -> Result<XcmHash, SendError> {
        let hash = pair.1.using_encoded(sp_io::hashing::blake2_256);
        SENT_XCM.with(|q| q.borrow_mut().push(pair));
        Ok(hash)
    }
}

parameter_types! {
    pub const RelayLocation: MultiLocation = MultiLocation::parent();
    pub const AnyNetwork: Option<NetworkId> = None;
    pub Ancestry: MultiLocation = Here.into();
    pub UnitWeightCost: u64 = 1_000;
}

pub type SovereignAccountOf = (AccountId32Aliases<AnyNetwork, AccountId>,);

pub type LocalAssetTransactor =
    XcmCurrencyAdapter<Balances, IsConcrete<RelayLocation>, SovereignAccountOf, AccountId, ()>;

type LocalOriginConverter = (
    SovereignSignedViaLocation<SovereignAccountOf, RuntimeOrigin>,
    SignedAccountId32AsNative<AnyNetwork, RuntimeOrigin>,
);

parameter_types! {
    pub const BaseXcmWeight: u64 = 1_000;
    pub CurrencyPerSecond: (xcm::latest::AssetId, u128, u128) = (Concrete(RelayLocation::get()), 1, 1);
    pub TrustedAssets: (MultiAssetFilter, MultiLocation) = (All.into(), Here.into());
    pub const MaxInstructions: u32 = 100;
    pub UniversalLocation: InteriorMultiLocation = Here;
    pub const MaxAssetsIntoHolding: u32 = 64;
}

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    AllowKnownQueryResponses<XcmPallet>,
    AllowSubscriptionsFrom<Everything>,
);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
    type RuntimeCall = RuntimeCall;
    type XcmSender = TestSendXcm;
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = LocalOriginConverter;
    type IsReserve = ();
    type IsTeleporter = ();
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
    type Trader = FixedRateOfFungible<CurrencyPerSecond, ()>;
    type ResponseHandler = XcmPallet;
    type AssetTrap = XcmPallet;
    type AssetClaims = XcmPallet;
    type SubscriptionService = XcmPallet;
    type AssetLocker = ();
    type AssetExchanger = ();
    type FeeManager = ();
    type MessageExporter = ();
    type UniversalAliases = Nothing;
    type UniversalLocation = UniversalLocation;
    type CallDispatcher = RuntimeCall;
    type SafeCallFilter = Everything;
    type PalletInstancesInfo = AllPalletsWithSystem;
    type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, AnyNetwork>;

impl pallet_xcm::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = TestSendXcm;
    type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Everything;
    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    type UniversalLocation = UniversalLocation;
    type MaxLockers = frame_support::traits::ConstU32<8>;
    type MaxRemoteLockConsumers = frame_support::traits::ConstU32<0>;
    type Currency = Balances;
    type CurrencyMatcher = IsConcrete<RelayLocation>;
    type AdminOrigin = EnsureRoot<AccountId>;
    type TrustedLockers = ();
    type SovereignAccountOf = AccountId32Aliases<(), AccountId32>;
    type RemoteLockConsumerIdentifier = ();
    type WeightInfo = pallet_xcm::TestWeightInfo;
}

const UNIT: u128 = 1000000000000;

orml_traits2::parameter_type_with_key! {
    pub CoreExistentialDeposits: |_currency_id: <Test as pallet_inv4::Config>::CoreId| -> Balance {
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
impl orml_traits2::currency::OnTransfer<AccountId, <Test as pallet_inv4::Config>::CoreId, Balance>
    for DisallowIfFrozen
{
    fn on_transfer(
        currency_id: <Test as pallet_inv4::Config>::CoreId,
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
impl orml_traits2::Happened<(AccountId, <Test as pallet_inv4::Config>::CoreId)>
    for HandleNewMembers
{
    fn happened((member, core_id): &(AccountId, <Test as pallet_inv4::Config>::CoreId)) {
        INV4::add_member(core_id, member)
    }
}

pub struct HandleRemovedMembers;
impl orml_traits2::Happened<(AccountId, <Test as pallet_inv4::Config>::CoreId)>
    for HandleRemovedMembers
{
    fn happened((member, core_id): &(AccountId, <Test as pallet_inv4::Config>::CoreId)) {
        INV4::remove_member(core_id, member)
    }
}

pub struct INV4TokenHooks;
impl
    orml_traits2::currency::MutationHooks<AccountId, <Test as pallet_inv4::Config>::CoreId, Balance>
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
    type CurrencyId = <Test as pallet_inv4::Config>::CoreId;
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
            who.clone(),
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

impl pallet_inv4::Config for Test {
    type MaxMetadata = MaxMetadata;
    type CoreId = u32;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type RuntimeCall = RuntimeCall;
    type MaxCallers = MaxCallers;
    type CoreSeedBalance = CoreSeedBalance;
    type AssetsProvider = CoreAssets;
    type RuntimeOrigin = RuntimeOrigin;
    type CoreCreationFee = CoreCreationFee;
    type FeeCharger = FeeCharger;
    type WeightInfo = pallet_inv4::weights::SubstrateWeight<Test>;

    type Tokens = Tokens;
    type KSMAssetId = RelayAssetId;
    type KSMCoreCreationFee = KSMCoreCreationFee;
    type MaxCallSize = ConstU32<51200>;

    type ParaId = ConstU32<2125>;
}

parameter_types! {
    pub ParaId: u32 = 2125u32;
    pub MaxWeightedLength: u32 = 100_000;
    pub INV4PalletIndex: u8 = 2u8;
}

impl pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Chains = Chains;
    type WeightInfo = weights::SubstrateWeight<Test>;
    type MaxXCMCallLength = ConstU32<100_000>;
    type MaintenanceOrigin = EnsureRoot<AccountId>;
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum Chains {
    ChainA,
    ChainB,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum Assets {
    AssetA,
    AssetB,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum ChainAssets {
    ChainA(Assets),
    ChainB(Assets),
}

impl ChainAssetsList for ChainAssets {
    type Chains = Chains;

    fn get_chain(&self) -> Self::Chains {
        match self {
            Self::ChainA(_) => Chains::ChainA,
            Self::ChainB(_) => Chains::ChainB,
        }
    }

    fn get_asset_location(&self) -> MultiLocation {
        match {
            match self {
                Self::ChainA(asset) => asset,
                Self::ChainB(asset) => asset,
            }
        } {
            Assets::AssetA => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(1234)),
            },

            Assets::AssetB => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2345)),
            },
        }
    }
}

impl ChainList for Chains {
    type Balance = Balance;
    type ChainAssets = ChainAssets;

    fn get_location(&self) -> MultiLocation {
        match self {
            Self::ChainA => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(1234)),
            },
            Self::ChainB => MultiLocation {
                parents: 1,
                interior: Junctions::X1(Junction::Parachain(2345)),
            },
        }
    }

    fn get_main_asset(&self) -> Self::ChainAssets {
        match self {
            Self::ChainA => ChainAssets::ChainA(Assets::AssetA),
            Self::ChainB => ChainAssets::ChainB(Assets::AssetB),
        }
    }
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
