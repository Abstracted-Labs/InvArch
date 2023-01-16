#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_std::vec::Vec;

#[cfg(feature = "std")]
use cumulus_primitives_core::relay_chain::Hash as PHash;

#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug, TypeInfo)]
pub struct CodeHashes(pub Vec<(u32, H256)>);

#[cfg(feature = "std")]
impl CodeHashes {
    pub async fn get_hashes(
        relay_parent: PHash,
        relay_chain_interface: &impl cumulus_relay_chain_interface::RelayChainInterface,
    ) -> Option<Self> {
        relay_chain_interface
            .get_storage_by_key(
                relay_parent,
                &hex_literal::hex!(
                    "cd710b30bd2eab0352ddcc26417aa194e2d1c22ba0a888147714a3487bd51c63"
                ),
            )
            .await
            .ok()?
            .map(|x| CodeHashes(Vec::<(u32, H256)>::decode(&mut x.as_slice()).unwrap()))
    }
}

#[cfg(feature = "std")]
#[async_trait::async_trait]
impl sp_inherents::InherentDataProvider for CodeHashes {
    fn provide_inherent_data(
        &self,
        inherent_data: &mut sp_inherents::InherentData,
    ) -> Result<(), sp_inherents::Error> {
        inherent_data.put_data(*b"codehash", &self)
    }

    async fn try_handle_error(
        &self,
        _: &sp_inherents::InherentIdentifier,
        _: &[u8],
    ) -> Option<Result<(), sp_inherents::Error>> {
        None
    }
}
