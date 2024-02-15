// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub use fendermint_app_options as options;
pub use fendermint_app_settings as settings;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    fmt::{self, writer::MakeWriterExt},
    layer::SubscriberExt,
};
mod cmd;

fn init_tracing(opts: &options::Options) -> Option<WorkerGuard> {
    let mut guard = None;

    let Some(log_level) = opts.tracing_level() else {
        return guard;
    };

    let registry = tracing_subscriber::registry();

    // add a file layer if log_dir is set
    let registry = registry.with(if let Some(log_dir) = &opts.log_dir {
        let filename = match &opts.log_file_prefix {
            Some(prefix) => format!("{}-{}", prefix, "fendermint"),
            None => "fendermint".to_string(),
        };

        let appender = RollingFileAppender::builder()
            .filename_prefix(filename)
            .filename_suffix("log")
            .rotation(Rotation::DAILY)
            .max_log_files(5)
            .build(log_dir)
            .expect("failed to initialize rolling file appender");

        let (non_blocking, g) = tracing_appender::non_blocking(appender);
        guard = Some(g);

        Some(
            fmt::Layer::new()
                .json()
                .with_writer(non_blocking.with_max_level(log_level))
                .with_target(false)
                .with_file(true)
                .with_line_number(true),
        )
    } else {
        None
    });

    // we also log all traces with level INFO or higher to stdout
    let registry = registry.with(
        tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout.with_max_level(tracing::Level::INFO))
            .with_target(false)
            .with_file(true)
            .with_line_number(true),
    );

    tracing::subscriber::set_global_default(registry).expect("Unable to set a global collector");

    guard
}
#[tokio::main]
async fn main() {
    let opts = options::parse();

    let _guard = init_tracing(&opts);

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
