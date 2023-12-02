use crate::{
    fee_handling::{FeeAsset, MultisigFeeHandler},
    origin::{INV4Origin, MultisigInternalOrigin},
    Config, Error,
};
use frame_support::{
    dispatch::{Dispatchable, GetDispatchInfo},
    pallet_prelude::*,
};

pub fn dispatch_call<T: Config>(
    core_id: <T as Config>::CoreId,
    fee_asset: &FeeAsset,
    call: <T as Config>::RuntimeCall,
) -> DispatchResultWithPostInfo
where
    T::AccountId: From<[u8; 32]>,
{
    let internal_origin = MultisigInternalOrigin::new(core_id);
    let multisig_account = internal_origin.to_account_id();
    let origin = INV4Origin::Multisig(internal_origin).into();

    let info = call.get_dispatch_info();
    let len = call.encode().len();

    let pre = <T::FeeCharger as MultisigFeeHandler<T>>::pre_dispatch(
        fee_asset,
        &multisig_account,
        &call,
        &info,
        len,
    )
    .map_err(|_| Error::<T>::CallFeePaymentFailed)?;

    let dispatch_result = call.dispatch(origin);

    let post = match dispatch_result {
        Ok(p) => p,
        Err(e) => e.post_info,
    };

    <T::FeeCharger as MultisigFeeHandler<T>>::post_dispatch(
        fee_asset,
        Some(pre),
        &info,
        &post,
        len,
        &dispatch_result.map(|_| ()).map_err(|e| e.error),
    )
    .map_err(|_| Error::<T>::CallFeePaymentFailed)?;

    dispatch_result
}
