#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};

/// IPS Id type
pub type IpsId = u64;
/// IPT Id type
pub type IptId = u64;

/// IPS info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen)]
pub struct IpsInfo<IptId, AccountId, Data, IpsMetadataOf> {
    // TODO: WIP
    /// IPS metadata
    pub metadata: IpsMetadataOf,
    /// Total issuance for the IPS
    pub total_issuance: IptId,
    /// IPS owner
    pub owner: AccountId,
    /// IPS Properties
    pub data: Data,
}

/// IPT Info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen)]
pub struct IptInfo<AccountId, Data, IptMetadataOf> {
    /// IPT owner
    pub owner: AccountId,
    /// IPT metadata
    pub metadata: IptMetadataOf,
    /// IPT data
    pub data: Data,
}
