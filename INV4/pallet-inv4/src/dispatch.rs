use crate::{origin::MultisigInternalOrigin, util::derive_ips_account, Config};
use frame_support::{
    dispatch::{Dispatchable, RawOrigin},
    pallet_prelude::*,
    traits::Contains,
};

pub fn dispatch_call<T: Config>(
    ips_id: <T as Config>::IpId,
    original_caller: Option<<T as frame_system::Config>::AccountId>,
    call: <T as Config>::Call,
) -> DispatchResultWithPostInfo {
    let origin = if <T as Config>::DispatchAsMultisigWhen::contains(&call) {
        super::Origin::<T>::Multisig(MultisigInternalOrigin {
            id: ips_id,
            original_caller,
        })
        .into()
    } else {
        RawOrigin::Signed(derive_ips_account::<T>(ips_id, original_caller.as_ref())).into()
    };

    call.dispatch(origin)
}
