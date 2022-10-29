use crate::{
    pallet::{self, Origin},
    Config,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{error::BadOrigin, RuntimeDebug};
use scale_info::TypeInfo;
use sp_runtime::traits::AtLeast32BitUnsigned;

#[derive(PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Clone, RuntimeDebug)]
pub enum INV4Origin<IpId: AtLeast32BitUnsigned, AccountId: Decode> {
    Multisig(MultisigInternalOrigin<IpId, AccountId>),
}

#[derive(PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Clone, RuntimeDebug)]
pub struct MultisigInternalOrigin<IpId: AtLeast32BitUnsigned, AccountId: Decode> {
    pub id: IpId,
    pub original_caller: Option<AccountId>,
}

pub fn ensure_multisig<T: Config, OuterOrigin>(
    o: OuterOrigin,
) -> Result<
    MultisigInternalOrigin<<T as pallet::Config>::IpId, <T as frame_system::Config>::AccountId>,
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
