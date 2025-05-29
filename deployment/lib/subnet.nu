use util.nu *

export def wait-for-subnet [] {
  while true {
    let result = (run-in-container cast chain-id '-r' $env.state.config.subnet.rpc_url | complete)
    if $result.exit_code == 0 {
      let chain_id = ($result.stdout | into int)
      if $chain_id == $env.state.config.subnet.chain_id {
        break
      } else {
        print $"Chain ID was ($chain_id) but expected ($env.state.config.subnet.chain_id)"
        exit 1
      }
    } else {
      print "waiting for subnet..."
      sleep 2sec
    }
  }
}

export def deploy-faucet-contract [] {
  let contract_name = "Faucet"
  forge-retry "deploy-faucet-contract" [
    script $"script/($contract_name).s.sol"
    --private-key $env.state.faucet_owner.private_key
    --tc DeployScript
    --sig "'run(uint256)'" (1e18)
    --rpc-url $env.state.config.subnet.rpc_url
    --broadcast -vv
    -g 100000
  ]

  let addr = (open $"($env.state.config.ipc_src_dir)/recall-contracts/broadcast/($contract_name).s.sol/($env.state.config.subnet.chain_id)/run-latest.json" |
    get returns | values | where internal_type == $"contract ($contract_name)" | get 0.value)
  do $env.state.update ({} | insert $"($contract_name | str downcase)_contract_address" $addr)
}


export def fund-faucet-contract [amount] {
  cast-retry "fund-faucet-contract" [
    send --private-key $env.state.faucet_owner.private_key
    --rpc-url $env.state.config.subnet.rpc_url
    --timeout 120
    --confirmations 10
    $env.state.Faucet_contract_address
    "'fund()'"
    --value $amount
  ]
}

export def faucet-set-drip-amount [drip_amount: float] {
  cast-retry "faucet-set-drip-amount" [
    send --private-key $env.state.faucet_owner.private_key
    --rpc-url $env.state.config.subnet.rpc_url
    --timeout 120
    --confirmations 10
    $env.state.Faucet_contract_address
    "'setDripAmount(uint256)'" $drip_amount
  ]
}

export def send-funds [src: record, dest:record, amount: float] {
  cast-retry "send-funds" [
    send --private-key $src.private_key
    --rpc-url $env.state.config.subnet.rpc_url
    --value $amount
    $dest.address
  ]
}

# WARNING: this command invokes `recall` CLI on your PATH!!!
export def set-network-admin [] {
  let cfg = ($env.state.config.workdir | path join "networks.toml")
  recall -c $cfg -n $env.state.config.network subnet config set-admin --private-key $env.state.validator0.private_key $env.state.network_admin.address
}

# WARNING: this command invokes `recall` CLI on your PATH!!!
export def set-network-config [] {
  let cfg = ($env.state.config.workdir | path join "networks.toml")
  recall -c $cfg -n $env.state.config.network subnet config set --private-key $env.state.network_admin.private_key ...[
    --blob-capacity (10 * 2 ** 40)
    --token-credit-rate (1e36)
    --blob-credit-debit-interval 600
    --blob-min-ttl 3600
    --blob-default-ttl 1209600
    --blob-delete-batch-size 100
    --account-debit-batch-size 1000
  ]
}
