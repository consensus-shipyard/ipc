// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub use fendermint_app_options as options;
pub use fendermint_app_settings as settings;

mod cmd;

fn init_tracing(opts: &options::Options) {
    match &opts.log_dir {
        Some(log_dir) => {
            let file = tracing_appender::rolling::never(log_dir, "fendermint.log");

            tracing_subscriber::fmt()
                .json()
                .compact()
                .with_writer(file)
                .with_target(false)
                .with_file(true)
                .with_line_number(true)
                .with_max_level(opts.tracing_level())
                .init();
        }
        None => {
            tracing_subscriber::fmt()
                .json()
                .compact()
                .with_target(false)
                .with_file(true)
                .with_line_number(true)
                .with_max_level(opts.tracing_level())
                .init();
        }
    };
}

#[tokio::main]
async fn main() {
    let opts = options::parse();

    init_tracing(&opts);

    if let Err(e) = cmd::exec(&opts).await {
        tracing::error!("failed to execute {:?}: {e:?}", opts);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use cid::Cid;
    use fendermint_rocksdb::{RocksDb, RocksDbConfig};
    use fendermint_vm_interpreter::fvm::bundle::bundle_path;
    use fvm::machine::Manifest;
    use fvm_ipld_car::load_car_unchecked;
    use fvm_ipld_encoding::CborStore;

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

        let open_db = || {
            RocksDb::open(path.clone(), &RocksDbConfig::default()).expect("error creating RocksDB")
        };
        let db = open_db();

        let cids = load_car_unchecked(&db, bundle_car.as_slice())
            .await
            .expect("error loading bundle CAR");

        let bundle_root = cids.first().expect("there should be 1 CID");

        // Close and reopen.
        drop(db);
        let db = open_db();

        let (manifest_version, manifest_data_cid): (u32, Cid) = db
            .get_cbor(bundle_root)
            .expect("error getting bundle root")
            .expect("bundle root was not in store");

        Manifest::load(&db, &manifest_data_cid, manifest_version).expect("error loading manifest");
    }
}
