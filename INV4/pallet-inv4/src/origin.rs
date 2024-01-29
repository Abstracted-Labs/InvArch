use crate::{
    account_derivation::CoreAccountDerivation,
    pallet::{self, Origin, Pallet},
    Config,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{error::BadOrigin, RuntimeDebug};
use scale_info::TypeInfo;

/// Origin representing a core by its id.
#[derive(PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Clone, RuntimeDebug)]
pub enum INV4Origin<T: pallet::Config> {
    Multisig(MultisigInternalOrigin<T>),
}

#[derive(PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Clone, RuntimeDebug)]
pub struct MultisigInternalOrigin<T: pallet::Config> {
    pub id: T::CoreId,
}

impl<T: pallet::Config> MultisigInternalOrigin<T>
where
    T::AccountId: From<[u8; 32]>,
{
    pub fn new(id: T::CoreId) -> Self {
        Self { id }
    }

    pub fn to_account_id(&self) -> T::AccountId {
        Pallet::<T>::derive_core_account(self.id)
    }
}

pub fn ensure_multisig<T: Config, OuterOrigin>(
    o: OuterOrigin,
) -> Result<MultisigInternalOrigin<T>, BadOrigin>
where
    OuterOrigin: Into<Result<pallet::Origin<T>, OuterOrigin>>,
{
    match o.into() {
        Ok(Origin::<T>::Multisig(internal)) => Ok(internal),
        _ => Err(BadOrigin),
    }
}
