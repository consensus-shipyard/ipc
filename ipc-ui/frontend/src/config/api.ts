// API Configuration
export const API_CONFIG = {
  baseURL: import.meta.env.VITE_API_BASE_URL || '', // Use relative URLs with Vite proxy
  wsURL: import.meta.env.VITE_WS_URL || `ws://${window.location.host}/ws`,
  timeout: 30000, // Increased to 30 seconds for blockchain operations
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
  discoverGateways: '/api/gateways-discover',
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
}