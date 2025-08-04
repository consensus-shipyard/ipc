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
  const selectedNetwork = computed(() =>
    networks.value.find(network => network.id === selectedNetworkId.value)
  )

  const availableNetworks = computed(() => networks.value)

  // Actions
  const initializeNetworks = () => {
    // Load networks from localStorage or use defaults
    const savedNetworks = localStorage.getItem('ipc-networks')
    const savedSelectedId = localStorage.getItem('ipc-selected-network')

    if (savedNetworks) {
      try {
        networks.value = JSON.parse(savedNetworks)
      } catch (error) {
        console.warn('Failed to parse saved networks, using defaults:', error)
        networks.value = [...DEFAULT_NETWORKS]
      }
    } else {
      networks.value = [...DEFAULT_NETWORKS]
    }

    if (savedSelectedId && networks.value.find(n => n.id === savedSelectedId)) {
      selectedNetworkId.value = savedSelectedId
    } else {
      // Select first network as default
      selectedNetworkId.value = networks.value[0]?.id || ''
    }
  }

  const selectNetwork = (networkId: string) => {
    const network = networks.value.find(n => n.id === networkId)
    if (network) {
      selectedNetworkId.value = networkId
      localStorage.setItem('ipc-selected-network', networkId)
    }
  }

  const addNetwork = (network: Omit<Network, 'id'>) => {
    const newNetwork: Network = {
      ...network,
      id: `custom-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      isDefault: false
    }

    networks.value.push(newNetwork)
    saveNetworks()

    return newNetwork.id
  }

  const updateNetwork = (networkId: string, updates: Partial<Omit<Network, 'id' | 'isDefault'>>) => {
    const index = networks.value.findIndex(n => n.id === networkId)
    if (index !== -1) {
      // Don't allow updating default networks' core properties
      if (networks.value[index].isDefault) {
        // Only allow updating name for default networks
        if (updates.name) {
          networks.value[index].name = updates.name
        }
      } else {
        networks.value[index] = { ...networks.value[index], ...updates }
      }
      saveNetworks()
      return true
    }
    return false
  }

  const removeNetwork = (networkId: string) => {
    const network = networks.value.find(n => n.id === networkId)

    // Don't allow removing default networks
    if (network?.isDefault) {
      return false
    }

    const index = networks.value.findIndex(n => n.id === networkId)
    if (index !== -1) {
      networks.value.splice(index, 1)

      // If the removed network was selected, select the first available network
      if (selectedNetworkId.value === networkId) {
        selectedNetworkId.value = networks.value[0]?.id || ''
        localStorage.setItem('ipc-selected-network', selectedNetworkId.value)
      }

      saveNetworks()
      return true
    }
    return false
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

  // Initialize on store creation
  initializeNetworks()

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