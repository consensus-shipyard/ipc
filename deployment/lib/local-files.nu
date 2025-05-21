
export def add-subnet-to-ipc-config [] {
 open $env.state.config.ipc_config_file |
  update subnets { $in | append {
    id: $env.state.subnet_id
    config: {
      network_type: "fevm"
      provider_http: $env.state.config.subnet.rpc_url
      gateway_addr: "0x77aa40b105843728088c0132e43fc44348881da8"
      registry_addr: "0x74539671a1d2f1c8f200826baba665179f53a1b7"
    }
  }} |
  to toml |
  save -f $env.state.config.ipc_config_file
}

export def write-subnet-config [dest: string, --bootstrap] {
  mut cfg = {
    address_network: "testnet"
    parent_chain: {
      chain_id: $env.state.config.parent_chain.chain_id
      addresses: {
        gateway: $env.state.gateway_address
        registry: $env.state.registry_address
        supply_source: $env.state.supply_source_address
        subnet_contract: $env.state.subnet_eth_address
        validator_gater: $env.state.validator_gater_address
        validator_rewarder: $env.state.validator_rewarder_address
      }
    }
    subnet: {
      subnet_id: $env.state.subnet_id
      chain_id: $env.state.config.subnet.chain_id
    }
    endpoints: {
      cometbft_rpc_servers: $env.state.config.subnet.cometbft_rpc_servers
      cometbft_persistent_peers: ($env.state.peers?.cometbft | default [] | uniq)
      fendermint_seeds: ($env.state.peers?.fendermint | default [] | uniq)
    }
  }

  if not $bootstrap {
    $cfg = ($cfg | merge deep {
      subnet: {
        addresses: {
          credit_manager: $env.state.creditManager_contract_address
          bucket_manager: $env.state.bucketManager_contract_address
          faucet_contract: $env.state.faucet_contract_address
          blob_manager: $env.state.blobManager_contract_address
        }
      }
      endpoints: {
        evm_rpc_url: $env.state.config.subnet.rpc_url
      }
    })
  }

  $cfg | save -f $dest
}

export def build-setup-docker-image [] {
  cd docker
  docker build ...[
    --build-arg $"fendermint_image=($env.state.config.fendermint_image)"
    -t $env.state.config.setup_image
    -f subnet-setup.Dockerfile .
  ]
}

export def build-fendermint-image [] {
  if $env.state.config.fendermint_image == "fendermint" {
    cd ../fendermint
    make docker-build
  }
}

export def set-fendermint-image [docker_compose_dir: string] {
  cd $"($docker_compose_dir)/config"
  let f = "node-default.toml"
  open $f | update images.fendermint $env.state.config.fendermint_image | save -f $f
}

# Write network config suitable for recall CLI into workdir.
export def write-recall-cli-config [] {
  let endpoints = match $env.state.config.network {
    "localnet" => ({
      subnet_config: {
        rpc_url: "http://localhost:26657"
        object_api_url: "http://localhost:8001"
        evm_rpc_url: "http://localhost:8645"
      }
      parent_network_config: {
        evm_rpc_url: "http://localhost:8545"
      }
    })
    "testnet" => {
      let base = $"($env.state.config.version).node-0.testnet.recall.network"
      {
        subnet_config: {
          rpc_url: $"https://api.($base)"
          object_api_url: $"https://objects.($base)"
          evm_rpc_url: $"https://evm.($base)"
        }
        parent_network_config: {
          evm_rpc_url: "https://api.calibration.node.glif.io"
        }
      }
    }
  }

  let contracts = {
    subnet_config: {
      chain_id: $env.state.config.subnet.chain_id
      subnet_id: $env.state.subnet_id
      evm_gateway_address: "0x77aa40b105843728088c0132e43fc44348881da8"
      evm_registry_address: "0x74539671a1d2f1c8f200826baba665179f53a1b7"
    }
    parent_network_config: {
      evm_gateway_address: $env.state.gateway_address
      evm_registry_address: $env.state.registry_address
      evm_supply_source_address: $env.state.supply_source_address
    }
  }

  let cfg = {} | insert $env.state.config.network ($endpoints | merge deep $contracts)

  $cfg | save -f ($env.state.config.workdir | path join "networks.toml")
}
