use crate::Config;
use frame_support::{dispatch::Dispatchable, pallet_prelude::*};

pub fn dispatch_call<T: Config>(
    ips_id: <T as Config>::IpId,
    original_caller: Option<<T as frame_system::Config>::AccountId>,
    call: <T as Config>::Call,
) -> DispatchResultWithPostInfo {
    let origin = call.dispatch_as((ips_id, original_caller));

    call.dispatch(origin)
}

pub trait DispatchAs<Origin, Id> {
    fn dispatch_as(&self, id: Id) -> Origin;
}
