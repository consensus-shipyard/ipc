// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use color_eyre::eyre::{bail, Result};
use dagger_sdk::{
    logging::StdLogger, Container, ContainerWithDirectoryOptsBuilder,
    ContainerWithEnvVariableOptsBuilder, ContainerWithExecOptsBuilder,
    ContainerWithFileOptsBuilder, DaggerConn, Directory, HostDirectoryOpts,
};
use fs_err as fs;
use std::future::Future;
use std::sync::Arc;

mod minimal;
use self::minimal::*;

/// Execute the lazily prepared container definition and convert to a `Result`
async fn run(container: &Container) -> Result<()> {
    let out_fut = container.stdout();
    let err_fut = container.stderr();
    let exit_code_fut = container.exit_code();
    let (out, err, exit_code) = tokio::join!(out_fut, err_fut, exit_code_fut);
    let out = out.inspect_err(|e| eprintln!("dagger err(stdout): {}", e));
    let err = err.inspect_err(|e| eprintln!("dagger err(stderr): {}", e));
    let exit_code = exit_code?;
    out?;
    err?;
    if exit_code == 0 {
        Ok(())
    } else {
        bail!("Exit code was non zero: {exit_code}")
    }
}

/// Simplify execution, split at whitespace
///
/// Note: Does not consider nested `"` nor escaping `\"`!
fn cmd(s: impl AsRef<str>) -> Vec<String> {
    Vec::from_iter(s.as_ref().split_whitespace().map(|x| x.to_string()))
}

/// Simplify access to the CWD
///
/// TODO: Should be the cargo manifest directory!
fn hrrd(client: &DaggerConn) -> Directory {
    dir(client, ".")
}

// TODO: caching of dagger requires to _not_ use `.directory().directory()`-chaining
fn dir(client: &DaggerConn, path: &str) -> Directory {
    client.host().directory_opts(
        path,
        HostDirectoryOpts {
            exclude: Some(vec!["node_modules", "target", "dagger"]),
            include: None,
        },
    )
}

trait WithCache {
    fn cache_node(self, client: &DaggerConn) -> Self;
    fn cache_rust(self, client: &DaggerConn) -> Self;
    fn cache_apt(self, client: &DaggerConn) -> Self;
}

impl WithCache for Container {
    fn cache_node(mut self, client: &DaggerConn) -> Container {
        let cache_volume_solidity = client.cache_volume("solidity");
        let cache_volume_node_modules = client.cache_volume("node_modules");
        let cache_volume_pnpm_store = client.cache_volume("pnpm");
        let cache_volume_npm_store = client.cache_volume("npm");

        let container = self;
        let container = container.with_mounted_cache("/root/.npm", cache_volume_npm_store.clone());
        let container =
            container.with_mounted_cache("/workdir/.pnpm-store", cache_volume_pnpm_store.clone());
        let container = container
            .with_mounted_cache("/workdir/node_modules", cache_volume_node_modules.clone());
        let container = container.with_mounted_cache(
            "/workdir/_compiled_contracts",
            cache_volume_solidity.clone(),
        );

        container
    }

    fn cache_rust(mut self, client: &DaggerConn) -> Container {
        let cache_volume_cargo = client.cache_volume("cargo");
        let cache_volume_rustup = client.cache_volume("rustup");
        let cache_volume_rustup_downloads = client.cache_volume("rustup_downloads");
        let cache_volume_target = client.cache_volume("target");

        let container = self;
        let container =
            container.with_mounted_cache("/workdir/target", cache_volume_target.clone());
        let container = container.with_mounted_cache("/root/.rustup", cache_volume_rustup.clone());
        let container = container.with_mounted_cache("/root/.cargo", cache_volume_cargo.clone());
        let container = container
            .with_mounted_cache("/usr/local/rustup", cache_volume_rustup_downloads.clone());

        container
    }

    fn cache_apt(mut self, client: &DaggerConn) -> Container {
        let cache_volume_aptititude = client.cache_volume("apt-cache");
        let cache_volume_var_cache = client.cache_volume("apt-lists");

        let container = self;
        let container = container.with_mounted_cache("/var/cache", cache_volume_var_cache.clone());
        let container =
            container.with_mounted_cache("/var/lib/apt/", cache_volume_aptititude.clone());

        container
    }
}

fn hccd(client: &DaggerConn) -> Directory {
    dir(client, "fendermint/actors/output")
}

#[derive(clap::Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Arg {
    #[command(subcommand)]
    what: Option<What>,
}

#[derive(Debug, Clone, Default, clap::Subcommand)]
enum What {
    #[default]
    #[command()]
    All,

    #[command()]
    MinimalContainer,

    #[command()]
    CargoTests,
}

use std::pin::Pin;

impl What {
    pub fn into_fut(
        self,
    ) -> Box<dyn FnMut(DaggerConn) -> Pin<Box<dyn Future<Output = Result<()>>>>> {
        match self {
            Self::All => Box::new(move |client: DaggerConn| {
                Box::pin(async move {
                    // TODO launch in parallel
                    prepare_fendermint_two_stage_build(client.clone()).await?;
                    run_cargo_test(client).await?;
                    Ok(())
                })
            }),
            Self::MinimalContainer => Box::new(move |client: DaggerConn| {
                Box::pin(async move {
                    prepare_fendermint_two_stage_build(client).await?;
                    Ok(())
                })
            }),
            Self::CargoTests => Box::new(move |client: DaggerConn| {
                Box::pin(async move {
                    run_cargo_test(client).await?;
                    Ok(())
                })
            }),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // dagger_sdk::logging::default_logging()?;

    let args = <Arg as clap::Parser>::try_parse()?;

    dagger_sdk::connect_opts(
        dagger_sdk::Config {
            workdir_path: None,
            config_path: None,
            timeout_ms: 2000,
            execute_timeout_ms: None,
            logger: Some(Arc::new(StdLogger::default())),
        },
        move |client| async move {
            let fx = args.what.unwrap_or_default().into_fut()(client);
            fx.await?;
            Ok(())
        },
    )
    .await?;

    Ok(())
}
