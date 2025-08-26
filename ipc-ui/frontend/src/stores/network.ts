import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
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

// Connection testing configuration
const CONNECTION_TEST_INTERVAL = 30000 // 30 seconds
const FAST_RETRY_INTERVAL = 5000 // 5 seconds for fast retries when disconnected
const MAX_FAST_RETRIES = 3 // Number of fast retries before falling back to normal interval

export const useNetworkStore = defineStore('network', () => {
  // State
  const networks = ref<Network[]>([])
  const selectedNetworkId = ref<string>('')
  const isLoading = ref(false)
  const networkStatuses = ref<Map<string, NetworkConnectionStatus>>(new Map())
  const isTestingConnection = ref(false)
  const connectionTestInterval = ref<number | null>(null)
  const fastRetryCount = ref(0)
  const isPeriodicTestingEnabled = ref(true)

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

  // Watch for network selection changes to restart connection testing
  watch(selectedNetworkId, (newNetworkId, oldNetworkId) => {
    if (newNetworkId !== oldNetworkId && newNetworkId) {
      console.log(`[NetworkStore] Network changed from ${oldNetworkId} to ${newNetworkId}`)

      // Reset fast retry count
      fastRetryCount.value = 0

      // Test connection immediately
      testSelectedNetworkConnection()

      // Restart periodic testing
      startPeriodicConnectionTesting()
    }
  })

  // Actions
  const selectNetwork = (networkId: string) => {
    const network = networks.value.find(n => n.id === networkId)
    if (network) {
      selectedNetworkId.value = networkId
      localStorage.setItem('ipc-selected-network', networkId)
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
    if (isTestingConnection.value) {
      console.log(`[NetworkStore] Connection test already in progress for ${network.name}`)
      return
    }

    isTestingConnection.value = true

    try {
      console.log(`[NetworkStore] Testing connection to ${network.name} (${network.rpcUrl})`)

      const status = await apiService.testNetworkConnection({
        network_id: network.id,
        network_name: network.name,
        rpc_url: network.rpcUrl,
        network_type: network.type
      })

      // Update the status
      const oldStatus = networkStatuses.value.get(network.id)
      networkStatuses.value.set(network.id, status)

      // Log status change
      if (!oldStatus || oldStatus.connected !== status.connected) {
        console.log(`[NetworkStore] Connection status changed for ${network.name}: ${status.connected ? 'CONNECTED' : 'DISCONNECTED'}`)

        if (status.connected) {
          fastRetryCount.value = 0 // Reset fast retry count on successful connection
        }
      }

      // If this is the selected network and it just connected, notify
      if (network.id === selectedNetworkId.value && status.connected && (!oldStatus || !oldStatus.connected)) {
        console.log(`[NetworkStore] Selected network ${network.name} is now connected`)
      }

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

      const oldStatus = networkStatuses.value.get(network.id)
      networkStatuses.value.set(network.id, failedStatus)

      // Log status change
      if (!oldStatus || oldStatus.connected !== failedStatus.connected) {
        console.log(`[NetworkStore] Connection failed for ${network.name}: ${failedStatus.error}`)
      }
    } finally {
      isTestingConnection.value = false
    }
  }

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

    // Test connection to selected network and start periodic testing
    if (selectedNetworkId.value) {
      await testSelectedNetworkConnection()
      startPeriodicConnectionTesting()
    }
  }

  // Initialize on store creation
  initializeNetworks()

  const startPeriodicConnectionTesting = () => {
    // Clear existing interval
    stopPeriodicConnectionTesting()

    if (!isPeriodicTestingEnabled.value) {
      console.log('[NetworkStore] Periodic connection testing is disabled')
      return
    }

    console.log('[NetworkStore] Starting periodic connection testing')

    const runPeriodicTest = () => {
      if (!selectedNetwork.value) return

      const currentStatus = selectedNetworkStatus.value
      let nextInterval = CONNECTION_TEST_INTERVAL

      // Use fast retry interval if we're disconnected and haven't exceeded max fast retries
      if (!currentStatus?.connected && fastRetryCount.value < MAX_FAST_RETRIES) {
        nextInterval = FAST_RETRY_INTERVAL
        fastRetryCount.value++
        console.log(`[NetworkStore] Using fast retry (${fastRetryCount.value}/${MAX_FAST_RETRIES}) - next test in ${nextInterval/1000}s`)
      } else if (!currentStatus?.connected) {
        console.log(`[NetworkStore] Max fast retries reached - using normal interval (${nextInterval/1000}s)`)
      }

      // Test the current selected network
      testSelectedNetworkConnection().then(() => {
        // Schedule next test
        if (isPeriodicTestingEnabled.value && selectedNetwork.value) {
          connectionTestInterval.value = setTimeout(runPeriodicTest, nextInterval)
        }
      })
    }

    // Schedule first test
    connectionTestInterval.value = setTimeout(runPeriodicTest, CONNECTION_TEST_INTERVAL)
  }

  const stopPeriodicConnectionTesting = () => {
    if (connectionTestInterval.value) {
      clearTimeout(connectionTestInterval.value)
      connectionTestInterval.value = null
      console.log('[NetworkStore] Stopped periodic connection testing')
    }
  }

  const enablePeriodicTesting = () => {
    isPeriodicTestingEnabled.value = true
    startPeriodicConnectionTesting()
  }

  const disablePeriodicTesting = () => {
    isPeriodicTestingEnabled.value = false
    stopPeriodicConnectionTesting()
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
    }

    saveNetworks()
    return true
  }

  const resetToDefaults = () => {
    networks.value = [...DEFAULT_NETWORKS]
    selectedNetworkId.value = DEFAULT_NETWORKS[0].id
    networkStatuses.value.clear()
    fastRetryCount.value = 0
    saveNetworks()
    localStorage.setItem('ipc-selected-network', selectedNetworkId.value)

    // Restart connection testing
    startPeriodicConnectionTesting()
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

  // Cleanup function for when the store is destroyed
  const cleanup = () => {
    stopPeriodicConnectionTesting()
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
    isPeriodicTestingEnabled: computed(() => isPeriodicTestingEnabled.value),

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
    testAllNetworkConnections,
    startPeriodicConnectionTesting,
    stopPeriodicConnectionTesting,
    enablePeriodicTesting,
    disablePeriodicTesting,
    cleanup
  }
})