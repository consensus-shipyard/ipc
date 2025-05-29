use ./util.nu
use ./parent-chain.nu
use ./local-files.nu
use ./subnet.nu


export def get-create-subnet-steps [get_funds_fn: closure] {
  [
    { name: "build_setup_image" fn: { local-files build-setup-docker-image} }
    { name: "create_ipc_config" fn: { parent-chain write-ipc-cli-config }}

    { name: "create_supply_source_owner_key" fn: { util create-wallet "supply_source_owner"} }
    { name: "supply_source_owner_ensure_funds" fn: $get_funds_fn }

    { name: "deploy_supply_source" fn: { parent-chain deploy-supply-source } }

    { name: "prepare_gateway_and_registry" fn: { parent-chain prepare-contract-stack-deployment } }
    { name: "deploy_gateway_and_registry" fn: { parent-chain deploy-contract-stack } }
    { name: "update_ipc_config" fn: { parent-chain write-ipc-cli-config }}

    { name: "create_validator0_key" fn: { util create-wallet "validator0"} }
    { name: "send_funds_to_validator0" fn: { parent-chain send-funds $env.state.validator0 20e18} }
    { name: "deploy_validator_rewarder" fn: { parent-chain deploy-validator-rewarder } }
    { name: "rewarder_grant_minter_role" fn: { parent-chain grant-minter-role $env.state.validator_rewarder_address }}
    { name: "deploy_validator_gater" fn: { parent-chain deploy-validator-gater} }
    { name: "mint_erc20" fn: { parent-chain mint-erc20 $env.state.validator0.address 1001e18} }
    { name: "approve_validator_power" fn: { parent-chain approve-validator-power $env.state.validator0 1e18 1000e18} }
    { name: "create_subnet" fn: { parent-chain create-subnet } }
    { name: "set_subnet_in_validator_gater" fn: { parent-chain set-subnet-in-validator-gater} }
    { name: "approve_subnet_contract" fn: { parent-chain approve-subnet-contract $env.state.validator0 1000e18} }
    { name: "prefund_validator" fn: { parent-chain prefund-validator $env.state.validator0 100e18} }
    { name: "join_subnet" fn: { parent-chain join-subnet $env.state.validator0 10} }
    { name: "transfer_funds" fn: { parent-chain cross-msg-to-subnet $env.state.validator0.address 5e18} }
    { name: "write_recall_subnet_config" fn: { local-files write-recall-cli-config } }
  ]
}

export def get-deploy-subnet-contracts-steps [set_up_contract_owner_steps: list] {
  [
    { name: "wait_for_subnet" fn: { subnet wait-for-subnet} }
    { name: "add_subnet_to_ipc_config" fn: { local-files add-subnet-to-ipc-config} }

    { name: "faucet_create_key" fn: { util create-wallet "faucet_owner"} }
    { name: "send_funds_to_faucet_owner" fn: { parent-chain send-funds $env.state.faucet_owner 3e18} }
    { name: "faucet_mint_erc20" fn: { parent-chain mint-erc20 $env.state.faucet_owner.address 1e30} }
    { name: "faucet_transfer_tokens_to_subnet" fn: { parent-chain cross-msg-to-subnet $env.state.faucet_owner.address (1e30 - 2e18)} }
    { name: "faucet_wait_for_funds" fn: { util wait-for-funds-on-subnet $env.state.faucet_owner.address} }
    { name: "deploy_faucet_contract" fn: { subnet deploy-faucet-contract} }
    { name: "fund_faucet_contract" fn: { subnet fund-faucet-contract (5e27)} }
    { name: "faucet_set_drip_amount" fn: { subnet faucet-set-drip-amount 5e18} }

    ...$set_up_contract_owner_steps
    { name: "validator_check_funds_on_subnet" fn: { util wait-for-funds-on-subnet $env.state.validator0.address} }
  ]
}

export def prepare-validator [name: string, max_power: float] {
  if $max_power < 1e18 {
    print "ERROR: Max power is too low, it must be provided in wei units"
    exit 1
  }
  [
    { name: "wait_for_subnet" fn: { subnet wait-for-subnet} }
    { name: $"($name)_get_funds_on_parent_chain" fn: { parent-chain send-funds ($env.state | get $name) 2e18} }
    { name: $"($name)_mint_erc20" fn: { parent-chain mint-erc20 ($env.state | get $name | get address) ($max_power * 2) } }
    { name: $"($name)_approve_validator_power" fn: { parent-chain approve-validator-power ($env.state | get $name) 1e18 $max_power} }
  ]
}

export def join-subnet [node_name: string, power: int] {
  if $power >= 1e18 {
    print "ERROR: Power is too high. It must be provided in full RECALL units."
    exit 1
  }
  [
    { name: $"($node_name)_approve_subnet_contract" fn: { parent-chain approve-subnet-contract ($env.state | get $node_name) (2 * $power * 1e18) }}
    { name: $"($node_name)_join_subnet" fn: { parent-chain join-subnet ($env.state | get $node_name) $power }}
  ]
}
