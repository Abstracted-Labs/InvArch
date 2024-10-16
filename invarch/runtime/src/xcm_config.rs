use crate::{Balance, Currencies, DealWithFees, TreasuryAccount};

use super::{
    assets::VARCH_ASSET_ID, common_types::AssetId, AccountId, AllPalletsWithSystem, AssetRegistry,
    Balances, ConstU32, MessageQueue, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime,
    RuntimeBlockWeights, RuntimeCall, RuntimeEvent, RuntimeOrigin, Weight, WeightToFee, XcmpQueue,
};
use codec::{Decode, Encode, MaxEncodedLen};
use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use frame_support::{
    parameter_types,
    traits::{Everything, Nothing, TransformOrigin},
};
use frame_system::EnsureRoot;
use orml_traits2::{
    location::AbsoluteReserveProvider, parameter_type_with_key, FixedConversionRateProvider,
    MultiCurrency,
};
use orml_xcm_support::{
    DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset,
};
use pallet_dao_staking::primitives::{
    CustomAggregateMessageOrigin, CustomMessageProcessor, CustomNarrowOriginToSibling,
    CustomParaIdToSibling,
};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{Convert, MaybeEquivalence},
    Perbill,
};
use sp_std::sync::Arc;
use xcm::{v3::MultiLocation, v4::prelude::*};
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, DescribeAllTerminal, DescribeFamily,
    EnsureXcmOrigin, FixedWeightBounds, HashedDescription, ParentIsPreset, RelayChainAsNative,
    SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
    SignedToAccountId32, SovereignSignedViaLocation, TakeRevenue, TakeWeightCredit,
    UsingComponents, WithComputedOrigin,
};
use xcm_executor::XcmExecutor;

parameter_types! {
    pub const RelayLocation: Location = Location::parent();
    pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
    pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: Location = Parachain(ParachainInfo::parachain_id().into()).into();
    pub UniversalLocation: InteriorLocation = [GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into())].into();
    pub const RelayAggregate: CustomAggregateMessageOrigin<AggregateMessageOrigin> = CustomAggregateMessageOrigin::Aggregate(AggregateMessageOrigin::Parent);
    pub SelfLocation: Location = Location::new(1, cumulus_primitives_core::Junctions::X1(Arc::new([Parachain(ParachainInfo::parachain_id().into());1])));
    pub LocalAssetLocation: Location = Location::new(0, Junctions::X1([Junction::GeneralIndex(VARCH_ASSET_ID.into())].into()));
}

/// Type for specifying how a `Location` can be converted into an `AccountId`.
///
/// This is used when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the default `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
    // Generate remote accounts according to polkadot standards
    HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
);

pub type NewLocalAssetTransactor = MultiCurrencyAdapter<
    Currencies,
    (),
    IsNativeConcrete<AssetId, CurrencyIdConvert>,
    AccountId,
    LocationToAccountId,
    AssetId,
    CurrencyIdConvert,
    DepositToAlternative<TreasuryAccount, Currencies, AssetId, AccountId, Balance>,
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
///
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
///
pub type XcmOriginToTransactDispatchOrigin = (
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
    pub const BaseXcmWeight: Weight = Weight::from_parts(100_000_000, 0);
    pub const MaxInstructions: u32 = 100;
    pub const MaxAssetsIntoHolding: u32 = 8;
}

pub struct ParentOrParentsPlurality;
impl frame_support::traits::Contains<Location> for ParentOrParentsPlurality {
    fn contains(location: &Location) -> bool {
        matches!(location.unpack(), (1, []) | (1, [Plurality { .. }]))
    }
}

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    // Parent and its plurality get free execution
    AllowUnpaidExecutionFrom<ParentOrParentsPlurality>,
    // Expected responses are OK.
    AllowKnownQueryResponses<PolkadotXcm>,
    // Subscriptions for version tracking are OK.
    AllowSubscriptionsFrom<Everything>,
    WithComputedOrigin<
        (
            AllowTopLevelPaidExecutionFrom<Everything>,
            // Subscriptions for version tracking are OK.
            AllowSubscriptionsFrom<Everything>,
        ),
        UniversalLocation,
        ConstU32<8>,
    >,
);

pub struct ToTreasury;
impl TakeRevenue for ToTreasury {
    fn take_revenue(revenue: Asset) {
        if let Asset {
            id: AssetId(location),
            fun: Fungible(amount),
        } = revenue
        {
            if let Some(currency_id) = CurrencyIdConvert::convert(location) {
                let _ = crate::Currencies::deposit(currency_id, &TreasuryAccount::get(), amount);
            }
        }
    }
}

pub type AssetRegistryWeightTrader = orml_asset_registry::AssetRegistryTrader<
    orml_asset_registry::FixedRateAssetRegistryTrader<MyFixedConversionRateProvider>,
    ToTreasury,
>;
pub struct MyFixedConversionRateProvider;
impl FixedConversionRateProvider for MyFixedConversionRateProvider {
    fn get_fee_per_second(location: &Location) -> Option<u128> {
        CurrencyIdConvert::convert(location.clone()).and_then(|id| {
            if id == VARCH_ASSET_ID {
                // for VARCH we use the UsingComponents Trader.
                None
            } else {
                crate::AssetRegistry::metadata(id)
                    .and_then(|metadata| Some(metadata.additional.fee_per_second))
            }
        })
    }
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
    type RuntimeCall = RuntimeCall;
    type XcmSender = XcmRouter;
    // How to withdraw and deposit an asset.
    type AssetTransactor = NewLocalAssetTransactor;
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
    type IsTeleporter = (); // Teleporting is disabled.
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
    type Trader = (
        UsingComponents<WeightToFee, LocalAssetLocation, AccountId, Balances, DealWithFees>,
        AssetRegistryWeightTrader,
    );
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
    type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
    type UniversalAliases = Nothing;
    type CallDispatcher = RuntimeCall;
    type SafeCallFilter = Everything;
    type Aliasers = Nothing;
    type TransactionalProcessor = xcm_builder::FrameTransactionalProcessor;
    type HrmpNewChannelOpenRequestHandler = ();
    type HrmpChannelAcceptedHandler = ();
    type HrmpChannelClosingHandler = ();
    type XcmRecorder = ();
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
    pub ReachableDest: Option<Location> = Some(Location::parent());
}

parameter_types! {
    pub const MaxLockers: u32 = 8;
    pub const MaxRemoteLockConsumers: u32 = 0;
}

impl pallet_xcm::Config for Runtime {
    type AdminOrigin = EnsureRoot<AccountId>;
    // ^ Override for AdvertisedXcmVersion default
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    type Currency = Balances;
    type CurrencyMatcher = ();
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type MaxLockers = MaxLockers;
    type MaxRemoteLockConsumers = MaxRemoteLockConsumers;
    type RemoteLockConsumerIdentifier = ();
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type SovereignAccountOf = ();
    type TrustedLockers = ();
    type UniversalLocation = UniversalLocation;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
    type WeightInfo = pallet_xcm::TestWeightInfo;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmReserveTransferFilter = Everything;
    type XcmRouter = XcmRouter;
    type XcmTeleportFilter = Nothing;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl orml_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SovereignOrigin = EnsureRoot<AccountId>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = PolkadotXcm;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
    type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
    type WeightInfo = ();
    type XcmpQueue = TransformOrigin<
        MessageQueue,
        CustomAggregateMessageOrigin<AggregateMessageOrigin>,
        ParaId,
        CustomParaIdToSibling,
    >;
    type MaxInboundSuspended = ConstU32<1000>;
    type MaxActiveOutboundChannels = sp_core::ConstU32<128>;
    type MaxPageSize = sp_core::ConstU32<{ 103 * 1024 }>;
}

// Deprecated
// impl cumulus_pallet_dmp_queue::Config for Runtime {
//     type RuntimeEvent = RuntimeEvent;
//     type DmpSink = EnqueueWithOrigin<MessageQueue, RelayAggregate>;
//     type WeightInfo = cumulus_pallet_dmp_queue::weights::SubstrateWeight<Self>;
// }

parameter_types! {
    pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
    pub const MessageQueueMaxStale: u32 = 8;
    pub const MessageQueueHeapSize: u32 = 128 * 1048;
    pub IdleMaxServiceWeight: Weight = Perbill::from_percent(15) * RuntimeBlockWeights::get().max_block;
}

impl pallet_message_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_message_queue::weights::SubstrateWeight<Self>;
    #[cfg(feature = "runtime-benchmarks")]
    type MessageProcessor = pallet_message_queue::mock_helpers::NoopMessageProcessor<
        CustomAggregateMessageOrigin<AggregateMessageOrigin>,
    >;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type MessageProcessor = CustomMessageProcessor<
        CustomAggregateMessageOrigin<AggregateMessageOrigin>,
        AggregateMessageOrigin,
        xcm_builder::ProcessXcmMessage<
            AggregateMessageOrigin,
            xcm_executor::XcmExecutor<XcmConfig>,
            RuntimeCall,
        >,
        RuntimeCall,
        Runtime,
    >;
    type Size = u32;
    type QueueChangeHandler = CustomNarrowOriginToSibling<XcmpQueue, Runtime>;
    type QueuePausedQuery = CustomNarrowOriginToSibling<XcmpQueue, Runtime>;
    type HeapSize = MessageQueueHeapSize;
    type MaxStale = MessageQueueMaxStale;
    type ServiceWeight = MessageQueueServiceWeight;
    type IdleMaxServiceWeight = IdleMaxServiceWeight;
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, Location> for AccountIdToMultiLocation {
    fn convert(account: AccountId) -> Location {
        [AccountId32 {
            network: None,
            id: account.into(),
        }]
        .into()
    }
}

const ASSET_HUB_PARA_ID: u32 = 1000;

parameter_type_with_key! {
    pub ParachainMinFee: |location: Location| -> Option<u128> {
        #[allow(clippy::match_ref_pats)] // false positive
        match (location.parents, location.first_interior()) {
            (1, Some(Parachain(ASSET_HUB_PARA_ID))) => Some(50_000_000),
            _ => None,
        }
    };
}

#[derive(Debug, Default, Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct AssetLocation(pub xcm::v3::Location);

impl From<AssetLocation> for Option<Location> {
    fn from(location: AssetLocation) -> Option<Location> {
        xcm_builder::WithLatestLocationConverter::<xcm::v3::Location>::convert_back(&location.0)
    }
}

impl TryFrom<Location> for AssetLocation {
    type Error = ();

    fn try_from(value: Location) -> Result<Self, Self::Error> {
        let loc: MultiLocation = value.try_into()?;
        Ok(AssetLocation(loc))
    }
}

pub struct CurrencyIdConvert;

impl Convert<AssetId, Option<Location>> for CurrencyIdConvert {
    fn convert(id: AssetId) -> Option<Location> {
        match id {
            VARCH_ASSET_ID => Some(Location {
                parents: 1,
                interior: [
                    Parachain(ParachainInfo::parachain_id().into()),
                    GeneralIndex(id.into()),
                ]
                .into(),
            }),
            _ => {
                if let Ok(Some(location)) = AssetRegistry::location(&id) {
                    AssetLocation(location).into()
                } else {
                    None
                }
            }
        }
    }
}

impl Convert<Location, Option<AssetId>> for CurrencyIdConvert {
    fn convert(location: Location) -> Option<AssetId> {
        let Location { parents, interior } = location.clone();

        match interior {
            Junctions::X2(a)
                if parents == 1
                    && a.contains(&Parachain(ParachainInfo::parachain_id().into()))
                    && a.contains(&GeneralIndex(VARCH_ASSET_ID.into())) =>
            {
                Some(VARCH_ASSET_ID)
            }
            Junctions::X1(a)
                if parents == 0 && a.contains(&GeneralIndex(VARCH_ASSET_ID.into())) =>
            {
                Some(VARCH_ASSET_ID)
            }
            _ => {
                let location: Option<AssetLocation> = location.try_into().ok();
                if let Some(location) = location {
                    AssetRegistry::location_to_asset_id(location.0)
                } else {
                    None
                }
            }
        }
    }
}

impl Convert<Asset, Option<AssetId>> for CurrencyIdConvert {
    fn convert(asset: Asset) -> Option<AssetId> {
        Self::convert(asset.id.0)
    }
}
