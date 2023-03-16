use crate::{
    common_types::CommonId, constants::currency::UNIT, AccountId, Balance, Balances, CoreAssets,
    DealWithFees, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin,
};
use codec::{Decode, Encode};
use frame_support::{parameter_types, traits::AsEnsureOriginWithArg};
use frame_system::{EnsureNever, EnsureRoot, RawOrigin};
use pallet_inv4::fee_handling::MultisigFeeHandler;
use pallet_transaction_payment::ChargeTransactionPayment;
use scale_info::TypeInfo;
use sp_runtime::traits::{SignedExtension, Zero};

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
    type WeightInfo = pallet_inv4::weights::SubstrateWeight<Runtime>;
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo, Debug)]
pub struct FeeCharger;

impl MultisigFeeHandler for FeeCharger {
    type AccountId = AccountId;
    type Call = RuntimeCall;
    type Pre = <ChargeTransactionPayment<Runtime> as SignedExtension>::Pre;

    fn pre_dispatch(
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
