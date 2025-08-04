import { computed } from 'vue'
import { useNetworkStore } from '@/stores/network'

// API Configuration - now reactive to selected network
const getNetworkStore = () => {
  try {
    return useNetworkStore()
  } catch {
    // If store is not available (during SSR or initial load), return defaults
    return null
  }
}

export const API_CONFIG = computed(() => {
  const networkStore = getNetworkStore()
  const selectedNetwork = networkStore?.selectedNetwork

  return {
    baseURL: import.meta.env.VITE_API_BASE_URL || '', // Use relative URLs with Vite proxy
    wsURL: import.meta.env.VITE_WS_URL || `ws://${window.location.host}/ws`,
    timeout: 30000, // Increased to 30 seconds for blockchain operations
    retryAttempts: 3,
    retryDelay: 1000,
    // Network-specific configuration
    networkRpcUrl: selectedNetwork?.rpcUrl,
    networkWsUrl: selectedNetwork?.wsUrl,
    networkChainId: selectedNetwork?.chainId,
    networkType: selectedNetwork?.type,
    networkId: selectedNetwork?.id,
  }
})

// Static configuration for operations that don't depend on network
export const STATIC_API_CONFIG = {
  baseURL: import.meta.env.VITE_API_BASE_URL || '',
  wsURL: import.meta.env.VITE_WS_URL || `ws://${window.location.host}/ws`,
  timeout: 30000,
  retryAttempts: 3,
  retryDelay: 1000,
}

// Specific timeouts for different operations
export const OPERATION_TIMEOUTS = {
  default: 30000,      // 30 seconds for general operations
  blockchain: 45000,   // 45 seconds for blockchain calls
  deployment: 60000,   // 60 seconds for deployments
  approval: 45000,     // 45 seconds for subnet approvals
}

// API endpoints
export const API_ENDPOINTS = {
  templates: '/api/templates',
  instances: '/api/instances',
  deploy: '/api/deploy',
  instance: (id: string) => `/api/instance?id=${encodeURIComponent(id)}`,
  gateways: '/api/gateways',
  gateway: (id: string) => `/api/gateways/${id}`,
  discoverGateways: '/api/gateways/discover',
  // Subnet statistics endpoints
  subnetStats: (id: string) => `/api/subnet/${encodeURIComponent(id)}/stats`,
  subnetStatus: (id: string) => `/api/subnet/${encodeURIComponent(id)}/status`,
  sendTestTx: (id: string) => `/api/subnet/${encodeURIComponent(id)}/test-transaction`,
  // Validator management endpoints
  validators: {
    add: '/api/validators/add',
    remove: '/api/validators/remove',
    updateStake: '/api/validators/update-stake',
    setFederatedPower: '/api/validators/set-federated-power',
  },
}

// Helper function to get current network headers
export const getNetworkHeaders = () => {
  const networkStore = getNetworkStore()
  const selectedNetwork = networkStore?.selectedNetwork

  if (!selectedNetwork) {
    return {}
  }

  return {
    'X-Network-Id': selectedNetwork.id,
    'X-Network-Name': selectedNetwork.name,
    'X-Network-RPC-URL': selectedNetwork.rpcUrl,
    'X-Network-Type': selectedNetwork.type,
    ...(selectedNetwork.wsUrl && { 'X-Network-WS-URL': selectedNetwork.wsUrl }),
    ...(selectedNetwork.chainId && { 'X-Network-Chain-ID': selectedNetwork.chainId.toString() }),
  }
}

// WebSocket message types
export interface WSMessage {
  type: string
  data?: any
  error?: string
}

export interface DeploymentProgress {
  deployment_id: string
  step: string
  progress: number
  status: 'pending' | 'in_progress' | 'completed' | 'failed'
  message?: string
  error?: string
  subnet_id?: string // The actual subnet ID generated during deployment
}