#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode};
use scale_info::TypeInfo;

pub type CommonId = u64;

#[derive(Encode, Decode, TypeInfo)]
pub enum AnyId<IpsId, IpfId> {
    IpsId(IpsId),
    IpfId(IpfId),
}
