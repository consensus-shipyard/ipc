import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

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

  // Computed
  const selectedNetwork = computed(() => {
    // Ensure networks.value is always an array to prevent find() errors
    if (!Array.isArray(networks.value)) {
      return undefined
    }
    return networks.value.find(network => network.id === selectedNetworkId.value)
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
  }

  // Initialize on store creation
  initializeNetworks()

  const selectNetwork = (networkId: string) => {
    const network = networks.value.find(n => n.id === networkId)
    if (network) {
      selectedNetworkId.value = networkId
      localStorage.setItem('ipc-selected-network', networkId)
      return true
    }
    return false
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
    saveNetworks()
    localStorage.setItem('ipc-selected-network', selectedNetworkId.value)
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
    isLoading,

    // Actions
    selectNetwork,
    addNetwork,
    updateNetwork,
    removeNetwork,
    resetToDefaults,
    validateNetwork,
    isNetworkNameUnique,
    initializeNetworks
  }
})