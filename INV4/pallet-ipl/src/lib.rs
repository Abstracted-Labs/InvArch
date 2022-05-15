#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::type_complexity)]

use frame_support::{pallet_prelude::*, traits::Currency as FSCurrency, Parameter};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, Member};

pub use pallet::*;

pub trait LicenseList {
    type IpfsHash: core::hash::Hash;
    type LicenseMetadata;

    fn get_hash_and_metadata(&self) -> (Self::LicenseMetadata, Self::IpfsHash);
}

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use core::iter::Sum;
    use primitives::OneOrPercent;
    use primitives::{utils::multi_account_id, IplInfo};
    use scale_info::prelude::fmt::Display;
    use sp_std::convert::TryInto;
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config {
        /// The IPL Pallet Events
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Currency
        type Currency: FSCurrency<Self::AccountId>;
        /// The units in which we record balances.
        type Balance: Member
            + Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + TypeInfo
            + Sum<<Self as pallet::Config>::Balance>
            + IsType<<Self as pallet_balances::Config>::Balance>;

        /// The IPL ID type
        type IplId: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + Display
            + MaxEncodedLen;

        type Licenses: Parameter
            + LicenseList<IpfsHash = <Self as frame_system::Config>::Hash, LicenseMetadata = Vec<u8>>;

        #[pallet::constant]
        type MaxLicenseMetadata: Get<u32>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as FSCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type IplInfoOf<T> = IplInfo<
        <T as frame_system::Config>::AccountId,
        <T as Config>::IplId,
        BoundedVec<u8, <T as Config>::MaxLicenseMetadata>,
        <T as frame_system::Config>::Hash,
    >;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn ipl_info)]
    /// Details of a multisig call.
    pub type Ipl<T: Config> = StorageMap<_, Blake2_128Concat, T::IplId, IplInfoOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn asset_weight_storage)]
    /// Details of a multisig call.
    pub type AssetWeight<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::IplId, Blake2_128Concat, T::IplId, OneOrPercent>;

    #[pallet::storage]
    #[pallet::getter(fn permissions)]
    /// Details of a multisig call.
    pub type Permissions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::IplId, T::IplId),
        Blake2_128Concat,
        [u8; 2],
        bool,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        PermissionSet(T::IplId, T::IplId, [u8; 2], bool),
        WeightSet(T::IplId, T::IplId, OneOrPercent),
    }

    /// Errors for IPF pallet
    #[pallet::error]
    pub enum Error<T> {
        IplDoesntExist,
        NoPermission,
        MaxMetadataExceeded,
    }

    /// Dispatch functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn set_permission(
            owner: OriginFor<T>,
            ipl_id: T::IplId,
            sub_asset: T::IplId,
            call_metadata: [u8; 2],
            permission: bool,
        ) -> DispatchResult {
            let owner = ensure_signed(owner)?;

            let ipl = Ipl::<T>::get(ipl_id).ok_or(Error::<T>::IplDoesntExist)?;

            ensure!(owner == ipl.owner, Error::<T>::NoPermission);

            Permissions::<T>::insert((ipl_id, sub_asset), call_metadata, permission);

            Self::deposit_event(Event::PermissionSet(
                ipl_id,
                sub_asset,
                call_metadata,
                permission,
            ));

            Ok(())
        }

        #[pallet::weight(100_000)] // TODO: Set correct weight
        pub fn set_asset_weight(
            owner: OriginFor<T>,
            ipl_id: T::IplId,
            sub_asset: T::IplId,
            asset_weight: OneOrPercent,
        ) -> DispatchResult {
            let owner = ensure_signed(owner)?;

            let ipl = Ipl::<T>::get(ipl_id).ok_or(Error::<T>::IplDoesntExist)?;

            ensure!(owner == ipl.owner, Error::<T>::NoPermission);

            AssetWeight::<T>::insert(ipl_id, sub_asset, asset_weight);

            Self::deposit_event(Event::WeightSet(ipl_id, sub_asset, asset_weight));

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    impl<T: Config> Pallet<T> {
        pub fn create(
            ipl_id: T::IplId,
            license: T::Licenses,
            execution_threshold: OneOrPercent,
            default_asset_weight: OneOrPercent,
            default_permission: bool,
        ) {
            Ipl::<T>::insert(
                ipl_id,
                IplInfo {
                    owner: multi_account_id::<T, T::IplId>(ipl_id, None),
                    id: ipl_id,
                    license: {
                        let (metadata, hash) = license.get_hash_and_metadata();
                        (
                            metadata
                                .try_into()
                                .map_err(|_| Error::<T>::MaxMetadataExceeded)
                                .unwrap(), // TODO: Remove unwrap.
                            hash,
                        )
                    },
                    execution_threshold,
                    default_asset_weight,
                    default_permission,
                },
            );
        }

        pub fn execution_threshold(ipl_id: T::IplId) -> Option<OneOrPercent> {
            Ipl::<T>::get(ipl_id).map(|ipl| ipl.execution_threshold)
        }

        pub fn asset_weight(ipl_id: T::IplId, sub_asset: T::IplId) -> Option<OneOrPercent> {
            AssetWeight::<T>::get(ipl_id, sub_asset)
                .or_else(|| Ipl::<T>::get(ipl_id).map(|ipl| ipl.default_asset_weight))
        }

        pub fn has_permission(
            ipl_id: T::IplId,
            sub_asset: T::IplId,
            call_metadata: [u8; 2],
        ) -> Option<bool> {
            Permissions::<T>::get((ipl_id, sub_asset), call_metadata)
                .or_else(|| Ipl::<T>::get(ipl_id).map(|ipl| ipl.default_permission))
        }
    }
}
