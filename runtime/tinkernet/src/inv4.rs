use crate::{
    common_types::CommonId,
    constants::currency::UNIT,
    xcm_config::{CheckingAccount, LocationToAccountId, RelayOrSiblingToAccountId},
    AccountId, Balance, Balances, CoreAssets, DealWithFees, Runtime, RuntimeCall, RuntimeEvent,
    RuntimeOrigin,
};
use codec::{Decode, Encode};
use frame_support::{
    parameter_types,
    traits::{
        fungibles::{Inspect, Mutate, MutateHold, Transfer, Unbalanced},
        Contains,
    },
};
use pallet_inv4::fee_handling::MultisigFeeHandler;
use pallet_transaction_payment::ChargeTransactionPayment;
use scale_info::TypeInfo;
use sp_core::{ConstU32, H256};
use sp_runtime::traits::{One, SignedExtension, Zero};
use xcm::latest::{
    AssetId::Concrete, Error as XcmError, Fungibility, Junction, Junctions, MultiAsset,
    MultiLocation, Result as XcmResult,
};
use xcm_builder::FungiblesMutateAdapter;
use xcm_executor::{
    traits::{Convert, Error as XcmExecutorError, MatchesFungibles, TransactAsset},
    Assets,
};

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
    type OnNewTokenAccount = ();
    type OnKilledTokenAccount = ();
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

pub type CoreAssetId = <Runtime as orml_tokens2::Config>::CurrencyId;

pub struct CoreAssetConvert;

impl Convert<CoreAssetId, MultiLocation> for CoreAssetConvert {
    fn convert(id: CoreAssetId) -> Result<MultiLocation, CoreAssetId> {
        Ok(MultiLocation::new(
            1,
            Junctions::X3(
                Junction::Parachain(2125),
                Junction::PalletInstance(72),
                Junction::GeneralIndex(id.into()),
            ),
        ))
    }
}

impl Convert<MultiLocation, CoreAssetId> for CoreAssetConvert {
    fn convert(location: MultiLocation) -> Result<CoreAssetId, MultiLocation> {
        match location {
            MultiLocation {
                parents: 0,
                interior: Junctions::X2(Junction::PalletInstance(72), Junction::GeneralIndex(index)),
            } => index.try_into().map_err(|_| location),

            MultiLocation {
                parents: 1,
                interior:
                    Junctions::X3(
                        Junction::Parachain(2125),
                        Junction::PalletInstance(72),
                        Junction::GeneralIndex(index),
                    ),
            } => index.try_into().map_err(|_| location),

            _ => Err(location),
        }
    }
}

impl Convert<MultiAsset, CoreAssetId> for CoreAssetConvert {
    fn convert(asset: MultiAsset) -> Result<CoreAssetId, MultiAsset> {
        if let MultiAsset {
            id: Concrete(ref location),
            ..
        } = asset
        {
            <Self as Convert<MultiLocation, CoreAssetId>>::convert(location.clone())
                .map_err(|_| asset)
        } else {
            Err(asset)
        }
    }
}

pub struct CheckCoreAssets;

impl Contains<CoreAssetId> for CheckCoreAssets {
    fn contains(t: &CoreAssetId) -> bool {
        CoreAssets::asset_exists(*t)
    }
}

pub struct ConvertCoreAssetBalance;

impl Convert<u128, Balance> for ConvertCoreAssetBalance {
    fn convert_ref(value: impl core::borrow::Borrow<u128>) -> Result<Balance, ()> {
        Ok(*value.borrow())
    }
}

pub struct MatchCoreAsset;

impl MatchesFungibles<CoreAssetId, <Runtime as orml_tokens2::Config>::Balance> for MatchCoreAsset {
    fn matches_fungibles(
        a: &MultiAsset,
    ) -> Result<(CoreAssetId, <Runtime as orml_tokens2::Config>::Balance), XcmExecutorError> {
        let (amount, id) = match (&a.fun, &a.id) {
            (Fungibility::Fungible(amount), Concrete(ref id)) => (amount, id),
            _ => return Err(XcmExecutorError::AssetNotFound),
        };

        let what: CoreAssetId =
            <CoreAssetConvert as Convert<MultiLocation, CoreAssetId>>::convert_ref(id)
                .map_err(|_| XcmExecutorError::AssetIdConversionFailed)?;

        Ok((what, *amount))
    }
}

pub struct CoreAssetsAdapter;

impl TransactAsset for CoreAssetsAdapter {
    fn can_check_in(origin: &MultiLocation, what: &MultiAsset) -> XcmResult {
        FungiblesMutateAdapter::<
            CoreAssets,
            MatchCoreAsset,
            LocationToAccountId,
            AccountId,
            CheckCoreAssets,
            CheckingAccount,
        >::can_check_in(origin, what)
    }

    fn check_in(origin: &MultiLocation, what: &MultiAsset) {
        FungiblesMutateAdapter::<
            CoreAssets,
            MatchCoreAsset,
            LocationToAccountId,
            AccountId,
            CheckCoreAssets,
            CheckingAccount,
        >::check_in(origin, what)
    }

    fn check_out(dest: &MultiLocation, what: &MultiAsset) {
        FungiblesMutateAdapter::<
            CoreAssets,
            MatchCoreAsset,
            LocationToAccountId,
            AccountId,
            CheckCoreAssets,
            CheckingAccount,
        >::check_out(dest, what)
    }

    fn deposit_asset(what: &MultiAsset, who: &MultiLocation) -> XcmResult {
        let (asset_id, amount) = MatchCoreAsset::matches_fungibles(what)?;

        let (who, is_relay_or_sibling) = match RelayOrSiblingToAccountId::convert(who.clone()) {
            Ok(account_id) => (account_id, true),
            Err(multilocation) => (
                LocationToAccountId::convert_ref(multilocation)
                    .map_err(|()| XcmExecutorError::AccountIdConversionFailed)?,
                false,
            ),
        };

        CoreAssets::mint_into(asset_id, &who, amount)
            .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

        if is_relay_or_sibling {
            CoreAssets::hold(asset_id, &who, amount)
                .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

            CoreAssets::set_total_issuance(asset_id, CoreAssets::total_issuance(asset_id) - amount);
        }

        Ok(())
    }

    fn withdraw_asset(what: &MultiAsset, who: &MultiLocation) -> Result<Assets, XcmError> {
        let (asset_id, amount) = MatchCoreAsset::matches_fungibles(what)?;

        let who = match RelayOrSiblingToAccountId::convert(who.clone()) {
            Ok(account_id) => {
                CoreAssets::release(asset_id, &account_id, amount, false)
                    .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

                CoreAssets::set_total_issuance(
                    asset_id,
                    CoreAssets::total_issuance(asset_id) + amount,
                );

                account_id
            }
            Err(multilocation) => LocationToAccountId::convert_ref(multilocation)
                .map_err(|()| XcmExecutorError::AccountIdConversionFailed)?,
        };

        CoreAssets::burn_from(asset_id, &who, amount)
            .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

        Ok(what.clone().into())
    }

    fn internal_transfer_asset(
        what: &MultiAsset,
        from: &MultiLocation,
        to: &MultiLocation,
    ) -> Result<Assets, XcmError> {
        let (asset_id, amount) = MatchCoreAsset::matches_fungibles(what)?;

        let source = match RelayOrSiblingToAccountId::convert(from.clone()) {
            Ok(account_id) => {
                CoreAssets::release(asset_id, &account_id, amount, false)
                    .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

                CoreAssets::set_total_issuance(
                    asset_id,
                    CoreAssets::total_issuance(asset_id) + amount,
                );

                account_id
            }
            Err(multilocation) => LocationToAccountId::convert_ref(multilocation)
                .map_err(|()| XcmExecutorError::AccountIdConversionFailed)?,
        };

        let (dest, is_dest_relay_or_sibling) = match RelayOrSiblingToAccountId::convert(to.clone())
        {
            Ok(account_id) => (account_id, true),
            Err(multilocation) => (
                LocationToAccountId::convert_ref(multilocation)
                    .map_err(|()| XcmExecutorError::AccountIdConversionFailed)?,
                false,
            ),
        };

        <CoreAssets as Transfer<AccountId>>::transfer(asset_id, &source, &dest, amount, true)
            .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

        if is_dest_relay_or_sibling {
            CoreAssets::hold(asset_id, &dest, amount)
                .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

            CoreAssets::set_total_issuance(asset_id, CoreAssets::total_issuance(asset_id) - amount);
        }

        Ok(what.clone().into())
    }
}
