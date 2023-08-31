use super::{
    AccountId, Balance, Balances, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime,
    RuntimeCall, RuntimeEvent, RuntimeOrigin, WeightToFee, XcmpQueue,
};
use crate::{
    assets::CORE_ASSET_ID, common_types::AssetId, constants::TreasuryAccount, AllPalletsWithSystem,
    AssetRegistry, Currencies, DealWithFees, UnknownTokens, Weight,
};
use codec::{Decode, Encode};
use cumulus_primitives_core::ParaId;
use frame_support::{
    match_types, parameter_types,
    traits::{Everything, Get, Nothing},
};
use frame_system::EnsureRoot;
use orml_asset_registry::{AssetRegistryTrader, FixedRateAssetRegistryTrader};
use orml_traits::{
    location::AbsoluteReserveProvider, parameter_type_with_key, FixedConversionRateProvider,
    MultiCurrency,
};
pub use orml_xcm_support::{
    DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset,
};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use scale_info::TypeInfo;
use sp_core::ConstU32;
use sp_runtime::traits::Convert;
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, EnsureXcmOrigin, FixedWeightBounds,
    ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeRevenue,
    TakeWeightCredit, UsingComponents, WithComputedOrigin,
};
use xcm_executor::XcmExecutor;

#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct AssetLocation(pub MultiLocation);

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

pub type Barrier = (
    // \/ FOR TESTING ONLY \/
    //xcm_builder::AllowUnpaidExecutionFrom<Everything>,
    // /\ FOR TESTING ONLY /\
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    WithComputedOrigin<AllowTopLevelPaidExecutionFrom<Everything>, UniversalLocation, ConstU32<8>>,
    // Parent and its plurality get free execution
    AllowUnpaidExecutionFrom<ParentOrParentsPlurality>,
    // Expected responses are OK.
    AllowKnownQueryResponses<PolkadotXcm>,
    // Subscriptions for version tracking are OK.
    AllowSubscriptionsFrom<Everything>,
);

parameter_types! {
    pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::get().into())));
}

parameter_types! {
    pub const RelayNetwork: NetworkId = NetworkId::Kusama;
    pub const RelayLocation: MultiLocation = MultiLocation::parent();
    pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
    pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into()));
}

match_types! {
    pub type ParentOrParentsPlurality: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Here } |
        MultiLocation { parents: 1, interior: X1(Plurality { .. }) }
    };
}

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
    // If XCM origin is an NFT from a registered chain we give it NftOrigin::Nft.
    pallet_nft_origins::NftMultilocationAsOrigin<RuntimeOrigin, Runtime>,
    // If XCM origin is an NFT Origin verifier from a registered chain we give it NftOrigin::Verifier.
    pallet_nft_origins::VerifierMultilocationAsOrigin<RuntimeOrigin, Runtime>,
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
    // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
    // recognized.
    RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognized.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
    // Native signed account converter; this just converts an `AccountId32` origin into a normal
    // `Origin::Signed` origin of the same 32-byte value.
    SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
    /// The amount of weight an XCM operation takes. This is a safe overestimate.
    pub const BaseXcmWeight: Weight = Weight::from_parts(100_000_000, 0);
    pub const MaxInstructions: u32 = 100;
    pub const MaxAssetsForTransfer: usize = 2;
}

pub struct CurrencyIdConvert;

impl Convert<AssetId, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: AssetId) -> Option<MultiLocation> {
        match id {
            CORE_ASSET_ID => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(ParachainInfo::get().into()),
                    GeneralIndex(id.into()),
                ),
            )),
            _ => AssetRegistry::multilocation(&id).unwrap_or_default(),
        }
    }
}

impl Convert<MultiLocation, Option<AssetId>> for CurrencyIdConvert {
    fn convert(location: MultiLocation) -> Option<AssetId> {
        match location {
            MultiLocation {
                parents,
                interior: X2(Parachain(id), GeneralIndex(index)),
            } if parents == 1
                && ParaId::from(id) == ParachainInfo::get()
                && (index as u32) == CORE_ASSET_ID =>
            {
                // Handling native asset for this parachain
                Some(CORE_ASSET_ID)
            }
            // handle reanchor canonical location: https://github.com/paritytech/polkadot/pull/4470
            MultiLocation {
                parents: 0,
                interior: X1(GeneralIndex(index)),
            } if (index as u32) == CORE_ASSET_ID => Some(CORE_ASSET_ID),
            // delegate to asset-registry
            _ => AssetRegistry::location_to_asset_id(location),
        }
    }
}

impl Convert<MultiAsset, Option<AssetId>> for CurrencyIdConvert {
    fn convert(asset: MultiAsset) -> Option<AssetId> {
        if let MultiAsset {
            id: Concrete(location),
            ..
        } = asset
        {
            Self::convert(location)
        } else {
            None
        }
    }
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
    fn convert(account: AccountId) -> MultiLocation {
        X1(AccountId32 {
            network: None,
            id: account.into(),
        })
        .into()
    }
}

parameter_types! {
    pub TNKRMultiLocation: MultiLocation = MultiLocation::new(0, X1(GeneralIndex(CORE_ASSET_ID.into())));
}

pub struct FeePerSecondProvider;
impl FixedConversionRateProvider for FeePerSecondProvider {
    fn get_fee_per_second(location: &MultiLocation) -> Option<u128> {
        AssetRegistry::fetch_metadata_by_location(location)?
            .additional
            .fee_per_second
    }
}

pub struct ToTreasury;
impl TakeRevenue for ToTreasury {
    fn take_revenue(revenue: MultiAsset) {
        if let MultiAsset {
            id: xcm::latest::AssetId::Concrete(id),
            fun: Fungibility::Fungible(amount),
        } = revenue
        {
            if let Some(currency_id) = CurrencyIdConvert::convert(id) {
                let _ = Currencies::deposit(currency_id, &TreasuryAccount::get(), amount);
            }
        }
    }
}

pub type Trader = (
    UsingComponents<WeightToFee, TNKRMultiLocation, AccountId, Balances, DealWithFees>,
    AssetRegistryTrader<FixedRateAssetRegistryTrader<FeePerSecondProvider>, ToTreasury>,
);

pub struct XcmConfig;

impl xcm_executor::Config for XcmConfig {
    type RuntimeCall = RuntimeCall;
    type XcmSender = XcmRouter;
    // How to withdraw and deposit an asset.
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
    type IsTeleporter = (); // disabled

    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
    type Trader = Trader;
    type ResponseHandler = PolkadotXcm;
    type AssetTrap = PolkadotXcm;
    type AssetClaims = PolkadotXcm;
    type SubscriptionService = PolkadotXcm;

    type UniversalLocation = UniversalLocation;
    type AssetLocker = ();
    type AssetExchanger = ();
    type FeeManager = ();
    type MessageExporter = ();
    type PalletInstancesInfo = AllPalletsWithSystem;
    type MaxAssetsIntoHolding = ConstU32<8>;
    type UniversalAliases = Nothing;
    type CallDispatcher = RuntimeCall;
    type SafeCallFilter = Everything;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = PolkadotXcm;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
    type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Runtime>;

    type PriceForSiblingDelivery = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

parameter_type_with_key! {
      pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
            None
      };
}

impl orml_xtokens::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type CurrencyId = AssetId;
    type CurrencyIdConvert = CurrencyIdConvert;
    type AccountIdToMultiLocation = AccountIdToMultiLocation;
    type SelfLocation = SelfLocation;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
    type BaseXcmWeight = BaseXcmWeight;
    type MaxAssetsForTransfer = MaxAssetsForTransfer;
    type MinXcmFee = ParachainMinFee;
    type MultiLocationsFilter = Everything;
    type ReserveProvider = AbsoluteReserveProvider;

    type UniversalLocation = UniversalLocation;
}

impl orml_unknown_tokens::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
}

impl orml_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SovereignOrigin = EnsureRoot<AccountId>;
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
      pub ReachableDest: Option<MultiLocation> = Some(MultiLocation::parent());
}

impl pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;

    type UniversalLocation = UniversalLocation;
    type Currency = Balances;
    type CurrencyMatcher = ();
    type TrustedLockers = ();
    type SovereignAccountOf = ();
    type MaxLockers = ConstU32<8>;
    type WeightInfo = pallet_xcm::TestWeightInfo;
    type AdminOrigin = EnsureRoot<AccountId>;
    type MaxRemoteLockConsumers = ConstU32<0>;
    type RemoteLockConsumerIdentifier = ();

    #[cfg(feature = "runtime-benchmarks")]
    type ReachableDest = ReachableDest;
}

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
    // The parent (Relay-chain) origin converts to the default `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type LocalAssetTransactor = MultiCurrencyAdapter<
    Currencies,
    UnknownTokens,
    IsNativeConcrete<AssetId, CurrencyIdConvert>,
    AccountId,
    LocationToAccountId,
    AssetId,
    CurrencyIdConvert,
    DepositToAlternative<TreasuryAccount, Currencies, AssetId, AccountId, Balance>,
>;

// Temporary \/

use codec::Compact;
use core::marker::PhantomData;
use sp_io::hashing::blake2_256;
use sp_std::prelude::*;
use xcm_executor::traits::Convert as XcmConvert;

/// Means of converting a location into a stable and unique descriptive identifier.
pub trait DescribeLocation {
    /// Create a description of the given `location` if possible. No two locations should have the
    /// same descriptor.
    fn describe_location(location: &MultiLocation) -> Option<Vec<u8>>;
}

impl<A: DescribeLocation, B: DescribeLocation, C: DescribeLocation, D: DescribeLocation>
    DescribeLocation for (A, B, C, D)
{
    fn describe_location(l: &MultiLocation) -> Option<Vec<u8>> {
        match A::describe_location(l) {
            Some(result) => Some(result),
            None => match B::describe_location(l) {
                Some(result) => Some(result),
                None => match C::describe_location(l) {
                    Some(result) => Some(result),
                    None => match D::describe_location(l) {
                        Some(result) => Some(result),
                        None => None,
                    },
                },
            },
        }
    }
}

pub struct DescribeTerminus;
impl DescribeLocation for DescribeTerminus {
    fn describe_location(l: &MultiLocation) -> Option<Vec<u8>> {
        match (l.parents, &l.interior) {
            (0, Here) => Some(Vec::new()),
            _ => return None,
        }
    }
}

pub struct DescribePalletTerminal;
impl DescribeLocation for DescribePalletTerminal {
    fn describe_location(l: &MultiLocation) -> Option<Vec<u8>> {
        match (l.parents, &l.interior) {
            (0, X1(PalletInstance(i))) => {
                Some((b"Pallet", Compact::<u32>::from(*i as u32)).encode())
            }
            _ => return None,
        }
    }
}

pub struct DescribeAccountId32Terminal;
impl DescribeLocation for DescribeAccountId32Terminal {
    fn describe_location(l: &MultiLocation) -> Option<Vec<u8>> {
        match (l.parents, &l.interior) {
            (0, X1(AccountId32 { id, .. })) => Some((b"AccountId32", id).encode()),
            _ => return None,
        }
    }
}

pub struct DescribeAccountKey20Terminal;
impl DescribeLocation for DescribeAccountKey20Terminal {
    fn describe_location(l: &MultiLocation) -> Option<Vec<u8>> {
        match (l.parents, &l.interior) {
            (0, X1(AccountKey20 { key, .. })) => Some((b"AccountKey20", key).encode()),
            _ => return None,
        }
    }
}

pub type DescribeAllTerminal = (
    DescribeTerminus,
    DescribePalletTerminal,
    DescribeAccountId32Terminal,
    DescribeAccountKey20Terminal,
);

pub struct DescribeFamily<DescribeInterior>(PhantomData<DescribeInterior>);
impl<Suffix: DescribeLocation> DescribeLocation for DescribeFamily<Suffix> {
    fn describe_location(l: &MultiLocation) -> Option<Vec<u8>> {
        match (l.parents, l.interior.first()) {
            (0, Some(Parachain(index))) => {
                let tail = l.interior.split_first().0;
                let interior = Suffix::describe_location(&tail.into())?;
                Some((b"ChildChain", Compact::<u32>::from(*index), interior).encode())
            }
            (1, Some(Parachain(index))) => {
                let tail = l.interior.split_first().0;
                let interior = Suffix::describe_location(&tail.into())?;
                Some((b"SiblingChain", Compact::<u32>::from(*index), interior).encode())
            }
            (1, _) => {
                let tail = l.interior.into();
                let interior = Suffix::describe_location(&tail)?;
                Some((b"ParentChain", interior).encode())
            }
            _ => return None,
        }
    }
}

pub struct HashedDescription<AccountId, Describe>(PhantomData<(AccountId, Describe)>);
impl<AccountId: From<[u8; 32]> + Clone + core::fmt::Debug, Describe: DescribeLocation>
    XcmConvert<MultiLocation, AccountId> for HashedDescription<AccountId, Describe>
{
    fn convert(value: MultiLocation) -> Result<AccountId, MultiLocation> {
        if let Some(description) = Describe::describe_location(&value) {
            log::trace!(
                  target: "xcm::LocationToAccountId",
                  "HashedDescription location: {:?}, account_id: {:?}",
                  value, AccountId::from(blake2_256(&description))
            );

            Ok(blake2_256(&description).into())
        } else {
            Err(value)
        }
    }
}
