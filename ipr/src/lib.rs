//! # Pallet IPR
//! Intellectual Property Replicas
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//! This pallet demonstrates how to produce a noted, tracked, & authorized copy of a IP File or a NFT featuring a standard that is interoperable & composable with the INV4 Protocol.
//!
//! ### Pallet Functions
//!
//! - `create` - Create a new IP Replica
//! - `delete` - Delete an IP Replica

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, Member, One};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + ipf::Config {
        /// The IPR Pallet Events
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The IPR ID type
        type IprId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;

        /// The maximum size of an IPS's metadata
        type MaxIprMetadata: Get<u32>;

        #[pallet::constant]
        type ExistentialDeposit: Get<<Self as pallet_assets::Config>::Balance>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as FSCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type IprIndexOf<T> = <T as Config>::IprId;

    pub type IprMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxIprMetadata>;

    pub type IprInfoOf<T> = IprInfo<
        <T as frame_system::Config>::AccountId,
        Vec<<T as ipf::Config>::IpfId>,
        IprMetadataOf<T>,
    >;

    pub type GenesisIpr<T> = (
        <T as frame_system::Config>::AccountId, // IPR owner
        Vec<u8>,                                // IPR metadata
        Vec<<T as ipf::Config>::IpfId>,         // IPR data
        Vec<ipf::GenesisIpfData<T>>,            // Vector of IPFs belong to this IPR
    );

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Next available IPR ID.
    #[pallet::storage]
    #[pallet::getter(fn next_ipr_id)]
    pub type NextIprId<T: Config> = StorageValue<_, T::IprId, ValueQuery>;

    /// Store IPR info
    ///
    /// Return `None` if IPR info not set of removed
    #[pallet::storage]
    #[pallet::getter(fn ipr_storage)]
    pub type IprStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::IprId, IprInfoOf<T>>;

    /// IPR existence check by owner and IPR ID
    #[pallet::storage]
    #[pallet::getter(fn ipr_by_owner)]
    pub type IprByOwner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // owner
        Blake2_128Concat,
        T::IprId,
        (),
    >;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        Created(T::AccountId, T::IprId),
        Deleted(T::AccountId, T::IprId),
    }

    /// Errors for IPR pallet
    #[pallet::error]
    pub enum Error<T> {
        /// No available IPR ID
        NoAvailableIprId,
        /// No available IPF ID
        NoAvailableIpfId,
        /// IPF (IprId, IpfId) not found
        IpfNotFound,
        /// IPR not found
        IprNotFound,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
        /// Can not destroy IPR
        CannotDestroyIpr,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create IP (Intellectual Property) Replica (IPR)
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn create_ipr(
            owner: OriginFor<T>,
            metadata: Vec<u8>,
            data: Vec<<T as ipf::Config>::IpfId>,
        ) -> DispatchResultWithPostInfo {
            NextIprId::<T>::try_mutate(|ipr_id| -> DispatchResultWithPostInfo {
                let creator = ensure_signed(owner.clone())?;

                let bounded_metadata: BoundedVec<u8, T::MaxIprMetadata> = metadata
                    .try_into()
                    .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

                let current_id = *ipr_id;
                *ipr_id = ipr_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableIprId)?;

                let info = IprInfo {
                    owner: ipr_account.clone(),
                    metadata: bounded_metadata,
                    data,
                };

                IprStorage::<T>::insert(current_id, info);
                IprByOwner::<T>::insert(ipr_account.clone(), current_id, ());

                Self::deposit_event(Event::Created(ipr_account, current_id));

                Ok(().into())
            })
        }

        /// Delete IP (Intellectual Property) Replica (IPR)
        #[pallet::weight(100_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn delete_ipr(owner: OriginFor<T>, ipr_id: T::IprId) -> DispatchResultWithPostInfo {
            IprStorage::<T>::try_mutate_exists(ipr_id, |ipr_info| -> DispatchResult {
                let owner = ensure_signed(owner)?;

                let info = ipr_info.take().ok_or(Error::<T>::IprNotFound)?;
                ensure!(info.owner == owner, Error::<T>::NoPermission);

                IprByOwner::<T>::remove(owner.clone(), ipr_id);

                Self::deposit_event(Event::Deleted(owner, ipr_id));

                Ok(())
            })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}
