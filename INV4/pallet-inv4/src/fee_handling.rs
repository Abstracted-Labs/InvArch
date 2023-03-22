use frame_support::{dispatch::Dispatchable, unsigned::TransactionValidityError};
use sp_runtime::{
    traits::{DispatchInfoOf, PostDispatchInfoOf},
    DispatchResult,
};

pub trait MultisigFeeHandler {
    type Pre;
    type AccountId;
    type Call: Dispatchable;

    fn pre_dispatch(
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError>;

    fn post_dispatch(
        pre: Option<Self::Pre>,
        info: &DispatchInfoOf<Self::Call>,
        post_info: &PostDispatchInfoOf<Self::Call>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError>;
}
