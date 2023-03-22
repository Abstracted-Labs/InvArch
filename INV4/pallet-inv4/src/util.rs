use crate::Config;
use codec::{Decode, Encode};
use frame_support::traits::Get;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::TrailingZeroInput;

/// Generates an `AccountId` using an `IpId` as the seed + the PalletId as salt.
pub fn derive_core_account<T: Config, CoreId, AccountId: Decode>(core_id: CoreId) -> AccountId
where
    (T::Hash, CoreId): Encode,
{
    let entropy = (
        //frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero()),
        T::GenesisHash::get(),
        core_id,
    )
        .using_encoded(blake2_256);

    Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
        .expect("infinite length input; no invalid inputs for type; qed")
}
