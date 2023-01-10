use core::marker::PhantomData;

use crate::{
    pallet::{self, Origin},
    util::derive_ips_account,
    Config,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{error::BadOrigin, RuntimeDebug};
use scale_info::TypeInfo;
use sp_runtime::traits::AtLeast32BitUnsigned;

#[derive(PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Clone, RuntimeDebug)]
pub enum INV4Origin<
    T: pallet::Config,
    IpId: AtLeast32BitUnsigned + Encode,
    AccountId: Decode + Encode + Clone,
> {
    Multisig(MultisigInternalOrigin<T, IpId, AccountId>),
}

#[derive(PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Clone, RuntimeDebug)]
pub struct MultisigInternalOrigin<
    T: pallet::Config,
    IpId: AtLeast32BitUnsigned + Encode,
    AccountId: Decode + Encode + Clone,
> {
    pub id: IpId,
    pub original_caller: Option<AccountId>,
    t: PhantomData<T>,
}

impl<
        T: pallet::Config,
        IpId: AtLeast32BitUnsigned + Encode,
        AccountId: Decode + Encode + Clone,
    > MultisigInternalOrigin<T, IpId, AccountId>
{
    pub fn to_account_id(&self) -> AccountId {
        derive_ips_account::<T, IpId, AccountId>(self.id.clone(), self.original_caller.as_ref())
    }
}

pub fn ensure_multisig<T: Config, OuterOrigin>(
    o: OuterOrigin,
) -> Result<
    MultisigInternalOrigin<T, <T as pallet::Config>::IpId, <T as frame_system::Config>::AccountId>,
    BadOrigin,
>
where
    OuterOrigin: Into<Result<pallet::Origin<T>, OuterOrigin>>,
{
    match o.into() {
        Ok(Origin::<T>::Multisig(internal)) => Ok(internal),
        _ => Err(BadOrigin),
    }
}
