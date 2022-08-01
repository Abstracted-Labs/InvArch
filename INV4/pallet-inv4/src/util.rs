use crate::Config;
use codec::{Decode, Encode};
use frame_support::traits::Get;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::TrailingZeroInput;

/// Generates an `AccountId` using an `IpId` as the seed + the PalletId as salt.
pub fn derive_ips_account<T: Config>(
    ips_id: T::IpId,
    original_caller: Option<T::AccountId>,
) -> T::AccountId {
    let entropy = if let Some(original_caller) = original_caller {
        (T::UniquePalletId::get(), ips_id, original_caller).using_encoded(blake2_256)
    } else {
        (T::UniquePalletId::get(), ips_id).using_encoded(blake2_256)
    };

    Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
        .expect("infinite length input; no invalid inputs for type; qed")
}
