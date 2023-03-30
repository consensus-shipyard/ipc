// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use fendermint_abci::ApplicationService;
use fendermint_app::{App, AppStore};
use fendermint_rocksdb::{namespaces, RocksDb, RocksDbConfig};
use fendermint_vm_interpreter::{
    bytes::BytesMessageInterpreter, chain::ChainMessageInterpreter, fvm::FvmMessageInterpreter,
    signed::SignedMessageInterpreter,
};
use tracing::info;

use crate::{cmd, options::RunArgs, settings::Settings};

cmd! {
  RunArgs(self, settings) {
    run(settings).await
  }
}

async fn run(settings: Settings) -> anyhow::Result<()> {
    let interpreter = FvmMessageInterpreter::<RocksDb>::new();
    let interpreter = SignedMessageInterpreter::new(interpreter);
    let interpreter = ChainMessageInterpreter::new(interpreter);
    let interpreter = BytesMessageInterpreter::new(interpreter);

    let ns = Namespaces::default();
    let db = open_db(&settings, &ns).context("error opening DB")?;

    let app: App<_, AppStore, _> = App::new(
        db,
        settings.builtin_actors_bundle(),
        ns.app,
        ns.state_hist,
        settings.db.state_hist_size,
        interpreter,
    )?;

    let service = ApplicationService(app);

    // Split it into components.
    let (consensus, mempool, snapshot, info) =
        tower_abci::split::service(service, settings.abci.bound);

    // Hand those components to the ABCI server. This is where tower layers could be added.
    let server = tower_abci::Server::builder()
        .consensus(consensus)
        .snapshot(snapshot)
        .mempool(mempool)
        .info(info)
        .finish()
        .context("error creating ABCI server")?;

    // Run the ABCI server.
    server
        .listen(settings.abci.listen_addr())
        .await
        .map_err(|e| anyhow!("error listening: {e}"))?;

    Ok(())
}

namespaces! {
    Namespaces {
        app,
        state_hist
    }
}

fn open_db(settings: &Settings, ns: &Namespaces) -> anyhow::Result<RocksDb> {
    let path = settings.data_dir().join("rocksdb");
    info!(
        path = path.to_string_lossy().into_owned(),
        "opening database"
    );
    let db = RocksDb::open(path, &RocksDbConfig::default())?;

    // Filter names first, then create, which is just a way to catch duplicates.
    let mut names = ns.values();
    names.retain(|name| !db.has_cf_handle(name));
    for name in names {
        db.new_cf_handle(name)?;
    }

    Ok(db)
}
