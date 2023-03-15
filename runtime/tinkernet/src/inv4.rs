use crate::{
    common_types::CommonId, constants::currency::UNIT, AccountId, Balance, Balances, CoreAssets,
    DealWithFees, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin,
};
use codec::{Decode, Encode};
use frame_support::{parameter_types, traits::AsEnsureOriginWithArg};
use frame_system::{EnsureNever, EnsureRoot, RawOrigin};
use pallet_transaction_payment::ChargeTransactionPayment;
use scale_info::TypeInfo;
use sp_runtime::traits::{SignedExtension, Zero};
use sp_std::vec::Vec;

parameter_types! {
    pub const MaxMetadata: u32 = 10000;
    pub const MaxCallers: u32 = 10000;
    pub const CoreSeedBalance: Balance = 1000000u128;
    pub const CoreCreationFee: Balance = UNIT * 100;
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
    type CoreCreationFee = CoreCreationFee;
    type CreationFeeHandler = DealWithFees;
    type FeeCharger = FeeCharger;
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo, Debug)]
pub struct FeeCharger;

impl Default for FeeCharger {
    fn default() -> Self {
        Self
    }
}

impl SignedExtension for FeeCharger {
    const IDENTIFIER: &'static str = ChargeTransactionPayment::<Runtime>::IDENTIFIER;
    type AccountId = <ChargeTransactionPayment<Runtime> as SignedExtension>::AccountId;
    type AdditionalSigned =
        <ChargeTransactionPayment<Runtime> as SignedExtension>::AdditionalSigned;
    type Call = <ChargeTransactionPayment<Runtime> as SignedExtension>::Call;
    type Pre = <ChargeTransactionPayment<Runtime> as SignedExtension>::Pre;

    fn additional_signed(
        &self,
    ) -> Result<Self::AdditionalSigned, frame_support::unsigned::TransactionValidityError> {
        ChargeTransactionPayment::<Runtime>::from(Zero::zero()).additional_signed()
    }

    fn metadata() -> Vec<sp_runtime::traits::SignedExtensionMetadata> {
        ChargeTransactionPayment::<Runtime>::metadata()
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, frame_support::unsigned::TransactionValidityError> {
        ChargeTransactionPayment::<Runtime>::from(Zero::zero()).pre_dispatch(who, call, info, len)
    }

    fn post_dispatch(
        pre: Option<Self::Pre>,
        info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
        post_info: &sp_runtime::traits::PostDispatchInfoOf<Self::Call>,
        len: usize,
        result: &sp_runtime::DispatchResult,
    ) -> Result<(), frame_support::unsigned::TransactionValidityError> {
        ChargeTransactionPayment::<Runtime>::post_dispatch(pre, info, post_info, len, result)
    }

    fn pre_dispatch_unsigned(
        call: &Self::Call,
        info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<(), frame_support::unsigned::TransactionValidityError> {
        ChargeTransactionPayment::<Runtime>::pre_dispatch_unsigned(call, info, len)
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> sp_api::TransactionValidity {
        ChargeTransactionPayment::<Runtime>::from(Zero::zero()).validate(who, call, info, len)
    }

    fn validate_unsigned(
        call: &Self::Call,
        info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> sp_api::TransactionValidity {
        ChargeTransactionPayment::<Runtime>::validate_unsigned(call, info, len)
    }
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
        CoreAssets::freeze_asset(
            RawOrigin::Signed(pallet_inv4::util::derive_core_account::<
                Runtime,
                CommonId,
                AccountId,
            >(asset_id))
            .into(),
            asset_id,
        )
    }

    fn thaw_asset(asset_id: CommonId) -> frame_support::dispatch::DispatchResult {
        CoreAssets::thaw_asset(
            RawOrigin::Signed(pallet_inv4::util::derive_core_account::<
                Runtime,
                CommonId,
                AccountId,
            >(asset_id))
            .into(),
            asset_id,
        )
    }
}
