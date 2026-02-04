// Copyright 2022-2026 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::sync::Arc;

use anyhow::{anyhow, bail, Context};
use async_stm::atomically_or_err;
use fendermint_rocksdb::blockstore::NamespaceBlockstore;
use fendermint_rocksdb::RocksDb;
use fendermint_vm_interpreter::fvm::interpreter::FvmMessagesInterpreter;
use fendermint_vm_interpreter::fvm::topdown::TopDownManager;
use fendermint_vm_interpreter::fvm::LegacyTopDownHandler;
use fendermint_vm_topdown::proxy::{IPCProviderProxy, IPCProviderProxyWithLatency};
use fendermint_vm_topdown::sync::launch_polling_syncer;
use fendermint_vm_topdown::voting::{publish_vote_loop, Error as VoteError, VoteTally};
use fendermint_vm_topdown::{CachedFinalityProvider, IPCParentFinality, Toggle};
use ipc_api::subnet_id::SubnetID;
use ipc_ipld_resolver::{Event as ResolverEvent, VoteRecord};
use ipc_provider::config::subnet::{EVMSubnet, SubnetConfig};
use ipc_provider::IpcProvider;
use libp2p::identity::secp256k1;
use libp2p::identity::Keypair;
use tokio::sync::broadcast::error::RecvError;

use crate::cmd::key::read_secret_key;
use crate::ipc::AppParentFinalityQuery;
use crate::ipc::AppVote;
use crate::{App, AppStore, BitswapBlockstore};
use fendermint_app_settings::{Settings, TopDownSettings};
use fendermint_storage::KVStore;

type TopDownFinalityProvider = Arc<Toggle<CachedFinalityProvider<IPCProviderProxyWithLatency>>>;

/// Legacy topdown background tasks which require a live `App` instance.
struct LegacyPostInit {
    agent_proxy: Arc<IPCProviderProxyWithLatency>,
    config: fendermint_vm_topdown::Config,
    parent_finality_provider: TopDownFinalityProvider,
    parent_finality_votes: VoteTally,
}

/// Result of topdown initialization performed before `App::new()`.
pub(super) struct TopDownInit {
    manager: TopDownManager<NamespaceBlockstore>,
    legacy_post_init: Option<LegacyPostInit>,
}

impl TopDownInit {
    pub(super) fn manager(&self) -> TopDownManager<NamespaceBlockstore> {
        self.manager.clone()
    }

    pub(super) async fn spawn_legacy_polling_syncer_if_needed(
        self,
        app: App<
            RocksDb,
            NamespaceBlockstore,
            AppStore,
            FvmMessagesInterpreter<NamespaceBlockstore>,
        >,
        tendermint_client: tendermint_rpc::HttpClient,
    ) -> anyhow::Result<()> {
        if let Some(p) = self.legacy_post_init {
            let app_parent_finality_query = AppParentFinalityQuery::new(app);
            tokio::spawn(async move {
                match launch_polling_syncer(
                    app_parent_finality_query,
                    p.config,
                    p.parent_finality_provider,
                    p.parent_finality_votes,
                    p.agent_proxy,
                    tendermint_client,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => tracing::error!("cannot launch polling syncer: {e}"),
                }
            });
        }
        Ok(())
    }
}

/// Initialize topdown (legacy or F3) before creating the `App`.
///
/// Returns the `TopDownManager` to be put into the interpreter, and a `TopDownInit` handle
/// with any required post-`App::new()` work.
pub(super) async fn start_topdown_if_enabled(
    settings: &Settings,
    db: &RocksDb,
    state_store: &NamespaceBlockstore,
    app_namespace: <AppStore as KVStore>::Namespace,
    bit_store_namespace: String,
    validator_keypair: Option<libp2p::identity::Keypair>,
    metrics_registry: Option<&prometheus::Registry>,
) -> anyhow::Result<TopDownInit> {
    // If topdown is disabled, return a disabled topdown manager and no post-init tasks.
    if !settings.topdown_enabled() {
        return Ok(TopDownInit {
            manager: TopDownManager::disabled(),
            legacy_post_init: None,
        });
    }

    let topdown_config = settings
        .ipc
        .topdown_config()
        .context("topdown is enabled but topdown config is missing")?;

    let f3_enabled_in_config = topdown_config.f3.is_some();
    let f3_state_in_committed_state =
        query_f3_state_in_committed_state(db, state_store, app_namespace.clone())?;
    let gateway_finality_in_committed_state =
        query_gateway_parent_finality_in_committed_state(db, state_store, app_namespace.clone())?;
    let gateway_event_cursor_in_committed_state =
        query_gateway_event_cursor_in_committed_state(db, state_store, app_namespace.clone())?;

    // Fail-fast consistency between config and committed state.
    //
    // - If committed state has F3 state, config must enable F3.
    // - If config enables F3, committed state must have initial F3 state.
    if f3_state_in_committed_state.is_some() && !f3_enabled_in_config {
        bail!("F3 is enabled in committed state but not in config");
    }
    if f3_enabled_in_config && f3_state_in_committed_state.is_none() {
        bail!("F3 is enabled in config but initial F3 state is missing in committed state");
    }

    if f3_enabled_in_config {
        if gateway_finality_in_committed_state.is_none() {
            bail!("F3 is enabled but gateway latest parent finality is missing in committed state");
        }
        return start_f3_topdown(
            settings,
            topdown_config,
            f3_state_in_committed_state,
            gateway_finality_in_committed_state,
            gateway_event_cursor_in_committed_state,
        )
        .await;
    }

    start_legacy_topdown(
        settings,
        topdown_config,
        validator_keypair,
        db.clone(),
        state_store.clone(),
        bit_store_namespace,
        metrics_registry,
    )
    .await
}

#[derive(Debug, Clone, Copy)]
struct GatewayEventCursor {
    applied_top_down_nonce: u64,
    next_power_change_config_number: u64,
}

fn query_f3_state_in_committed_state(
    db: &RocksDb,
    state_store: &NamespaceBlockstore,
    app_namespace: <AppStore as KVStore>::Namespace,
) -> anyhow::Result<Option<fendermint_vm_actor_interface::f3_light_client::GetStateResponse>> {
    // Query F3 state from committed state once (used for fail-fast + F3 cache init).
    let exec_state =
        crate::app::create_read_only_exec_state::<_, _, AppStore>(db, state_store, app_namespace)
            .context("failed to create read-only exec state")?;

    let f3_state_in_committed_state = match exec_state {
        Some(mut state) => crate::app::query_f3_state(&mut state)
            .context("failed to query F3 state from committed state")?,
        None => None,
    };

    Ok(f3_state_in_committed_state)
}

fn query_gateway_parent_finality_in_committed_state(
    db: &RocksDb,
    state_store: &NamespaceBlockstore,
    app_namespace: <AppStore as KVStore>::Namespace,
) -> anyhow::Result<Option<IPCParentFinality>> {
    type ROStore = fendermint_vm_interpreter::fvm::store::ReadOnlyBlockstore<
        std::sync::Arc<NamespaceBlockstore>,
    >;
    // Query the gateway's latest parent finality from committed/genesis state once.
    let exec_state =
        crate::app::create_read_only_exec_state::<_, _, AppStore>(db, state_store, app_namespace)
            .context("failed to create read-only exec state")?;

    let latest = match exec_state {
        Some(mut state) => {
            let gw =
                fendermint_vm_interpreter::fvm::state::ipc::GatewayCaller::<ROStore>::default();
            Some(gw.get_latest_parent_finality(&mut state)?)
        }
        None => None,
    };

    Ok(latest)
}

fn query_gateway_event_cursor_in_committed_state(
    db: &RocksDb,
    state_store: &NamespaceBlockstore,
    app_namespace: <AppStore as KVStore>::Namespace,
) -> anyhow::Result<Option<GatewayEventCursor>> {
    type ROStore = fendermint_vm_interpreter::fvm::store::ReadOnlyBlockstore<
        std::sync::Arc<NamespaceBlockstore>,
    >;
    let exec_state =
        crate::app::create_read_only_exec_state::<_, _, AppStore>(db, state_store, app_namespace)
            .context("failed to create read-only exec state")?;

    let cursor = match exec_state {
        Some(mut state) => {
            let gw =
                fendermint_vm_interpreter::fvm::state::ipc::GatewayCaller::<ROStore>::default();
            let applied_top_down_nonce = gw.applied_top_down_nonce(&mut state)?;
            let (next_cfg, _start_cfg) = gw.tracker_configuration_numbers(&mut state)?;
            Some(GatewayEventCursor {
                applied_top_down_nonce,
                next_power_change_config_number: next_cfg,
            })
        }
        None => None,
    };

    Ok(cursor)
}

fn make_resolver_service(
    settings: &Settings,
    db: RocksDb,
    state_store: NamespaceBlockstore,
    bit_store_ns: String,
) -> anyhow::Result<ipc_ipld_resolver::Service<libipld::DefaultParams, AppVote>> {
    let bit_store = NamespaceBlockstore::new(db, bit_store_ns).context("error creating bit DB")?;
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

    tracing::info!("init ipc provider with subnet: {}", subnet.id);
    let ipc_provider = IpcProvider::new_with_subnet(None, subnet)?;
    IPCProviderProxy::new(ipc_provider, settings.ipc.subnet_id.clone())
}

async fn start_legacy_topdown(
    settings: &Settings,
    topdown_config: &TopDownSettings,
    validator_keypair: Option<libp2p::identity::Keypair>,
    db: RocksDb,
    state_store: NamespaceBlockstore,
    bit_store_ns: String,
    metrics_registry: Option<&prometheus::Registry>,
) -> anyhow::Result<TopDownInit> {
    let parent_finality_votes = VoteTally::empty();
    // Resolver is required for legacy mode (vote gossip + quorum collection).
    if !settings.resolver_enabled() {
        bail!("IPLD Resolver is disabled but legacy topdown is enabled");
    }

    let mut service = make_resolver_service(settings, db, state_store.clone(), bit_store_ns)?;

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

    // NOTE: Legacy topdown can run in a non-validator mode.
    //
    // Non-validator nodes should still start up and subscribe to votes (so they can
    // observe quorum and execute committed checkpoints), but they cannot *publish*
    // votes without a validator keypair.
    if let Some(key) = validator_keypair {
        let parent_finality_votes_for_votes_loop = parent_finality_votes.clone();
        let vote_interval = settings.ipc.vote_interval;
        let vote_timeout = settings.ipc.vote_timeout;
        let own_subnet_id_for_votes_loop = own_subnet_id.clone();
        let client_for_votes_loop = client.clone();

        tracing::info!("starting the parent finality vote gossip loop...");
        tokio::spawn(async move {
            publish_vote_loop(
                parent_finality_votes_for_votes_loop,
                vote_interval,
                vote_timeout,
                key,
                own_subnet_id_for_votes_loop,
                client_for_votes_loop,
                |height, block_hash| {
                    AppVote::ParentFinality(IPCParentFinality { height, block_hash })
                },
            )
            .await
        });
    } else {
        tracing::warn!(
            "validator key missing; legacy topdown enabled but vote publishing is disabled (non-validator mode)"
        );
    }

    tracing::info!("subscribing to gossip...");
    let rx = service.subscribe();
    let parent_finality_votes_for_resolver = parent_finality_votes.clone();
    tokio::spawn(async move {
        dispatch_resolver_events(rx, parent_finality_votes_for_resolver).await;
    });

    tracing::info!("starting the IPLD Resolver Service...");
    tokio::spawn(async move {
        if let Err(e) = service.run().await {
            tracing::error!("IPLD Resolver Service failed: {e:#}")
        }
    });

    tracing::info!("legacy topdown finality enabled");

    let mut config = fendermint_vm_topdown::Config::new(
        topdown_config.chain_head_delay,
        topdown_config.polling_interval,
        topdown_config.exponential_back_off,
        topdown_config.exponential_retry_limit,
    )
    .with_proposal_delay(topdown_config.proposal_delay)
    .with_max_proposal_range(topdown_config.max_proposal_range);

    if let Some(v) = topdown_config.max_cache_blocks {
        tracing::info!(value = v, "setting max cache blocks");
        config = config.with_max_cache_blocks(v);
    }

    let ipc_provider = {
        let p = make_ipc_provider_proxy(settings)?;
        Arc::new(IPCProviderProxyWithLatency::new(p))
    };

    let finality_provider =
        CachedFinalityProvider::uninitialized(config.clone(), ipc_provider.clone()).await?;

    let parent_finality_provider: TopDownFinalityProvider =
        Arc::new(Toggle::enabled(finality_provider));

    let manager = TopDownManager::legacy(LegacyTopDownHandler::new(
        parent_finality_provider.clone(),
        parent_finality_votes.clone(),
    ));

    Ok(TopDownInit {
        manager,
        legacy_post_init: Some(LegacyPostInit {
            agent_proxy: ipc_provider,
            config,
            parent_finality_provider,
            parent_finality_votes: parent_finality_votes.clone(),
        }),
    })
}

async fn start_f3_topdown(
    settings: &Settings,
    topdown_config: &TopDownSettings,
    f3_state_in_committed_state: Option<
        fendermint_vm_actor_interface::f3_light_client::GetStateResponse,
    >,
    gateway_finality_in_committed_state: Option<IPCParentFinality>,
    gateway_event_cursor_in_committed_state: Option<GatewayEventCursor>,
) -> anyhow::Result<TopDownInit> {
    let f3_config = topdown_config
        .f3
        .as_ref()
        .context("F3 is enabled in config but missing F3 config section")?;

    let f3_state = f3_state_in_committed_state
        .context("F3 is enabled in config but initial F3 state is missing in committed state")?;
    let initial_instance = f3_state.processed_instance_id;
    // Epoch cursor comes from the gateway contract (seeded at genesis).
    let initial_epoch = gateway_finality_in_committed_state
        .context("F3 enabled but gateway latest parent finality missing in committed state")?
        .height as fvm_shared::clock::ChainEpoch;
    let gateway_cursor = gateway_event_cursor_in_committed_state
        .context("F3 enabled but gateway event cursor missing in committed state")?;

    let db_path = Some(settings.data_dir().join("proof-cache"));
    let cache = Arc::new(
        fendermint_vm_topdown_proof_service::ProofCache::new_with_persistence(
            initial_epoch,
            initial_instance,
            f3_config.proof_service.cache_config.clone(),
            db_path.as_ref().expect("db_path always set here"),
        )?,
    );

    let handler = fendermint_vm_interpreter::fvm::F3TopDownHandler::new(cache);
    let proof_cache = handler.proof_cache().clone();

    let mut proof_config = f3_config.proof_service.clone();
    proof_config.parent_rpc_url = topdown_config.parent_http_endpoint.to_string();

    if !proof_config.enabled {
        tracing::info!("F3 proof service disabled in configuration");
    } else {
        tracing::info!("F3 proof service enabled");

        use fendermint_vm_topdown_proof_service::ProofGeneratorService;
        let subnet_id: SubnetID = settings.ipc.subnet_id.clone();
        let service = ProofGeneratorService::new(
            proof_config.clone(),
            proof_cache.clone(),
            &subnet_id,
            initial_instance,
            fendermint_vm_topdown_proof_service::power_entries_from_actor(&f3_state.power_table),
            gateway_cursor.applied_top_down_nonce,
            gateway_cursor.next_power_change_config_number,
        )
        .await
        .context("Failed to create F3 proof service")?;

        tracing::info!(
            f3_network = proof_config.f3_network_name(&subnet_id),
            lookahead = proof_config.cache_config.lookahead_instances,
            "F3 proof service initialized successfully"
        );

        tokio::spawn(async move {
            service.run().await;
        });
    }

    Ok(TopDownInit {
        manager: TopDownManager::f3_with_retry_config(
            handler,
            fendermint_vm_interpreter::fvm::topdown::F3ExecutionCacheRetryConfig {
                backoff_initial: f3_config.execution_cache_retry.backoff_initial,
                backoff_max: f3_config.execution_cache_retry.backoff_max,
                critical_after: f3_config.execution_cache_retry.critical_after,
                error_after: f3_config.execution_cache_retry.error_after,
            },
        ),
        legacy_post_init: None,
    })
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

    Ok(Config {
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
    })
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
                Err(
                    e @ (
                        VoteError::Uninitialized // early vote, we're not ready yet
                        | VoteError::UnpoweredValidator(_) // maybe arrived too early or too late, or spam
                        | VoteError::UnexpectedBlock(_, _) // won't happen here
                    ),
                ) => {
                    tracing::debug!(error = e.to_string(), "failed to handle vote");
                }
                _ => {
                    tracing::debug!("vote handled");
                }
            };
        }
    }
}
