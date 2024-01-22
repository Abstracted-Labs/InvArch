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

/// Asset to be used by the multisig for paying fees transaction fees.
#[derive(Clone, TypeInfo, Encode, Decode, MaxEncodedLen, Debug, PartialEq, Eq)]
pub enum FeeAsset {
    Native,
    Relay,
}

pub enum FeeAssetNegativeImbalance<NativeNegativeImbalance, RelayNegativeImbalance> {
    Native(NativeNegativeImbalance),
    Relay(RelayNegativeImbalance),
}

/// Fee handler trait.
/// This should be implemented properly in the runtime to account for native and non-native assets.
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
