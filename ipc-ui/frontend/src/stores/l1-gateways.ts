import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { apiService } from '@/services/api'
import { useNetworkStore } from './network'

export interface L1GatewayConfig {
  id: string
  name: string
  address: string
  registry_address: string
  network_id: string
  network_name: string
  chain_id: number
  deployed_at: string
  deployer_address: string
  is_default: boolean
  description?: string
}

export interface L1GatewayConfigFile {
  default_gateway?: string
  gateways: L1GatewayConfig[]
  last_updated: string
}

export const useL1GatewaysStore = defineStore('l1Gateways', () => {
  // State
  const l1Gateways = ref<L1GatewayConfig[]>([])
  const selectedGatewayId = ref<string>('')
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Get network store for current network context
  const networkStore = useNetworkStore()

  // Computed
  const selectedGateway = computed(() => {
    return l1Gateways.value.find(gateway => gateway.id === selectedGatewayId.value)
  })

  const availableGateways = computed(() => {
    // Filter gateways for current network
    const currentNetwork = networkStore.selectedNetwork
    if (!currentNetwork) {
      console.log('[L1GatewaysStore] No current network selected, showing all gateways')
      return l1Gateways.value
    }

    // Map network ID to expected subnet path using chainId
    const expectedRootPath = currentNetwork.chainId ? `/r${currentNetwork.chainId}` : null

    const filtered = l1Gateways.value.filter(gateway => {
      // Check if gateway's network_id matches the expected root path for current network
      if (expectedRootPath && gateway.network_id === expectedRootPath) {
        return true
      }

      // Fallback: legacy hardcoded checks for backwards compatibility
      if (currentNetwork.id === 'calibration' && gateway.network_id === '/r314159') {
        return true
      }

      if (currentNetwork.id === 'local-anvil' && gateway.network_id === '/r31337') {
        return true
      }

      return false
    })

    console.log(`[L1GatewaysStore] Filtered ${filtered.length} gateways for network ${currentNetwork.id} (${expectedRootPath})`)
    return filtered
  })

  const defaultGateway = computed(() => {
    return availableGateways.value.find(gateway => gateway.is_default) || availableGateways.value[0]
  })

  // Actions
  const loadL1Gateways = async () => {
    isLoading.value = true
    error.value = null

    try {
      console.log('[L1GatewaysStore] Loading L1 gateways...')

      // Load L1 gateways configuration file
      const response = await apiService.getL1GatewayConfig()

      console.log('[L1GatewaysStore] API response:', response.data)

      // The API returns data in ApiResponse format: { success, data: { gateways, default_gateway, ... } }
      if (response.data?.data?.gateways) {
        l1Gateways.value = response.data.data.gateways
        console.log('[L1GatewaysStore] Loaded gateways:', l1Gateways.value)

        // Set selected gateway from config or default
        if (response.data.data.default_gateway) {
          selectedGatewayId.value = response.data.data.default_gateway
          console.log('[L1GatewaysStore] Set selected gateway from config:', selectedGatewayId.value)
        } else if (defaultGateway.value) {
          selectedGatewayId.value = defaultGateway.value.id
          console.log('[L1GatewaysStore] Set selected gateway to default:', selectedGatewayId.value)
        }
      } else {
        console.warn('[L1GatewaysStore] No gateways found in response')
      }
    } catch (err) {
      console.error('Failed to load L1 gateways:', err)
      error.value = err instanceof Error ? err.message : 'Failed to load L1 gateways'
    } finally {
      isLoading.value = false
    }
  }

  const selectGateway = async (gatewayId: string) => {
    if (selectedGatewayId.value === gatewayId) return

    const gateway = l1Gateways.value.find(g => g.id === gatewayId)
    if (!gateway) {
      throw new Error(`Gateway with ID ${gatewayId} not found`)
    }

    selectedGatewayId.value = gatewayId

    try {
      // Update config.toml with new gateway selection
      await apiService.updateL1GatewaySelection(gatewayId)
      console.log(`Selected L1 gateway: ${gateway.name} (${gateway.address})`)
    } catch (err) {
      console.error('Failed to update gateway selection:', err)
      error.value = err instanceof Error ? err.message : 'Failed to update gateway selection'
      throw err
    }
  }

  const addL1Gateway = async (gateway: Omit<L1GatewayConfig, 'id' | 'deployed_at'>) => {
    try {
      const response = await apiService.addL1Gateway(gateway)

      if (response.data?.gateway) {
        l1Gateways.value.push(response.data.gateway)

        // If this is the first gateway for this network, make it default
        if (availableGateways.value.length === 1) {
          await selectGateway(response.data.gateway.id)
        }
      }

      return response.data?.gateway
    } catch (err) {
      console.error('Failed to add L1 gateway:', err)
      error.value = err instanceof Error ? err.message : 'Failed to add L1 gateway'
      throw err
    }
  }

  const removeL1Gateway = async (gatewayId: string) => {
    try {
      await apiService.removeL1Gateway(gatewayId)

      const index = l1Gateways.value.findIndex(g => g.id === gatewayId)
      if (index >= 0) {
        l1Gateways.value.splice(index, 1)

        // If we removed the selected gateway, select another one
        if (selectedGatewayId.value === gatewayId && defaultGateway.value) {
          await selectGateway(defaultGateway.value.id)
        }
      }
    } catch (err) {
      console.error('Failed to remove L1 gateway:', err)
      error.value = err instanceof Error ? err.message : 'Failed to remove L1 gateway'
      throw err
    }
  }

  const updateGatewayAsDefault = async (gatewayId: string) => {
    try {
      // Remove default from all gateways
      l1Gateways.value.forEach(g => g.is_default = false)

      // Set new default
      const gateway = l1Gateways.value.find(g => g.id === gatewayId)
      if (gateway) {
        gateway.is_default = true
        await selectGateway(gatewayId)
      }
    } catch (err) {
      console.error('Failed to update default gateway:', err)
      throw err
    }
  }

  // Watch network changes to load appropriate gateways
  watch(() => networkStore.selectedNetworkId, async (newNetworkId) => {
    if (newNetworkId) {
      await loadL1Gateways()
    }
  })

  return {
    // State
    l1Gateways: computed(() => l1Gateways.value),
    selectedGatewayId: computed(() => selectedGatewayId.value),
    selectedGateway,
    availableGateways,
    defaultGateway,
    isLoading: computed(() => isLoading.value),
    error: computed(() => error.value),

    // Actions
    loadL1Gateways,
    selectGateway,
    addL1Gateway,
    removeL1Gateway,
    updateGatewayAsDefault
  }
})