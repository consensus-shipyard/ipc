/**
 * Type definitions for subnet-related entities
 */

export interface Validator {
  address: string
  stake: string
  power: number
  status: string
  // Additional properties for federated mode
  current_power?: number
  next_power?: number
  waiting?: boolean
  // Additional properties for collateral mode
  initial_balance?: number
}

export interface SubnetInstance {
  id: string
  name: string
  status: string
  template?: string
  parent: string
  created_at: string
  validators: Validator[]
  config: SubnetConfig
  data?: SubnetData
}

export interface SubnetData {
  id?: string
  name?: string
  status?: string
  template?: string
  parent?: string
  created_at?: string
  validator_count?: number
  validators?: Validator[]
  config?: SubnetConfig
  [key: string]: unknown
}

export interface SubnetConfig {
  permissionMode?: 'federated' | 'collateral' | 'static' | 'root' | 'unknown'
  gateway_addr?: unknown
  registry_addr?: unknown
  deployer_address?: string
  parent_endpoint?: string
  minValidatorStake?: string
  minValidators?: number
  bottomupCheckPeriod?: number
  supplySourceKind?: string
  collateralSourceKind?: string
  minCrossMsgFee?: string
  [key: string]: unknown
}

export interface ChainStats {
  total_supply: string
  circulating_supply: string
  fees_collected: string
  active_validators: number
  last_checkpoint: string
  uptime: string
  block_height: number
  transaction_count: number
  tps: number
  avg_block_time: number
  latest_block_time: string
  consensus_status: string
  validators_online: number
  pending_transactions?: number
}

export interface SubnetStatus {
  status: string
  message: string
  is_active: boolean
  block_height: number
  validators_online: number
  consensus_status: 'healthy' | 'degraded' | 'offline' | string
  sync_status?: 'synced' | 'syncing' | 'behind' | string
}

export interface NodeConfigData {
  validatorAddress: string
  configYaml: string
  commands: NodeCommands
  filename: string
}

export interface NodeCommands {
  commands: NodeCommand[]
  prerequisites: string[]
  notes: string[]
}

export interface NodeCommand {
  step: number
  title: string
  description: string
  command: string
  required: boolean
  condition?: string
}

export interface TestTransactionData {
  type: 'transfer' | 'contract_call' | 'simple'
  network: 'subnet' | 'l1'
  from: string
  to: string
  amount: string
  data: string
  gas_limit: number
}

export interface NewValidator {
  address: string
  pubkey: string
  power: number
  collateral: number
  initialBalance: number
}

export interface BulkValidator {
  address: string
  pubkey: string
  power: number
  isNew?: boolean
}

// API Request Types
export interface AddValidatorData {
  subnetId: string
  address: string
  permissionMode: 'federated' | 'collateral' | 'static'
  collateral?: number
  initialBalance?: number
  pubkey?: string
  power?: number
}

export interface RemoveValidatorData {
  subnetId: string
  address: string
}

export interface UpdateStakeData {
  subnetId: string
  address: string
  amount: number
  action: 'stake' | 'unstake'
}

export interface FederatedPowerData {
  subnetId: string
  fromAddress: string
  validators: Array<{
    address: string
    pubkey: string
    power: number
  }>
}
