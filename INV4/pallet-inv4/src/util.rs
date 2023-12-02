use crate::{Config, Pallet};
use codec::{Compact, Decode, Encode};
use frame_support::traits::Get;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::TrailingZeroInput;
use xcm::latest::{BodyId, BodyPart, Junction, Junctions};

/// Generates an `AccountId` using an `IpId` as the seed + the PalletId as salt.
pub fn derive_core_account_old<T: Config, CoreId, AccountId: Decode>(core_id: CoreId) -> AccountId
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

pub trait CoreAccountConversion<T: Config> {
    fn derive_core_account(core_id: T::CoreId) -> T::AccountId;
    fn core_location(core_id: T::CoreId) -> Junctions;
}

impl<T: Config> CoreAccountConversion<T> for Pallet<T>
where
    T::AccountId: From<[u8; 32]>,
{
    fn derive_core_account(core_id: T::CoreId) -> T::AccountId {
        (
            b"GlobalConsensus",
            T::GlobalNetworkId::get(),
            b"Parachain",
            Compact::<u32>::from(T::ParaId::get()),
            (b"Body", BodyId::Index(core_id.into()), BodyPart::Voice).encode(),
        )
            .using_encoded(blake2_256)
            .into()
    }

    fn core_location(core_id: T::CoreId) -> Junctions {
        Junctions::X3(
            Junction::GlobalConsensus(T::GlobalNetworkId::get()),
            Junction::Parachain(T::ParaId::get()),
            Junction::Plurality {
                id: BodyId::Index(core_id.into()),
                part: BodyPart::Voice,
            },
        )
    }
}
