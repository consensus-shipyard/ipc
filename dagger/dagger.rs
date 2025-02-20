use std::path::Path;
use std::sync::Arc;

use color_eyre::eyre::{self, bail, eyre, Result};
use dagger_sdk::{
    logging::{StdLogger, TracingLogger},
    Container, ContainerBuildOpts, ContainerBuildOptsBuilder, ContainerWithMountedCacheOpts,
    ContainerWithMountedCacheOptsBuilder, ContainerWithMountedDirectoryOpts, DaggerConn, Directory,
    File, HostDirectoryOpts, Service,
};
use fs_err as fs;
use rand::Rng;

async fn run(container: &Container) -> eyre::Result<()> {
    let out_fut = container.stdout();
    let err_fut = container.stderr();
    let exit_code_fut = container.exit_code();
    let (out, err, exit_code) = tokio::join!(out_fut, err_fut, exit_code_fut);
    let out = out.inspect_err(|e| eprintln!("dagger err(stdout): {}", e.to_string()));
    let err = err.inspect_err(|e| eprintln!("dagger err(stderr): {}", e.to_string()));
    let exit_code = exit_code?;
    out?;
    err?;
    if exit_code == 0 {
        Ok(())
    } else {
        bail!("Exit code was non zero: {exit_code}")
    }
}

async fn prepare_anvil_service(client: DaggerConn) -> eyre::Result<Service> {
    let context_dir = client.host().directory("./contracts/");

    let container = client.container().build_opts(
        context_dir,
        ContainerBuildOptsBuilder::default()
            .dockerfile("docker/Dockerfile")
            .build()?,
    );

    let out_fut = container.stdout();
    let err_fut = container.stderr();
    let exit_code_fut = container.exit_code();
    let (out, err, exit_code) = tokio::join!(out_fut, err_fut, exit_code_fut);
    let out = dbg!(out?);
    let err = dbg!(err?);

    Ok(container.as_service())
}

async fn prepare_ipc_cli(client: DaggerConn) -> eyre::Result<Container> {
    let context_dir = client.host().directory("fendermint/");

    let container = client.container().build_opts(
        context_dir,
        ContainerBuildOptsBuilder::default()
            .dockerfile("docker/runner.Dockerfile")
            .build()?,
    );

    Ok(container)
}

async fn test_service() -> eyre::Result<()> {
    todo!();
    Ok(())
}

async fn prepare_fendermint_container(client: DaggerConn) -> eyre::Result<Container> {
    let context_dir = client.host().directory("fendermint");

    let container = client.container().build_opts(
        context_dir,
        ContainerBuildOptsBuilder::default()
            .dockerfile("docker/runner.Dockerfile")
            .build()?,
    );

    Ok(container)
}

fn cmd(s: impl AsRef<str>) -> Vec<String> {
    Vec::from_iter(s.as_ref().split_whitespace().map(|x| x.to_string()))
}

fn host_repo_root_dir(client: &DaggerConn) -> Directory {
    let repo_root_dir = client.host().directory_opts(
        ".",
        HostDirectoryOpts {
            exclude: Some(vec!["node_modules", "target"]),
            include: None,
        },
    );
    repo_root_dir
}

fn define_contracts_container(client: DaggerConn) -> Result<Container> {
    let d = host_repo_root_dir(&client);
    let opts = ContainerBuildOptsBuilder::default()
        .dockerfile("contracts/docker/builder.Dockerfile")
        .build()?;

    Ok(client
        .container()
        .from("docker.io/library/node:latest")
        .with_mounted_directory("/workdir", d.clone())
        .with_exec(cmd("apt-get update -y"))
        .with_exec(cmd("apt-get install -y curl"))
        // .build_opts(d.clone(), opts)
        .with_workdir("/workdir/contracts")
        .with_exec(cmd("npm install -g pnpm"))
        .with_exec(vec!["sh", "-c", "curl -L https://foundry.paradigm.xyz | bash && /root/.foundry/bin/foundryup --install 0.3.0"])
        .with_exec(cmd("make gen")))
}

fn define_crates_container(client: DaggerConn) -> Result<Container> {
    let d = host_repo_root_dir(&client);

    let opts = ContainerBuildOptsBuilder::default()
        .dockerfile("fendermint/docker/builder.local.Dockerfile")
        .build()?;

    Ok(client
        .container()
        .from("docker.io/rust:bookworm")
        .with_mounted_directory("/workdir", d.clone())
        // TODO
        // .with_mounted_directory("/output", output_dir.clone())
        .with_workdir("/workdir")
        .with_exec(cmd("apt-get update"))
        .with_exec(cmd(
            "apt-get install -y build-essential clang cmake protobuf-compiler",
        ))
        .with_exec(cmd("cargo b -p fendermint_app")))
    // .build_opts(d.clone(), opts))
}

async fn prepare_fendermint_two_stage_build(client: DaggerConn) -> eyre::Result<Container> {
    let repo_root_dir = host_repo_root_dir(&client);

    let fendermint_dir = client.host().directory("fendermint");

    fs::create_dir_all("output")?;
    let output_dir = client.host().directory("output");

    let contracts_gen = define_contracts_container(client.clone())?;
    run(&contracts_gen).await?;
    let output = contracts_gen.file("/output");
    let output = contracts_gen.file("output");

    let crates_def = define_crates_container(client.clone())?.with_file("/foo", output); // TODO more files

    run(&crates_def).await?;
    let f_fendermint = contracts_gen.file("fendermint");
    let f_ipc = contracts_gen.file("ipc-cli");

    // let cache_volume_aptititude = client.cache_volume("aptitude");
    // let cache_volume_cargo = client.cache_volume("cargo");
    // let cache_volume_rustup = client.cache_volume("rustup");
    // let cache_volume_target = client.cache_volume("target");
    // let cache_volume_solidity = client.cache_volume("solidity");

    println!("Extracting .car files from container");

    let car_extra = contracts_gen.file("/app/output/custom_extra_actors.car");
    let car_builtin = contracts_gen.file("/app/output/builtin_actors.car");

    let runner = client
        .container()
        .with_file("/usr/local/bin/fendermint", f_fendermint)
        .with_file("/usr/local/bin/ipc-cli", f_ipc)
        .with_file("/tmp/extra.car", car_extra)
        .with_file("/tmp/builtin.car", car_builtin)
        .build_opts(
            repo_root_dir.clone(),
            ContainerBuildOptsBuilder::default()
                .dockerfile("docker/runner.Dockerfile")
                .build()?,
        );
    run(&runner).await?;

    Ok(runner)
}

// async fn end_to_end_tests(client: DaggerConn) -> eyre::Result<()> {
//     let container = prepare_fendermint_container(client).await?;
//     // container.with_service_binding("joe", service)
//     run(&container.with_workdir("contracts").with_exec(vec!["make", "gen"])).await;
//     run(&container.with_workdir("fendermint").with_env_variable("PROFILE", "release").with_exec(vec!["make", "e2e-only"])).await;
//     Ok(())
// }

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    // dagger_sdk::logging::default_logging()?;

    dagger_sdk::connect_opts(
        dagger_sdk::Config {
            workdir_path: None,
            config_path: None,
            timeout_ms: 2000,
            execute_timeout_ms: None,
            logger: Some(Arc::new(StdLogger::default())),
        },
        |client| async move {
            let _fendermint_featherweight =
                prepare_fendermint_two_stage_build(client).await.unwrap();

            // let anvil = prepare_anvil_service(client).await?;
            // let client = prepare_ipc_cli(client).await?;

            Ok(())
        },
    )
    .await?;

    Ok(())
}
