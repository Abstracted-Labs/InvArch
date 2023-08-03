use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    traits::{fungibles::Credit, Currency},
    unsigned::TransactionValidityError,
};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{DispatchInfoOf, PostDispatchInfoOf},
    DispatchResult,
};

#[derive(Clone, TypeInfo, Encode, Decode, MaxEncodedLen, Debug, PartialEq, Eq)]
pub enum FeeAsset {
    TNKR,
    KSM,
}

pub enum FeeAssetNegativeImbalance<TNKRNegativeImbalance, KSMNegativeImbalance> {
    TNKR(TNKRNegativeImbalance),
    KSM(KSMNegativeImbalance),
}

pub trait MultisigFeeHandler<T: Config> {
    type Pre;

    fn pre_dispatch(
        asset: &FeeAsset,
        who: &T::AccountId,
        call: &<T as Config>::RuntimeCall,
        info: &DispatchInfoOf<<T as Config>::RuntimeCall>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError>;

    fn post_dispatch(
        asset: &FeeAsset,
        pre: Option<Self::Pre>,
        info: &DispatchInfoOf<<T as Config>::RuntimeCall>,
        post_info: &PostDispatchInfoOf<<T as Config>::RuntimeCall>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError>;

    fn handle_creation_fee(
        imbalance: FeeAssetNegativeImbalance<
            <T::Currency as Currency<T::AccountId>>::NegativeImbalance,
            Credit<T::AccountId, T::Tokens>,
        >,
    );
}
