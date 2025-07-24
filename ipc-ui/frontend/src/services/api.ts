import axios, { type AxiosInstance, type AxiosResponse, AxiosError } from 'axios'
import { API_CONFIG, API_ENDPOINTS } from '../config/api'

// Create axios instance
const api: AxiosInstance = axios.create({
  baseURL: API_CONFIG.baseURL,
  timeout: API_CONFIG.timeout,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor
api.interceptors.request.use(
  (config) => {
    console.log(`API Request: ${config.method?.toUpperCase()} ${config.url}`)
    return config
  },
  (error) => {
    console.error('API Request Error:', error)
    return Promise.reject(error)
  }
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
const retryRequest = async (fn: () => Promise<any>, attempts = API_CONFIG.retryAttempts): Promise<any> => {
  try {
    return await fn()
  } catch (error) {
    if (attempts > 0 && axios.isAxiosError(error) && error.code !== 'ERR_CANCELED') {
      console.log(`Retrying request in ${API_CONFIG.retryDelay}ms... (${API_CONFIG.retryAttempts - attempts + 1}/${API_CONFIG.retryAttempts})`)
      await new Promise(resolve => setTimeout(resolve, API_CONFIG.retryDelay))
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

  // Instances
  async getInstances() {
    return retryRequest(() => api.get(API_ENDPOINTS.instances))
  },

  async getInstance(id: string) {
    return retryRequest(() => api.get(API_ENDPOINTS.instance(id)))
  },

  // Gateways
  async getGateways() {
    return retryRequest(() => api.get(API_ENDPOINTS.gateways))
  },

  async getGateway(id: string) {
    return retryRequest(() => api.get(API_ENDPOINTS.gateway(id)))
  },

  async discoverGateways() {
    return retryRequest(() => api.get(API_ENDPOINTS.discoverGateways))
  },

  // Deployment
  async deploy(config: any) {
    return retryRequest(() => api.post(API_ENDPOINTS.deploy, config))
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