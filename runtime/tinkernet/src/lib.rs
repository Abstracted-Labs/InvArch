// Copyright 2021-2022 InvArch Association.
// This file is part of InvArch.

// InvArch is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// InvArch is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with InvArch.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod xcm_config;

use codec::{Decode, Encode};
use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
pub use frame_support::{
    construct_runtime, match_types, parameter_types,
    traits::{
        AsEnsureOriginWithArg, Contains, Currency, EqualPrivilegeOnly, Everything, FindAuthor,
        Imbalance, KeyOwnerProofSystem, Nothing, OnUnbalanced, Randomness, StorageInfo,
    },
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        ConstantMultiplier, DispatchClass, IdentityFee, Weight, WeightToFeeCoefficient,
        WeightToFeeCoefficients, WeightToFeePolynomial,
    },
    BoundedVec, ConsensusEngineId, PalletId,
};
use frame_support::{dispatch::RawOrigin, pallet_prelude::EnsureOrigin};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot, EnsureSigned,
};
use pallet_transaction_payment::Multiplier;
use polkadot_runtime_common::SlowAdjustingFeeUpdate;
use scale_info::TypeInfo;
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, H160};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{
        AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, Verify,
    },
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, FixedPointNumber, MultiSignature, Perquintill,
};
pub use sp_runtime::{Perbill, Permill};
use sp_std::{marker::PhantomData, prelude::*};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

use xcm::latest::prelude::BodyId;

/// Import the ipf pallet.
pub use pallet_ipf as ipf;

/// Import the inv4 pallet.
pub use pallet_inv4 as inv4;

use inv4::ipl::LicenseList;

// Weights
mod weights;

use sp_core::crypto::ByteArray;

pub struct FindAuthorTruncated<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
    fn find_author<'a, I>(digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        if let Some(author_index) = F::find_author(digests) {
            let authority_id = Aura::authorities()[author_index as usize].clone();
            return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]));
        }
        None
    }
}

mod constants;
use constants::currency::*;
mod common_types;
use common_types::*;
mod assets;
mod inflation;
mod staking;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
///
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    // Remove this before next runtime upgrade
    (CheckedInflationMigration,),
>;

pub type CheckedInflationMigration =
    pallet_checked_inflation::migrations::first_time::InitializeStorages<Runtime>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
    use sp_runtime::{generic, traits::BlakeTwo256};

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
    }
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("tinkernet_node"),
    impl_name: create_runtime_str!("tinkernet_node"),
    authoring_version: 1,
    spec_version: 14,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 12000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The existential deposit. Set to 1/10 of the Connected Relay Chain
pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND.saturating_div(2);

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

pub const SS58_PREFIX: u16 = 117u16;

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;

    // This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
    //  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
    // `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
    // the lazy contract deletion.
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
    pub const SS58Prefix: u16 = SS58_PREFIX;

    pub const BlockHashCount: BlockNumber = 1200;
}

pub struct BaseFilter;
impl Contains<Call> for BaseFilter {
    fn contains(_c: &Call) -> bool {
        // !matches!(
        //     c,
        //     Call::XTokens(_)
        //         | Call::PolkadotXcm(_)
        //         | Call::OrmlXcm(_)
        //         | Call::Currencies(_)
        //         | Call::Tokens(_)
        // )
        true
    }
}

pub struct MaintenanceFilter;
impl Contains<Call> for MaintenanceFilter {
    fn contains(c: &Call) -> bool {
        !matches!(
            c,
            Call::Balances(_)
                | Call::Vesting(_)
                | Call::XTokens(_)
                | Call::PolkadotXcm(_)
                | Call::OrmlXcm(_)
                | Call::Currencies(_)
                | Call::Tokens(_)
        )
    }
}

/// The hooks we want to run in Maintenance Mode
pub struct MaintenanceHooks;

impl frame_support::traits::OnInitialize<BlockNumber> for MaintenanceHooks {
    fn on_initialize(n: BlockNumber) -> Weight {
        AllPalletsWithSystem::on_initialize(n)
    }
}

impl frame_support::traits::OnRuntimeUpgrade for MaintenanceHooks {
    fn on_runtime_upgrade() -> Weight {
        AllPalletsWithSystem::on_runtime_upgrade()
    }
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        AllPalletsWithSystem::pre_upgrade()
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        AllPalletsWithSystem::post_upgrade()
    }
}

impl frame_support::traits::OnFinalize<BlockNumber> for MaintenanceHooks {
    fn on_finalize(n: BlockNumber) {
        AllPalletsWithSystem::on_finalize(n)
    }
}

impl frame_support::traits::OnIdle<BlockNumber> for MaintenanceHooks {
    fn on_idle(_n: BlockNumber, _max_weight: Weight) -> Weight {
        Weight::zero()
    }
}

impl frame_support::traits::OffchainWorker<BlockNumber> for MaintenanceHooks {
    fn offchain_worker(n: BlockNumber) {
        AllPalletsWithSystem::offchain_worker(n)
    }
}

impl pallet_maintenance_mode::Config for Runtime {
    type Event = Event;
    type NormalCallFilter = BaseFilter;
    type MaintenanceCallFilter = MaintenanceFilter;
    type MaintenanceOrigin = EnsureRoot<AccountId>;
    // We use AllPalletsReversedWithSystemFirst because we dont want to change the hooks in normal
    // operation
    type NormalExecutiveHooks = AllPalletsWithSystem;
    type MaintenanceExecutiveHooks = MaintenanceHooks;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = MaintenanceMode;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    /// The set code logic, just the default since we're not a parachain.
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = (CollatorSelection,);
}

parameter_types! {
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    /// Relay Chain `TransactionByteFee` / 10
    pub const TransactionByteFee: Balance = 10 * MICROUNIT;
    pub const OperationalFeeMultiplier: u8 = 5;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);

    pub const WeightToFeeScalar: Balance = 150;
}

pub struct ToStakingPot;
impl OnUnbalanced<NegativeImbalance> for ToStakingPot {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        let staking_pot = PotId::get().into_account_truncating();
        Balances::resolve_creating(&staking_pot, amount);
    }
}

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(mut fees) = fees_then_tips.next() {
            if let Some(tips) = fees_then_tips.next() {
                // Merge with fee, for now we send everything to the treasury
                tips.merge_into(&mut fees);
            }

            let (to_collators, to_treasury) = fees.ration(50, 50);

            Treasury::on_unbalanced(to_treasury);
            ToStakingPot::on_unbalanced(to_collators);
        }
    }
}

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        let p = UNIT / 500;
        let q = Balance::from(ExtrinsicBaseWeight::get().ref_time());
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}

impl pallet_transaction_payment::Config for Runtime {
    type Event = Event;
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
    type WeightToFee = WeightToFee;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
    type OnSystemEvent = ();
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type DmpMessageHandler = DmpQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type OutboundXcmpMessageSource = XcmpQueue;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
    pub const Period: u32 = 6 * HOURS;
    pub const Offset: u32 = 0;
    pub const MaxAuthorities: u32 = 100_000;
}

impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    // we don't have stash and controller, thus we don't need the convert as well.
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = CollatorSelection;
    // Essentially just Aura, but lets be pedantic.
    type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
    pub const PotId: PalletId = PalletId(*b"ia/Potst");
    pub const MaxCandidates: u32 = 50;
    pub const MinCandidates: u32 = 5;
    pub const SessionLength: BlockNumber = 6 * HOURS;
    pub const MaxInvulnerables: u32 = 100;
    pub const ExecutiveBody: BodyId = BodyId::Executive;
}

// We allow root only to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;

impl pallet_collator_selection::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type UpdateOrigin = CollatorSelectionUpdateOrigin;
    type PotId = PotId;
    type MaxCandidates = MaxCandidates;
    type MinCandidates = MinCandidates;
    type MaxInvulnerables = MaxInvulnerables;
    // should be a multiple of session or things will get inconsistent
    type KickThreshold = Period;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ValidatorRegistration = Session;
    type WeightInfo = pallet_collator_selection::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    // The maximum size of an IPF's metadata
    pub const MaxIpfMetadata: u32 = 10000;
}

impl ipf::Config for Runtime {
    // The maximum size of an IPF's metadata
    type MaxIpfMetadata = MaxIpfMetadata;
    // The IPF ID type
    type IpfId = u64;
    // Th IPF pallet events
    type Event = Event;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Encode, Decode, TypeInfo, Eq, PartialEq)]
pub enum InvArchLicenses {
    /// Apache License 2.0 | https://choosealicense.com/licenses/apache-2.0/
    Apache2,
    /// GNU General Public License v3.0 | https://choosealicense.com/licenses/gpl-3.0/
    GPLv3,
    /// GNU General Public License v2.0 | https://choosealicense.com/licenses/gpl-2.0/
    GPLv2,
    /// GNU Affero General Public License v3.0 | https://choosealicense.com/licenses/agpl-3.0/
    AGPLv3,
    /// GNU Lesser General Public License v3.0 | https://choosealicense.com/licenses/lgpl-3.0/
    LGPLv3,
    /// MIT License | https://choosealicense.com/licenses/mit/
    MIT,
    /// ISC License | https://choosealicense.com/licenses/isc/
    ISC,
    /// Mozilla Public License 2.0 | https://choosealicense.com/licenses/mpl-2.0/
    MPLv2,
    /// Boost Software License 1.0 | https://choosealicense.com/licenses/bsl-1.0/
    BSLv1,
    /// The Unlicense | https://choosealicense.com/licenses/unlicense/
    Unlicense,
    /// Creative Commons Zero v1.0 Universal | https://choosealicense.com/licenses/cc0-1.0/
    CC0_1,
    /// Creative Commons Attribution 4.0 International | https://choosealicense.com/licenses/cc-by-4.0/
    CC_BY_4,
    /// Creative Commons Attribution Share Alike 4.0 International | https://choosealicense.com/licenses/cc-by-sa-4.0/
    CC_BY_SA_4,
    /// Creative Commons Attribution-NoDerivatives 4.0 International | https://creativecommons.org/licenses/by-nd/4.0/
    CC_BY_ND_4,
    /// Creative Commons Attribution-NonCommercial 4.0 International | http://creativecommons.org/licenses/by-nc/4.0/
    CC_BY_NC_4,
    /// Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International | http://creativecommons.org/licenses/by-nc-sa/4.0/
    CC_BY_NC_SA_4,
    /// Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International | http://creativecommons.org/licenses/by-nc-nd/4.0/
    CC_BY_NC_ND_4,
    /// SIL Open Font License 1.1 | https://choosealicense.com/licenses/ofl-1.1/
    OFL_1_1,
    /// Dapper Labs' NFT License Version 2.0 | https://www.nftlicense.org/
    NFT_License_2,
    Custom(
        BoundedVec<u8, <Runtime as inv4::Config>::MaxMetadata>,
        <Runtime as frame_system::Config>::Hash,
    ),
}

impl LicenseList<Runtime> for InvArchLicenses {
    /// Returns the license name as bytes and the IPFS hash of the licence on IPFS
    fn get_hash_and_metadata(
        &self,
    ) -> (
        BoundedVec<u8, <Runtime as inv4::Config>::MaxMetadata>,
        <Runtime as frame_system::Config>::Hash,
    ) {
        match self {
            InvArchLicenses::Apache2 => (
                vec![
                    65, 112, 97, 99, 104, 101, 32, 76, 105, 99, 101, 110, 115, 101, 32, 50, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    7, 57, 92, 251, 234, 183, 217, 144, 220, 196, 201, 132, 176, 249, 18, 224, 237,
                    201, 2, 113, 146, 78, 111, 152, 92, 71, 16, 228, 87, 39, 81, 142,
                ]
                .into(),
            ),
            InvArchLicenses::GPLv3 => (
                vec![
                    71, 78, 85, 32, 71, 101, 110, 101, 114, 97, 108, 32, 80, 117, 98, 108, 105, 99,
                    32, 76, 105, 99, 101, 110, 115, 101, 32, 118, 51, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    72, 7, 169, 24, 30, 7, 200, 69, 232, 27, 10, 138, 130, 253, 91, 158, 210, 95,
                    127, 37, 85, 41, 106, 136, 66, 116, 64, 35, 252, 195, 69, 253,
                ]
                .into(),
            ),
            InvArchLicenses::GPLv2 => (
                vec![
                    71, 78, 85, 32, 71, 101, 110, 101, 114, 97, 108, 32, 80, 117, 98, 108, 105, 99,
                    32, 76, 105, 99, 101, 110, 115, 101, 32, 118, 50, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    83, 11, 214, 48, 75, 23, 172, 31, 175, 110, 63, 110, 178, 73, 2, 178, 184, 21,
                    246, 188, 76, 84, 217, 226, 18, 136, 59, 165, 230, 221, 238, 176,
                ]
                .into(),
            ),
            InvArchLicenses::AGPLv3 => (
                vec![
                    71, 78, 85, 32, 65, 102, 102, 101, 114, 111, 32, 71, 101, 110, 101, 114, 97,
                    108, 32, 80, 117, 98, 108, 105, 99, 32, 76, 105, 99, 101, 110, 115, 101, 32,
                    118, 51, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    16, 157, 152, 89, 106, 226, 188, 217, 72, 112, 106, 206, 65, 165, 183, 196, 92,
                    139, 38, 166, 5, 26, 115, 178, 28, 146, 161, 129, 62, 94, 35, 237,
                ]
                .into(),
            ),
            InvArchLicenses::LGPLv3 => (
                vec![
                    71, 78, 85, 32, 76, 101, 115, 115, 101, 114, 32, 71, 101, 110, 101, 114, 97,
                    108, 32, 80, 117, 98, 108, 105, 99, 32, 76, 105, 99, 101, 110, 115, 101, 32,
                    118, 51, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    41, 113, 123, 121, 57, 73, 217, 57, 239, 157, 246, 130, 231, 72, 190, 228, 200,
                    196, 32, 236, 163, 234, 84, 132, 137, 143, 25, 250, 176, 138, 20, 72,
                ]
                .into(),
            ),
            InvArchLicenses::MIT => (
                vec![77, 73, 84, 32, 76, 105, 99, 101, 110, 115, 101]
                    .try_into()
                    .unwrap(),
                [
                    30, 110, 34, 127, 171, 16, 29, 6, 239, 45, 145, 39, 222, 102, 84, 140, 102,
                    230, 120, 249, 189, 170, 34, 83, 199, 156, 9, 49, 150, 152, 11, 200,
                ]
                .into(),
            ),
            InvArchLicenses::ISC => (
                vec![73, 83, 67, 32, 76, 105, 99, 101, 110, 115, 101]
                    .try_into()
                    .unwrap(),
                [
                    119, 124, 140, 27, 203, 222, 251, 174, 95, 70, 118, 187, 129, 69, 225, 96, 227,
                    232, 195, 7, 229, 132, 185, 27, 190, 77, 151, 87, 106, 54, 147, 44,
                ]
                .into(),
            ),
            InvArchLicenses::MPLv2 => (
                vec![
                    77, 111, 122, 105, 108, 108, 97, 32, 80, 117, 98, 108, 105, 99, 32, 76, 105,
                    99, 101, 110, 115, 101, 32, 50, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    22, 230, 111, 228, 166, 207, 221, 50, 16, 229, 13, 232, 100, 77, 102, 184, 158,
                    79, 129, 211, 209, 102, 176, 109, 87, 105, 70, 160, 64, 123, 111, 125,
                ]
                .into(),
            ),
            InvArchLicenses::BSLv1 => (
                vec![
                    66, 111, 111, 115, 116, 32, 83, 111, 102, 116, 119, 97, 114, 101, 32, 76, 105,
                    99, 101, 110, 115, 101, 32, 49, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    174, 124, 16, 124, 106, 249, 123, 122, 241, 56, 223, 75, 59, 68, 65, 204, 73,
                    69, 88, 196, 145, 163, 233, 220, 238, 63, 99, 237, 91, 2, 44, 204,
                ]
                .into(),
            ),
            InvArchLicenses::Unlicense => (
                vec![84, 104, 101, 32, 85, 110, 108, 105, 99, 101, 110, 115, 101]
                    .try_into()
                    .unwrap(),
                [
                    208, 213, 16, 2, 240, 247, 235, 52, 119, 223, 47, 248, 137, 215, 165, 255, 76,
                    216, 178, 1, 189, 80, 159, 6, 76, 219, 36, 87, 18, 95, 66, 69,
                ]
                .into(),
            ),
            InvArchLicenses::CC0_1 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    90, 101, 114, 111, 32, 118, 49, 46, 48, 32, 85, 110, 105, 118, 101, 114, 115,
                    97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    157, 190, 198, 99, 94, 106, 166, 7, 57, 110, 33, 230, 148, 72, 5, 109, 159,
                    142, 83, 41, 164, 67, 188, 195, 189, 191, 36, 11, 61, 171, 27, 20,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 32, 52, 46, 48, 32, 73,
                    110, 116, 101, 114, 110, 97, 116, 105, 111, 110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    40, 210, 60, 93, 221, 27, 242, 205, 66, 90, 61, 65, 117, 72, 161, 102, 0, 242,
                    255, 168, 0, 82, 46, 245, 187, 126, 239, 220, 22, 231, 141, 195,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_SA_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 32, 83, 104, 97, 114, 101,
                    32, 65, 108, 105, 107, 101, 32, 52, 46, 48, 32, 73, 110, 116, 101, 114, 110,
                    97, 116, 105, 111, 110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    250, 189, 246, 254, 64, 139, 178, 19, 24, 92, 176, 241, 128, 91, 98, 105, 205,
                    149, 22, 98, 175, 178, 74, 187, 181, 189, 44, 158, 64, 117, 224, 61,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_ND_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 45, 78, 111, 68, 101, 114,
                    105, 118, 97, 116, 105, 118, 101, 115, 32, 52, 46, 48, 32, 73, 110, 116, 101,
                    114, 110, 97, 116, 105, 111, 110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    50, 75, 4, 246, 125, 55, 242, 42, 183, 14, 224, 101, 36, 251, 72, 169, 71, 35,
                    92, 129, 50, 38, 165, 223, 90, 240, 205, 149, 113, 56, 115, 85,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_NC_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 45, 78, 111, 110, 67, 111,
                    109, 109, 101, 114, 99, 105, 97, 108, 32, 52, 46, 48, 32, 73, 110, 116, 101,
                    114, 110, 97, 116, 105, 111, 110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    30, 62, 213, 3, 26, 115, 233, 140, 111, 241, 54, 179, 119, 44, 203, 198, 240,
                    172, 227, 68, 101, 15, 57, 156, 29, 234, 167, 155, 66, 200, 219, 146,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_NC_SA_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 45, 78, 111, 110, 67, 111,
                    109, 109, 101, 114, 99, 105, 97, 108, 45, 83, 104, 97, 114, 101, 65, 108, 105,
                    107, 101, 32, 52, 46, 48, 32, 73, 110, 116, 101, 114, 110, 97, 116, 105, 111,
                    110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    52, 186, 173, 229, 107, 225, 22, 146, 198, 254, 191, 247, 180, 34, 43, 39, 219,
                    40, 4, 143, 186, 8, 23, 44, 210, 224, 186, 201, 166, 41, 158, 121,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_NC_ND_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 45, 78, 111, 110, 67, 111,
                    109, 109, 101, 114, 99, 105, 97, 108, 45, 78, 111, 68, 101, 114, 105, 118, 97,
                    116, 105, 118, 101, 115, 32, 52, 46, 48, 32, 73, 110, 116, 101, 114, 110, 97,
                    116, 105, 111, 110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    127, 207, 189, 44, 174, 24, 37, 236, 169, 209, 80, 31, 171, 44, 32, 63, 200,
                    40, 59, 177, 185, 27, 199, 7, 96, 93, 98, 43, 219, 226, 216, 52,
                ]
                .into(),
            ),
            InvArchLicenses::OFL_1_1 => (
                vec![
                    83, 73, 76, 32, 79, 112, 101, 110, 32, 70, 111, 110, 116, 32, 76, 105, 99, 101,
                    110, 115, 101, 32, 49, 46, 49,
                ]
                .try_into()
                .unwrap(),
                [
                    44, 228, 173, 234, 177, 180, 217, 203, 36, 28, 127, 255, 113, 162, 181, 151,
                    240, 101, 203, 142, 246, 219, 177, 3, 77, 139, 82, 210, 87, 200, 140, 196,
                ]
                .into(),
            ),
            InvArchLicenses::NFT_License_2 => (
                vec![
                    78, 70, 84, 32, 76, 105, 99, 101, 110, 115, 101, 32, 86, 101, 114, 115, 105,
                    111, 110, 32, 50, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    126, 111, 159, 224, 78, 176, 72, 197, 201, 197, 30, 50, 31, 166, 61, 182, 81,
                    131, 149, 233, 202, 149, 92, 62, 241, 34, 86, 196, 64, 243, 112, 152,
                ]
                .into(),
            ),
            InvArchLicenses::Custom(metadata, hash) => (metadata.clone(), *hash),
        }
    }
}

parameter_types! {
    pub const MaxMetadata: u32 = 10000;
    pub const MaxCallers: u32 = 10000;
    pub const MaxLicenseMetadata: u32 = 10000;
}

impl inv4::Config for Runtime {
    // The maximum size of an IPS's metadata
    type MaxMetadata = MaxMetadata;
    // The IPS ID type
    type IpId = CommonId;
    // The IPS Pallet Events
    type Event = Event;
    // Currency
    type Currency = Balances;
    // The ExistentialDeposit
    type ExistentialDeposit = ExistentialDeposit;

    type Balance = Balance;

    type Call = Call;
    type MaxCallers = MaxCallers;
    type WeightToFee = WeightToFee;
    type MaxSubAssets = MaxCallers;
    type Licenses = InvArchLicenses;
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(1);
    pub const ProposalBondMinimum: Balance = 100 * UNIT;
    pub const SpendPeriod: BlockNumber = DAYS;
    pub const Burn: Permill = Permill::from_percent(1);
    pub const TreasuryPalletId: PalletId = PalletId(*b"ia/trsry");
    pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type ApproveOrigin = EnsureRoot<AccountId>;
    type RejectOrigin = EnsureRoot<AccountId>;
    type Event = Event;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = ();
    type BurnDestination = ();
    type SpendFunds = ();
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
    type MaxApprovals = MaxApprovals;
    type ProposalBondMaximum = ();
    type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;
}

parameter_types! {
      pub const ResourceSymbolLimit: u32 = 10;
      pub const PartsLimit: u32 = 25;
      pub const MaxPriorities: u32 = 25;
      pub const CollectionSymbolLimit: u32 = 100;
      pub const MaxResourcesOnMint: u32 = 100;
      pub const NestingBudget: u32 = 20;
}

impl pallet_rmrk_core::Config for Runtime {
    type Event = Event;
    type ProtocolOrigin = frame_system::EnsureRoot<AccountId>;
    type ResourceSymbolLimit = ResourceSymbolLimit;
    type PartsLimit = PartsLimit;
    type MaxPriorities = MaxPriorities;
    type CollectionSymbolLimit = CollectionSymbolLimit;
    type MaxResourcesOnMint = MaxResourcesOnMint;
    type NestingBudget = NestingBudget;
    type WeightInfo = pallet_rmrk_core::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
      pub const MaxPropertiesPerTheme: u32 = 100;
      pub const MaxCollectionsEquippablePerPart: u32 = 100;
}

impl pallet_rmrk_equip::Config for Runtime {
    type Event = Event;
    type MaxPropertiesPerTheme = MaxPropertiesPerTheme;
    type MaxCollectionsEquippablePerPart = MaxCollectionsEquippablePerPart;
}

parameter_types! {
      pub const CollectionDeposit: Balance = 10 * MILLIUNIT;
      pub const ItemDeposit: Balance = UNIT;
      pub const KeyLimit: u32 = 32;
      pub const ValueLimit: u32 = 256;
      pub const UniquesMetadataDepositBase: Balance = 10 * MILLIUNIT;
      pub const AttributeDepositBase: Balance = 10 * MILLIUNIT;
      pub const DepositPerByte: Balance = MILLIUNIT;
      pub const UniquesStringLimit: u32 = 128;
}

impl pallet_uniques::Config for Runtime {
    type Event = Event;
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

parameter_types! {
    pub const MinVestedTransfer: Balance = UNIT;
    pub const MaxVestingSchedules: u32 = 50u32;
}

parameter_types! {
      pub InvarchAccounts: Vec<AccountId> = vec![
          // Tinkernet Root Account (i53Pqi67ocj66W81cJNrUvjjoM3RcAsGhXVTzREs5BRfwLnd7)
          hex_literal::hex!["f430c3461d19cded0bb3195af29d2b0379a96836c714ceb8e64d3f10902cec55"].into(),
          // Tinkernet Rewards Account (i4zTcKHr38MbSUrhFLVKHG5iULhYttBVrqVon2rv6iWcxQwQQ)
          hex_literal::hex!["725bf57f1243bf4b06e911a79eb954d1fe1003f697ef5db9640e64d6e30f9a42"].into(),
          // Tinkernet Treasury Pallet Account
          TreasuryPalletId::get().into_account_truncating(),
      ];
}

pub struct EnsureInvarchAccount;
impl EnsureOrigin<Origin> for EnsureInvarchAccount {
    type Success = AccountId;

    fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
        Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
            RawOrigin::Signed(caller) => {
                if InvarchAccounts::get().contains(&caller) {
                    Ok(caller)
                } else {
                    Err(Origin::from(Some(caller)))
                }
            }
            r => Err(Origin::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> Origin {
        let zero_account_id =
            AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
                .expect("infinite length input; no invalid inputs for type; qed");
        Origin::from(RawOrigin::Signed(zero_account_id))
    }
}

impl orml_vesting::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type MinVestedTransfer = MinVestedTransfer;
    type VestedTransferOrigin = EnsureInvarchAccount;
    type WeightInfo = ();
    type MaxVestingSchedules = MaxVestingSchedules;
    type BlockNumberProvider = System;
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
        RuntimeBlockWeights::get().max_block;
    // Retry a scheduled item every 25 blocks (5 minute) until the preimage exists.
    pub const NoPreimagePostponement: Option<u32> = Some(5 * MINUTES);
    pub const MaxScheduledPerBlock: u32 = 50u32;
}

impl pallet_scheduler::Config for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = ();
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type PreimageProvider = Preimage;
    type NoPreimagePostponement = NoPreimagePostponement;
}

parameter_types! {
    // Max size 4MB allowed: 4096 * 1024
    pub const PreimageMaxSize: u32 = 4096 * 1024;
      pub const PreimageBaseDeposit: Balance = deposit(2, 64);
      pub const PreimageByteDeposit: Balance = deposit(0, 1);
}

impl pallet_preimage::Config for Runtime {
    type WeightInfo = ();
    type Event = Event;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type MaxSize = PreimageMaxSize;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
}

parameter_types! {
    pub BasicDeposit: Balance = 5 * UNIT;
    pub FieldDeposit: Balance = 2 * UNIT;
    pub const MaxAdditionalFields: u32 = 5;
    pub const MaxRegistrars: u32 = 10;
    pub const MaxSubAccounts: u32 = 10;
    pub SubAccountDeposit: Balance = 5 * UNIT;
}

impl pallet_identity::Config for Runtime {
    type BasicDeposit = BasicDeposit;
    type Currency = Balances;
    type Event = Event;
    type FieldDeposit = FieldDeposit;
    type ForceOrigin = EnsureRoot<AccountId>;
    type MaxAdditionalFields = MaxAdditionalFields;
    type MaxRegistrars = MaxRegistrars;
    type MaxSubAccounts = MaxSubAccounts;
    type RegistrarOrigin = EnsureRoot<AccountId>;
    type Slashed = Treasury;
    type SubAccountDeposit = SubAccountDeposit;
    type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
      pub DepositBase: Balance = deposit(1, 88);
      pub DepositFactor: Balance = deposit(0, 32);
      pub const MaxSignatories: u16 = 50;
}

impl pallet_multisig::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = ();
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // System support stuff
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        Utility: pallet_utility::{Pallet, Call, Event} = 1,
        ParachainSystem: cumulus_pallet_parachain_system::{
            Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
        } = 2,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
        ParachainInfo: parachain_info::{Pallet, Storage, Config} = 4,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 5,
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 6,
        MaintenanceMode: pallet_maintenance_mode::{Pallet, Call, Config, Storage, Event} = 7,

        // Monetary stuff
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 11,
            Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 12,

        // Collator support. The order of there 4 are important and shale not change.
        Authorship: pallet_authorship::{Pallet, Call, Storage } = 20,
        CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
        Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,
        AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 24,

        // XCM helpers
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 30,
        PolkadotXcm: pallet_xcm::{Pallet, Event<T>, Origin, Config, Call} = 31,
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 32,
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 33,

        // FRAME
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 40,
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 41,
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 42,
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 43,

        // InvArch stuff
        Ipf: ipf::{Pallet, Call, Storage, Event<T>} = 70,
        INV4: inv4::{Pallet, Call, Storage, Event<T>} = 71,
        CheckedInflation: pallet_checked_inflation::{Pallet, Storage, Event<T>, Call} = 75,
        OcifStaking: pallet_ocif_staking::{Pallet, Call, Storage, Event<T>} = 76,

        Uniques: pallet_uniques::{Pallet, Storage, Event<T>} = 80,
        RmrkCore: pallet_rmrk_core::{Pallet, Call, Event<T>, Storage} = 81,
        RmrkEquip: pallet_rmrk_equip::{Pallet, Call, Event<T>, Storage} = 82,

        OrmlXcm: orml_xcm = 90,
        Vesting: orml_vesting::{Pallet, Storage, Call, Event<T>, Config<T>} = 91,
        XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 92,
        UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 93,
        AssetRegistry: orml_asset_registry::{Pallet, Call, Config<T>, Storage, Event<T>} = 94,
        Currencies: orml_currencies::{Pallet, Call} = 95,
        Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>, Config<T>} = 96,
    }
);

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_session, SessionBench::<Runtime>]
        [pallet_timestamp, Timestamp]
        [pallet_collator_selection, CollatorSelection]
        [cumulus_pallet_xcmp_queue, XcmpQueue]
    );
}

impl_runtime_apis! {
    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities().into_inner()
        }
    }

    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
            ParachainSystem::collect_collation_info(header)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade() -> (Weight, Weight) {
            log::info!("try-runtime::on_runtime_upgrade.");
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade().map_err(|err|{
                log::info!("try-runtime::on_runtime_upgrade failed with: {:?}", err);
                err
            }).unwrap();
            (weight, RuntimeBlockWeights::get().max_block)
        }

        fn execute_block_no_check(block: Block) -> Weight {
            Executive::execute_block_no_check(block)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

            let mut list = Vec::<BenchmarkList>::new();

            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();

            return (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey};

            use frame_system_benchmarking::Pallet as SystemBench;
            impl frame_system_benchmarking::Config for Runtime {}

            use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
            impl cumulus_pallet_session_benchmarking::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmarks!(params, batches);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
    fn check_inherents(
        block: &Block,
        relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
    ) -> sp_inherents::CheckInherentsResult {
        let relay_chain_slot = relay_state_proof
            .read_slot()
            .expect("Could not read the relay chain slot from the proof");

        let inherent_data =
            cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
                relay_chain_slot,
                sp_std::time::Duration::from_secs(6),
            )
            .create_inherent_data()
            .expect("Could not create the timestamp inherent data");

        inherent_data.check_extrinsics(block)
    }
}

cumulus_pallet_parachain_system::register_validate_block! {
    Runtime = Runtime,
    BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
    CheckInherents = CheckInherents,
}
