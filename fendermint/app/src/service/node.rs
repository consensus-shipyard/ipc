// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, bail, Context};
use fendermint_abci::ApplicationService;
use fendermint_rocksdb::{blockstore::NamespaceBlockstore, namespaces, RocksDb, RocksDbConfig};
use fendermint_vm_interpreter::fvm::interpreter::FvmMessagesInterpreter;
use fendermint_vm_interpreter::fvm::observe::register_metrics as register_interpreter_metrics;
use fendermint_vm_interpreter::fvm::upgrades::UpgradeScheduler;
use fendermint_vm_snapshot::{SnapshotManager, SnapshotParams};
use fendermint_vm_topdown::observe::register_metrics as register_topdown_metrics;
use fvm_shared::address::{current_network, Address, Network};
use ipc_observability::observe::register_metrics as register_default_metrics;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tracing::info;

use crate::cmd::key::read_secret_key;
use crate::observe::register_metrics as register_consensus_metrics;
use crate::{App, AppConfig, AppStore};
use fendermint_app_settings::{AccountKind, Settings};

use super::topdown::start_topdown_if_enabled;
use fendermint_vm_interpreter::fvm::end_block_hook::EndBlockManager;

// Database collection names.
namespaces! {
    Namespaces {
        app,
        state_hist,
        state_store,
        bit_store
    }
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

    let topdown = start_topdown_if_enabled(
        &settings,
        &db,
        &state_store,
        ns.app.clone(),
        ns.bit_store.clone(),
        validator_keypair,
        metrics_registry.as_ref(),
    )
    .await?;

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
        topdown.manager(),
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

    topdown
        .spawn_post_init_tasks(app.clone(), tendermint_client.clone())
        .await?;

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

fn to_address(sk: &fendermint_crypto::SecretKey, kind: &AccountKind) -> anyhow::Result<Address> {
    let pk = sk.public_key().serialize();
    match kind {
        AccountKind::Regular => Ok(Address::new_secp256k1(&pk)?),
        AccountKind::Ethereum => Ok(Address::from(
            fendermint_vm_actor_interface::eam::EthAddress::new_secp256k1(&pk)?,
        )),
    }
}
