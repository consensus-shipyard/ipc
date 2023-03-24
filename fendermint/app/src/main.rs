// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use clap::Parser;
use fendermint_abci::ApplicationService;
use fendermint_app::{
    options::{Command, Options},
    settings::Settings,
    App, AppStore,
};
use fendermint_rocksdb::{RocksDb, RocksDbConfig};
use fendermint_vm_interpreter::{
    bytes::BytesMessageInterpreter, chain::ChainMessageInterpreter, fvm::FvmMessageInterpreter,
    signed::SignedMessageInterpreter,
};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let opts = Options::parse();

    // Log events to stdout.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(opts.tracing_level())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    if let Some(Command::Run { ref mode }) = opts.command {
        let config_dir = match opts.config_dir() {
            Some(d) if d.is_dir() => d,
            Some(d) if d.exists() => panic!("config '{d:?}' is a not a directory"),
            Some(d) => panic!("config '{d:?}' does not exist"),
            None => panic!("could not find a config directory to use"),
        };

        let settings = Settings::new(config_dir, mode).expect("error parsing settings");

        let interpreter = FvmMessageInterpreter::<RocksDb>::new();
        let interpreter = SignedMessageInterpreter::new(interpreter);
        let interpreter = ChainMessageInterpreter::new(interpreter);
        let interpreter = BytesMessageInterpreter::new(interpreter);

        let db = open_db(&settings).expect("error opening DB");
        let app_ns = db.new_cf_handle("app").unwrap();
        let state_hist_ns = db.new_cf_handle("state_hist").unwrap();

        let app = App::<_, AppStore, _>::new(
            db,
            settings.builtin_actors_bundle,
            app_ns,
            state_hist_ns,
            interpreter,
        );

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
            .expect("error creating ABCI server");

        // Run the ABCI server.
        server
            .listen(settings.abci.listen_addr())
            .await
            .expect("error listening to ABCI requests");
    }
}

fn open_db(settings: &Settings) -> anyhow::Result<RocksDb> {
    let path = settings.data_dir.join("rocksdb");
    let db = RocksDb::open(path, &RocksDbConfig::default())?;
    Ok(db)
}

#[cfg(test)]
mod tests {
    use fendermint_rocksdb::{RocksDb, RocksDbConfig};
    use fendermint_vm_interpreter::fvm::bundle::bundle_path;
    use fvm_ipld_car::load_car_unchecked;

    #[tokio::test]
    async fn load_car() {
        // Just to see if dependencies compile together, see if we can load an actor bundle into a temporary RocksDB.
        // Run it with `cargo run -p fendermint_app`

        // Not loading the actors from the library any more. It would be possible, as long as dependencies are aligned.
        // let bundle_car = actors_v10::BUNDLE_CAR;

        let bundle_path = bundle_path();
        let bundle_car = std::fs::read(&bundle_path)
            .unwrap_or_else(|_| panic!("failed to load bundle CAR from {bundle_path:?}"));

        let dir = tempfile::Builder::new()
            .tempdir()
            .expect("error creating temporary path for db");
        let path = dir.path().join("rocksdb");
        let db =
            RocksDb::open(path.clone(), &RocksDbConfig::default()).expect("error creating RocksDB");

        let _cids = load_car_unchecked(&db, bundle_car.as_slice())
            .await
            .expect("error loading bundle CAR");
    }
}
