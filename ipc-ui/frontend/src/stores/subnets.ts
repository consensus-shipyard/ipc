import { defineStore } from 'pinia'
import { ref, computed, watch, readonly } from 'vue'
import { apiService } from '@/services/api'
import { useNetworkStore } from './network'

export interface SubnetInstance {
  id: string
  name: string
  status: string
  template: string
  parent: string
  gateway_address?: string
  validator_count: number
  created_at: string
  deployment_id?: string
  validators: Array<{
    address: string
    stake: string
    power: number
    status: string
  }>
  config: Record<string, any>
}

export const useSubnetsStore = defineStore('subnets', () => {
  // State
  const allSubnets = ref<SubnetInstance[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const lastFetch = ref<Date | null>(null)

  // Get network store to react to network changes
  const networkStore = useNetworkStore()

  // Computed filtered subnets based on selected network
  const filteredSubnets = computed(() => {
    const selectedNetwork = networkStore.selectedNetwork
    if (!selectedNetwork) return allSubnets.value

    // Filter subnets based on network
    return allSubnets.value.filter(subnet => {
      // Check if subnet belongs to the selected network
      // This can be based on the parent network, gateway address, or other network identifiers
      const subnetNetworkId = extractNetworkFromSubnet(subnet, selectedNetwork)
      return subnetNetworkId === selectedNetwork.id
    })
  })

  // Helper function to extract network information from subnet
  const extractNetworkFromSubnet = (subnet: SubnetInstance, selectedNetwork: any): string => {
    // Use the chain ID from the selected network to construct the root network path
    if (selectedNetwork.chainId) {
      const expectedRootPath = `/r${selectedNetwork.chainId}`
      // Check if parent starts with the expected root path for this network
      if (subnet.parent.startsWith(expectedRootPath)) {
        return selectedNetwork.id
      }
    }

    // Fallback: if no chainId is available, try legacy hardcoded checks for backwards compatibility
    if (selectedNetwork.id === 'calibration') {
      return subnet.parent.startsWith('/r314159') ? 'calibration' : 'unknown'
    }

    if (selectedNetwork.id === 'local-anvil') {
      return subnet.parent.startsWith('/r31337') ? 'local-anvil' : 'unknown'
    }

    return 'unknown'
  }

  // Actions
  const fetchSubnets = async (force = false, retryCount = 0) => {
    // Don't fetch if we recently fetched and it's not forced
    if (!force && lastFetch.value && Date.now() - lastFetch.value.getTime() < 30000) {
      return
    }

    const maxRetries = 2
    const timeout = 15000 // 15 second timeout

    try {
      loading.value = true
      error.value = null

      console.log(`[SubnetsStore] Fetching subnets (attempt ${retryCount + 1})...`)

      // Create timeout promise
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Request timeout - API call took too long')), timeout)
      })

      // Race between API call and timeout
      const response = await Promise.race([
        apiService.getInstances(),
        timeoutPromise
      ]) as any

      if (response.data && response.data.data) {
        allSubnets.value = response.data.data
        lastFetch.value = new Date()
        console.log(`[SubnetsStore] Successfully fetched ${response.data.data.length} subnets`)
      }
    } catch (err: any) {
      console.error(`[SubnetsStore] Error fetching subnets (attempt ${retryCount + 1}):`, err)

      // Retry logic for network errors or timeouts
      if (retryCount < maxRetries && (
        err.message.includes('timeout') ||
        err.message.includes('Network Error') ||
        err.message.includes('ECONNREFUSED')
      )) {
        console.log(`[SubnetsStore] Retrying in 2 seconds... (${retryCount + 1}/${maxRetries})`)
        setTimeout(() => {
          fetchSubnets(force, retryCount + 1)
        }, 2000)
        return
      }

      error.value = err?.message || 'Failed to load subnets'
    } finally {
      loading.value = false
    }
  }

  // Refresh subnets
  const refreshSubnets = () => fetchSubnets(true)

  // Get subnet by ID
  const getSubnetById = (id: string) => {
    return filteredSubnets.value.find(subnet => subnet.id === id)
  }

  // Get recent subnets (for sidebar)
  const recentSubnets = computed(() => {
    return filteredSubnets.value
      .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
      .slice(0, 5)
  })

  // Get active subnets
  const activeSubnets = computed(() => {
    return filteredSubnets.value.filter(subnet =>
      subnet.status === 'active' || subnet.status === 'running'
    )
  })

  // Watch for network changes and refresh data
  watch(() => networkStore.selectedNetwork?.id, (newNetworkId, oldNetworkId) => {
    if (newNetworkId !== oldNetworkId && newNetworkId) {
      console.log(`Network changed to ${newNetworkId}, refreshing subnets...`)
      refreshSubnets()
    }
  })

  return {
    // State
    allSubnets: readonly(allSubnets),
    loading: readonly(loading),
    error: readonly(error),
    lastFetch: readonly(lastFetch),

    // Computed
    filteredSubnets,
    recentSubnets,
    activeSubnets,

    // Actions
    fetchSubnets,
    refreshSubnets,
    getSubnetById
  }
})