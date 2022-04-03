use codec::{Decode, Encode};
use core::convert::{TryFrom, TryInto};
use frame_support::log::{error, trace};
use invarch_runtime_primitives::CommonId;
use log::debug;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

use pallet_ipf::*;
use pallet_ips::*;
use pallet_ipt::*;

use super::Runtime;

pub fn to_account_id(account: &[u8]) -> Result<sp_runtime::AccountId32, ()> {
    sp_runtime::AccountId32::try_from(account)
}

/// Contract extension for the InvArch chain.
pub struct InvarchExtension;

impl ChainExtension<Runtime> for InvarchExtension {
    fn call<E: Ext>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
    where
        <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        trace!(
            target: "runtime",
            "[ChainExtension]|call|func_id:{:}",
            func_id
        );

        match func_id {
            /// ipf.mint
            5000 => {
                let mut env = env.buf_in_buf_out();
                let (metadata, data): (Vec<u8>, <Runtime as SysConfig>::Hash) =
                    env.read_as_unbounded(env.in_len())?;

                let id: <Runtime as pallet_ipf::Config>::IpfId =
                    pallet_ipf::NextIpfId::<Runtime>::get();

                pallet_ipf::Pallet::<Runtime>::internal_mint(
                    to_account_id(env.ext().address().as_ref())
                        .map_err(|_| {
                            DispatchError::Other(
                                "ChainExtension failed to convert contract account to AccountId32",
                            )
                        })
                        .unwrap(),
                    metadata
                        .to_vec()
                        .try_into()
                        .map_err(|_| DispatchError::Other("Metadata exceeds limit"))?,
                    data,
                )?;

                env.write(&id.encode(), false, None)
                    .map_err(|_| DispatchError::Other("ChainExtension failed to call ipf.mint"))?;
            }

            /// ipf.burn
            5001 => {
                let mut env = env.buf_in_buf_out();
                let ipf_id: CommonId = env.read_as()?;

                pallet_ipf::Pallet::<Runtime>::internal_burn(
                    to_account_id(env.ext().address().as_ref())
                        .map_err(|_| {
                            DispatchError::Other(
                                "ChainExtension failed to convert contract account to AccountId32",
                            )
                        })
                        .unwrap(),
                    ipf_id,
                )?;
            }

            /// ips.createIps
            5100 => {
                let mut env = env.buf_in_buf_out();
                let (metadata, data, allow_replica): (Vec<u8>, Vec<CommonId>, bool) =
                    env.read_as_unbounded(env.in_len())?;

                let id: <Runtime as pallet_ips::Config>::IpsId =
                    pallet_ips::NextIpsId::<Runtime>::get();

                pallet_ips::Pallet::<Runtime>::internal_create_ips(
                    to_account_id(env.ext().address().as_ref())
                        .map_err(|_| {
                            DispatchError::Other(
                                "ChainExtension failed to convert contract account to AccountId32",
                            )
                        })
                        .unwrap(),
                    metadata
                        .try_into()
                        .map_err(|_| DispatchError::Other("Metadata exceeds limit"))?,
                    data.try_into()
                        .map_err(|_| DispatchError::Other("Data exceeds limit"))?,
                    allow_replica,
                )?;

                env.write(&id.encode(), false, None)
                    .map_err(|_| DispatchError::Other("ChainExtension failed to call ipf.mint"))?;
            }

            /// ips.append
            5102 => {
                let mut env = env.buf_in_buf_out();
                let (ips_id, assets, new_metadata): (
                    CommonId,
                    Vec<
                        invarch_primitives::AnyId<
                            <Runtime as pallet_ipf::Config>::IpfId,
                            <Runtime as pallet_ips::Config>::IpsId,
                        >,
                    >,
                    Option<Vec<u8>>,
                ) = env.read_as_unbounded(env.in_len())?;

                pallet_ips::Pallet::<Runtime>::internal_append(
                    to_account_id(env.ext().address().as_ref())
                        .map_err(|_| {
                            DispatchError::Other(
                                "ChainExtension failed to convert contract account to AccountId32",
                            )
                        })
                        .unwrap(),
                    ips_id,
                    assets,
                    new_metadata,
                )?;
            }

            /// ips.remove
            5103 => {
                let mut env = env.buf_in_buf_out();
                let (ips_id, assets, new_metadata): (
                    CommonId,
                    Vec<(
                        invarch_primitives::AnyId<
                            <Runtime as pallet_ipf::Config>::IpfId,
                            <Runtime as pallet_ips::Config>::IpsId,
                        >,
                        <Runtime as SysConfig>::AccountId,
                    )>,
                    Option<Vec<u8>>,
                ) = env.read_as_unbounded(env.in_len())?;

                pallet_ips::Pallet::<Runtime>::internal_remove(
                    to_account_id(env.ext().address().as_ref())
                        .map_err(|_| {
                            DispatchError::Other(
                                "ChainExtension failed to convert contract account to AccountId32",
                            )
                        })
                        .unwrap(),
                    ips_id,
                    assets,
                    new_metadata,
                )?;
            }

            /// ips.allowReplica
            5104 => {
                let mut env = env.buf_in_buf_out();
                let ips_id: CommonId = env.read_as_unbounded(env.in_len())?;

                pallet_ips::Pallet::<Runtime>::internal_allow_replica(
                    to_account_id(env.ext().address().as_ref())
                        .map_err(|_| {
                            DispatchError::Other(
                                "ChainExtension failed to convert contract account to AccountId32",
                            )
                        })
                        .unwrap(),
                    ips_id,
                )?;
            }

            /// ips.disallowReplica
            5105 => {
                let mut env = env.buf_in_buf_out();
                let ips_id: CommonId = env.read_as_unbounded(env.in_len())?;

                pallet_ips::Pallet::<Runtime>::internal_disallow_replica(
                    to_account_id(env.ext().address().as_ref())
                        .map_err(|_| {
                            DispatchError::Other(
                                "ChainExtension failed to convert contract account to AccountId32",
                            )
                        })
                        .unwrap(),
                    ips_id,
                )?;
            }

            /// ips.createReplica
            5106 => {
                let mut env = env.buf_in_buf_out();
                let ips_id: CommonId = env.read_as_unbounded(env.in_len())?;

                pallet_ips::Pallet::<Runtime>::internal_create_replica(
                    to_account_id(env.ext().address().as_ref())
                        .map_err(|_| {
                            DispatchError::Other(
                                "ChainExtension failed to convert contract account to AccountId32",
                            )
                        })
                        .unwrap(),
                    ips_id,
                )?;
            }

            /// ipt.mint
            5201 => {
                let mut env = env.buf_in_buf_out();
                let (target, ipt_id, amount): (
                    <Runtime as SysConfig>::AccountId,
                    CommonId,
                    <Runtime as pallet_ipt::Config>::Balance,
                ) = env.read_as_unbounded(env.in_len())?;

                pallet_ipt::Pallet::<Runtime>::internal_mint(
                    to_account_id(env.ext().address().as_ref())
                        .map_err(|_| {
                            DispatchError::Other(
                                "ChainExtension failed to convert contract account to AccountId32",
                            )
                        })
                        .unwrap(),
                    target,
                    ipt_id,
                    amount,
                )?;
            }

            _ => {
                error!("Called an unregistered `func_id`: {:}", func_id);
                return Err(DispatchError::Other("Unimplemented func_id"));
            }
        }

        Ok(RetVal::Converging(0))
    }

    fn enabled() -> bool {
        true
    }
}
