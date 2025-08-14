import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { apiService } from '@/services/api'

export enum SubnetLifecycleState {
  Deploying = 'deploying',
  Deployed = 'deployed',
  Initializing = 'initializing',
  WaitingForValidators = 'waiting_for_validators',
  Active = 'active',
  Syncing = 'syncing',
  Healthy = 'healthy',
  Degraded = 'degraded',
  Offline = 'offline',
  Failed = 'failed',
  Unknown = 'unknown'
}

export interface SubnetStatusInfo {
  lifecycle_state: SubnetLifecycleState
  genesis_available: boolean
  validator_count: number
  active_validators: number
  permission_mode?: string
  deployment_time?: string
  last_block_time?: string
  error_message?: string
  next_action_required?: string
}

export interface ValidatorInfo {
  address: string
  stake: string
  status: string
  power?: string
  is_active?: boolean
}

export interface SubnetInstance {
  id: string
  name: string
  parent: string
  status: SubnetLifecycleState
  status_info: SubnetStatusInfo
  validators: ValidatorInfo[]
  created_at?: string
  config?: {
    permissionMode?: string
    gateway_addr?: string
    registry_addr?: string
    [key: string]: any
  }
  // Per-subnet loading state
  isLoading?: boolean
  loadError?: string | null
}

export interface SubnetHierarchyNode {
  subnet: SubnetInstance
  children: SubnetHierarchyNode[]
  depth: number
}

export const useSubnetsStore = defineStore('subnets', () => {
  // State
  const subnets = ref<SubnetInstance[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const lastUpdated = ref<Date | null>(null)
  const subnetLoadingStates = ref<Map<string, boolean>>(new Map())
  const subnetErrors = ref<Map<string, string>>(new Map())

  // Computed
  const subnetCount = computed(() => subnets.value.length)

  const healthySubnets = computed(() =>
    subnets.value.filter(s => s.status === SubnetLifecycleState.Healthy)
  )

  const problematicSubnets = computed(() =>
    subnets.value.filter(s => [
      SubnetLifecycleState.Failed,
      SubnetLifecycleState.Offline,
      SubnetLifecycleState.Degraded
    ].includes(s.status))
  )

  const waitingSubnets = computed(() =>
    subnets.value.filter(s => [
      SubnetLifecycleState.WaitingForValidators,
      SubnetLifecycleState.Initializing,
      SubnetLifecycleState.Deploying
    ].includes(s.status))
  )

  const subnetsByParent = computed(() => {
    const groups: { [parent: string]: SubnetInstance[] } = {}
    subnets.value.forEach(subnet => {
      const parent = subnet.parent || 'unknown'
      if (!groups[parent]) {
        groups[parent] = []
      }
      groups[parent].push(subnet)
    })
    return groups
  })

  const subnetHierarchy = computed((): SubnetHierarchyNode[] => {
    const nodeMap = new Map<string, SubnetHierarchyNode>()
    const roots: SubnetHierarchyNode[] = []

    // Create nodes for all subnets
    subnets.value.forEach(subnet => {
      nodeMap.set(subnet.id, {
        subnet,
        children: [],
        depth: 0
      })
    })

    // Build hierarchy
    subnets.value.forEach(subnet => {
      const node = nodeMap.get(subnet.id)!

      if (subnet.parent && nodeMap.has(subnet.parent)) {
        const parentNode = nodeMap.get(subnet.parent)!
        parentNode.children.push(node)
        node.depth = parentNode.depth + 1
      } else {
        roots.push(node)
      }
    })

    return roots
  })

  // Actions
  const loadSubnets = async (showInitialLoading = true) => {
    if (showInitialLoading) {
      isLoading.value = true
    }
    error.value = null

    try {
      const response = await apiService.getInstances()

      if (response.data?.data) {
        // Get existing subnet IDs to preserve loading states
        const existingSubnetIds = new Set(subnets.value.map(s => s.id))

        // Map the API response to our enhanced SubnetInstance interface
        const newSubnets = response.data.data.map((subnet: any): SubnetInstance => ({
          id: subnet.id,
          name: subnet.name,
          parent: subnet.parent,
          status: subnet.status as SubnetLifecycleState,
          status_info: {
            lifecycle_state: subnet.status_info?.lifecycle_state || SubnetLifecycleState.Unknown,
            genesis_available: subnet.status_info?.genesis_available || false,
            validator_count: subnet.status_info?.validator_count || 0,
            active_validators: subnet.status_info?.active_validators || 0,
            permission_mode: subnet.status_info?.permission_mode,
            deployment_time: subnet.status_info?.deployment_time,
            last_block_time: subnet.status_info?.last_block_time,
            error_message: subnet.status_info?.error_message,
            next_action_required: subnet.status_info?.next_action_required
          },
          validators: subnet.validators || [],
          created_at: subnet.created_at,
          config: subnet.config,
          isLoading: subnetLoadingStates.value.get(subnet.id) || false,
          loadError: subnetErrors.value.get(subnet.id) || null
        }))

        subnets.value = newSubnets
        lastUpdated.value = new Date()
        console.log(`Loaded ${subnets.value.length} subnets with enhanced status`)
      }
    } catch (err) {
      console.error('Failed to load subnets:', err)
      error.value = err instanceof Error ? err.message : 'Failed to load subnets'
    } finally {
      if (showInitialLoading) {
        isLoading.value = false
      }
    }
  }

  const setSubnetLoading = (subnetId: string, loading: boolean) => {
    subnetLoadingStates.value.set(subnetId, loading)
    // Update the subnet in the array
    const subnet = subnets.value.find(s => s.id === subnetId)
    if (subnet) {
      subnet.isLoading = loading
    }
  }

  const setSubnetError = (subnetId: string, error: string | null) => {
    if (error) {
      subnetErrors.value.set(subnetId, error)
    } else {
      subnetErrors.value.delete(subnetId)
    }
    // Update the subnet in the array
    const subnet = subnets.value.find(s => s.id === subnetId)
    if (subnet) {
      subnet.loadError = error
    }
  }

  const loadSubnetDetails = async (subnetId: string) => {
    setSubnetLoading(subnetId, true)
    setSubnetError(subnetId, null)

    try {
      // For now, refresh all subnets but could be optimized to load individual subnet
      await loadSubnets(false)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load subnet details'
      setSubnetError(subnetId, errorMessage)
      console.error(`Failed to load subnet ${subnetId}:`, err)
    } finally {
      setSubnetLoading(subnetId, false)
    }
  }

  const getSubnet = (id: string): SubnetInstance | undefined => {
    return subnets.value.find(subnet => subnet.id === id)
  }

  const refreshSubnet = async (id: string) => {
    try {
      // For now, refresh all subnets - could be optimized later
      await loadSubnets()
    } catch (err) {
      console.error(`Failed to refresh subnet ${id}:`, err)
      throw err
    }
  }

  // Initialize
  loadSubnets()

  return {
    // State
    subnets: computed(() => subnets.value),
    isLoading: computed(() => isLoading.value),
    error: computed(() => error.value),
    lastUpdated: computed(() => lastUpdated.value),

    // Computed
    subnetCount,
    healthySubnets,
    problematicSubnets,
    waitingSubnets,
    subnetsByParent,
    subnetHierarchy,

    // Actions
    loadSubnets,
    getSubnet,
    refreshSubnet,
    setSubnetLoading,
    setSubnetError,
    loadSubnetDetails
  }
})