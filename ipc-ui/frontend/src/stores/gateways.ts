import { defineStore } from 'pinia'
import { ref, computed, watch, readonly } from 'vue'
import { apiService } from '@/services/api'
import { useNetworkStore } from './network'

export interface GatewayInfo {
  id: string
  name?: string
  address: string
  registry_address: string
  deployer_address: string
  parent_network: string
  subnet_count: number
  is_active: boolean
  deployed_at: string
}

export interface ContractInfo {
  id: string
  name: string
  type: 'gateway' | 'registry' | 'custom'
  address: string
  deployer: string
  network: string
  deployed_at: string
  status: 'active' | 'inactive'
  description?: string
  subnets_created?: number
  actions: string[]
}

export const useGatewaysStore = defineStore('gateways', () => {
  // State
  const allGateways = ref<GatewayInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const lastFetch = ref<Date | null>(null)

  // Get network store to react to network changes
  const networkStore = useNetworkStore()

  // Computed filtered gateways based on selected network
  const filteredGateways = computed(() => {
    const selectedNetwork = networkStore.selectedNetwork
    if (!selectedNetwork) return allGateways.value

    // Filter gateways based on network
    return allGateways.value.filter(gateway => {
      const gatewayNetworkId = extractNetworkFromGateway(gateway, selectedNetwork)
      return gatewayNetworkId === selectedNetwork.id
    })
  })

  // Transform gateways into contract format for the contracts view
  const contracts = computed((): ContractInfo[] => {
    const contracts: ContractInfo[] = []

    // Add gateway contracts
    filteredGateways.value.forEach(gateway => {
      contracts.push({
        id: `gateway-${gateway.id}`,
        name: gateway.name || `Gateway (${gateway.address.slice(0, 8)}...)`,
        type: 'gateway',
        address: gateway.address,
        deployer: gateway.deployer_address,
        network: gateway.parent_network,
        deployed_at: gateway.deployed_at,
        status: gateway.is_active ? 'active' : 'inactive',
        subnets_created: gateway.subnet_count,
        actions: ['inspect', 'configure', 'approve-subnets']
      })

      // Add corresponding registry contract
      if (gateway.registry_address) {
        contracts.push({
          id: `registry-${gateway.id}`,
          name: `Registry for ${gateway.name || 'Gateway'}`,
          type: 'registry',
          address: gateway.registry_address,
          deployer: gateway.deployer_address,
          network: gateway.parent_network,
          deployed_at: gateway.deployed_at,
          status: gateway.is_active ? 'active' : 'inactive',
          description: 'Subnet registry contract for managing subnet registrations',
          actions: ['inspect', 'manage-registrations']
        })
      }
    })

    return contracts
  })

  // Helper function to extract network information from gateway
  const extractNetworkFromGateway = (gateway: GatewayInfo, selectedNetwork: any): string => {
    // Use the chain ID from the selected network to construct the root network path
    if (selectedNetwork.chainId) {
      const expectedRootPath = `/r${selectedNetwork.chainId}`
      // Check if parent_network contains the expected root path for this network
      if (gateway.parent_network.includes(expectedRootPath)) {
        return selectedNetwork.id
      }
    }

    // Fallback: if no chainId is available, try legacy hardcoded checks for backwards compatibility
    if (selectedNetwork.id === 'calibration') {
      return gateway.parent_network.includes('/r314159') ? 'calibration' : 'unknown'
    }

    if (selectedNetwork.id === 'local-anvil') {
      return gateway.parent_network.includes('/r31337') ? 'local-anvil' : 'unknown'
    }

    return 'unknown'
  }

  // Actions
  const fetchGateways = async (force = false, retryCount = 0) => {
    // Don't fetch if we recently fetched and it's not forced
    if (!force && lastFetch.value && Date.now() - lastFetch.value.getTime() < 30000) {
      return
    }

    const maxRetries = 2
    const timeout = 15000 // 15 second timeout

    try {
      loading.value = true
      error.value = null

      console.log(`[GatewaysStore] Fetching gateways (attempt ${retryCount + 1})...`)

      // Create timeout promise
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Request timeout - API call took too long')), timeout)
      })

      // Race between API call and timeout
      const response = await Promise.race([
        apiService.discoverGateways(),
        timeoutPromise
      ]) as any

      if (response.data && response.data.data) {
        allGateways.value = response.data.data
        lastFetch.value = new Date()
        console.log(`[GatewaysStore] Successfully fetched ${response.data.data.length} gateways`)
      }
    } catch (err: any) {
      console.error(`[GatewaysStore] Error fetching gateways (attempt ${retryCount + 1}):`, err)

      // Retry logic for network errors or timeouts
      if (retryCount < maxRetries && (
        err.message.includes('timeout') ||
        err.message.includes('Network Error') ||
        err.message.includes('ECONNREFUSED')
      )) {
        console.log(`[GatewaysStore] Retrying in 2 seconds... (${retryCount + 1}/${maxRetries})`)
        setTimeout(() => {
          fetchGateways(force, retryCount + 1)
        }, 2000)
        return
      }

      error.value = err?.message || 'Failed to load gateways'
    } finally {
      loading.value = false
    }
  }

  // Refresh gateways
  const refreshGateways = () => fetchGateways(true)

  // Get gateway by ID
  const getGatewayById = (id: string) => {
    return filteredGateways.value.find(gateway => gateway.id === id)
  }

  // Get contract by ID
  const getContractById = (id: string) => {
    return contracts.value.find(contract => contract.id === id)
  }

  // Get active gateways
  const activeGateways = computed(() => {
    return filteredGateways.value.filter(gateway => gateway.is_active)
  })

  // Get gateway contracts only
  const gatewayContracts = computed(() => {
    return contracts.value.filter(contract => contract.type === 'gateway')
  })

  // Get registry contracts only
  const registryContracts = computed(() => {
    return contracts.value.filter(contract => contract.type === 'registry')
  })

  // Watch for network changes and refresh data
  watch(() => networkStore.selectedNetwork?.id, (newNetworkId, oldNetworkId) => {
    if (newNetworkId !== oldNetworkId && newNetworkId) {
      console.log(`Network changed to ${newNetworkId}, refreshing gateways...`)
      refreshGateways()
    }
  })

  return {
    // State
    allGateways: readonly(allGateways),
    loading: readonly(loading),
    error: readonly(error),
    lastFetch: readonly(lastFetch),

    // Computed
    filteredGateways,
    contracts,
    activeGateways,
    gatewayContracts,
    registryContracts,

    // Actions
    fetchGateways,
    refreshGateways,
    getGatewayById,
    getContractById
  }
})