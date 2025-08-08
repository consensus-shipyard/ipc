import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { apiService, type NetworkConnectionStatus } from '@/services/api'

export interface Network {
  id: string
  name: string
  rpcUrl: string
  wsUrl?: string
  chainId?: number
  type: 'mainnet' | 'testnet' | 'local' | 'custom'
  isDefault?: boolean
}

const DEFAULT_NETWORKS: Network[] = [
  {
    id: 'calibration',
    name: 'Calibration Testnet',
    rpcUrl: 'https://api.calibration.node.glif.io/rpc/v1',
    wsUrl: 'wss://api.calibration.node.glif.io/rpc/v1',
    chainId: 314159,
    type: 'testnet',
    isDefault: true
  },
  {
    id: 'local-anvil',
    name: 'Local Anvil',
    rpcUrl: 'http://localhost:8545',
    wsUrl: 'ws://localhost:8545',
    chainId: 31337,
    type: 'local',
    isDefault: true
  }
]

export const useNetworkStore = defineStore('network', () => {
  // State
  const networks = ref<Network[]>([])
  const selectedNetworkId = ref<string>('')
  const isLoading = ref(false)
  const networkStatuses = ref<Map<string, NetworkConnectionStatus>>(new Map())
  const isTestingConnection = ref(false)

  // Computed
  const selectedNetwork = computed(() => {
    // Ensure networks.value is always an array to prevent find() errors
    if (!Array.isArray(networks.value)) {
      return undefined
    }
    return networks.value.find(network => network.id === selectedNetworkId.value)
  })

  const selectedNetworkStatus = computed(() => {
    if (!selectedNetworkId.value) return null
    return networkStatuses.value.get(selectedNetworkId.value) || null
  })

  const availableNetworks = computed(() => networks.value)

  // Actions
  const initializeNetworks = async () => {
    // Load networks from localStorage or use defaults
    const savedNetworks = localStorage.getItem('ipc-networks')
    const savedSelectedId = localStorage.getItem('ipc-selected-network')

    if (savedNetworks) {
      try {
        const parsed = JSON.parse(savedNetworks)
        if (Array.isArray(parsed)) {
          networks.value = parsed
        }
      } catch (error) {
        console.error('Failed to parse saved networks:', error)
        networks.value = [...DEFAULT_NETWORKS]
      }
    } else {
      networks.value = [...DEFAULT_NETWORKS]
    }

    // Set selected network
    if (savedSelectedId && networks.value.find(n => n.id === savedSelectedId)) {
      selectedNetworkId.value = savedSelectedId
    } else {
      selectedNetworkId.value = networks.value[0]?.id || DEFAULT_NETWORKS[0].id
    }

    console.log('[NetworkStore] Networks initialized')

    // Test connection to selected network
    if (selectedNetworkId.value) {
      await testSelectedNetworkConnection()
    }
  }

  // Initialize on store creation
  initializeNetworks()

  const selectNetwork = (networkId: string) => {
    const network = networks.value.find(n => n.id === networkId)
    if (network) {
      selectedNetworkId.value = networkId
      localStorage.setItem('ipc-selected-network', networkId)

      // Test connection to newly selected network
      testSelectedNetworkConnection()

      return true
    }
    return false
  }

  const testSelectedNetworkConnection = async () => {
    const network = selectedNetwork.value
    if (!network) return

    await testNetworkConnection(network)
  }

  const testNetworkConnection = async (network: Network) => {
    isTestingConnection.value = true

    try {
      const status = await apiService.testNetworkConnection({
        network_id: network.id,
        network_name: network.name,
        rpc_url: network.rpcUrl,
        network_type: network.type
      })

      networkStatuses.value.set(network.id, status)
      console.log(`[NetworkStore] Connection test for ${network.name}:`, status.connected ? 'CONNECTED' : 'FAILED')
    } catch (error) {
      console.error(`[NetworkStore] Failed to test connection for ${network.name}:`, error)

      // Create a failed status
      const failedStatus: NetworkConnectionStatus = {
        network_id: network.id,
        network_name: network.name,
        rpc_url: network.rpcUrl,
        connected: false,
        error: error instanceof Error ? error.message : 'Connection test failed',
        last_checked: new Date().toISOString()
      }

      networkStatuses.value.set(network.id, failedStatus)
    } finally {
      isTestingConnection.value = false
    }
  }

  const testAllNetworkConnections = async () => {
    const testPromises = networks.value.map(network => testNetworkConnection(network))
    await Promise.allSettled(testPromises)
  }

  const addNetwork = (network: Omit<Network, 'id'>) => {
    const errors = validateNetwork(network)
    if (errors.length > 0) {
      throw new Error(errors.join(', '))
    }

    // Generate unique ID
    const id = `custom-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`
    const newNetwork: Network = {
      ...network,
      id,
      isDefault: false
    }

    networks.value.push(newNetwork)
    saveNetworks()

    // Test connection to new network
    testNetworkConnection(newNetwork)

    return newNetwork
  }

  const updateNetwork = (id: string, updates: Partial<Network>) => {
    const networkIndex = networks.value.findIndex(n => n.id === id)
    if (networkIndex === -1) return false

    const network = networks.value[networkIndex]

    // Don't allow updating default networks' core properties
    if (network.isDefault) {
      const allowedUpdates = { name: updates.name }
      Object.assign(network, allowedUpdates)
    } else {
      // Validate the updated network
      const updatedNetwork = { ...network, ...updates }
      const errors = validateNetwork(updatedNetwork)
      if (errors.length > 0) {
        throw new Error(errors.join(', '))
      }
      Object.assign(network, updates)
    }

    saveNetworks()

    // Re-test connection if RPC URL changed
    if (updates.rpcUrl) {
      testNetworkConnection(network)
    }

    return true
  }

  const removeNetwork = (id: string) => {
    const network = networks.value.find(n => n.id === id)
    if (!network) return false

    // Don't allow removing default networks
    if (network.isDefault) {
      throw new Error('Cannot remove default networks')
    }

    networks.value = networks.value.filter(n => n.id !== id)

    // Remove connection status
    networkStatuses.value.delete(id)

    // If the removed network was selected, select the first available
    if (selectedNetworkId.value === id) {
      selectedNetworkId.value = networks.value[0]?.id || DEFAULT_NETWORKS[0].id
      localStorage.setItem('ipc-selected-network', selectedNetworkId.value)

      // Test connection to newly selected network
      if (selectedNetworkId.value) {
        testSelectedNetworkConnection()
      }
    }

    saveNetworks()
    return true
  }

  const resetToDefaults = () => {
    networks.value = [...DEFAULT_NETWORKS]
    selectedNetworkId.value = DEFAULT_NETWORKS[0].id
    networkStatuses.value.clear()
    saveNetworks()
    localStorage.setItem('ipc-selected-network', selectedNetworkId.value)

    // Test connection to default network
    testSelectedNetworkConnection()
  }

  const saveNetworks = () => {
    localStorage.setItem('ipc-networks', JSON.stringify(networks.value))
  }

  // Validation helpers
  const validateNetwork = (network: Partial<Network>): string[] => {
    const errors: string[] = []

    if (!network.name?.trim()) {
      errors.push('Network name is required')
    }

    if (!network.rpcUrl?.trim()) {
      errors.push('RPC URL is required')
    } else {
      try {
        new URL(network.rpcUrl)
      } catch {
        errors.push('RPC URL must be a valid URL')
      }
    }

    if (network.wsUrl?.trim()) {
      try {
        new URL(network.wsUrl)
      } catch {
        errors.push('WebSocket URL must be a valid URL')
      }
    }

    if (network.chainId !== undefined && (network.chainId < 1 || !Number.isInteger(network.chainId))) {
      errors.push('Chain ID must be a positive integer')
    }

    return errors
  }

  const isNetworkNameUnique = (name: string, excludeId?: string): boolean => {
    return !networks.value.some(n =>
      n.name.toLowerCase() === name.toLowerCase() && n.id !== excludeId
    )
  }

  return {
    // State
    networks: availableNetworks,
    selectedNetwork,
    selectedNetworkId: computed(() => selectedNetworkId.value),
    selectedNetworkStatus,
    networkStatuses: computed(() => networkStatuses.value),
    isLoading,
    isTestingConnection,

    // Actions
    selectNetwork,
    addNetwork,
    updateNetwork,
    removeNetwork,
    resetToDefaults,
    validateNetwork,
    isNetworkNameUnique,
    initializeNetworks,
    testSelectedNetworkConnection,
    testNetworkConnection,
    testAllNetworkConnections
  }
})