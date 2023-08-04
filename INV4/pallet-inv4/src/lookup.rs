use crate::{Config, CoreByAccount, CoreStorage, Pallet};
use core::marker::PhantomData;
use frame_support::error::LookupError;
use sp_runtime::{traits::StaticLookup, MultiAddress};

impl<T: Config> Pallet<T> {
    pub fn lookup_core(core_id: T::CoreId) -> Option<T::AccountId> {
        CoreStorage::<T>::get(core_id).map(|core| core.account)
    }

    pub fn lookup_address(a: MultiAddress<T::AccountId, T::CoreId>) -> Option<T::AccountId> {
        match a {
            MultiAddress::Id(i) => Some(i),
            MultiAddress::Index(i) => Self::lookup_core(i),
            _ => None,
        }
    }
}

pub struct INV4Lookup<T: Config>(PhantomData<T>);

impl<T: Config> StaticLookup for INV4Lookup<T> {
    type Source = MultiAddress<T::AccountId, T::CoreId>;
    type Target = T::AccountId;

    fn lookup(a: Self::Source) -> Result<Self::Target, LookupError> {
        Pallet::<T>::lookup_address(a).ok_or(LookupError)
    }

    fn unlookup(a: Self::Target) -> Self::Source {
        match CoreByAccount::<T>::get(&a) {
            Some(core_id) => MultiAddress::Index(core_id),
            None => MultiAddress::Id(a),
        }
    }
}
