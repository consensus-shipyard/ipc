// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, bail, Context};
use async_stm::atomically_or_err;
use fendermint_abci::ApplicationService;
use fendermint_crypto::SecretKey;
use fendermint_rocksdb::{blockstore::NamespaceBlockstore, namespaces, RocksDb, RocksDbConfig};
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_interpreter::fvm::interpreter::FvmMessagesInterpreter;
use fendermint_vm_interpreter::fvm::observe::register_metrics as register_interpreter_metrics;
use fendermint_vm_interpreter::fvm::topdown::TopDownManager;
use fendermint_vm_interpreter::fvm::upgrades::UpgradeScheduler;
use fendermint_vm_snapshot::{SnapshotManager, SnapshotParams};
use fendermint_vm_topdown::observe::register_metrics as register_topdown_metrics;
use fendermint_vm_topdown::proxy::{IPCProviderProxy, IPCProviderProxyWithLatency};
use fendermint_vm_topdown::sync::launch_polling_syncer;
use fendermint_vm_topdown::voting::{publish_vote_loop, Error as VoteError, VoteTally};
use fendermint_vm_topdown::{CachedFinalityProvider, IPCParentFinality, Toggle};
use fvm_shared::address::{current_network, Address, Network};
use ipc_ipld_resolver::{Event as ResolverEvent, VoteRecord};
use ipc_observability::observe::register_metrics as register_default_metrics;
use ipc_provider::config::subnet::{EVMSubnet, SubnetConfig};
use ipc_provider::IpcProvider;
use libp2p::identity::secp256k1;
use libp2p::identity::Keypair;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tracing::info;

use crate::cmd::key::read_secret_key;
use crate::ipc::{AppParentFinalityQuery, AppVote};
use crate::observe::register_metrics as register_consensus_metrics;
use crate::{App, AppConfig, AppStore, BitswapBlockstore};
use fendermint_app_settings::{AccountKind, Settings};

use fendermint_vm_interpreter::fvm::end_block_hook::EndBlockManager;
use filecoin_f3_gpbft::PowerEntries;

// Database collection names.
namespaces! {
    Namespaces {
        app,
        state_hist,
        state_store,
        bit_store
    }
}

/// Post-`App::new()` tasks for the chosen topdown mode.
///
/// Some topdown background tasks depend on having a live `App` instance (e.g. legacy polling syncer),
/// so we collect them here and run them in one place right after app creation.
enum TopDownPostInit {
    None,
    Legacy {
        agent_proxy: Arc<IPCProviderProxyWithLatency>,
        config: fendermint_vm_topdown::Config,
        parent_finality_provider: Arc<Toggle<CachedFinalityProvider<IPCProviderProxyWithLatency>>>,
        parent_finality_votes: VoteTally,
    },
    F3 {
        proof_config: fendermint_vm_topdown_proof_service::ProofServiceConfig,
        proof_cache: Arc<fendermint_vm_topdown_proof_service::ProofCache>,
    },
}

/// Runs the ABCI server. If a CancellationToken is provided (i.e. Some(token)),
/// the server future is wrapped with cancellation logic. Otherwise, it just awaits the server future.
pub async fn run(
    settings: Settings,
    cancel_token: Option<CancellationToken>,
) -> anyhow::Result<()> {
    let tendermint_rpc_url = settings.tendermint_rpc_url()?;
    tracing::info!("Connecting to Tendermint at {tendermint_rpc_url}");

    let tendermint_client: tendermint_rpc::HttpClient =
        tendermint_rpc::HttpClient::new(tendermint_rpc_url)
            .context("failed to create Tendermint client")?;

    // Prometheus metrics
    let metrics_registry = if settings.metrics.enabled {
        let registry = prometheus::Registry::new_custom(
            Some("ipc".to_string()),
            Some([("subnet_id".to_string(), settings.ipc.subnet_id.to_string())].into()),
        )
        .context("failed to create Prometheus registry")?;

        register_default_metrics(&registry).context("failed to register default metrics")?;
        register_topdown_metrics(&registry).context("failed to register topdown metrics")?;
        register_interpreter_metrics(&registry)
            .context("failed to register interpreter metrics")?;
        register_consensus_metrics(&registry).context("failed to register consensus metrics")?;

        Some(registry)
    } else {
        None
    };

    let validator = match settings.validator_key {
        Some(ref key) => {
            let sk = key.path(settings.home_dir());
            if sk.exists() && sk.is_file() {
                let sk = read_secret_key(&sk).context("failed to read validator key")?;
                let addr = to_address(&sk, &key.kind)?;
                tracing::info!("validator key address: {addr} detected");
                Some((sk, addr))
            } else {
                bail!("validator key does not exist: {}", sk.to_string_lossy());
            }
        }
        None => {
            tracing::debug!("validator key not configured");
            None
        }
    };

    let validator_keypair = validator.as_ref().map(|(sk, _)| {
        let mut bz = sk.serialize();
        let sk = libp2p::identity::secp256k1::SecretKey::try_from_bytes(&mut bz)
            .expect("secp256k1 secret key");
        let kp = libp2p::identity::secp256k1::Keypair::from(sk);
        libp2p::identity::Keypair::from(kp)
    });

    let testing_settings = match settings.testing.as_ref() {
        Some(_) if current_network() == Network::Mainnet => {
            bail!("testing settings are not allowed on Mainnet");
        }
        other => other,
    };

    let ns = Namespaces::default();
    let db = open_db(&settings, &ns).context("error opening DB")?;

    // Blockstore for actors.
    let state_store =
        NamespaceBlockstore::new(db.clone(), ns.state_store).context("error creating state DB")?;

    let parent_finality_votes = VoteTally::empty();

    let topdown_enabled = settings.topdown_enabled();
    let topdown_config = if topdown_enabled {
        Some(
            settings
                .ipc
                .topdown_config()
                .context("topdown is enabled but topdown config is missing")?,
        )
    } else {
        None
    };

    // Decide topdown mode once.
    // - If F3 is enabled, legacy (vote-based) topdown must NOT run.
    // - If genesis state indicates F3 is enabled, config must also enable it (fail fast).
    let f3_enabled_in_config = topdown_config
        .as_ref()
        .and_then(|tc| tc.f3.as_ref())
        .is_some();

    // Query F3 state from committed/genesis state once (used for fail-fast + F3 cache init).
    let app_namespace = ns.app.clone();
    let exec_state = crate::app::create_read_only_exec_state::<_, _, AppStore>(
        &db,
        &state_store,
        app_namespace.clone(),
    )
    .context("failed to create read-only exec state")?;
    let f3_state_in_genesis = match exec_state {
        Some(mut state) => crate::app::query_f3_state(&mut state)
            .context("failed to query F3 state from genesis")?,
        None => None,
    };

    if f3_state_in_genesis.is_some() && !f3_enabled_in_config {
        bail!("F3 is enabled in genesis but not in config");
    }

    // Start the chosen topdown mode (and its background tasks) and build the TopDownManager once.
    let (top_down_manager, topdown_post_init) = if !topdown_enabled {
        let parent_finality_provider = Arc::new(Toggle::disabled());
        let top_down_manager = TopDownManager::new(
            parent_finality_provider,
            parent_finality_votes.clone(),
            None,
        );
        (top_down_manager, TopDownPostInit::None)
    } else if f3_enabled_in_config {
        let (f3_handler, proof_config, proof_cache) = start_f3_topdown(
            &settings,
            topdown_config
                .as_ref()
                .expect("topdown_config must exist when topdown is enabled"),
            f3_state_in_genesis,
        )?;

        // Legacy provider is disabled in F3 mode.
        let parent_finality_provider = Arc::new(Toggle::disabled());
        let top_down_manager = TopDownManager::new(
            parent_finality_provider,
            parent_finality_votes.clone(),
            Some(f3_handler),
        );

        (
            top_down_manager,
            TopDownPostInit::F3 {
                proof_config,
                proof_cache,
            },
        )
    } else {
        let (parent_finality_provider, ipc_tuple) = start_legacy_topdown(
            &settings,
            topdown_config
                .as_ref()
                .expect("topdown_config must exist when topdown is enabled"),
            validator_keypair,
            parent_finality_votes.clone(),
            db.clone(),
            state_store.clone(),
            ns.bit_store,
            metrics_registry.as_ref(),
        )
        .await?;

        let top_down_manager = TopDownManager::new(
            parent_finality_provider.clone(),
            parent_finality_votes.clone(),
            None,
        );

        let post_init = if let Some((agent_proxy, config)) = ipc_tuple {
            TopDownPostInit::Legacy {
                agent_proxy,
                config,
                parent_finality_provider,
                parent_finality_votes: parent_finality_votes.clone(),
            }
        } else {
            TopDownPostInit::None
        };

        (top_down_manager, post_init)
    };

    // Start a snapshot manager in the background.
    let snapshots = if settings.snapshots.enabled {
        let (manager, client) = SnapshotManager::new(
            state_store.clone(),
            SnapshotParams {
                snapshots_dir: settings.snapshots_dir(),
                download_dir: settings.snapshots.download_dir(),
                block_interval: settings.snapshots.block_interval,
                chunk_size: settings.snapshots.chunk_size_bytes,
                hist_size: settings.snapshots.hist_size,
                last_access_hold: settings.snapshots.last_access_hold,
                sync_poll_interval: settings.snapshots.sync_poll_interval,
            },
        )
        .context("failed to create snapshot manager")?;

        tracing::info!("starting the SnapshotManager...");
        let tendermint_client = tendermint_client.clone();
        tokio::spawn(async move { manager.run(tendermint_client).await });

        Some(client)
    } else {
        info!("snapshots disabled");
        None
    };

    let end_block_manager = EndBlockManager::new();
    let interpreter = FvmMessagesInterpreter::new(
        end_block_manager,
        top_down_manager,
        UpgradeScheduler::new(),
        testing_settings.is_none_or(|t| t.push_chain_meta),
        settings.abci.block_max_msgs,
        settings.fvm.gas_overestimation_rate,
        settings.fvm.gas_search_step,
    );

    let app: App<_, _, AppStore, _> = App::new(
        AppConfig {
            app_namespace: ns.app,
            state_hist_namespace: ns.state_hist,
            // keep all state history for light client validation
            state_hist_size: 0,
            halt_height: settings.halt_height,
        },
        db,
        state_store,
        interpreter,
        snapshots,
    )?;

    // Run any post-init tasks for the chosen topdown mode in one place.
    match topdown_post_init {
        TopDownPostInit::None => {}
        TopDownPostInit::F3 {
            proof_config,
            proof_cache,
        } => {
            if !proof_config.enabled {
                tracing::info!("F3 proof service disabled in configuration");
            } else {
                tracing::info!("F3 proof service enabled");

                use fendermint_vm_topdown_proof_service::ProofGeneratorService;
                let service = ProofGeneratorService::new(
                    proof_config.clone(),
                    proof_cache.clone(),
                    &settings.ipc.subnet_id,
                    0,                    // Service will fetch actual instance ID from cache
                    PowerEntries(vec![]), // Service will fetch actual power table from parent
                )
                .await
                .context("Failed to create F3 proof service")?;

                tracing::info!(
                    f3_network = proof_config.f3_network_name(&settings.ipc.subnet_id),
                    lookahead = proof_config.cache_config.lookahead_instances,
                    "F3 proof service initialized successfully"
                );

                tokio::spawn(async move {
                    service.run().await;
                });
            }
        }
        TopDownPostInit::Legacy {
            agent_proxy,
            config,
            parent_finality_provider,
            parent_finality_votes,
        } => {
            let app_parent_finality_query = AppParentFinalityQuery::new(app.clone());
            tokio::spawn(async move {
                match launch_polling_syncer(
                    app_parent_finality_query,
                    config,
                    parent_finality_provider,
                    parent_finality_votes,
                    agent_proxy,
                    tendermint_client,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => tracing::error!("cannot launch polling syncer: {e}"),
                }
            });
        }
    }

    // Start the metrics on a background thread.
    if let Some(registry) = metrics_registry {
        info!(
            listen_addr = settings.metrics.listen.to_string(),
            "serving metrics"
        );
        let mut builder = prometheus_exporter::Builder::new(settings.metrics.listen.try_into()?);
        builder.with_registry(registry);
        let _ = builder.start().context("failed to start metrics server")?;
    } else {
        info!("metrics disabled");
    }

    let service = ApplicationService(app);

    // Split it into components.
    let (consensus, mempool, snapshot, info) =
        tower_abci::split::service(service, settings.abci.bound);

    // Hand those components to the ABCI server. This is where tower layers could be added.
    // TODO: Check out the examples about load shedding in `info` requests.
    let server = tower_abci::v037::Server::builder()
        .consensus(
            // Limiting the concurrency to 1 here because the `AplicationService::poll_ready` always
            // reports `Ready`, because it doesn't know which request it's going to get.
            // Not limiting the concurrency to 1 can lead to transactions being applied
            // in different order across nodes. The buffer size has to be large enough
            // to allow all in-flight requests to not block message handling in
            // `tower_abci::Connection::run`, which could lead to deadlocks.
            // With ABCI++ we need to be able to handle all block transactions plus the begin/end/commit
            // around it. With ABCI 2.0 we'll get the block as a whole, which makes this easier.
            ServiceBuilder::new()
                .buffer(settings.abci.block_max_msgs + 3)
                .concurrency_limit(1)
                .service(consensus),
        )
        .snapshot(snapshot)
        .mempool(mempool)
        .info(info)
        .finish()
        .context("error creating ABCI server")?;

    // Run the ABCI server.
    if let Some(token) = cancel_token {
        select! {
            res = server.listen(settings.abci.listen.to_string()) => {
                res.map_err(|e| anyhow!("error listening: {e}"))?
            }
            _ = token.cancelled() => {
                info!("Cancellation requested. Shutting down ABCI server.");
            }
        }
    } else {
        server
            .listen(settings.abci.listen.to_string())
            .await
            .map_err(|e| anyhow!("error listening: {e}"))?
    }

    Ok(())
}

/// Open database with all
fn open_db(settings: &Settings, ns: &Namespaces) -> anyhow::Result<RocksDb> {
    let path = settings.data_dir().join("rocksdb");
    info!(
        path = path.to_string_lossy().into_owned(),
        "opening database"
    );
    let config = RocksDbConfig {
        compaction_style: settings.db.compaction_style.to_string(),
        ..Default::default()
    };
    let db = RocksDb::open_cf(path, &config, ns.values().iter())?;
    Ok(db)
}

fn make_resolver_service(
    settings: &Settings,
    db: RocksDb,
    state_store: NamespaceBlockstore,
    bit_store_ns: String,
) -> anyhow::Result<ipc_ipld_resolver::Service<libipld::DefaultParams, AppVote>> {
    // Blockstore for Bitswap.
    let bit_store = NamespaceBlockstore::new(db, bit_store_ns).context("error creating bit DB")?;

    // Blockstore for Bitswap with a fallback on the actor store for reads.
    let bitswap_store = BitswapBlockstore::new(state_store, bit_store);

    let config = to_resolver_config(settings).context("error creating resolver config")?;

    let service = ipc_ipld_resolver::Service::new(config, bitswap_store)
        .context("error creating IPLD Resolver Service")?;

    Ok(service)
}

fn make_ipc_provider_proxy(settings: &Settings) -> anyhow::Result<IPCProviderProxy> {
    let topdown_config = settings.ipc.topdown_config()?;
    let subnet = ipc_provider::config::Subnet {
        id: settings
            .ipc
            .subnet_id
            .parent()
            .ok_or_else(|| anyhow!("subnet has no parent"))?,
        config: SubnetConfig::Fevm(EVMSubnet {
            provider_http: topdown_config
                .parent_http_endpoint
                .to_string()
                .parse()
                .unwrap(),
            provider_timeout: topdown_config.parent_http_timeout,
            auth_token: topdown_config.parent_http_auth_token.as_ref().cloned(),
            registry_addr: topdown_config.parent_registry,
            gateway_addr: topdown_config.parent_gateway,
        }),
    };
    info!("init ipc provider with subnet: {}", subnet.id);

    let ipc_provider = IpcProvider::new_with_subnet(None, subnet)?;
    IPCProviderProxy::new(ipc_provider, settings.ipc.subnet_id.clone())
}

/// Start legacy (vote-based) topdown finality.
///
/// This is the only place where we start legacy topdown background tasks:
/// - optional resolver service
/// - optional vote gossip loop
/// - create the parent finality provider + config needed later by the polling syncer
async fn start_legacy_topdown(
    settings: &Settings,
    topdown_config: &fendermint_app_settings::TopDownSettings,
    validator_keypair: Option<libp2p::identity::Keypair>,
    parent_finality_votes: VoteTally,
    db: RocksDb,
    state_store: NamespaceBlockstore,
    bit_store_ns: String,
    metrics_registry: Option<&prometheus::Registry>,
) -> anyhow::Result<(
    Arc<Toggle<CachedFinalityProvider<IPCProviderProxyWithLatency>>>,
    Option<(
        Arc<IPCProviderProxyWithLatency>,
        fendermint_vm_topdown::Config,
    )>,
)> {
    // Resolver is optional but only meaningful for legacy mode.
    if settings.resolver_enabled() {
        let mut service = make_resolver_service(settings, db, state_store.clone(), bit_store_ns)?;

        // Register all metrics from the IPLD resolver stack
        if let Some(registry) = metrics_registry {
            service
                .register_metrics(registry)
                .context("failed to register IPLD resolver metrics")?;
        }

        let client = service.client();
        let own_subnet_id = settings.ipc.subnet_id.clone();

        client
            .add_provided_subnet(own_subnet_id.clone())
            .context("error adding own provided subnet.")?;

        if let Some(key) = validator_keypair {
            let parent_finality_votes = parent_finality_votes.clone();
            let vote_interval = settings.ipc.vote_interval;
            let vote_timeout = settings.ipc.vote_timeout;
            tracing::info!("starting the parent finality vote gossip loop...");
            tokio::spawn(async move {
                publish_vote_loop(
                    parent_finality_votes,
                    vote_interval,
                    vote_timeout,
                    key,
                    own_subnet_id,
                    client,
                    |height, block_hash| {
                        AppVote::ParentFinality(IPCParentFinality { height, block_hash })
                    },
                )
                .await
            });
        } else {
            tracing::info!("validator key missing; parent finality vote gossip disabled");
        }

        tracing::info!("subscribing to gossip...");
        let rx = service.subscribe();
        let parent_finality_votes = parent_finality_votes.clone();
        tokio::spawn(async move {
            dispatch_resolver_events(rx, parent_finality_votes).await;
        });

        tracing::info!("starting the IPLD Resolver Service...");
        tokio::spawn(async move {
            if let Err(e) = service.run().await {
                tracing::error!("IPLD Resolver Service failed: {e:#}")
            }
        });
    } else {
        tracing::info!("IPLD Resolver disabled.");
    }

    // Build legacy finality provider.
    info!("legacy topdown finality enabled");
    let mut config = fendermint_vm_topdown::Config::new(
        topdown_config.chain_head_delay,
        topdown_config.polling_interval,
        topdown_config.exponential_back_off,
        topdown_config.exponential_retry_limit,
    )
    .with_proposal_delay(topdown_config.proposal_delay)
    .with_max_proposal_range(topdown_config.max_proposal_range);

    if let Some(v) = topdown_config.max_cache_blocks {
        info!(value = v, "setting max cache blocks");
        config = config.with_max_cache_blocks(v);
    }

    let ipc_provider = {
        let p = make_ipc_provider_proxy(settings)?;
        Arc::new(IPCProviderProxyWithLatency::new(p))
    };

    let finality_provider =
        CachedFinalityProvider::uninitialized(config.clone(), ipc_provider.clone()).await?;
    let parent_finality_provider = Arc::new(Toggle::enabled(finality_provider));

    Ok((parent_finality_provider, Some((ipc_provider, config))))
}

/// Start F3 (proof-based) topdown finality.
///
/// Returns the configured handler (for `TopDownManager`) plus the proof-service config
/// used later to spawn the background proof generator.
fn start_f3_topdown(
    settings: &Settings,
    topdown_config: &fendermint_app_settings::TopDownSettings,
    f3_state_in_genesis: Option<(u64, Option<fvm_shared::clock::ChainEpoch>)>,
) -> anyhow::Result<(
    fendermint_vm_interpreter::fvm::F3FinalityHandler,
    fendermint_vm_topdown_proof_service::ProofServiceConfig,
    Arc<fendermint_vm_topdown_proof_service::ProofCache>,
)> {
    let f3_config = topdown_config
        .f3
        .as_ref()
        .context("F3 is enabled in config but missing F3 config section")?;

    let (initial_instance, initial_epoch) = match f3_state_in_genesis {
        Some((inst, Some(epoch))) => (inst, epoch),
        Some((inst, None)) => (inst, 0),
        None => bail!("F3 is enabled in config but initial F3 state is missing in genesis"),
    };

    let db_path = Some(settings.data_dir().join("proof-cache"));
    let cache = Arc::new(
        fendermint_vm_topdown_proof_service::ProofCache::new_with_persistence(
            initial_epoch,
            initial_instance,
            f3_config.proof_service.cache_config.clone(),
            db_path.as_ref().expect("db_path always set here"),
        )?,
    );

    let handler = fendermint_vm_interpreter::fvm::F3FinalityHandler::new(
        cache,
        settings.ipc.subnet_id.to_string(),
    );
    let proof_cache = handler.proof_cache().clone();

    let mut proof_config = f3_config.proof_service.clone();
    proof_config.parent_rpc_url = topdown_config.parent_http_endpoint.to_string();

    Ok((handler, proof_config, proof_cache))
}

fn to_resolver_config(settings: &Settings) -> anyhow::Result<ipc_ipld_resolver::Config> {
    use ipc_ipld_resolver::{
        Config, ConnectionConfig, ContentConfig, DiscoveryConfig, MembershipConfig, NetworkConfig,
    };

    let r = &settings.resolver;

    let local_key: Keypair = {
        let path = r.network.local_key(settings.home_dir());
        let sk = read_secret_key(&path)?;
        let sk = secp256k1::SecretKey::try_from_bytes(sk.serialize())?;
        secp256k1::Keypair::from(sk).into()
    };

    let network_name = format!(
        "ipld-resolver-{}-{}",
        settings.ipc.subnet_id.root_id(),
        r.network.network_name
    );

    let config = Config {
        connection: ConnectionConfig {
            listen_addr: r.connection.listen_addr.clone(),
            external_addresses: r.connection.external_addresses.clone(),
            expected_peer_count: r.connection.expected_peer_count,
            max_incoming: r.connection.max_incoming,
            max_peers_per_query: r.connection.max_peers_per_query,
            event_buffer_capacity: r.connection.event_buffer_capacity,
        },
        network: NetworkConfig {
            local_key,
            network_name,
        },
        discovery: DiscoveryConfig {
            static_addresses: r.discovery.static_addresses.clone(),
            target_connections: r.discovery.target_connections,
            enable_kademlia: r.discovery.enable_kademlia,
        },
        membership: MembershipConfig {
            static_subnets: r.membership.static_subnets.clone(),
            max_subnets: r.membership.max_subnets,
            publish_interval: r.membership.publish_interval,
            min_time_between_publish: r.membership.min_time_between_publish,
            max_provider_age: r.membership.max_provider_age,
        },
        content: ContentConfig {
            rate_limit_bytes: r.content.rate_limit_bytes,
            rate_limit_period: r.content.rate_limit_period,
        },
    };

    Ok(config)
}

fn to_address(sk: &SecretKey, kind: &AccountKind) -> anyhow::Result<Address> {
    let pk = sk.public_key().serialize();
    match kind {
        AccountKind::Regular => Ok(Address::new_secp256k1(&pk)?),
        AccountKind::Ethereum => Ok(Address::from(EthAddress::new_secp256k1(&pk)?)),
    }
}

async fn dispatch_resolver_events(
    mut rx: tokio::sync::broadcast::Receiver<ResolverEvent<AppVote>>,
    parent_finality_votes: VoteTally,
) {
    loop {
        match rx.recv().await {
            Ok(event) => match event {
                ResolverEvent::ReceivedPreemptive(_, _) => {}
                ResolverEvent::ReceivedVote(vote) => {
                    dispatch_vote(*vote, &parent_finality_votes).await;
                }
            },
            Err(RecvError::Lagged(n)) => {
                tracing::warn!("the resolver service skipped {n} gossip events")
            }
            Err(RecvError::Closed) => {
                tracing::error!("the resolver service stopped receiving gossip");
                return;
            }
        }
    }
}

async fn dispatch_vote(vote: VoteRecord<AppVote>, parent_finality_votes: &VoteTally) {
    match vote.content {
        AppVote::ParentFinality(f) => {
            let res = atomically_or_err(|| {
                parent_finality_votes.add_vote(
                    vote.public_key.clone(),
                    f.height,
                    f.block_hash.clone(),
                )
            })
            .await;

            match res {
                Err(e @ VoteError::Equivocation(_, _, _, _)) => {
                    tracing::warn!(error = e.to_string(), "failed to handle vote");
                }
                Err(e @ (
                VoteError::Uninitialized // early vote, we're not ready yet
                | VoteError::UnpoweredValidator(_) // maybe arrived too early or too late, or spam
                | VoteError::UnexpectedBlock(_, _) // won't happen here
                )) => {
                    tracing::debug!(error = e.to_string(), "failed to handle vote");
                }
                _ => {
                    tracing::debug!("vote handled");
                }
            };
        }
    }
}
