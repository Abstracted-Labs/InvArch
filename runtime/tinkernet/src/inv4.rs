use crate::{
    assets::{RelayAssetId, KSM_ASSET_ID},
    common_types::{AssetId, CommonId},
    constants::currency::UNIT,
    fee_handling::DealWithKSMFees,
    AccountId, Balance, Balances, CoreAssets, DealWithFees, Runtime, RuntimeCall, RuntimeEvent,
    RuntimeOrigin, Tokens,
};
use codec::{Decode, Encode};
use frame_support::{
    parameter_types,
    traits::{fungibles::Credit, Currency, OnUnbalanced},
};
use pallet_asset_tx_payment::ChargeAssetTxPayment;
use pallet_inv4::fee_handling::{FeeAsset, FeeAssetNegativeImbalance, MultisigFeeHandler};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::traits::{SignedExtension, Zero};

parameter_types! {
    pub const MaxMetadata: u32 = 10000;
    pub const MaxCallers: u32 = 10000;
    pub const CoreSeedBalance: Balance = 1000000u128;
    pub const CoreCreationFee: Balance = UNIT * 100;
    pub const GenesisHash: <Runtime as frame_system::Config>::Hash = H256([
        212, 46, 150, 6, 169, 149, 223, 228, 51, 220, 121, 85, 220, 42, 112, 244, 149, 243, 80,
        243, 115, 218, 162, 0, 9, 138, 232, 68, 55, 129, 106, 210,
    ]);

    pub const KSMCoreCreationFee: Balance = UNIT;
    pub const MaxCallSize: u32 = 50 * 1024;
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
    type CoreCreationFee = CoreCreationFee;
    type FeeCharger = FeeCharger;
    type GenesisHash = GenesisHash;
    type WeightInfo = pallet_inv4::weights::SubstrateWeight<Runtime>;

    type Tokens = Tokens;
    type KSMAssetId = RelayAssetId;
    type KSMCoreCreationFee = KSMCoreCreationFee;

    type MaxCallSize = MaxCallSize;
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo, Debug)]
pub struct FeeCharger;

impl MultisigFeeHandler<Runtime> for FeeCharger {
    type Pre = (
        // tip
        Balance,
        // who paid the fee
        AccountId,
        // imbalance resulting from withdrawing the fee
        pallet_asset_tx_payment::InitialPayment<Runtime>,
        // asset_id for the transaction payment
        Option<AssetId>,
    );

    fn pre_dispatch(
        fee_asset: &FeeAsset,
        who: &AccountId,
        call: &RuntimeCall,
        info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
        len: usize,
    ) -> Result<Self::Pre, frame_support::unsigned::TransactionValidityError> {
        match fee_asset {
            FeeAsset::TNKR => ChargeAssetTxPayment::<Runtime>::from(Zero::zero(), None)
                .pre_dispatch(who, call, info, len),

            FeeAsset::KSM => {
                ChargeAssetTxPayment::<Runtime>::from(Zero::zero(), Some(KSM_ASSET_ID))
                    .pre_dispatch(who, call, info, len)
            }
        }
    }

    fn post_dispatch(
        fee_asset: &FeeAsset,
        pre: Option<Self::Pre>,
        info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
        post_info: &sp_runtime::traits::PostDispatchInfoOf<RuntimeCall>,
        len: usize,
        result: &sp_runtime::DispatchResult,
    ) -> Result<(), frame_support::unsigned::TransactionValidityError> {
        match fee_asset {
            FeeAsset::TNKR => {
                ChargeAssetTxPayment::<Runtime>::post_dispatch(pre, info, post_info, len, result)
            }

            FeeAsset::KSM => {
                ChargeAssetTxPayment::<Runtime>::post_dispatch(pre, info, post_info, len, result)
            }
        }
    }

    fn handle_creation_fee(
        imbalance: FeeAssetNegativeImbalance<
            <Balances as Currency<AccountId>>::NegativeImbalance,
            Credit<AccountId, Tokens>,
        >,
    ) {
        match imbalance {
            FeeAssetNegativeImbalance::TNKR(imb) => DealWithFees::on_unbalanced(imb),

            FeeAssetNegativeImbalance::KSM(imb) => DealWithKSMFees::on_unbalanced(imb),
        }
    }
}

// pub struct DisallowIfFrozen;
// impl
//     orml_tokens_fork::OnTransfer<
//         pallet_inv4::multisig::MultisigMemberOf<Runtime>,
//         <Runtime as pallet_inv4::Config>::CoreId,
//         Balance,
//     > for DisallowIfFrozen
// {
//     fn on_transfer(
//         currency_id: <Runtime as pallet_inv4::Config>::CoreId,
//         _from: &pallet_inv4::multisig::MultisigMemberOf<Runtime>,
//         _to: &pallet_inv4::multisig::MultisigMemberOf<Runtime>,
//         _amount: Balance,
//     ) -> sp_runtime::DispatchResult {
//         if let Some(true) = crate::INV4::is_asset_frozen(currency_id) {
//             Err(sp_runtime::DispatchError::Token(
//                 sp_runtime::TokenError::Frozen,
//             ))
//         } else {
//             Ok(())
//         }
//     }
// }

// pub struct HandleNewMembers;
// impl
//     orml_tokens_fork::Happened<(
//         pallet_inv4::multisig::MultisigMemberOf<Runtime>,
//         <Runtime as pallet_inv4::Config>::CoreId,
//     )> for HandleNewMembers
// {
//     fn happened(
//         (member, core_id): &(
//             pallet_inv4::multisig::MultisigMemberOf<Runtime>,
//             <Runtime as pallet_inv4::Config>::CoreId,
//         ),
//     ) {
//         crate::INV4::add_member(core_id, member)
//     }
// }

// pub struct HandleRemovedMembers;
// impl
//     orml_tokens_fork::Happened<(
//         pallet_inv4::multisig::MultisigMemberOf<Runtime>,
//         <Runtime as pallet_inv4::Config>::CoreId,
//     )> for HandleRemovedMembers
// {
//     fn happened(
//         (member, core_id): &(
//             pallet_inv4::multisig::MultisigMemberOf<Runtime>,
//             <Runtime as pallet_inv4::Config>::CoreId,
//         ),
//     ) {
//         crate::INV4::remove_member(core_id, member)
//     }
// }

// pub struct INV4TokenHooks;
// impl
//     orml_tokens_fork::MutationHooks<
//         pallet_inv4::multisig::MultisigMemberOf<Runtime>,
//         <Runtime as pallet_inv4::Config>::CoreId,
//         Balance,
//     > for INV4TokenHooks
// {
//     type PreTransfer = DisallowIfFrozen;
//     type OnDust = ();
//     type OnSlash = ();
//     type PreDeposit = ();
//     type PostDeposit = ();
//     type PostTransfer = ();
//     type OnNewTokenAccount = HandleNewMembers;
//     type OnKilledTokenAccount = HandleRemovedMembers;
// }

impl pallet_core_assets::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type CurrencyId = <Runtime as pallet_inv4::Config>::CoreId;
    type WeightInfo = ();

    type AccountId = pallet_inv4::multisig::MultisigMemberOf<Runtime>;
    type Lookup =
        sp_runtime::traits::IdentityLookup<pallet_inv4::multisig::MultisigMemberOf<Runtime>>;
}
