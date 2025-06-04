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

def --env set-up-recall-cli [private_key: string] {
  let cfg = ($env.state.config.workdir | path join "networks.toml")
  $env.RECALL_NETWORK_CONFIG_FILE = $cfg
  $env.RECALL_NETWORK = $env.state.config.network
  $env.RECALL_PRIVATE_KEY = $private_key
}

# WARNING: this command invokes `recall` CLI on your PATH!!!
export def set-network-admin [] {
  set-up-recall-cli $env.state.validator0.private_key
  recall subnet config set-admin $env.state.network_admin.address
}

# WARNING: this command invokes `recall` CLI on your PATH!!!
export def set-network-config [] {
  set-up-recall-cli $env.state.validator0.private_key
  recall subnet config set ...[
    --blob-capacity (10 * 2 ** 40)
    --token-credit-rate (1e36)
    --blob-credit-debit-interval 600
    --blob-min-ttl 3600
    --blob-default-ttl 1209600
    --blob-delete-batch-size 100
    --account-debit-batch-size 1000
  ]
}

export def run-recall-cli-test [] {
  set-up-recall-cli $env.state.faucet_owner.private_key
  recall account info
  recall account credit stats
  recall account credit buy 2
  recall account credit stats
  let addr = recall bucket create --alias test1 | tee {print} | from json | get address | inspect
  recall bucket add -a $addr --key a1 ./set-up-nu.sh
  recall bucket get -a $addr a1
}
