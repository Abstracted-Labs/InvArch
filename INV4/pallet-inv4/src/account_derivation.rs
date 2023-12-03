use crate::{Config, Pallet};
use codec::{Compact, Encode};
use sp_io::hashing::blake2_256;
use xcm::latest::{BodyId, BodyPart, Junction, Junctions};

pub trait CoreAccountDerivation<T: Config> {
    fn derive_core_account(core_id: T::CoreId) -> T::AccountId;
    fn core_location(core_id: T::CoreId) -> Junctions;
}

impl<T: Config> CoreAccountDerivation<T> for Pallet<T>
where
    T::AccountId: From<[u8; 32]>,
{
    fn derive_core_account(core_id: T::CoreId) -> T::AccountId {
        (
            b"GlobalConsensus",
            T::GLOBAL_NETWORK_ID,
            b"Parachain",
            Compact::<u32>::from(T::PARA_ID),
            (b"Body", BodyId::Index(core_id.into()), BodyPart::Voice).encode(),
        )
            .using_encoded(blake2_256)
            .into()
    }

    fn core_location(core_id: T::CoreId) -> Junctions {
        Junctions::X3(
            Junction::GlobalConsensus(T::GLOBAL_NETWORK_ID),
            Junction::Parachain(T::PARA_ID),
            Junction::Plurality {
                id: BodyId::Index(core_id.into()),
                part: BodyPart::Voice,
            },
        )
    }
}
