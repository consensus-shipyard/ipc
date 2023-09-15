// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use fendermint_abci::ApplicationService;
use fendermint_app::{App, AppConfig, AppStore, BitswapBlockstore};
use fendermint_rocksdb::{blockstore::NamespaceBlockstore, namespaces, RocksDb, RocksDbConfig};
use fendermint_vm_interpreter::{
    bytes::{BytesMessageInterpreter, ProposalPrepareMode},
    chain::{ChainMessageInterpreter, CheckpointPool},
    fvm::FvmMessageInterpreter,
    signed::SignedMessageInterpreter,
};
use fendermint_vm_resolver::ipld::IpldResolver;
use libp2p::identity::secp256k1;
use libp2p::identity::Keypair;
use tracing::info;

use crate::cmd::key::read_secret_key;
use crate::{cmd, options::run::RunArgs, settings::Settings};

cmd! {
  RunArgs(self, settings) {
    run(settings).await
  }
}

/// Run the Fendermint ABCI Application.
///
/// This method acts as our composition root.
async fn run(settings: Settings) -> anyhow::Result<()> {
    let client = tendermint_rpc::HttpClient::new(settings.tendermint_rpc_url()?)
        .context("failed to create Tendermint client")?;

    let validator_key = {
        let sk = settings.validator_key();
        if sk.exists() && sk.is_file() {
            Some(read_secret_key(&sk).context("failed to read validator key")?)
        } else {
            tracing::debug!("validator key not configured");
            None
        }
    };

    let interpreter = FvmMessageInterpreter::<NamespaceBlockstore, _>::new(
        client,
        validator_key,
        settings.contracts_dir(),
        settings.fvm.gas_overestimation_rate,
        settings.fvm.gas_search_step,
        settings.fvm.exec_in_check,
    );
    let interpreter = SignedMessageInterpreter::new(interpreter);
    let interpreter = ChainMessageInterpreter::new(interpreter);
    let interpreter =
        BytesMessageInterpreter::new(interpreter, ProposalPrepareMode::AppendOnly, false);

    let ns = Namespaces::default();
    let db = open_db(&settings, &ns).context("error opening DB")?;

    // Blockstore for actors.
    let state_store =
        NamespaceBlockstore::new(db.clone(), ns.state_store).context("error creating state DB")?;

    let resolve_pool = CheckpointPool::new();

    if settings.resolver_enabled() {
        let service =
            make_resolver_service(&settings, db.clone(), state_store.clone(), ns.bit_store)?;

        let client = service.client();

        let own_subnet_id = settings.resolver.subnet_id.clone();

        client
            .add_provided_subnet(own_subnet_id.clone())
            .context("error adding own provided subnet.")?;

        let resolver = IpldResolver::new(
            client,
            resolve_pool.queue(),
            settings.resolver.retry_delay,
            own_subnet_id,
        );

        tracing::info!("starting the IPLD Resolver Service...");
        tokio::spawn(async move {
            if let Err(e) = service.run().await {
                tracing::error!("IPLD Resolver Service failed: {e:#}")
            }
        });

        tracing::info!("starting the IPLD Resolver...");
        tokio::spawn(async move { resolver.run().await });
    } else {
        tracing::info!("IPLD Resolver disabled.")
    }

    let app: App<_, _, AppStore, _> = App::new(
        AppConfig {
            app_namespace: ns.app,
            state_hist_namespace: ns.state_hist,
            state_hist_size: settings.db.state_hist_size,
            builtin_actors_bundle: settings.builtin_actors_bundle(),
        },
        db,
        state_store,
        interpreter,
        resolve_pool,
    )?;

    let service = ApplicationService(app);

    // Split it into components.
    let (consensus, mempool, snapshot, info) =
        tower_abci::split::service(service, settings.abci.bound);

    // Hand those components to the ABCI server. This is where tower layers could be added.
    let server = tower_abci::v037::Server::builder()
        .consensus(consensus)
        .snapshot(snapshot)
        .mempool(mempool)
        .info(info)
        .finish()
        .context("error creating ABCI server")?;

    // Run the ABCI server.
    server
        .listen(settings.abci.listen.addr())
        .await
        .map_err(|e| anyhow!("error listening: {e}"))?;

    Ok(())
}

namespaces! {
    Namespaces {
        app,
        state_hist,
        state_store,
        bit_store
    }
}

/// Open database with all
fn open_db(settings: &Settings, ns: &Namespaces) -> anyhow::Result<RocksDb> {
    let path = settings.data_dir().join("rocksdb");
    info!(
        path = path.to_string_lossy().into_owned(),
        "opening database"
    );
    let db = RocksDb::open_cf(path, &RocksDbConfig::default(), ns.values().iter())?;
    Ok(db)
}

fn make_resolver_service(
    settings: &Settings,
    db: RocksDb,
    state_store: NamespaceBlockstore,
    bit_store_ns: String,
) -> anyhow::Result<ipc_ipld_resolver::Service<libipld::DefaultParams>> {
    // Blockstore for Bitswap.
    let bit_store = NamespaceBlockstore::new(db, bit_store_ns).context("error creating bit DB")?;

    // Blockstore for Bitswap with a fallback on the actor store for reads.
    let bitswap_store = BitswapBlockstore::new(state_store, bit_store);

    let config = to_resolver_config(settings).context("error creating resolver config")?;

    let service = ipc_ipld_resolver::Service::new(config, bitswap_store)
        .context("error creating IPLD Resolver Service")?;

    Ok(service)
}

fn to_resolver_config(settings: &Settings) -> anyhow::Result<ipc_ipld_resolver::Config> {
    use ipc_ipld_resolver::{
        Config, ConnectionConfig, ContentConfig, DiscoveryConfig, MembershipConfig, NetworkConfig,
    };

    let r = &settings.resolver;

    let local_key = {
        let path = r.network.local_key(settings.home_dir());
        let sk = read_secret_key(&path)?;
        let sk = secp256k1::SecretKey::from_bytes(sk.serialize())?;
        Keypair::Secp256k1(secp256k1::Keypair::from(sk))
    };

    let network_name = format!(
        "ipld-resolver-{}-{}",
        r.subnet_id.root_id(),
        r.network.network_name
    );

    let config = Config {
        connection: ConnectionConfig {
            listen_addr: r.connection.listen_addr.clone(),
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
