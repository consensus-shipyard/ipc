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
  const loadSubnets = async () => {
    isLoading.value = true
    error.value = null

    try {
      const response = await apiService.getInstances()

            if (response.data?.data) {
        // Map the API response to our enhanced SubnetInstance interface
        subnets.value = response.data.data.map((subnet: any): SubnetInstance => ({
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
          config: subnet.config
        }))

        lastUpdated.value = new Date()
        console.log(`Loaded ${subnets.value.length} subnets with enhanced status`)
      }
    } catch (err) {
      console.error('Failed to load subnets:', err)
      error.value = err instanceof Error ? err.message : 'Failed to load subnets'
    } finally {
      isLoading.value = false
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
    refreshSubnet
  }
})