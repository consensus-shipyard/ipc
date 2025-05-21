use util.nu *

def rpc-url [] {
  if ("token" in $env.state.config.parent_chain) {
    $"($env.state.config.parent_chain.rpc_url)?token=($env.state.config.parent_chain.token)"
  } else {
    $env.state.config.parent_chain.rpc_url
  }
}

def balance [addr: string] {
  cast balance --rpc-url (rpc-url) $addr | into float
}

export def ensure-funds [address: string] {
  let b = (balance $address)
  log $"Balance of ($address): ($b / 1e18)"
  if $b == 0 {
    print $"(ansi green)==== Goto to https://faucet.calibnet.chainsafe-fil.io/funds.html and fund ($address) (ansi reset)"
  } else {
    return
  }
  loop {
    sleep 15sec
    let b = (balance $address)
    log $"balance: ($b / 1e18)"
    if $b == 0 {
      log "waiting for funds"
    } else {
      break
    }
  }
}

export def write-ipc-cli-config [] {
  mkdir $env.state.config.ipc_config_dir

  let gateway_addr = if "gateway_address" in $env.state {$env.state.gateway_address} else {"0x0000000000000000000000000000000000000000"}
  let registry_addr = if "registry_address" in $env.state {$env.state.registry_address} else {"0x0000000000000000000000000000000000000000"}

  let token = if ("token" in $env.state.config.parent_chain) {{auth_token: $env.state.config.parent_chain.token}} else {{}}
  let calibration = {
    id: $"/r($env.state.config.parent_chain.chain_id)"
    config: ({
      network_type: "fevm"
      provider_http: $env.state.config.parent_chain.rpc_url
      gateway_addr: $gateway_addr
      registry_addr: $registry_addr
    } | merge $token)
  }

  let cfg = {
    keystore_path: "/fendermint/.ipc"
    subnets: [$calibration]
  }
  $cfg | save -f $env.state.config.ipc_config_file

  do $env.state.update {parent: $calibration}
}

export def deploy-validator-gater [] {
  run-in-container ...[
    $"cd ($env.state.config.docker_ipc_src_dir)/recall-contracts;"
    "forge clean;"
    "forge install;"
    forge script script/ValidatorGater.s.sol
    --private-key $env.state.validator0.private_key
    --rpc-url (rpc-url)
    --tc DeployScript --sig "'run()'"
    --broadcast -g (get-gas-estimate-multiplier) -vv
  ]

  let validator_gater = (open $"($env.state.config.ipc_src_dir)/recall-contracts/broadcast/ValidatorGater.s.sol/($env.state.config.parent_chain.chain_id)/run-latest.json" |
    get transactions | where contractName == ERC1967Proxy | get 0.contractAddress)
  do $env.state.update {validator_gater_address: $validator_gater}
}

export def approve-validator-power [validator, min: float, max: float] {
  cast-retry "approve-validator-power" [
    send --private-key $env.state.validator0.private_key
    --rpc-url (rpc-url)
    --timeout 120
    --confirmations 10
    $env.state.validator_gater_address
    "'approve(address,uint256,uint256)'"
    $validator.address
    $min $max
  ]
}

export def create-subnet [] {
  let result = (run-in-container ipc-cli ...[
    subnet create
    --from $env.state.validator0.address
    --parent $"/r($env.state.config.parent_chain.chain_id)"
    --min-validators 0
    --min-validator-stake 1
    --bottomup-check-period $env.state.config.bottomup_check_period
    --active-validators-limit 40
    --permission-mode collateral
    --supply-source-kind erc20
    --supply-source-address $env.state.supply_source_address
    --validator-gater $env.state.validator_gater_address
    --validator-rewarder $env.state.validator_rewarder_address
    --collateral-source-kind erc20
    --collateral-source-address $env.state.supply_source_address
  ] | complete)
  if $result.exit_code != 0 {
    print $result.stdout
    print $result.stderr
    exit $result.exit_code
  }
  let subnet_id = ($result.stdout | tee { print }| lines | find "created subnet actor" | get 0 | str replace -r '.*\/r' '/r' | str trim)

  do $env.state.update {
    subnet_id: $subnet_id
    subnet_eth_address: (f4-to-eth ($subnet_id | split row '/' | get 2))
  }
}

export def set-subnet-in-validator-gater [] {
  cast-retry "set-subnet-in-validator-gater" [
    send --private-key $env.state.validator0.private_key
    --rpc-url (rpc-url)
    --timeout 120
    --confirmations 10
    $env.state.validator_gater_address
    "'setSubnet((uint64,address[]))'"
    $"'\(($env.state.config.parent_chain.chain_id), [($env.state.subnet_eth_address)])'"
  ]
}

export def approve-subnet-contract [validator, amount: float] {
  cast-retry "approve-subnet-contract" [
    send --private-key $validator.private_key
    --rpc-url (rpc-url)
    --timeout 120
    --confirmations 10
    $env.state.supply_source_address
    "'approve(address,uint256)'"
    $env.state.subnet_eth_address $amount
  ]
}

export def prefund-validator [validator, amount: float] {
  cast-retry "prefund-validator" [
    send --private-key $validator.private_key
    --rpc-url (rpc-url)
    --timeout 120
    --confirmations 10
    $env.state.subnet_eth_address
    "'preFund(uint256)'"
    $amount
  ]
}

export def join-subnet [validator, stake: int] {
  run-in-container ipc-cli ...[
    subnet join
    --from $validator.address
    --subnet $env.state.subnet_id
    --collateral $stake
  ]
}

export def stake [validator, stake: int] {
  run-in-container ipc-cli ...[
    subnet stake
    --from $validator.address
    --subnet $env.state.subnet_id
    --collateral $stake
  ]
}

export def unstake [validator, stake: int] {
  run-in-container ipc-cli ...[
    subnet unstake
    --from $validator.address
    --subnet $env.state.subnet_id
    --collateral $stake
  ]
}

# Anvil does not tollerate too high gas estimate multipliers in some cases.
def get-gas-estimate-multiplier [] {
  let s = $env.state
  if $s.config.network == "localnet" {
    130
  } else {
    $s.config.parent_chain.gas_estimate_multiplier
  }
}

export def deploy-supply-source [] {
  run-in-container ...[
    $"cd ($env.state.config.docker_ipc_src_dir)/recall-contracts;"
    'forge clean;'
    'forge install;'
    forge script script/Recall.s.sol
    --private-key $env.state.supply_source_owner.private_key
    --rpc-url (rpc-url)
    --tc DeployScript
    --sig "'run()'"
    --broadcast
    -g (get-gas-estimate-multiplier)
    -vv
  ]

  let supply_source = (open $"($env.state.config.ipc_src_dir)/recall-contracts/broadcast/Recall.s.sol/($env.state.config.parent_chain.chain_id)/run-latest.json" | get returns | values | where internal_type == "contract Recall" | get 0.value)

  do $env.state.update {supply_source_address: $supply_source }
}

export def deploy-validator-rewarder [] {
  run-in-container ...[
    $"cd ($env.state.config.docker_ipc_src_dir)/recall-contracts;"
    "forge clean;"
    "forge install;"
    forge script script/ValidatorRewarder.s.sol
    --private-key $env.state.validator0.private_key
    --rpc-url (rpc-url)
    --tc DeployScript
    --sig "'run(address)'" $env.state.supply_source_address
    --broadcast -g (get-gas-estimate-multiplier) -vv
  ]

  let contract_address = (open $"($env.state.config.ipc_src_dir)/recall-contracts/broadcast/ValidatorRewarder.s.sol/($env.state.config.parent_chain.chain_id)/run-latest.json" | get returns | values | where internal_type == "contract ValidatorRewarder" | get 0.value)

  do $env.state.update {validator_rewarder_address: $contract_address }
}

export def "prepare-contract-stack-deployment" [] {
  run-in-container ...[
    "set -ex;"
    $"rm -rf ($env.state.config.docker_ipc_src_dir)/node_modules;"
    cd $"($env.state.config.docker_ipc_src_dir)/contracts;"
    rm -rf "deployments;"
    "npm install;"
  ]
}

export def "deploy-contract-stack" [] {
  mut denv = {
    REGISTRY_CREATION_PRIVILEGES: "unrestricted"
    RPC_URL: (rpc-url)
    PRIVATE_KEY: $env.state.supply_source_owner.private_key
  }
  mut stack_network = $env.state.config.parent_chain.network
  if $env.state.config.network == "localnet" {
    $stack_network = "auto"
    $denv = ($denv | merge {
      CHAIN_ID: $env.state.config.parent_chain.chain_id
      RPC_URL: $env.state.config.parent_chain.rpc_url
    })
  }
  let out = (run-in-container --denv $denv ...[
    "set -ex;"
    cd $"($env.state.config.docker_ipc_src_dir)/contracts;"
    make deploy-stack $"NETWORK=($stack_network)"
    ] | tee {print} | lines)

  def extract_field [pattern] {
    $out | find $pattern | ansi strip | split row -r '\s+' | get 3 | str trim
  }

  do $env.state.update {
    gateway_address: (extract_field 'GatewayDiamond deployed')
    registry_address: (extract_field 'SubnetRegistryDiamond deployed')
  }
}

export def grant-minter-role [address: string] {
  let minter_role = (run-in-container cast keccak "MINTER_ROLE")
  cast-retry "grant-minter-role" [
    send --private-key $env.state.supply_source_owner.private_key
    --rpc-url (rpc-url)
    --timeout 120
    $env.state.supply_source_address
    "'grantRole(bytes32,address)'" $minter_role $address
  ]
}

export def send-funds [dest, amount: float, --from-private-key: string] {
  let pk = if ($from_private_key | is-empty) {$env.state.supply_source_owner.private_key} else {$from_private_key}
  cast-retry "send-funds" [
    send --private-key $pk
    --rpc-url (rpc-url)
    --timeout 120
    --confirmations 10
    --value $amount
    $dest.address
  ]
}

export def mint-erc20 [address: string, amount: float] {
   cast-retry "mint-erc20" [
    send --private-key $env.state.supply_source_owner.private_key
    --rpc-url (rpc-url)
    --timeout 120
    --confirmations 10
    $env.state.supply_source_address
    "'mint(address,uint256)'"
    $address
    $amount
  ]
}

export def cross-msg-to-subnet [address: string, amount: float] {
  run-in-container ipc-cli ...[
    cross-msg fund-with-token
    --from $address
    --subnet $env.state.subnet_id
    --approve $amount
  ]
}
