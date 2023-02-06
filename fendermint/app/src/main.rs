// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::sync::Arc;

use fendermint_abci::ApplicationService;
use fendermint_app::app;
use fendermint_vm_interpreter::{
    bytes::BytesMessageInterpreter, chain::ChainMessageInterpreter, fvm::FvmMessageInterpreter,
    signed::SignedMessageInterpreter,
};
use forest_db::rocks::RocksDb;

#[tokio::main]
async fn main() {
    let interpreter = FvmMessageInterpreter::<RocksDb>::new();
    let interpreter = SignedMessageInterpreter::new(interpreter);
    let interpreter = ChainMessageInterpreter::new(interpreter);
    let interpreter = BytesMessageInterpreter::new(interpreter);

    let db = open_db();
    let app = app::App::new(db, interpreter);
    let _service = ApplicationService(app);
}

fn open_db() -> Arc<RocksDb> {
    todo!()
}

#[cfg(test)]
mod tests {
    use forest_db::rocks::RocksDb;
    use forest_db::rocks_config::RocksDbConfig;
    use fvm_ipld_car::load_car_unchecked;

    #[tokio::test]
    async fn load_car() {
        // Just to see if dependencies compile together, see if we can load an actor bundle into a temporary RocksDB.
        // Run it with `cargo run -p fendermint_app`

        let bundle = actors_v10::BUNDLE_CAR;

        let dir = tempfile::Builder::new()
            .tempdir()
            .expect("error creating temporary path for db");
        let path = dir.path().join("rocksdb");
        let db =
            RocksDb::open(path.clone(), &RocksDbConfig::default()).expect("error creating RocksDB");

        let _cids = load_car_unchecked(&db, bundle)
            .await
            .expect("error loading bundle CAR");
    }
}
