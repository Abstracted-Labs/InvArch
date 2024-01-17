use crate::{Config, Pallet};
use codec::{Compact, Encode};
use frame_support::traits::Get;
use sp_io::hashing::blake2_256;
use xcm::latest::{BodyId, BodyPart, Junction, Junctions};

// Trait providing the XCM location and the derived account of a core.
pub trait CoreAccountDerivation<T: Config> {
    fn derive_core_account(core_id: T::CoreId) -> T::AccountId;
    fn core_location(core_id: T::CoreId) -> Junctions;
}

impl<T: Config> CoreAccountDerivation<T> for Pallet<T>
where
    T::AccountId: From<[u8; 32]>,
{
    fn derive_core_account(core_id: T::CoreId) -> T::AccountId {
        // HashedDescription of the core location from the perspective of a sibling chain.
        // This derivation allows the local account address to match the account address in other parachains.
        // Reference: https://github.com/paritytech/polkadot-sdk/blob/master/polkadot/xcm/xcm-builder/src/location_conversion.rs
        blake2_256(
            &(
                b"SiblingChain",
                Compact::<u32>::from(T::ParaId::get()),
                (b"Body", BodyId::Index(core_id.into()), BodyPart::Voice).encode(),
            )
                .encode(),
        )
        .into()
    }

    fn core_location(core_id: T::CoreId) -> Junctions {
        // Core location is defined as a plurality within the parachain.
        Junctions::X2(
            Junction::Parachain(T::ParaId::get()),
            Junction::Plurality {
                id: BodyId::Index(core_id.into()),
                part: BodyPart::Voice,
            },
        )
    }
}
