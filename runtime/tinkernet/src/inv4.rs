use crate::{
    common_types::CommonId, constants::currency::UNIT, AccountId, Balance, Balances, CoreAssets,
    DealWithFees, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin,
};
use codec::{Decode, Encode};
use frame_support::{parameter_types, traits::Contains};
use pallet_asset_tx_payment::{ChargeAssetTxPayment, InitialPayment};
use pallet_inv4::fee_handling::{FeeAsset, MultisigFeeHandler};
use pallet_transaction_payment::ChargeTransactionPayment;
use scale_info::TypeInfo;
use sp_core::{ConstU32, H256};
use sp_runtime::traits::{One, SignedExtension, Zero};

parameter_types! {
    pub const MaxMetadata: u32 = 10000;
    pub const MaxCallers: u32 = 10000;
    pub const CoreSeedBalance: Balance = 1000000u128;
    pub const CoreCreationFee: Balance = UNIT * 100;
    pub const GenesisHash: <Runtime as frame_system::Config>::Hash = H256([
        212, 46, 150, 6, 169, 149, 223, 228, 51, 220, 121, 85, 220, 42, 112, 244, 149, 243, 80,
        243, 115, 218, 162, 0, 9, 138, 232, 68, 55, 129, 106, 210,
    ]);
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
    // type AssetFreezer = AssetFreezer;
    type CoreCreationFee = CoreCreationFee;
    type CreationFeeHandler = DealWithFees;
    type FeeCharger = FeeCharger;
    type GenesisHash = GenesisHash;
    type WeightInfo = pallet_inv4::weights::SubstrateWeight<Runtime>;
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo, Debug)]
pub struct FeeCharger;

impl MultisigFeeHandler for FeeCharger {
    type AccountId = AccountId;
    type Call = RuntimeCall;
    type Pre = (
        u128,
        AccountId,
        Option<pallet_balances::NegativeImbalance<Runtime>>,
        InitialPayment<Runtime>,
        Option<u32>,
    );

    fn pre_dispatch(
        fee_asset: &FeeAsset,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, frame_support::unsigned::TransactionValidityError> {
        match fee_asset {
            FeeAsset::TNKR => ChargeTransactionPayment::<Runtime>::from(Zero::zero())
                .pre_dispatch(who, call, info, len)
                .map(|(x, y, z)| (x, y, z, InitialPayment::Nothing, None)),

            FeeAsset::KSM => ChargeAssetTxPayment::<Runtime>::from(Zero::zero(), 1.into())
                .pre_dispatch(who, call, info, len)
                .map(|(x, y, z, a)| (x, y, None, z, a)),
        }
    }

    fn post_dispatch(
        fee_asset: &FeeAsset,
        pre: Option<Self::Pre>,
        info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
        post_info: &sp_runtime::traits::PostDispatchInfoOf<Self::Call>,
        len: usize,
        result: &sp_runtime::DispatchResult,
    ) -> Result<(), frame_support::unsigned::TransactionValidityError> {
        match fee_asset {
            FeeAsset::TNKR => ChargeTransactionPayment::<Runtime>::post_dispatch(
                pre.map(|(x, y, z, _, _)| (x, y, z)),
                info,
                post_info,
                len,
                result,
            ),
            FeeAsset::KSM => ChargeAssetTxPayment::<Runtime>::post_dispatch(
                pre.map(|(x, y, _, z, a)| (x, y, z, a)),
                info,
                post_info,
                len,
                result,
            ),
        }
    }
}

orml_traits2::parameter_type_with_key! {
    pub CoreExistentialDeposits: |_currency_id: <Runtime as pallet_inv4::Config>::CoreId| -> Balance {
        Balance::one()
    };
}

pub struct CoreDustRemovalWhitelist;
impl Contains<AccountId> for CoreDustRemovalWhitelist {
    fn contains(_: &AccountId) -> bool {
        true
    }
}

pub struct DisallowIfFrozen;
impl
    orml_traits2::currency::OnTransfer<AccountId, <Runtime as pallet_inv4::Config>::CoreId, Balance>
    for DisallowIfFrozen
{
    fn on_transfer(
        currency_id: <Runtime as pallet_inv4::Config>::CoreId,
        _from: &AccountId,
        _to: &AccountId,
        _amount: Balance,
    ) -> sp_runtime::DispatchResult {
        if let Some(true) = crate::INV4::is_asset_frozen(currency_id) {
            Err(sp_runtime::DispatchError::Token(
                sp_runtime::TokenError::Frozen,
            ))
        } else {
            Ok(())
        }
    }
}

pub struct HandleNewMembers;
impl orml_traits2::Happened<(AccountId, <Runtime as pallet_inv4::Config>::CoreId)>
    for HandleNewMembers
{
    fn happened((member, core_id): &(AccountId, <Runtime as pallet_inv4::Config>::CoreId)) {
        crate::INV4::add_member(core_id, member)
    }
}

pub struct HandleRemovedMembers;
impl orml_traits2::Happened<(AccountId, <Runtime as pallet_inv4::Config>::CoreId)>
    for HandleRemovedMembers
{
    fn happened((member, core_id): &(AccountId, <Runtime as pallet_inv4::Config>::CoreId)) {
        crate::INV4::remove_member(core_id, member)
    }
}

pub struct INV4TokenHooks;
impl
    orml_traits2::currency::MutationHooks<
        AccountId,
        <Runtime as pallet_inv4::Config>::CoreId,
        Balance,
    > for INV4TokenHooks
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

impl orml_tokens2::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type Amount = i128;
    type CurrencyId = <Runtime as pallet_inv4::Config>::CoreId;
    type WeightInfo = ();
    type ExistentialDeposits = CoreExistentialDeposits;
    type MaxLocks = ConstU32<0u32>;
    type MaxReserves = ConstU32<0u32>;
    type DustRemovalWhitelist = CoreDustRemovalWhitelist;
    type ReserveIdentifier = [u8; 8];
    type CurrencyHooks = INV4TokenHooks;
}
