use crate::{
    common_types::AssetId,
    constants::{StakingPotAccount, TreasuryAccount},
    AccountId, Balance, Balances, DealWithFees, ExtrinsicBaseWeight, Runtime, RuntimeBlockWeights,
    RuntimeCall, Weight,
};
use codec::{Decode, Encode};
use frame_support::{
    dispatch::{DispatchClass, Pays},
    parameter_types,
    traits::{
        fungible::Inspect as FungibleInspect,
        tokens::{WithdrawConsequence, WithdrawReasons},
        Contains, Currency, ExistenceRequirement, Imbalance, OnUnbalanced,
    },
    weights::{
        WeightToFee, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
};
use orml_tokens::{CurrencyAdapter, NegativeImbalance as OrmlNegativeImbalance};
use pallet_transaction_payment::{FeeDetails, InclusionFee};
use scale_info::TypeInfo;
use smallvec::smallvec;
use sp_core::ConstU128;
use sp_runtime::{
    traits::{DispatchInfoOf, PostDispatchInfoOf, SignedExtension, Zero},
    transaction_validity::{
        InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
    },
    DispatchResult, FixedPointNumber, Perbill,
};

#[derive(Default)]
pub enum InitialPayment {
    /// No initial fee was payed.
    #[default]
    Nothing,
    /// The initial fee was payed in the native currency.
    Native(Option<<Balances as Currency<AccountId>>::NegativeImbalance>),
    /// The initial fee was payed in an asset.
    Asset(OrmlNegativeImbalance<Runtime, GetKSM>),
}
// TODO: Turn this into a pallet.
// TODO: Write RPC interface to query fee info.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub struct ChargeNativeOrRelayToken {
    #[codec(compact)]
    tip: Balance,
    asset_id: Option<AssetId>,
}

fn compute_ksm_fee(
    len: u32,
    weight: Weight,
    tip: Balance,
    pays_fee: Pays,
    class: DispatchClass,
) -> FeeDetails<Balance> {
    if pays_fee == Pays::Yes {
        // the adjustable part of the fee.
        let unadjusted_weight_fee = KusamaWeightToFee::weight_to_fee(&weight);
        let multiplier = pallet_transaction_payment::Pallet::<Runtime>::next_fee_multiplier();
        // final adjusted weight fee.
        let adjusted_weight_fee = multiplier.saturating_mul_int(unadjusted_weight_fee);

        // length fee. this is adjusted via `LengthToFee`.
        let len_fee = KusamaLengthToFee::weight_to_fee(&Weight::from_ref_time(len as u64));

        let base_fee =
            KusamaWeightToFee::weight_to_fee(&RuntimeBlockWeights::get().get(class).base_extrinsic);

        FeeDetails {
            inclusion_fee: Some(InclusionFee {
                base_fee,
                len_fee,
                adjusted_weight_fee,
            }),
            tip,
        }
    } else {
        FeeDetails {
            inclusion_fee: None,
            tip,
        }
    }
}

impl ChargeNativeOrRelayToken {
    /// Utility constructor. Used only in client/factory code.
    pub fn from(tip: Balance, asset_id: Option<AssetId>) -> Self {
        Self { tip, asset_id }
    }

    /// Fee withdrawal logic that dispatches to either `OnChargeAssetTransaction` or
    /// `OnChargeTransaction`.
    fn withdraw_fee(
        &self,
        who: &AccountId,
        call: &RuntimeCall,
        info: &DispatchInfoOf<RuntimeCall>,
        len: usize,
    ) -> Result<(Balance, InitialPayment), TransactionValidityError> {
        match self.asset_id {
            None => {
                let fee = pallet_transaction_payment::Pallet::<Runtime>::compute_fee(
                    len as u32, info, self.tip,
                );

                debug_assert!(
                    self.tip <= fee,
                    "tip should be included in the computed fee"
                );

                if fee.is_zero() {
                    Ok((fee, InitialPayment::Nothing))
                } else {
                    <pallet_transaction_payment::CurrencyAdapter::<Balances, DealWithFees> as pallet_transaction_payment::OnChargeTransaction<Runtime>>::withdraw_fee(
                        who, call, info, fee, self.tip,
                    )
                        .map(|i| (fee, InitialPayment::Native(i)))
                        .map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })
                }
            }
            Some(1u32) => {
                let fee =
                    compute_ksm_fee(len as u32, info.weight, self.tip, info.pays_fee, info.class)
                        .final_fee();

                debug_assert!(
                    self.tip <= fee,
                    "tip should be included in the computed fee"
                );

                if fee.is_zero() {
                    Ok((fee, InitialPayment::Nothing))
                } else {
                    FilteredTransactionCharger::withdraw_fee(who, call, info, 1u32, fee, self.tip)
                        .map(|i| (fee, InitialPayment::Asset(i)))
                }
            }
            _ => Err(InvalidTransaction::Payment.into()),
        }
    }
}

impl sp_std::fmt::Debug for ChargeNativeOrRelayToken {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(
            f,
            "ChargeNativeOrRelayToken<{:?}, {:?}>",
            self.tip,
            self.asset_id.encode()
        )
    }
    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl SignedExtension for ChargeNativeOrRelayToken {
    const IDENTIFIER: &'static str = "ChargeAssetTxPayment";
    type AccountId = AccountId;
    type Call = RuntimeCall;
    type AdditionalSigned = ();
    type Pre = (
        // tip
        Balance,
        // who paid the fee
        Self::AccountId,
        // imbalance resulting from withdrawing the fee
        InitialPayment,
        // asset_id for the transaction payment
        Option<AssetId>,
    );

    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> TransactionValidity {
        use pallet_transaction_payment::ChargeTransactionPayment;
        let (fee, _) = self.withdraw_fee(who, call, info, len)?;
        let priority = ChargeTransactionPayment::<Runtime>::get_priority(info, len, self.tip, fee);
        Ok(ValidTransaction {
            priority,
            ..Default::default()
        })
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        let (_fee, initial_payment) = self.withdraw_fee(who, call, info, len)?;
        Ok((self.tip, who.clone(), initial_payment, self.asset_id))
    }

    fn post_dispatch(
        pre: Option<Self::Pre>,
        info: &DispatchInfoOf<Self::Call>,
        post_info: &PostDispatchInfoOf<Self::Call>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        if let Some((tip, who, initial_payment, _)) = pre {
            match initial_payment {
                InitialPayment::Native(already_withdrawn) => {
                    pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::post_dispatch(
                        Some((tip, who, already_withdrawn)),
                        info,
                        post_info,
                        len,
                        result,
                    )?;
                }
                InitialPayment::Asset(already_withdrawn) => {
                    let actual_fee = compute_ksm_fee(
                        len as u32,
                        post_info.calc_actual_weight(info),
                        tip,
                        post_info.pays_fee(info),
                        info.class,
                    )
                    .final_fee();

                    FilteredTransactionCharger::correct_and_deposit_fee(
                        &who,
                        info,
                        post_info,
                        actual_fee,
                        tip,
                        already_withdrawn,
                    )?;
                }
                InitialPayment::Nothing => {
                    // `actual_fee` should be zero here for any signed extrinsic. It would be
                    // non-zero here in case of unsigned extrinsics as they don't pay fees but
                    // `compute_actual_fee` is not aware of them. In both cases it's fine to just
                    // move ahead without adjusting the fee, though, so we do nothing.
                    debug_assert!(tip.is_zero(), "tip should be zero if initial fee was zero.");
                }
            }
        }

        Ok(())
    }
}

pub struct KusamaWeightToFee;
impl WeightToFeePolynomial for KusamaWeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        // in Kusama, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
        let p = (1_000_000_000_000 / 30) / 100;
        let q = 10 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}

pub type KusamaLengthToFee =
    frame_support::weights::ConstantMultiplier<Balance, ConstU128<10_000_000_000>>;

pub struct KSMEnabledPallets;
impl Contains<RuntimeCall> for KSMEnabledPallets {
    fn contains(t: &RuntimeCall) -> bool {
        matches!(t, RuntimeCall::INV4(_) | RuntimeCall::Rings(_))
    }
}

pub struct FilteredTransactionCharger;
impl FilteredTransactionCharger {
    fn withdraw_fee(
        who: &AccountId,
        call: &RuntimeCall,
        _dispatch_info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
        asset_id: AssetId,
        fee: Balance,
        _tip: Balance,
    ) -> Result<
        OrmlNegativeImbalance<Runtime, GetKSM>,
        frame_support::unsigned::TransactionValidityError,
    > {
        if KSMEnabledPallets::contains(call) && asset_id == 1 {
            let can_withdraw = CurrencyAdapter::<Runtime, GetKSM>::can_withdraw(who, fee);

            if !matches!(can_withdraw, WithdrawConsequence::Success) {
                return Err(InvalidTransaction::Payment.into());
            }

            CurrencyAdapter::<Runtime, GetKSM>::withdraw(
                who,
                fee,
                WithdrawReasons::FEE,
                ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| TransactionValidityError::from(InvalidTransaction::Payment))
        } else {
            Err(TransactionValidityError::from(InvalidTransaction::Payment))
        }
    }

    fn correct_and_deposit_fee(
        who: &AccountId,
        _dispatch_info: &DispatchInfoOf<RuntimeCall>,
        _post_info: &PostDispatchInfoOf<RuntimeCall>,
        corrected_fee: Balance,
        tip: Balance,
        paid: OrmlNegativeImbalance<Runtime, GetKSM>,
    ) -> Result<(), TransactionValidityError> {
        let refund_amount = paid.peek().saturating_sub(corrected_fee);

        let refund_imbalance = CurrencyAdapter::<Runtime, GetKSM>::deposit_into_existing(
            who,
            refund_amount,
        )
        .unwrap_or_else(|_| {
            <CurrencyAdapter<Runtime, GetKSM> as Currency<AccountId>>::PositiveImbalance::zero()
        });

        let adjusted_paid = paid
            .offset(refund_imbalance)
            .same()
            .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;

        let (tip, fee) = adjusted_paid.split(tip);

        DealWithKSMFees::on_unbalanceds(Some(fee).into_iter().chain(Some(tip)));

        Ok(())
    }
}

pub struct DealWithKSMFees;
impl OnUnbalanced<OrmlNegativeImbalance<Runtime, GetKSM>> for DealWithKSMFees {
    fn on_unbalanceds<B>(
        mut fees_then_tips: impl Iterator<Item = OrmlNegativeImbalance<Runtime, GetKSM>>,
    ) {
        if let Some(mut fees) = fees_then_tips.next() {
            if let Some(tips) = fees_then_tips.next() {
                // Merge with fee, for now we send everything to the treasury
                tips.merge_into(&mut fees);
            }

            Self::on_unbalanced(fees);
        }
    }

    fn on_unbalanced(amount: OrmlNegativeImbalance<Runtime, GetKSM>) {
        let (to_collators, to_treasury) = amount.ration(50, 50);

        CurrencyAdapter::<Runtime, GetKSM>::resolve_creating(&TreasuryAccount::get(), to_treasury);

        CurrencyAdapter::<Runtime, GetKSM>::resolve_creating(
            &StakingPotAccount::get(),
            to_collators,
        );
    }
}

parameter_types! {
    pub const GetKSM: AssetId = 1u32;
}
