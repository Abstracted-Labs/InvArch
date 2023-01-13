use crate::{
    origin::{INV4Origin, MultisigInternalOrigin},
    Config,
};
use frame_support::{dispatch::Dispatchable, pallet_prelude::*};

pub fn dispatch_call<T: Config>(
    ips_id: <T as Config>::IpId,
    original_caller: Option<<T as frame_system::Config>::AccountId>,
    call: <T as Config>::Call,
) -> DispatchResultWithPostInfo {
    let origin =
        INV4Origin::Multisig(MultisigInternalOrigin::new((ips_id, original_caller))).into();

    call.dispatch(origin)
}
