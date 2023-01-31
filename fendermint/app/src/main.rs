// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

#[tokio::main]
async fn main() {
    println!("Soon.")
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
