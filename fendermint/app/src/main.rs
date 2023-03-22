// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{path::PathBuf, str::FromStr};

use fendermint_abci::ApplicationService;
use fendermint_app::{app, store::AppStore};
use fendermint_rocksdb::RocksDb;
use fendermint_vm_interpreter::{
    bytes::BytesMessageInterpreter, chain::ChainMessageInterpreter, fvm::FvmMessageInterpreter,
    signed::SignedMessageInterpreter,
};

#[tokio::main]
async fn main() {
    let interpreter = FvmMessageInterpreter::<RocksDb>::new();
    let interpreter = SignedMessageInterpreter::new(interpreter);
    let interpreter = ChainMessageInterpreter::new(interpreter);
    let interpreter = BytesMessageInterpreter::new(interpreter);

    let db = open_db();
    let bundle_path = bundle_path();
    let app_ns = db.new_cf_handle("app").unwrap();
    let state_hist_ns = db.new_cf_handle("state_hist").unwrap();
    let app = app::App::<_, AppStore, _>::new(db, bundle_path, app_ns, state_hist_ns, interpreter);
    let _service = ApplicationService(app);
}

fn open_db() -> RocksDb {
    todo!()
}

fn bundle_path() -> PathBuf {
    let bundle_path = std::env::var("BUILTIN_ACTORS_BUNDLE")
        .unwrap_or_else(|_| "../../../builtin-actors/output/bundle.car".to_owned());

    PathBuf::from_str(&bundle_path).expect("malformed bundle path")
}

#[cfg(test)]
mod tests {
    use fendermint_rocksdb::{RocksDb, RocksDbConfig};
    use fvm_ipld_car::load_car_unchecked;

    use crate::bundle_path;

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
