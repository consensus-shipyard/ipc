use ./local-files.nu
use ./state-engine.nu
use ./util.nu *

export def run-localnet-node [
  ix: int, # node index
  dc_repo: string, # recall-docker-compose repo to clone
  dc_branch: string, # recall-docker-compose branch
  --bootstrap, # run only essential services required to deploy subnet contracts
  ] {

  let node_name = $"node-($ix)"
  let node_dir = ($env.state.config.workdir | path join $node_name | path expand)
  let repo = if ($dc_repo | str starts-with "..") { $dc_repo | path expand} else { $dc_repo }
  mkdir $node_dir
  cd $node_dir
  if ($node_dir | path join ".git" | path exists) {
    git checkout $dc_branch
    git pull
  } else {
    git clone --branch $dc_branch $repo .
  }

  local-files write-subnet-config ($node_dir | path join "config/network-localnet.toml") --bootstrap=$bootstrap
  write-localnet-node-config $ix $bootstrap
  ./init-workdir ./workdir
  do {
    cd ./workdir
    docker compose up -d
  }

  let ids = (./workdir/node-tools show-peer-ids | from yaml)
  do $env.state.update {
    peers: {
      cometbft: [$"($ids.cometbft_id)@localnet-($node_name)-cometbft-1:26656"]
      fendermint: [$"/dns/localnet-($node_name)-fendermint-1/tcp/26655/p2p/($ids.fendermint_id)"]
    }
  }
}

export def stop-node [ix: int] {
  let node_name = $"node-($ix)"
  let node_dir = ($env.state.config.workdir | path join $node_name | path expand)

  if ($node_dir | path exists) {
    cd ($node_dir + "/workdir")
    docker compose down
  } else {
    echo $"Directory ($node_dir) does not exist, skipping."
  }
}

def write-localnet-node-config [node_ix: int, bootstrap: bool] {
  let node_name = $"node-([$node_ix 0] | math max)"
  let output_file = ($env.state.config.workdir | path join $node_name| path join "config/node.toml")

  let enable = {
    sync: ($node_ix > 0)
    relayer: ($node_ix == 0)
    registrar: ($node_ix == 0 and not $bootstrap)
    s3: ($node_ix == 0 and not $bootstrap)
  }
  mut cfg = {
    network_name: localnet
    node_name: $node_name
    project_name: $"localnet-($node_name)"
    node_private_key: ($env.state | get $"validator($node_ix)" | get private_key)

    images: {
      fendermint: $env.state.config.fendermint_image
    }
    parent_endpoint: {
      url: "http://localnet-anvil:8545"
    }
    networking: {
      docker_network_subnet: $"10.222.($node_ix).0/24"
      host_bind_ip: ""
    }
    services: {
      cometbft_statesync_enable: $enable.sync
      relayer_checkpoint_interval_sec: 10
    }
    localnet: {
      enable: true
      network: recall-localnet
    }
  }

  if $node_ix == 0 {
    $cfg = ($cfg | merge deep {
      localnet: {
        cli_bind_host: "127.0.0.1"
      }
    })
  }
  if $enable.relayer {
    $cfg = ($cfg | merge deep {
      relayer: {
        enable: true

        # FIXME: This is 3rd anvil key. Make it dynamic.
        private_key: "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6"
      }
    })
  }
  if $enable.registrar {
    $cfg = ($cfg | merge deep {
      registrar: {
        enable: true
        faucet_owner_private_key: $env.state.faucet_owner.private_key
        turnstile_secret_key: nonsense
        trusted_proxy_ips: "10.0.0.0" # dummy
      }
    })
  }
  if $enable.s3 {
    $cfg = ($cfg | merge deep {
      recall_s3: {
        enable: true
        access_key: "user1"
        secret_key: "hello-recall"
        domain: "localhost"
      }
    })
  }

  $cfg | save -f $output_file
}

export def run-anvil [workdir: string] {
  docker build -t anvil -f ./docker/anvil.Dockerfile ./docker
  let found = (docker network ls | lines | find "recall-localnet" | length)
  if $found == 0 {
    docker network create recall-localnet
  }
  let anvil_dir = $"($workdir)/anvil"
  mkdir $anvil_dir
  docker run ...[
    --rm -d
    -u $"(id -u):(id -g)"
    --name localnet-anvil
    -p 127.0.0.1:8545:8545
    --network recall-localnet
    -v $"($anvil_dir):/workdir"
    anvil
  ]
}

export def stop-anvil [] {
  # We want a graceful stop so that anvil can dump all its state to the state file.
  # Without this wait anvil can hang on termination.
  sleep 1sec
  docker stop localnet-anvil

  # Verify the state file was created
  if not ( $env.state.config.workdir | path join "anvil/state" | path exists) {
    print "ERROR: anvil failed to dump its state"
    exit 5
  }
}

export def stop-network [workdir: string, --force] {
  if $force {
    docker ps -a --format json | lines | each {from json} | where Names =~ $"localnet-" | each {docker rm -f $in.ID}
  } else {
    glob ($workdir + "/node-*") | reverse | each {|dir|
      cd ($dir | path join "workdir")
      docker compose down
    }

    let state = state-engine read-state (state-file  $workdir)
    if (docker ps | lines | find localnet-anvil | length) == 1 {
      stop-anvil
    }
    do $state.update { graceful_shutdown: true}
  }
}

export def build-dind-image [local_image_tag: any, push_multi_arch_tags: any] {
  let found = (docker buildx ls | lines | find "multi-arch-builder" | length)
  if $found == 0 {
    docker buildx create --name multi-arch-builder --driver docker-container
  }

  def build-local [tag:string] {
    docker buildx build -t $tag --load -f docker/localnet.Dockerfile .
  }

  if ($local_image_tag | is-not-empty) {
    build-local $local_image_tag
  } else if ($push_multi_arch_tags | is-not-empty) {
    let tags = $push_multi_arch_tags | split row ',' | each {|tag| [-t $tag]} | flatten
    docker buildx build --builder=multi-arch-builder --platform linux/amd64,linux/arm64 --push ...$tags -f docker/localnet.Dockerfile .
  } else {
    build-local recall-localnet
  }
}

export def wait-for-sync [ ix: int ] {
  loop {
    print "== calling cometbft..."
    let result = (run-in-container curl '-s' '-m' 2 $"http://localnet-node-($ix)-cometbft-1:26657/status" | complete)
    if $result.exit_code == 0 {
      let cu = ($result.stdout | from json | get result.sync_info.catching_up )
      if $cu == false {
        break
      }
    } else {
      print $"== stdout: ($result.stdout)"
      print $"== stderr: ($result.stderr)"
    }
    print "waiting for sync..."
    sleep 10sec
  }
}

export def wait-for-cometbft [ ix: int ] {
  loop {
    print "== calling cometbft..."
    let result = (run-in-container curl '-s' '-m' 2 $"http://localnet-node-($ix)-cometbft-1:26657/status" | complete)
    if $result.exit_code == 0 {
      let block_height = ($result.stdout | from json | get result.sync_info.latest_block_height )
      if ($block_height | str length) > 0 and ($block_height | into int) > 0 {
        break
      }
    }
    print "waiting for cometbft..."
    sleep 2sec
  }
}

# Create workdir and the config file
export def init-state [
  workdir: string,
  fendermint_image: string,
  --parent-rpc-url: string = "http://localnet-anvil:8545",
  ] {

  let base_config = (get-base-config $workdir "localnet" $fendermint_image)
  let cfg = {
    bottomup_check_period: 10
    docker_network: "recall-localnet"
    parent_chain: {
      rpc_url: $parent_rpc_url
      gas_estimate_multiplier: 10000
      network: "localnet"
      chain_id: 31337
    }
    subnet: {
      chain_id: 248163216
      rpc_url: "http://localnet-node-0-ethapi-1:8545"
      cometbft_rpc_servers: [ "http://localnet-node-0-cometbft-1:26657" ]
    }
  }
  let state = {
    config: ($base_config | merge $cfg)
  }
  state-engine update-state ($workdir | path join "state.yml") $state
}
