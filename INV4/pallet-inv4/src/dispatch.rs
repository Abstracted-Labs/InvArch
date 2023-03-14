use crate::{
    origin::{INV4Origin, MultisigInternalOrigin},
    Config,
};
use frame_support::{dispatch::Dispatchable, pallet_prelude::*};

pub fn dispatch_call<T: Config>(
    core_id: <T as Config>::CoreId,
    call: <T as Config>::RuntimeCall,
) -> DispatchResultWithPostInfo {
    let origin = INV4Origin::Multisig(MultisigInternalOrigin::new(core_id)).into();

    call.dispatch(origin)
}
