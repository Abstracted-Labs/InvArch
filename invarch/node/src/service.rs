//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

// Local Runtime Types
use invarch_runtime::{api, native_version as _native_version, opaque::Block, Hash};

use cumulus_client_cli::CollatorOptions;
use cumulus_client_consensus_aura::{AuraConsensus, BuildAuraConsensusParams, SlotProportion};
use cumulus_client_consensus_common::{
    ParachainBlockImport as TParachainBlockImport, ParachainConsensus,
};
use cumulus_client_network::BlockAnnounceValidator;
use cumulus_client_service::{
    build_relay_chain_interface, prepare_node_config, start_collator, start_full_node,
    StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;
use cumulus_primitives_parachain_inherent::{
    MockValidationDataInherentDataProvider, MockXcmConfig,
};
use cumulus_relay_chain_interface::{RelayChainError, RelayChainInterface, RelayChainResult};
use jsonrpsee::RpcModule;
use polkadot_service::CollatorPair;
use sc_consensus::ImportQueue;
use sc_executor::NativeElseWasmExecutor;
use sc_network::{NetworkBlock, NetworkService};
use sc_service::{
    ChainSpec, Configuration, PartialComponents, TFullBackend, TFullClient, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::traits::BlakeTwo256;
use std::{sync::Arc, time::Duration};
use substrate_prometheus_endpoint::Registry;

// Our native executor instance.
pub struct ParachainNativeExecutor;

impl sc_executor::NativeExecutionDispatch for ParachainNativeExecutor {
    /// Only enable the benchmarking host functions when we actually want to benchmark.
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    /// Otherwise we only use the default Substrate host functions.
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        _native_version()
    }
}

type ParachainExecutor = NativeElseWasmExecutor<ParachainNativeExecutor>;

type ParachainClient = TFullClient<Block, invarch_runtime::RuntimeApi, ParachainExecutor>;

type ParachainBackend = TFullBackend<Block>;

type ParachainBlockImport = TParachainBlockImport<Block, Arc<ParachainClient>, ParachainBackend>;

pub trait ChainIdentify {
    fn is_solo_dev(&self) -> bool;
}

impl ChainIdentify for Box<dyn ChainSpec> {
    fn is_solo_dev(&self) -> bool {
        self.id().starts_with("invarch-solo-dev")
    }
}

//type Executor = sp_io::SubstrateHostFunctions;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
#[allow(clippy::type_complexity)]
pub fn new_partial<BIQ>(
    config: &Configuration,
    build_import_queue: BIQ,
) -> Result<
    PartialComponents<
        ParachainClient,
        ParachainBackend,
        (),
        sc_consensus::DefaultImportQueue<Block, ParachainClient>,
        sc_transaction_pool::FullPool<Block, ParachainClient>,
        (
            ParachainBlockImport,
            Option<Telemetry>,
            Option<TelemetryWorkerHandle>,
        ),
    >,
    sc_service::Error,
>
where
    sc_client_api::StateBackendFor<ParachainBackend, Block>: sp_api::StateBackend<BlakeTwo256>,
    BIQ: FnOnce(
        Arc<ParachainClient>,
        ParachainBlockImport,
        &Configuration,
        Option<TelemetryHandle>,
        &TaskManager,
    ) -> Result<
        sc_consensus::DefaultImportQueue<Block, ParachainClient>,
        sc_service::Error,
    >,
{
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = ParachainExecutor::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
        config.runtime_cache_size,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, invarch_runtime::RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let block_import = ParachainBlockImport::new(client.clone(), backend.clone());

    let import_queue = build_import_queue(
        client.clone(),
        block_import.clone(),
        config,
        telemetry.as_ref().map(|telemetry| telemetry.handle()),
        &task_manager,
    )?;

    let params = PartialComponents {
        backend,
        client,
        import_queue,
        keystore_container,
        task_manager,
        transaction_pool,
        select_chain: (),
        other: (block_import, telemetry, telemetry_worker_handle),
    };

    Ok(params)
}

async fn _build_relay_chain_interface(
    polkadot_config: Configuration,
    parachain_config: &Configuration,
    telemetry_worker_handle: Option<TelemetryWorkerHandle>,
    task_manager: &mut TaskManager,
    collator_options: CollatorOptions,
    hwbench: Option<sc_sysinfo::HwBench>,
) -> RelayChainResult<(
    Arc<(dyn RelayChainInterface + 'static)>,
    Option<CollatorPair>,
)> {
    build_relay_chain_interface(
        polkadot_config,
        parachain_config,
        telemetry_worker_handle,
        task_manager,
        collator_options,
        hwbench,
    )
    .await

    // match collator_options.relay_chain_rpc_url {
    //     Some(relay_chain_url) => {
    //         let client = create_client_and_start_worker(relay_chain_url, task_manager).await?;
    //         Ok((
    //             Arc::new(RelayChainRpcInterface::new(client)) as Arc<_>,
    //             None,
    //         ))
    //     }
    //     None => build_inprocess_relay_chain(
    //         polkadot_config,
    //         parachain_config,
    //         telemetry_worker_handle,
    //         task_manager,
    //         hwbench,
    //     ),
    // }
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[allow(clippy::too_many_arguments)]
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<RB, BIQ, BIC>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
    id: ParaId,
    _rpc_ext_builder: RB,
    build_import_queue: BIQ,
    build_consensus: BIC,
    hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)>
where
    sc_client_api::StateBackendFor<ParachainBackend, Block>: sp_api::StateBackend<BlakeTwo256>,
    RB: Fn(Arc<ParachainClient>) -> Result<RpcModule<()>, sc_service::Error> + Send + 'static,
    BIQ: FnOnce(
            Arc<ParachainClient>,
            ParachainBlockImport,
            &Configuration,
            Option<TelemetryHandle>,
            &TaskManager,
        ) -> Result<
            sc_consensus::DefaultImportQueue<Block, ParachainClient>,
            sc_service::Error,
        > + 'static,
    BIC: FnOnce(
        Arc<ParachainClient>,
        ParachainBlockImport,
        Option<&Registry>,
        Option<TelemetryHandle>,
        &TaskManager,
        Arc<dyn RelayChainInterface>,
        Arc<sc_transaction_pool::FullPool<Block, ParachainClient>>,
        Arc<NetworkService<Block, Hash>>,
        SyncCryptoStorePtr,
        bool,
    ) -> Result<Box<dyn ParachainConsensus<Block>>, sc_service::Error>,
{
    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial::<BIQ>(&parachain_config, build_import_queue)?;
    let (block_import, mut telemetry, telemetry_worker_handle) = params.other;

    let client = params.client.clone();
    let backend = params.backend.clone();
    let mut task_manager = params.task_manager;

    let (relay_chain_interface, collator_key) = build_relay_chain_interface(
        polkadot_config,
        &parachain_config,
        telemetry_worker_handle,
        &mut task_manager,
        collator_options.clone(),
        hwbench,
    )
    .await
    .map_err(|e| match e {
        RelayChainError::ServiceError(polkadot_service::Error::Sub(x)) => x,
        s => s.to_string().into(),
    })?;

    let block_announce_validator = BlockAnnounceValidator::new(relay_chain_interface.clone(), id);

    let force_authoring = parachain_config.force_authoring;
    let validator = parachain_config.role.is_authority();
    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();

    // let import_queue = cumulus_client_service::SharedImportQueue::new(params.import_queue);

    let import_queue_service = params.import_queue.service();

    let (network, system_rpc_tx, tx_handler_controller, start_network) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &parachain_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue: params.import_queue,
            block_announce_validator_builder: Some(Box::new(|_| {
                Box::new(block_announce_validator)
            })),
            warp_sync: None,
        })?;

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                deny_unsafe,
                command_sink: None,
            };

            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        rpc_builder,
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        config: parachain_config,
        keystore: params.keystore_container.sync_keystore(),
        backend: backend.clone(),
        network: network.clone(),
        system_rpc_tx,
        tx_handler_controller,
        telemetry: telemetry.as_mut(),
    })?;

    let announce_block = {
        let network = network.clone();
        Arc::new(move |hash, data| network.announce_block(hash, data))
    };

    let relay_chain_slot_duration = Duration::from_secs(6);

    if validator {
        let parachain_consensus = build_consensus(
            client.clone(),
            block_import,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|t| t.handle()),
            &task_manager,
            relay_chain_interface.clone(),
            transaction_pool,
            network,
            params.keystore_container.sync_keystore(),
            force_authoring,
        )?;

        let spawner = task_manager.spawn_handle();

        let params = StartCollatorParams {
            para_id: id,
            block_status: client.clone(),
            announce_block,
            client: client.clone(),
            task_manager: &mut task_manager,
            relay_chain_interface,
            spawner,
            parachain_consensus,
            import_queue: import_queue_service,
            collator_key: collator_key.expect("Command line arguments do not allow this. qed"),
            relay_chain_slot_duration,
        };

        start_collator(params).await?;
    } else {
        let params = StartFullNodeParams {
            client: client.clone(),
            announce_block,
            task_manager: &mut task_manager,
            para_id: id,
            relay_chain_interface,
            relay_chain_slot_duration,
            import_queue: import_queue_service,
        };

        start_full_node(params)?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}

use sc_client_api::HeaderBackend;

/// Build the import queue for the parachain runtime.
#[allow(clippy::type_complexity)]
pub fn parachain_build_import_queue(
    client: Arc<ParachainClient>,
    block_import: ParachainBlockImport,
    config: &Configuration,
    telemetry: Option<TelemetryHandle>,
    task_manager: &TaskManager,
) -> Result<sc_consensus::DefaultImportQueue<Block, ParachainClient>, sc_service::Error> {
    let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

    cumulus_client_consensus_aura::import_queue::<
        sp_consensus_aura::sr25519::AuthorityPair,
        _,
        _,
        _,
        _,
        _,
    >(cumulus_client_consensus_aura::ImportQueueParams {
        block_import,
        client,
        create_inherent_data_providers: move |_, _| async move {
            let time = sp_timestamp::InherentDataProvider::from_system_time();

            let slot =
                sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                    *time,
                    slot_duration,
                );

            Ok((slot, time))
        },
        registry: config.prometheus_registry(),
        spawner: &task_manager.spawn_essential_handle(),
        telemetry,
    })
    .map_err(Into::into)
}

/// Start a parachain node.
pub async fn start_parachain_node(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
    id: ParaId,
    hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
    start_node_impl::<_, _, _>(
        parachain_config,
        polkadot_config,
        collator_options,
        id,
        |_| Ok(RpcModule::new(())),
        parachain_build_import_queue,
        |client,
        block_import,
         prometheus_registry,
         telemetry,
         task_manager,
         relay_chain_interface,
         transaction_pool,
         sync_oracle,
         keystore,
         force_authoring| {
            let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

            let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
                task_manager.spawn_handle(),
                client.clone(),
                transaction_pool,
                prometheus_registry,
                telemetry.clone(),
            );

            Ok(AuraConsensus::build::<
                sp_consensus_aura::sr25519::AuthorityPair,
                _,
                _,
                _,
                _,
                _,
                _,
            >(BuildAuraConsensusParams {
                proposer_factory,
                create_inherent_data_providers: move |_, (relay_parent, validation_data)| {
                    let relay_chain_interface = relay_chain_interface.clone();
                    async move {
                        let parachain_inherent =
                            cumulus_primitives_parachain_inherent::ParachainInherentData::create_at(
                                relay_parent,
                                &relay_chain_interface,
                                &validation_data,
                                id,
                            ).await;
                        let time = sp_timestamp::InherentDataProvider::from_system_time();

                        let slot =
                            sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                                *time,
                                slot_duration,
                            );

                        let parachain_inherent = parachain_inherent.ok_or_else(|| {
                            Box::<dyn std::error::Error + Send + Sync>::from(
                                "Failed to create parachain inherent",
                            )
                        })?;
                        Ok((slot, time, parachain_inherent))
                    }
                },
                block_import,
                para_client: client,
                backoff_authoring_blocks: Option::<()>::None,
                sync_oracle,
                keystore,
                force_authoring,
                slot_duration,
                // We got around 500ms for proposing
                block_proposal_slot_portion: SlotProportion::new(1f32 / 24f32),
                // And a maximum of 750ms if slots are skipped
                max_block_proposal_slot_portion: Some(SlotProportion::new(1f32 / 16f32)),
                telemetry,
            }))
        }, hwbench
    )
    .await
}

pub fn start_solo_dev(config: Configuration) -> Result<TaskManager, sc_service::error::Error>
where
    sc_client_api::StateBackendFor<ParachainBackend, Block>: sp_api::StateBackend<BlakeTwo256>,
{
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (mut telemetry, _),
    } = new_solo_partial(&config)?;

    let (network, system_rpc_tx, tx_handler_controller, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync: None,
        })?;

    let prometheus_registry = config.prometheus_registry().cloned();

    let role = config.role.clone();
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;

    let command_sink = if role.is_authority() {
        let proposer_factory = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
        let client_for_cidp = client.clone();

        let aura = sc_consensus_aura::start_aura::<
            sp_consensus_aura::sr25519::AuthorityPair,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
        >(sc_consensus_aura::StartAuraParams {
            slot_duration: sc_consensus_aura::slot_duration(&*client)?,
            client: client.clone(),
            select_chain,
            block_import: instant_finalize::InstantFinalizeBlockImport::new(client.clone()),
            proposer_factory,
            create_inherent_data_providers: move |block: Hash, ()| {
                let current_para_block = client_for_cidp
                    .number(block)
                    .expect("Header lookup should succeed")
                    .expect("Header passed in as parent should be present in backend.");
                let client_for_xcm = client_for_cidp.clone();

                async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                    let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

                    let mocked_parachain = MockValidationDataInherentDataProvider {
                        current_para_block,
                        relay_offset: 1000,
                        relay_blocks_per_para_block: 2,
                        xcm_config: MockXcmConfig::new(
                            &*client_for_xcm,
                            block,
                            Default::default(),
                            Default::default(),
                        ),
                        raw_downward_messages: vec![],
                        raw_horizontal_messages: vec![],
                        para_blocks_per_relay_epoch: 0,
                        relay_randomness_config: (),
                    };

                    Ok((slot, timestamp, mocked_parachain))
                }
            },
            force_authoring,
            backoff_authoring_blocks,
            keystore: keystore_container.sync_keystore(),
            sync_oracle: network.clone(),
            justification_sync_link: network.clone(),
            // We got around 500ms for proposing
            block_proposal_slot_portion: SlotProportion::new(1f32 / 24f32),
            // And a maximum of 750ms if slots are skipped
            max_block_proposal_slot_portion: Some(SlotProportion::new(1f32 / 16f32)),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            compatibility_mode: Default::default(),
        })?;

        // the AURA authoring task is considered essential, i.e. if it
        // fails we take down the service with it.
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("aura", Some("block-authoring"), aura);

        None
    } else {
        None
    };

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                deny_unsafe,
                command_sink: command_sink.clone(),
            };

            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        rpc_builder,
        client,
        transaction_pool,
        task_manager: &mut task_manager,
        config,
        keystore: keystore_container.sync_keystore(),
        backend,
        network,
        system_rpc_tx,
        tx_handler_controller,
        telemetry: telemetry.as_mut(),
    })?;

    network_starter.start_network();

    Ok(task_manager)
}

#[allow(clippy::type_complexity)]
pub fn new_solo_partial(
    config: &Configuration,
) -> Result<
    PartialComponents<
        ParachainClient,
        ParachainBackend,
        sc_consensus::LongestChain<ParachainBackend, Block>,
        sc_consensus::DefaultImportQueue<Block, ParachainClient>,
        sc_transaction_pool::FullPool<Block, ParachainClient>,
        (Option<Telemetry>, Option<TelemetryWorkerHandle>),
    >,
    sc_service::Error,
>
where
    sc_client_api::StateBackendFor<ParachainBackend, Block>: sp_api::StateBackend<BlakeTwo256>,
{
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = ParachainExecutor::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
        config.runtime_cache_size,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, invarch_runtime::RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let import_queue = {
        // aura import queue
        let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
        let client_for_cidp = client.clone();

        sc_consensus_aura::import_queue::<sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _>(
            sc_consensus_aura::ImportQueueParams {
                block_import: client.clone(),
                justification_import: None,
                client: client.clone(),
                create_inherent_data_providers: move |block: Hash, ()| {
                    let current_para_block = client_for_cidp
                        .number(block)
                        .expect("Header lookup should succeed")
                        .expect("Header passed in as parent should be present in backend.");
                    let client_for_xcm = client_for_cidp.clone();

                    async move {
                        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                        let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp,
                        slot_duration,
                    );

                        let mocked_parachain = MockValidationDataInherentDataProvider {
                            current_para_block,
                            relay_offset: 1000,
                            relay_blocks_per_para_block: 1,
                            xcm_config: MockXcmConfig::new(
                                &*client_for_xcm,
                                block,
                                Default::default(),
                                Default::default(),
                            ),
                            raw_downward_messages: vec![],
                            raw_horizontal_messages: vec![],
                            para_blocks_per_relay_epoch: 0,
                            relay_randomness_config: (),
                        };

                        Ok((slot, timestamp, mocked_parachain))
                    }
                },
                spawner: &task_manager.spawn_essential_handle(),
                registry: config.prometheus_registry(),
                check_for_equivocation: Default::default(),
                telemetry: telemetry.as_ref().map(|x| x.handle()),
                compatibility_mode: Default::default(),
            },
        )?
    };

    let params = PartialComponents {
        backend: backend.clone(),
        client,
        import_queue,
        keystore_container,
        task_manager,
        transaction_pool,
        select_chain: sc_consensus::LongestChain::new(backend),
        other: (telemetry, telemetry_worker_handle),
    };

    Ok(params)
}

mod instant_finalize {
    use sc_consensus::BlockImport;
    use sp_runtime::traits::Block as BlockT;

    pub struct InstantFinalizeBlockImport<I>(I);

    impl<I> InstantFinalizeBlockImport<I> {
        /// Create a new instance.
        pub fn new(inner: I) -> Self {
            Self(inner)
        }
    }

    #[async_trait::async_trait]
    impl<Block, I> BlockImport<Block> for InstantFinalizeBlockImport<I>
    where
        Block: BlockT,
        I: BlockImport<Block> + Send,
    {
        type Error = I::Error;
        type Transaction = I::Transaction;

        async fn check_block(
            &mut self,
            block: sc_consensus::BlockCheckParams<Block>,
        ) -> Result<sc_consensus::ImportResult, Self::Error> {
            self.0.check_block(block).await
        }

        async fn import_block(
            &mut self,
            mut block_import_params: sc_consensus::BlockImportParams<Block, Self::Transaction>,
            cache: std::collections::HashMap<sp_consensus::CacheKeyId, Vec<u8>>,
        ) -> Result<sc_consensus::ImportResult, Self::Error> {
            block_import_params.finalized = true;
            self.0.import_block(block_import_params, cache).await
        }
    }
}
