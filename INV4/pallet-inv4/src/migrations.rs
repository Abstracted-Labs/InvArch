use super::*;
use frame_support::{
    dispatch::GetStorageVersion,
    pallet_prelude::*,
    traits::{Get, OnRuntimeUpgrade},
    weights::Weight,
};
use log::{info, warn};

pub mod v1 {
    use core::convert::TryInto;
    use frame_support::BoundedVec;
    use primitives::{CoreInfo, OneOrPercent};
    use rmrk_traits::collection::Collection;
    use rmrk_traits::nft::Nft;
    use rmrk_traits::primitives::CollectionId;
    use rmrk_traits::ResourceInfoMin;
    use sp_core::H256;
    use sp_std::vec;
    use sp_std::vec::Vec;

    use super::*;

    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
    pub enum Parentage<AccountId, IpsId> {
        Parent(AccountId),
        Child(IpsId, AccountId),
    }

    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
    enum IpsType<IpsId> {
        Normal,
        Replica(IpsId),
    }

    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
    struct IpInfo<AccountId, Data, IpsMetadataOf, IpId, Balance, LicenseMetadata, Hash> {
        pub parentage: Parentage<AccountId, IpId>,
        pub metadata: IpsMetadataOf,
        pub data: Data,
        pub ips_type: IpsType<IpId>,
        pub allow_replica: bool,
        pub supply: Balance,
        pub license: (LicenseMetadata, Hash),
        pub execution_threshold: OneOrPercent,
        pub default_asset_weight: OneOrPercent,
        pub default_permission: bool,
    }

    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
    enum AnyId<IpsId, IpfId, RmrkNftTuple, RmrkCollectionId> {
        IpfId(IpfId),
        RmrkNft(RmrkNftTuple),
        RmrkCollection(RmrkCollectionId),
        IpsId(IpsId),
    }

    type AnyIdOf = AnyId<u32, u64, (u32, u32), u32>;

    type IpsMetadataOf = BoundedVec<u8, ConstU32<10000>>;

    type IpInfoOf<T> = IpInfo<
        <T as frame_system::Config>::AccountId,
        BoundedVec<AnyIdOf, ConstU32<10000>>,
        IpsMetadataOf,
        u32,
        u128,
        BoundedVec<u8, ConstU32<10000>>,
        <T as frame_system::Config>::Hash,
    >;

    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
    pub struct IpfInfo<AccountId, Data, IpfMetadataOf> {
        /// IPF owner
        pub owner: AccountId,
        /// Original IPF author
        pub author: AccountId,
        /// IPF metadata
        pub metadata: IpfMetadataOf,
        /// IPF data
        pub data: Data,
    }

    type IpfMetadataOf = BoundedVec<u8, ConstU32<10000>>;
    type IpfInfoOf<T> = IpfInfo<
        <T as frame_system::Config>::AccountId,
        <T as frame_system::Config>::Hash,
        IpfMetadataOf,
    >;

    pub fn migrate_ip_storage_to_core_storage<
        T: Config + pallet_rmrk_core::Config + pallet_uniques::Config,
    >()
    where
        u32: Into<T::CoreId>
            + Into<CollectionId>
            + Into<<T as pallet_uniques::Config>::CollectionId>
            + Into<<T as pallet_uniques::Config>::ItemId>,

        <T as frame_system::Config>::Hash: IsType<H256>,

        [u8; 32]: Into<T::AccountId>,
    {
        let total_ips = frame_support::migration::storage_key_iter::<
            u32,
            IpInfoOf<T>,
            Blake2_128Concat,
        >(b"INV4", b"IpStorage")
        .count();

        let total_ipf = frame_support::migration::storage_key_iter::<
            u64,
            IpfInfoOf<T>,
            Blake2_128Concat,
        >(b"Ipf", b"IpfStorage")
        .count();

        info!("Attempting to migrate {} IPS into Cores.", total_ips);
        info!("Attempting to migrate {} IPF into RMRK NFTs.", total_ipf);

        let mut ips_migrated = 0;
        let mut ipf_migrated = 0;

        frame_support::migration::storage_key_iter::<u32, IpInfoOf<T>, Blake2_128Concat>(
            b"INV4",
            b"IpStorage",
        )
        .for_each(|(ips_id, ips)| {
            if let Parentage::Parent(account) = ips.parentage {
                CoreStorage::<T>::insert(
                    Into::<T::CoreId>::into(ips_id),
                    CoreInfo {
                        account: account.clone(),
                        metadata: ips
                            .metadata
                            .into_inner()
                            .try_into()
                            .expect("IPS metadata should always fit in Core metadata."),
                        execution_threshold: ips.execution_threshold,
                        default_asset_weight: ips.default_asset_weight,
                        default_permission: ips.default_permission,
                    },
                );

                let symbol = {
                    let mut c = "Core".encode();
                    c.append(&mut ips_id.encode());
                    c
                };

                pallet_rmrk_core::Pallet::<T>::collection_create(
                    account.clone(),
                    ips_id.into(),
                    BoundedVec::default(),
                    None,
                    symbol
                        .try_into()
                        .expect("Collection symbol is always below max."),
                )
                .expect("Creating the collection should always succeed.");

                ips.data.into_iter().enumerate().for_each(|(i, any_id)| {
                    if let AnyId::IpfId(ipf_id) = any_id {
                        if let Some(ipf) = frame_support::migration::take_storage_item::<
                            u64,
                            IpfInfoOf<T>,
                            Blake2_128Concat,
                        >(b"Ipf", b"IpfStorage", ipf_id)
                        {
                            pallet_rmrk_core::Pallet::<T>::nft_mint(
                                account.clone(),
                                account.clone(),
                                (i as u32).into(),
                                ips_id.into(),
                                None,
                                None,
                                ipf.metadata.to_vec().try_into().expect("IPF metadata should always fit in RMRK NFT metadata."),
                                true,
                                Some(
                                    vec![ResourceInfoMin {
                                        id: ipf_id as u32,
                                        resource: rmrk_traits::ResourceTypes::Basic(
                                            rmrk_traits::BasicResource {
                                                metadata: ipf
                                                    .data
                                                    .into()
                                                    .as_bytes()
                                                    .to_vec()
                                                    .try_into()
                                                    .expect("IPF data should always fit in RMRK Resource metadata."),
                                            },
                                        ),
                                    }]
                                    .try_into()
                                    .expect("Resources vec with a single item should always fit in RMRK Core resource bounded vec."),
                                ),
                            )
                            .expect("Minting the NFT should always succeed.");

                            ipf_migrated += 1;
                        }
                    }
                });

                ips_migrated += 1;
            }
        });

        let next_id =
            frame_support::migration::get_storage_value::<u32>(b"INV4", b"NextIpId", &[]).unwrap();

        pallet_rmrk_core::Pallet::<T>::collection_create(
            [
                2u8, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 2,
            ]
            .into(),
            next_id.into(),
            BoundedVec::default(),
            None,
            b"MIGR"
                .to_vec()
                .try_into()
                .expect("Collection symbol is always below max."),
        )
        .expect("Creating the collection should always succeed.");

        frame_support::migration::storage_key_iter::<
                u64,
            IpfInfoOf<T>,
            Blake2_128Concat,
            >(b"Ipf", b"IpfStorage").enumerate().for_each(|(i, (ipf_id, ipf))| {
                pallet_rmrk_core::Pallet::<T>::nft_mint(
                    [
                        2u8, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                        2, 2, 2, 2, 2,
                    ]
                        .into(),
                    [
                        2u8, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                        2, 2, 2, 2, 2,
                    ]
                        .into(),
                                (i as u32).into(),
                                next_id.into(),
                                None,
                                None,
                                ipf.metadata.to_vec().try_into().expect("IPF metadata should always fit in RMRK NFT metadata."),
                                true,
                                Some(
                                    vec![ResourceInfoMin {
                                        id: ipf_id as u32,
                                        resource: rmrk_traits::ResourceTypes::Basic(
                                            rmrk_traits::BasicResource {
                                                metadata: ipf
                                                    .data
                                                    .into()
                                                    .as_bytes()
                                                    .to_vec()
                                                    .try_into()
                                                    .expect("IPF data should always fit in RMRK Resource metadata."),
                                            },
                                        ),
                                    }]
                                    .try_into()
                                    .expect("Resources vec with a single item should always fit in RMRK Core resource bounded vec."),
                                ),
                            )
                            .expect("Minting the NFT should always succeed.");

                            ipf_migrated += 1;
            });

        info!("Migrated {} IPS into Cores.", ips_migrated);
        info!(
            "Extra check: {} Cores, {} NFT Collections",
            CoreStorage::<T>::iter().count(),
            pallet_rmrk_core::Collections::<T>::iter().count()
        );

        info!("Migrated {} IPF into RMRK NFTs.", ipf_migrated);
        info!(
            "Extra check: {}",
            pallet_rmrk_core::Nfts::<T>::iter().count()
        );
    }

    pub fn migrate_ip_owner_to_core_account<T: Config>()
    where
        u32: Into<T::CoreId>,
    {
        let mut ips_migrated = 0;

        frame_support::migration::storage_key_iter::<(T::AccountId, u32), (), Blake2_128Concat>(
            b"INV4",
            b"IpsByOwner",
        )
        .for_each(|((account, ips_id), _)| {
            CoreByAccount::<T>::insert(account, ips_id.into());

            ips_migrated += 1;
        });

        info!(
            "Migrated {} IPS accounts into CoreByAccount storage.",
            ips_migrated
        );
        info!("Extra check: {}", CoreByAccount::<T>::iter().count());
    }

    pub fn migrate_next_id<T: Config>()
    where
        u32: Into<T::CoreId>,
    {
        let next_id =
            frame_support::migration::take_storage_value::<u32>(b"INV4", b"NextIpId", &[]).unwrap();

        NextCoreId::<T>::put(next_id.into());

        info!("Migrated NextIpId {} into NextCoreId.", next_id);
        info!("Extra check: {}", NextCoreId::<T>::get());
    }

    pub fn migrate_balance_and_total_issuance<T: Config>()
    where
        u32: Into<T::CoreId>,
    {
        let entries = frame_support::migration::storage_key_iter::<
            (u32, Option<u32>, T::AccountId),
            BalanceOf<T>,
            Blake2_128Concat,
        >(b"INV4", b"Balance")
        .count();

        info!(
            "Attempting to migrate {} entries from INV4.Balance storage.",
            entries
        );

        let mut migrated = 0;

        frame_support::migration::storage_key_iter::<
            (u32, Option<u32>, T::AccountId),
            BalanceOf<T>,
            Blake2_128Concat,
        >(b"INV4", b"Balance")
        .for_each(|((ips_id, token, account), balance)| {
            Balances::<T>::insert::<(T::CoreId, Option<T::CoreId>, T::AccountId), BalanceOf<T>>(
                (ips_id.into(), token.map(|x| x.into()), account),
                balance,
            );
            TotalIssuance::<T>::mutate(ips_id.into(), token.map(|x| x.into()), |issuance| {
                *issuance += balance;
            });

            migrated += 1;
        });

        info!("Migrated {} entries from Balance to Balances.", migrated);
        info!("Extra check: {}", Balances::<T>::iter_keys().count());
    }

    pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
    impl<T: Config + pallet_rmrk_core::Config + pallet_uniques::Config> OnRuntimeUpgrade
        for MigrateToV1<T>
    where
        u32: Into<T::CoreId>
            + Into<CollectionId>
            + Into<<T as pallet_uniques::Config>::CollectionId>
            + Into<<T as pallet_uniques::Config>::ItemId>,

        <T as frame_system::Config>::Hash: IsType<H256>,

        [u8; 32]: Into<T::AccountId>,
    {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
            frame_support::ensure!(
                Pallet::<T>::current_storage_version() == 0,
                "Required v0 before upgrading to v1"
            );

            Ok(Default::default())
        }

        fn on_runtime_upgrade() -> Weight {
            let current = Pallet::<T>::current_storage_version();

            if current == 1 {
                migrate_ip_storage_to_core_storage::<T>();
                migrate_ip_owner_to_core_account::<T>();
                migrate_next_id::<T>();
                migrate_balance_and_total_issuance::<T>();

                current.put::<Pallet<T>>();

                info!("v1 applied successfully");
                T::DbWeight::get().reads_writes(10, 10)
            } else {
                warn!("Skipping v1, should be removed");
                T::DbWeight::get().reads(1)
            }
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
            frame_support::ensure!(
                Pallet::<T>::on_chain_storage_version() == 1,
                "v1 not applied"
            );

            Ok(())
        }
    }
}
