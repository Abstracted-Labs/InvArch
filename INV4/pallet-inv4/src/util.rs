use codec::{Decode, Encode};
use sp_arithmetic::traits::Zero;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::TrailingZeroInput;

/// Generates an `AccountId` using an `IpId` as the seed + the PalletId as salt.
pub fn derive_core_account<T: frame_system::Config, CoreId, AccountId: Decode>(
    core_id: CoreId,
) -> AccountId
where
    (T::Hash, CoreId): Encode,
{
    let entropy = (
        frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero()),
        core_id,
    )
        .using_encoded(blake2_256);

    Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
        .expect("infinite length input; no invalid inputs for type; qed")
}
