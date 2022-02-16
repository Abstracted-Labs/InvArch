#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum Parentage<AccountId, IpsId> {
    Parent(AccountId),
    Child(IpsId),
}

/// IPS info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub struct IpsInfo<AccountId, Data, IpsMetadataOf, IpsId> {
    /// IPS parentage
    pub parentage: Parentage<AccountId, IpsId>,
    /// IPS metadata
    pub metadata: IpsMetadataOf,
    /// IPS Properties
    pub data: Data,
}

/// IPT Info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub struct IpfInfo<AccountId, Data, IpfMetadataOf> {
    /// IPT owner
    pub owner: AccountId,
    /// IPT metadata
    pub metadata: IpfMetadataOf,
    /// IPT data
    pub data: Data,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub enum AnyId<IpsId, IpfId> {
    IpsId(IpsId),
    IpfId(IpfId),
}

pub mod utils {
    use codec::{Decode, Encode};
    use sp_io::hashing::blake2_256;

    pub fn multi_account_id<T: frame_system::Config, IpsId: Encode>(
        ips_id: IpsId,
    ) -> <T as frame_system::Config>::AccountId {
        let entropy = (b"modlpy/utilisuba", ips_id).using_encoded(blake2_256);
        T::AccountId::decode(&mut &entropy[..]).unwrap() // TODO: Remove unwrap.
    }
}
