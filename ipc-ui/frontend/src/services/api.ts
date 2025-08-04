import axios, { type AxiosInstance, type AxiosResponse, AxiosError } from 'axios'
import { STATIC_API_CONFIG, API_ENDPOINTS, OPERATION_TIMEOUTS, getNetworkHeaders } from '../config/api'

// Create axios instance
const api: AxiosInstance = axios.create({
  baseURL: STATIC_API_CONFIG.baseURL,
  timeout: STATIC_API_CONFIG.timeout,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Create a separate instance for blockchain operations with longer timeout
const blockchainApi: AxiosInstance = axios.create({
  baseURL: STATIC_API_CONFIG.baseURL,
  timeout: OPERATION_TIMEOUTS.blockchain,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Create instance for deployment operations with even longer timeout
const deploymentApi: AxiosInstance = axios.create({
  baseURL: STATIC_API_CONFIG.baseURL,
  timeout: OPERATION_TIMEOUTS.deployment,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Create instance for approval operations
const approvalApi: AxiosInstance = axios.create({
  baseURL: STATIC_API_CONFIG.baseURL,
  timeout: OPERATION_TIMEOUTS.approval,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Function to add network headers to all axios instances
const addNetworkHeaders = () => {
  const networkHeaders = getNetworkHeaders()

  // Update headers for all instances
  Object.assign(api.defaults.headers, networkHeaders)
  Object.assign(blockchainApi.defaults.headers, networkHeaders)
  Object.assign(deploymentApi.defaults.headers, networkHeaders)
  Object.assign(approvalApi.defaults.headers, networkHeaders)
}

// Request interceptor
api.interceptors.request.use(
  (config) => {
    // Add network headers to each request
    const networkHeaders = getNetworkHeaders()
    Object.assign(config.headers, networkHeaders)

    console.log(`API Request: ${config.method?.toUpperCase()} ${config.url}`)
    if (Object.keys(networkHeaders).length > 0) {
      console.log('Network Headers:', networkHeaders)
    }
    return config
  },
  (error) => {
    console.error('API Request Error:', error)
    return Promise.reject(error)
  }
)

// Add request interceptors to other instances as well
blockchainApi.interceptors.request.use(
  (config) => {
    const networkHeaders = getNetworkHeaders()
    Object.assign(config.headers, networkHeaders)
    console.log(`Blockchain API Request: ${config.method?.toUpperCase()} ${config.url}`)
    return config
  },
  (error) => Promise.reject(error)
)

deploymentApi.interceptors.request.use(
  (config) => {
    const networkHeaders = getNetworkHeaders()
    Object.assign(config.headers, networkHeaders)
    console.log(`Deployment API Request: ${config.method?.toUpperCase()} ${config.url}`)
    return config
  },
  (error) => Promise.reject(error)
)

approvalApi.interceptors.request.use(
  (config) => {
    const networkHeaders = getNetworkHeaders()
    Object.assign(config.headers, networkHeaders)
    console.log(`Approval API Request: ${config.method?.toUpperCase()} ${config.url}`)
    return config
  },
  (error) => Promise.reject(error)
)

// Response interceptor
api.interceptors.response.use(
  (response: AxiosResponse) => {
    console.log(`API Response: ${response.status} ${response.config.url}`)
    return response
  },
  (error: AxiosError) => {
    console.error('API Response Error:', error.response?.status, error.message)

    // Handle common error scenarios
    if (error.response?.status === 404) {
      console.warn('Resource not found:', error.config?.url)
    } else if (error.response?.status === 500) {
      console.error('Server error:', error.response.data)
    } else if (error.code === 'ECONNREFUSED' || error.code === 'ERR_NETWORK') {
      console.error('Backend server is not reachable. Make sure ipc-cli ui is running.')
    }

    return Promise.reject(error)
  }
)

// Retry function for failed requests
const retryRequest = async (fn: () => Promise<any>, attempts = STATIC_API_CONFIG.retryAttempts): Promise<any> => {
  try {
    return await fn()
  } catch (error) {
    if (attempts > 0 && axios.isAxiosError(error) && error.code !== 'ERR_CANCELED') {
      console.log(`Retrying request in ${STATIC_API_CONFIG.retryDelay}ms... (${STATIC_API_CONFIG.retryAttempts - attempts + 1}/${STATIC_API_CONFIG.retryAttempts})`)
      await new Promise(resolve => setTimeout(resolve, STATIC_API_CONFIG.retryDelay))
      return retryRequest(fn, attempts - 1)
    }
    throw error
  }
}

// API service functions
export const apiService = {
  // Health check
  async healthCheck(): Promise<boolean> {
    try {
      await api.get('/api/health')
      return true
    } catch {
      return false
    }
  },

  // Templates
  async getTemplates() {
    return retryRequest(() => api.get(API_ENDPOINTS.templates))
  },

  // Instances (using blockchain API for longer timeout)
  async getInstances() {
    return retryRequest(() => blockchainApi.get(API_ENDPOINTS.instances))
  },

  async getInstance(id: string) {
    return retryRequest(() => blockchainApi.get(API_ENDPOINTS.instance(id)))
  },

  // Chain statistics for subnets
  async getSubnetStats(subnetId: string) {
    return retryRequest(() => blockchainApi.get(API_ENDPOINTS.subnetStats(subnetId)))
  },

  async getSubnetStatus(subnetId: string) {
    return retryRequest(() => blockchainApi.get(API_ENDPOINTS.subnetStatus(subnetId)))
  },

  // Test transaction functionality
  async sendTestTransaction(subnetId: string, testTxData: TestTransactionRequest) {
    return retryRequest(() => blockchainApi.post(API_ENDPOINTS.sendTestTx(subnetId), testTxData))
  },

  // Gateways
  async getGateways() {
    return retryRequest(() => api.get(API_ENDPOINTS.gateways))
  },

  async getGateway(id: string) {
    return retryRequest(() => api.get(API_ENDPOINTS.gateway(id)))
  },

  async discoverGateways() {
    return retryRequest(() => api.post(API_ENDPOINTS.discoverGateways))
  },

  // Contracts
  async getContracts() {
    return retryRequest(() => api.get('/api/contracts'))
  },

  async getContract(id: string) {
    return retryRequest(() => api.get(`/api/contracts/${id}`))
  },

  async inspectContract(contractAddress: string) {
    return retryRequest(() => api.get(`/api/contracts/inspect/${contractAddress}`))
  },

  async configureContract(contractId: string, config: any) {
    return retryRequest(() => api.put(`/api/contracts/${contractId}/configure`, config))
  },

  async upgradeContract(contractId: string, upgradeData: any) {
    return retryRequest(() => api.post(`/api/contracts/${contractId}/upgrade`, upgradeData))
  },

  async getContractABI(contractAddress: string) {
    return retryRequest(() => api.get(`/api/contracts/${contractAddress}/abi`))
  },

  // Deployment (using deployment API for extended timeout)
  async deploy(config: DeploymentRequest) {
    return retryRequest(() => deploymentApi.post(API_ENDPOINTS.deploy, config))
  },

  // Subnet approval (using approval API for extended timeout)
  async approveSubnet(subnetId: string, gatewayOwnerAddress: string) {
    return retryRequest(() => approvalApi.post(`/api/subnets/${encodeURIComponent(subnetId)}/approve`, {
      from: gatewayOwnerAddress
    }))
  },

  // Validator management
  async addValidator(validatorData: ValidatorManagementRequest) {
    return retryRequest(() => api.post(API_ENDPOINTS.validators.add, validatorData))
  },

  async removeValidator(validatorData: ValidatorRemovalRequest) {
    return retryRequest(() => api.post(API_ENDPOINTS.validators.remove, validatorData))
  },

  async updateValidatorStake(stakeData: ValidatorStakeUpdateRequest) {
    return retryRequest(() => api.post(API_ENDPOINTS.validators.updateStake, stakeData))
  },

  async setFederatedPower(powerData: FederatedPowerRequest) {
    return retryRequest(() => api.post(API_ENDPOINTS.validators.setFederatedPower, powerData))
  },

  // Configuration management
  async saveConfig(config: any) {
    return retryRequest(() => api.post('/api/config/save', config))
  },

  async loadConfig(name: string) {
    return retryRequest(() => api.get(`/api/config/${name}`))
  },

  async listConfigs() {
    return retryRequest(() => api.get('/api/config/list'))
  },
}

export default api

interface DeploymentProgress {
  deployment_id: string
  step: string
  progress: number
  status: 'pending' | 'in_progress' | 'completed' | 'failed'
  message?: string
  error?: string
  subnet_id?: string // The actual subnet ID generated during deployment
}

// Validator management interfaces
interface ValidatorManagementRequest {
  subnetId: string
  address: string
  permissionMode: 'federated' | 'collateral' | 'static'
  collateral?: number // For collateral mode
  initialBalance?: number // For collateral mode
  pubkey?: string // For federated mode
  power?: number // For federated mode
}

interface ValidatorRemovalRequest {
  subnetId: string
  address: string
}

interface ValidatorStakeUpdateRequest {
  subnetId: string
  address: string
  amount: number
  action: 'stake' | 'unstake'
}

interface FederatedPowerRequest {
  subnetId: string
  fromAddress: string
  validators: Array<{
    address: string
    pubkey: string
    power: number
  }>
}

interface DeploymentRequest {
  template: string
  config: any
}

// Test transaction interface
interface TestTransactionRequest {
  type: 'transfer' | 'contract_call' | 'simple'
  network: 'subnet' | 'l1'
  from?: string
  to?: string
  amount?: string
  data?: string
  gas_limit?: number
}

// Chain statistics interfaces
interface ChainStats {
  block_height: number
  latest_block_time: string
  transaction_count: number
  validator_count: number
  tps: number
  avg_block_time: number
  last_checkpoint: string
  network_hash_rate?: string
  pending_transactions?: number
}

interface SubnetStatus {
  is_active: boolean
  last_block_time: string
  block_height: number
  validators_online: number
  consensus_status: 'healthy' | 'degraded' | 'offline'
  sync_status: 'synced' | 'syncing' | 'behind'
}