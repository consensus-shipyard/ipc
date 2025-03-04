// TODO use digests rather than tags for reproducibility

use super::*;

/// Create a container definition which is able to compile the contracts
pub fn define_contracts_container(client: DaggerConn) -> Result<Container> {
    let compiled_contracts_dir = "/workdir/compiled_contracts";

    let container = client.container()
        .from("docker.io/library/node:latest")
    .with_mounted_directory("/workdir", hrrd(&client))
    .with_mounted_directory(compiled_contracts_dir, hccd(&client))
    .cache_apt(&client)
    .with_exec(cmd("apt-get update -y"))
    .with_exec(cmd("apt-get install -y curl which"))
    .with_workdir("/workdir/contracts")
    .with_exec(cmd("ls -al"))
    .cache_node(&client)
    .with_exec(cmd("npm install -g pnpm"))
    .with_exec(cmd("pnpm install"))
    .with_exec(vec!["sh", "-c", "curl -L https://foundry.paradigm.xyz | bash && /root/.foundry/bin/foundryup --install 0.3.0"])
    .with_exec(cmd("git submodule update --init --recursive"))
    .with_exec(cmd(format!("mkdir -p {compiled_contracts_dir}")))
    .with_env_variable_opts("PATH", "${PATH}:/root/.foundry/bin", ContainerWithEnvVariableOptsBuilder::default().expand(true).build()?)
    .with_exec(cmd("which forge"))
    // actually build the contracts
    // TODO investigate caching issue further
    .with_exec(cmd(format!("forge -vvv build -C ./src/ --lib-paths ./lib/ --via-ir --sizes --skip test --out={compiled_contracts_dir}")));

    Ok(container)
}

pub async fn run_cargo_test(client: DaggerConn) -> Result<()> {
    let container = define_crates_container(client.clone())?;
    let container = container
        .cache_rust(&client)
        .with_exec(cmd("cargo test --workspace"));
    crate::run(&container).await?;
    Ok(())
}

pub fn define_crates_container(client: DaggerConn) -> Result<Container> {
    let container = client
        .container()
        .from("docker.io/rust:bookworm")
        .with_mounted_directory("/workdir", hrrd(&client))
        .with_mounted_directory("/workdir/_compiled_contracts", hccd(&client))
        .with_workdir("/workdir")
        .cache_apt(&client)
        .with_exec(cmd("apt-get update"))
        .with_exec(cmd(
            "apt-get install -y build-essential clang cmake protobuf-compiler",
        ))
        .cache_rust(&client)
        // actually build the rust binaries
        .with_exec(cmd("cargo b -p fendermint_app -p ipc-cli"))
        // see what was created
        .with_exec(cmd("ls -al /workdir/target/debug"))
        .with_exec(cmd("ls -al output"));

    Ok(container)
}

pub async fn prepare_fendermint_two_stage_build(client: DaggerConn) -> Result<Container> {
    fs::create_dir_all("_compiled_contracts")?;

    let contracts_out = dir(&client, "./contracts/out");
    let node_app_config = dir(&client, "./fendermint/app/config");

    let contracts_gen = define_contracts_container(client.clone())?;
    run(&contracts_gen).await?;

    let crates_def = define_crates_container(client.clone())?;
    run(&crates_def).await?;

    let f_fendermint = crates_def.file("/workdir/target/debug/fendermint");
    let f_ipc = crates_def.file("/workdir/target/debug/ipc-cli");

    let car_extra =
        contracts_gen.file("/workdir/fendermint/actors/output/custom_actors_bundle.car");

    // prepare the to-be-published "runner" container
    let runner = client
        .container()
        .from("docker.io/debian:bookworm-slim")
        .with_file_opts(
                "/usr/local/bin/fendermint",
            f_fendermint,
            ContainerWithFileOptsBuilder::default()
                .permissions(0o755_isize)
                .build()?,
        )
        .with_file_opts(
            "/usr/local/bin/ipc-cli",
            f_ipc,
            ContainerWithFileOptsBuilder::default()
                .permissions(0o755_isize)
                .build()?,
        )
        .with_exec(cmd("ls -al"))
        .cache_apt(&client)
        .with_exec(vec!["sh", "-c", "apt-get update && \
            apt-get install -y libssl3 ca-certificates curl"])
        .with_env_variable("FM_HOME_DIR", "/fendermint")
        .with_exposed_port(26658)
        .with_exposed_port(8445)
        .with_exposed_port(9184)
        .with_entrypoint(cmd("docker-entry.sh"))
        .with_default_terminal_cmd(cmd("run"))
        // TODO STOPSIGNAL SIGTERM
        .with_env_variable("FM_ABCI__LISTEN__HOST", "0.0.0.0")
        .with_env_variable("FM_ETH__LISTEN__HOST", "0.0.0.0")
        .with_env_variable("FM_METRICS__LISTEN__HOST", "0.0.0.0")        
        .with_exec(cmd("mkdir -p /fendermint/logs"))
        .with_exec(cmd("chmod 777 /fendermint/logs"))
        .with_file("/usr/local/bin/docker-entry.sh", hrrd(&client).file("fendermint/docker/docker-entry.sh"))
        .with_file("/fendermint/custom_actors_bundle.car", car_extra)
        // TODO FIXME - this is insane
        .with_exec(cmd("mkdir -p /fendermint/builtin-actors/output"))
        .with_exec_opts(cmd("curl -L -o /fendermint/builtin-actors/output/bundle.car https://github.com/filecoin-project/builtin-actors/releases/download/${BUILTIN_ACTORS_TAG}/builtin-actors-mainnet.car"), ContainerWithExecOptsBuilder::default().expand(true).build()?)
        .with_directory("/fendermint/contracts", contracts_out)
        .with_directory_opts("/fendermint/config", node_app_config, ContainerWithDirectoryOptsBuilder::default().exclude(vec![".git", ".gitignore", ".*"]).build()?);

    run(&runner).await?;

    // TODO extract file and publish gh release IFF tagged
    // TODO
    // TODO runner.publish(address).await?;

    Ok(runner)
}
