//! Core Account Derivation.
//!
//! ## Overview
//!
//! This module defines a method for generating account addresses, and how it's implemented within this
//! pallet. We use a custom derivation scheme to ensure that when a multisig is created, its AccountId
//! remains consistent across different parachains, promoting seamless interaction.
//!
//! ### The module contains:
//! - `CoreAccountDerivation` trait: The interface for our derivation method.
//! - Pallet implementation: The specific logic used to derive AccountIds.

use crate::{Config, Pallet};
use codec::{Compact, Encode};
use frame_support::traits::Get;
use sp_io::hashing::blake2_256;
use xcm::latest::{BodyId, BodyPart, Junction, Junctions};
/// Trait providing the XCM location and the derived account of a core.
pub trait CoreAccountDerivation<T: Config> {
    /// Derives the core's AccountId.
    fn derive_core_account(core_id: T::CoreId) -> T::AccountId;
    /// Specifies a core's location.
    fn core_location(core_id: T::CoreId) -> Junctions;
}

impl<T: Config> CoreAccountDerivation<T> for Pallet<T>
where
    T::AccountId: From<[u8; 32]>,
{
    /// HashedDescription of the core location from the perspective of a sibling chain.
    /// This derivation allows the local account address to match the account address in other parachains.
    /// Reference: https://github.com/paritytech/polkadot-sdk/blob/master/polkadot/xcm/xcm-builder/src/location_conversion.rs
    fn derive_core_account(core_id: T::CoreId) -> T::AccountId {
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
    /// Core location is defined as a plurality within the parachain.
    fn core_location(core_id: T::CoreId) -> Junctions {
        Junctions::X2(
            Junction::Parachain(T::ParaId::get()),
            Junction::Plurality {
                id: BodyId::Index(core_id.into()),
                part: BodyPart::Voice,
            },
        )
    }
}
