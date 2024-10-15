//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

// std
use std::{sync::Arc, time::Duration};

use cumulus_client_cli::CollatorOptions;
// Local Runtime Types
use invarch_runtime::{opaque::Block, Hash, RuntimeApi};

// Cumulus Imports
use cumulus_client_collator::service::CollatorService;
#[docify::export(lookahead_collator)]
use cumulus_client_consensus_aura::{
    collators::lookahead::{self as aura, Params as AuraParams},
    SlotProportion,
};
use cumulus_client_consensus_common::ParachainBlockImport as TParachainBlockImport;
use cumulus_client_consensus_proposer::Proposer;
use cumulus_client_parachain_inherent::{MockValidationDataInherentDataProvider, MockXcmConfig};
use cumulus_client_service::{
    build_network, build_relay_chain_interface, prepare_node_config, start_relay_chain_tasks,
    BuildNetworkParams, CollatorSybilResistance, DARecoveryProfile, ParachainHostFunctions,
    StartRelayChainTasksParams,
};
#[docify::export(cumulus_primitives)]
use cumulus_primitives_core::{
    relay_chain::{CollatorPair, ValidationCode},
    ParaId,
};
use cumulus_relay_chain_interface::{OverseerHandle, RelayChainInterface};
use sc_client_api::HeaderBackend;

// Substrate Imports
use frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE;
use prometheus_endpoint::Registry;
use sc_client_api::Backend;
use sc_consensus::ImportQueue;
use sc_executor::{HeapAllocStrategy, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_network::{NetworkBlock, NotificationMetrics};
use sc_service::{Configuration, PartialComponents, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_core::Encode;
use sp_keystore::KeystorePtr;

// #[docify::export(wasm_executor)]
pub type ParachainExecutor = WasmExecutor<ParachainHostFunctions>;

type ParachainClient = TFullClient<Block, RuntimeApi, ParachainExecutor>;

type ParachainBackend = TFullBackend<Block>;

type ParachainBlockImport = TParachainBlockImport<Block, Arc<ParachainClient>, ParachainBackend>;

/// Assembly of PartialComponents (enough to run chain ops subcommands)
pub type Service = PartialComponents<
    ParachainClient,
    ParachainBackend,
    (),
    sc_consensus::DefaultImportQueue<Block>,
    sc_transaction_pool::FullPool<Block, ParachainClient>,
    (
        ParachainBlockImport,
        Option<Telemetry>,
        Option<TelemetryWorkerHandle>,
    ),
>;

pub type ServiceSolo = PartialComponents<
    ParachainClient,
    ParachainBackend,
    sc_consensus::LongestChain<ParachainBackend, Block>,
    sc_consensus::DefaultImportQueue<Block>,
    sc_transaction_pool::FullPool<Block, ParachainClient>,
    (Option<Telemetry>, Option<TelemetryWorkerHandle>),
>;

pub trait ChainIdentify {
    fn is_solo_dev(&self) -> bool;
}

impl ChainIdentify for Box<dyn sc_service::ChainSpec> {
    fn is_solo_dev(&self) -> bool {
        self.id().starts_with("invarch-solo-dev")
    }
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
#[docify::export(component_instantiation)]
pub fn new_partial(config: &Configuration) -> Result<Service, sc_service::Error> {
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

    let heap_pages = config
        .default_heap_pages
        .map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static {
            extra_pages: h as _,
        });

    let executor = ParachainExecutor::builder()
        .with_execution_method(config.wasm_method)
        .with_onchain_heap_alloc_strategy(heap_pages)
        .with_offchain_heap_alloc_strategy(heap_pages)
        .with_max_runtime_instances(config.max_runtime_instances)
        .with_runtime_cache_size(config.runtime_cache_size)
        .build();

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts_record_import::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
            true,
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
    );

    Ok(PartialComponents {
        backend,
        client,
        import_queue,
        keystore_container,
        task_manager,
        transaction_pool,
        select_chain: (),
        other: (block_import, telemetry, telemetry_worker_handle),
    })
}

/// Build the import queue for the parachain runtime.
fn build_import_queue(
    client: Arc<ParachainClient>,
    block_import: ParachainBlockImport,
    config: &Configuration,
    telemetry: Option<TelemetryHandle>,
    task_manager: &TaskManager,
) -> sc_consensus::DefaultImportQueue<Block> {
    cumulus_client_consensus_aura::equivocation_import_queue::fully_verifying_import_queue::<
        sp_consensus_aura::sr25519::AuthorityPair,
        _,
        _,
        _,
        _,
    >(
        client,
        block_import,
        move |_, _| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            Ok(timestamp)
        },
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
        telemetry,
    )
}

#[allow(clippy::too_many_arguments)]
fn start_consensus(
    client: Arc<ParachainClient>,
    backend: Arc<ParachainBackend>,
    block_import: ParachainBlockImport,
    prometheus_registry: Option<&Registry>,
    telemetry: Option<TelemetryHandle>,
    task_manager: &TaskManager,
    relay_chain_interface: Arc<dyn RelayChainInterface>,
    transaction_pool: Arc<sc_transaction_pool::FullPool<Block, ParachainClient>>,
    keystore: KeystorePtr,
    relay_chain_slot_duration: Duration,
    para_id: ParaId,
    collator_key: CollatorPair,
    overseer_handle: OverseerHandle,
    announce_block: Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
) -> Result<(), sc_service::Error> {
    let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
        task_manager.spawn_handle(),
        client.clone(),
        transaction_pool,
        prometheus_registry,
        telemetry.clone(),
    );

    let proposer = Proposer::new(proposer_factory);

    let collator_service = CollatorService::new(
        client.clone(),
        Arc::new(task_manager.spawn_handle()),
        announce_block,
        client.clone(),
    );

    let params = AuraParams {
        create_inherent_data_providers: move |_, ()| async move { Ok(()) },
        block_import,
        para_client: client.clone(),
        para_backend: backend,
        relay_client: relay_chain_interface,
        code_hash_provider: move |block_hash| {
            client
                .code_at(block_hash)
                .ok()
                .map(|c| ValidationCode::from(c).hash())
        },
        keystore,
        collator_key,
        para_id,
        overseer_handle,
        relay_chain_slot_duration,
        proposer,
        collator_service,
        authoring_duration: Duration::from_millis(2000),
        reinitialize: false,
    };
    let fut = aura::run::<Block, sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _, _, _, _>(
        params,
    );
    task_manager
        .spawn_essential_handle()
        .spawn("aura", None, fut);

    Ok(())
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
pub async fn start_parachain_node(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
    para_id: ParaId,
    hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial(&parachain_config)?;
    let (block_import, mut telemetry, telemetry_worker_handle) = params.other;
    let net_config = sc_network::config::FullNetworkConfiguration::<
        _,
        _,
        sc_network::NetworkWorker<Block, Hash>,
    >::new(&parachain_config.network);

    let client = params.client.clone();
    let backend = params.backend.clone();
    let mut task_manager = params.task_manager;

    let (relay_chain_interface, collator_key) = build_relay_chain_interface(
        polkadot_config,
        &parachain_config,
        telemetry_worker_handle,
        &mut task_manager,
        collator_options.clone(),
        hwbench.clone(),
    )
    .await
    .map_err(|e| sc_service::Error::Application(Box::new(e) as Box<_>))?;

    let validator = parachain_config.role.is_authority();
    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let import_queue_service = params.import_queue.service();

    // NOTE: because we use Aura here explicitly, we can use `CollatorSybilResistance::Resistant`
    // when starting the network.
    let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
        build_network(BuildNetworkParams {
            parachain_config: &parachain_config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            para_id,
            spawn_handle: task_manager.spawn_handle(),
            relay_chain_interface: relay_chain_interface.clone(),
            import_queue: params.import_queue,
            sybil_resistance_level: CollatorSybilResistance::Resistant, // because of Aura
        })
        .await?;

    if parachain_config.offchain_worker.enabled {
        use futures::FutureExt;

        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-work",
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                keystore: Some(params.keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: Arc::new(network.clone()),
                is_validator: parachain_config.role.is_authority(),
                enable_http_requests: false,
                custom_extensions: move |_| vec![],
            })
            .run(client.clone(), task_manager.spawn_handle())
            .boxed(),
        );
    }

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                deny_unsafe,
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
        keystore: params.keystore_container.keystore(),
        backend: backend.clone(),
        network,
        sync_service: sync_service.clone(),
        system_rpc_tx,
        tx_handler_controller,
        telemetry: telemetry.as_mut(),
    })?;

    if let Some(hwbench) = hwbench {
        sc_sysinfo::print_hwbench(&hwbench);
        // Here you can check whether the hardware meets your chains' requirements. Putting a link
        // in there and swapping out the requirements for your own are probably a good idea. The
        // requirements for a para-chain are dictated by its relay-chain.
        match SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench) {
            Err(err) if validator => {
                log::warn!(
				"⚠️  The hardware does not meet the minimal requirements {} for role 'Authority'.",
				err
			);
            }
            _ => {}
        }

        if let Some(ref mut telemetry) = telemetry {
            let telemetry_handle = telemetry.handle();
            task_manager.spawn_handle().spawn(
                "telemetry_hwbench",
                None,
                sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
            );
        }
    }

    let announce_block = {
        let sync_service = sync_service.clone();
        Arc::new(move |hash, data| sync_service.announce_block(hash, data))
    };

    let relay_chain_slot_duration = Duration::from_secs(6);

    let overseer_handle = relay_chain_interface
        .overseer_handle()
        .map_err(|e| sc_service::Error::Application(Box::new(e)))?;

    start_relay_chain_tasks(StartRelayChainTasksParams {
        client: client.clone(),
        announce_block: announce_block.clone(),
        para_id,
        relay_chain_interface: relay_chain_interface.clone(),
        task_manager: &mut task_manager,
        da_recovery_profile: if validator {
            DARecoveryProfile::Collator
        } else {
            DARecoveryProfile::FullNode
        },
        import_queue: import_queue_service,
        relay_chain_slot_duration,
        recovery_handle: Box::new(overseer_handle.clone()),
        sync_service: sync_service.clone(),
    })?;

    if validator {
        start_consensus(
            client.clone(),
            backend,
            block_import,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|t| t.handle()),
            &task_manager,
            relay_chain_interface,
            transaction_pool,
            params.keystore_container.keystore(),
            relay_chain_slot_duration,
            para_id,
            collator_key.expect("Command line arguments do not allow this. qed"),
            overseer_handle,
            announce_block,
        )?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}

pub async fn start_solo_dev(
    config: Configuration,
    para_id: ParaId,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
    start_solo_node_impl(config, para_id).await
}

#[sc_tracing::logging::prefix_logs_with("Parachain")]
pub async fn start_solo_node_impl(
    config: Configuration,
    para_id: ParaId,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
    let parachain_config = prepare_node_config(config);

    let params = new_partial_solo(&parachain_config, para_id)?;
    let (mut telemetry, _telemetry_worker_handle) = params.other;
    let net_config = sc_network::config::FullNetworkConfiguration::<
        _,
        _,
        sc_network::NetworkWorker<Block, Hash>,
    >::new(&parachain_config.network);

    let client = params.client.clone();
    let backend = params.backend.clone();
    let mut task_manager = params.task_manager;

    let transaction_pool = params.transaction_pool.clone();
    let import_queue = params.import_queue;
    let select_chain = params.select_chain;
    let force_authoring = parachain_config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let role = parachain_config.role.clone();

    let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &parachain_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_params: None,
            net_config,
            block_relay: None,
            metrics: NotificationMetrics::new(None),
        })?;

    if parachain_config.offchain_worker.enabled {
        use futures::FutureExt;

        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-work",
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                keystore: Some(params.keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: Arc::new(network.clone()),
                is_validator: parachain_config.role.is_authority(),
                enable_http_requests: false,
                custom_extensions: move |_| vec![],
            })
            .run(client.clone(), task_manager.spawn_handle())
            .boxed(),
        );
    }

    if role.is_authority() {
        let proposer_factory = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            None,
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
                let additional_key_values = Some(vec![(
                    array_bytes::hex2bytes_unchecked(
                        "1cb6f36e027abb2091cfb5110ab5087f06155b3cd9a8c9e5e9a23fd5dc13a5ed",
                    ),
                    sp_consensus_aura::Slot::from_timestamp(
                        sp_timestamp::Timestamp::current(),
                        slot_duration,
                    )
                    .encode(),
                )]);
                let current_para_block_head = client_for_cidp
                    .expect_header(block)
                    .ok()
                    .map(|h| (polkadot_primitives::HeadData(h.encode())));
                async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                    let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                    *timestamp,
                    slot_duration,
                );
                    let mocked_parachain = MockValidationDataInherentDataProvider {
                        current_para_block,
                        current_para_block_head,
                        para_id,
                        relay_offset: 1000,
                        relay_blocks_per_para_block: 1,
                        xcm_config: MockXcmConfig::new(&*client_for_xcm, block, Default::default()),
                        raw_downward_messages: vec![],
                        raw_horizontal_messages: vec![],
                        para_blocks_per_relay_epoch: 0,
                        relay_randomness_config: (),
                        additional_key_values,
                    };

                    Ok((slot, timestamp, mocked_parachain))
                }
            },
            force_authoring,
            backoff_authoring_blocks,
            keystore: params.keystore_container.keystore(),
            sync_oracle: sync_service.clone(),
            justification_sync_link: sync_service.clone(),
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
    }

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                deny_unsafe,
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
        keystore: params.keystore_container.keystore(),
        backend: backend.clone(),
        network,
        sync_service: sync_service.clone(),
        system_rpc_tx,
        tx_handler_controller,
        telemetry: telemetry.as_mut(),
    })?;

    network_starter.start_network();

    Ok((task_manager, client))
}

pub fn new_partial_solo(
    config: &Configuration,
    para_id: ParaId,
) -> Result<ServiceSolo, sc_service::Error> {
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

    let heap_pages = config
        .default_heap_pages
        .map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static {
            extra_pages: h as _,
        });

    let executor = ParachainExecutor::builder()
        .with_execution_method(config.wasm_method)
        .with_onchain_heap_alloc_strategy(heap_pages)
        .with_offchain_heap_alloc_strategy(heap_pages)
        .with_max_runtime_instances(config.max_runtime_instances)
        .with_runtime_cache_size(config.runtime_cache_size)
        .build();

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts_record_import::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
            true,
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
        let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;
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
                    let current_para_block_head = client_for_cidp
                        .expect_header(block)
                        .ok()
                        .map(|h| (polkadot_primitives::HeadData(h.encode())));
                    let additional_key_values = Some(vec![(
                        array_bytes::hex2bytes_unchecked(
                            "1cb6f36e027abb2091cfb5110ab5087f06155b3cd9a8c9e5e9a23fd5dc13a5ed",
                        ),
                        sp_consensus_aura::Slot::from_timestamp(
                            sp_timestamp::Timestamp::current(),
                            slot_duration,
                        )
                        .encode(),
                    )]);

                    async move {
                        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                        let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp,
                        slot_duration,
                    );

                        let mocked_parachain = MockValidationDataInherentDataProvider {
                            current_para_block,
                            current_para_block_head,
                            para_id,
                            relay_offset: 1000,
                            relay_blocks_per_para_block: 1,
                            xcm_config: MockXcmConfig::new(
                                &*client_for_xcm,
                                block,
                                Default::default(),
                            ),
                            raw_downward_messages: vec![],
                            raw_horizontal_messages: vec![],
                            para_blocks_per_relay_epoch: 0,
                            relay_randomness_config: (),
                            additional_key_values,
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

    Ok(PartialComponents {
        backend: backend.clone(),
        client,
        import_queue,
        keystore_container,
        task_manager,
        transaction_pool,
        select_chain: sc_consensus::LongestChain::new(backend),
        other: (telemetry, telemetry_worker_handle),
    })
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
        I: BlockImport<Block> + Send + std::marker::Sync,
    {
        type Error = I::Error;

        async fn check_block(
            &self,
            block: sc_consensus::BlockCheckParams<Block>,
        ) -> Result<sc_consensus::ImportResult, Self::Error> {
            self.0.check_block(block).await
        }

        async fn import_block(
            &mut self,
            mut block_import_params: sc_consensus::BlockImportParams<Block>,
        ) -> Result<sc_consensus::ImportResult, Self::Error> {
            block_import_params.finalized = true;
            self.0.import_block(block_import_params).await
        }
    }
}
