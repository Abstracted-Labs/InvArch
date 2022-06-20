use super::pallet::*;
use frame_support::pallet_prelude::*;
use frame_system::ensure_signed;
use frame_system::pallet_prelude::*;
use primitives::{OneOrPercent, Parentage};

use parity_wasm::elements::{ExportEntry, ImportEntry};
use sp_sandbox::{SandboxEnvironmentBuilder, SandboxInstance, SandboxMemory};

pub trait LicenseList<T: Config> {
    fn get_hash_and_metadata(
        &self,
    ) -> (
        BoundedVec<u8, <T as Config>::MaxMetadata>,
        <T as frame_system::Config>::Hash,
    );
}

impl<T: Config> Pallet<T> {
    pub(crate) fn inner_set_permission(
        owner: OriginFor<T>,
        ipl_id: T::IpId,
        sub_asset: T::IpId,
        call_metadata: [u8; 2],
        permission: BoolOrWasm<T>,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        // Wasm permissions disabled for now. Too new for Tinkernet.
        ensure!(
            matches!(permission, BoolOrWasm::<T>::Bool(_)),
            Error::<T>::WasmPermissionsDisabled
        );

        let ip = IpStorage::<T>::get(ipl_id).ok_or(Error::<T>::IpDoesntExist)?;

        match ip.parentage {
            Parentage::Parent(ips_account) => {
                ensure!(ips_account == owner, Error::<T>::NoPermission)
            }
            Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
        }

        if let BoolOrWasm::<T>::Wasm(ref wasm) = permission {
            let module = parity_wasm::elements::Module::from_bytes(&wasm)
                .map_err(|_| Error::<T>::InvalidWasmPermission)?;

            ensure!(
                if let Some(import_section) = module.import_section() {
                    import_section
                        .entries()
                        .iter()
                        .any(|entry: &ImportEntry| entry.module() == "e" && entry.field() == "m")
                } else {
                    false
                },
                Error::<T>::InvalidWasmPermission
            );

            ensure!(
                if let Some(export_section) = module.export_section() {
                    export_section
                        .entries()
                        .iter()
                        .any(|entry: &ExportEntry| entry.field() == "c")
                } else {
                    false
                },
                Error::<T>::InvalidWasmPermission
            );
        }

        Permissions::<T>::insert((ipl_id, sub_asset), call_metadata, permission.clone());

        Self::deposit_event(Event::PermissionSet(
            ipl_id,
            sub_asset,
            call_metadata,
            permission,
        ));

        Ok(())
    }

    pub(crate) fn inner_set_asset_weight(
        owner: OriginFor<T>,
        ipl_id: T::IpId,
        sub_asset: T::IpId,
        asset_weight: OneOrPercent,
    ) -> DispatchResult {
        let owner = ensure_signed(owner)?;

        let ip = IpStorage::<T>::get(ipl_id).ok_or(Error::<T>::IpDoesntExist)?;

        match ip.parentage {
            Parentage::Parent(ips_account) => {
                ensure!(ips_account == owner, Error::<T>::NoPermission)
            }
            Parentage::Child(..) => return Err(Error::<T>::NotParent.into()),
        }

        AssetWeight::<T>::insert(ipl_id, sub_asset, asset_weight);

        Self::deposit_event(Event::WeightSet(ipl_id, sub_asset, asset_weight));

        Ok(())
    }

    pub fn execution_threshold(ipl_id: T::IpId) -> Option<OneOrPercent> {
        IpStorage::<T>::get(ipl_id).map(|ipl| ipl.execution_threshold)
    }

    pub fn asset_weight(ipl_id: T::IpId, sub_asset: T::IpId) -> Option<OneOrPercent> {
        AssetWeight::<T>::get(ipl_id, sub_asset)
            .or_else(|| IpStorage::<T>::get(ipl_id).map(|ipl| ipl.default_asset_weight))
    }

    pub fn has_permission(
        ipl_id: T::IpId,
        sub_asset: T::IpId,
        call_metadata: [u8; 2],
        call_arguments: BoundedVec<u8, T::MaxWasmPermissionBytes>,
    ) -> Result<bool, Error<T>> {
        Permissions::<T>::get((ipl_id, sub_asset), call_metadata).map_or(
            IpStorage::<T>::get(ipl_id)
                .map(|ipl| ipl.default_permission)
                .ok_or(Error::<T>::IpDoesntExist),
            |bool_or_wasm| -> Result<bool, Error<T>> {
                match bool_or_wasm {
                    BoolOrWasm::<T>::Bool(b) => Ok(b),
                    BoolOrWasm::<T>::Wasm(wasm) => {
                        let args = call_arguments.as_slice();

                        let mut env =
                            sp_sandbox::default_executor::EnvironmentDefinitionBuilder::new();
                        let mem = sp_sandbox::default_executor::Memory::new(1u32, Some(1u32))
                            .map_err(|_| Error::<T>::WasmPermissionFailedExecution)?;
                        mem.set(1u32, args)
                            .map_err(|_| Error::<T>::WasmPermissionFailedExecution)?;
                        env.add_memory("e", "m", mem);

                        let mut instance =
                            sp_sandbox::default_executor::Instance::new(&wasm, &env, &mut ())
                                .map_err(|_| Error::<T>::WasmPermissionFailedExecution)?;

                        if let sp_sandbox::ReturnValue::Value(sp_sandbox::Value::I32(integer)) =
                            instance
                                .invoke("c", &[sp_sandbox::Value::I32(args.len() as i32)], &mut ())
                                .map_err(|_| Error::<T>::WasmPermissionFailedExecution)?
                        {
                            Ok(!matches!(integer, 0))
                        } else {
                            Err(Error::<T>::InvalidWasmPermission)
                        }
                    }
                }
            },
        )
    }
}
