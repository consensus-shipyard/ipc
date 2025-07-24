// API Configuration
export const API_CONFIG = {
  baseURL: import.meta.env.VITE_API_BASE_URL || '', // Use relative URLs with Vite proxy
  wsURL: import.meta.env.VITE_WS_URL || `ws://${window.location.host}/ws`,
  timeout: 10000,
  retryAttempts: 3,
  retryDelay: 1000,
}

// API endpoints
export const API_ENDPOINTS = {
  templates: '/api/templates',
  instances: '/api/instances',
  deploy: '/api/deploy',
  instance: (id: string) => `/api/instances/${id}`,
  gateways: '/api/gateways',
  gateway: (id: string) => `/api/gateways/${id}`,
  discoverGateways: '/api/gateways/discover',
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