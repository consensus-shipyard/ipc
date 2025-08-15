<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import ProgressiveLoader from '../components/common/ProgressiveLoader.vue'
import { useNetworkStore } from '../stores/network'
import { useSubnetsStore, type SubnetInstance } from '../stores/subnets'

interface SubnetNode {
  subnet: SubnetInstance
  children: SubnetNode[]
  depth: number
}

// Stores
const subnetsStore = useSubnetsStore()
const networkStore = useNetworkStore()

// State
const expandedNodes = ref<Set<string>>(new Set())
const viewMode = ref<'hierarchy' | 'list'>('hierarchy')

// Use store data instead of local state
const subnets = computed(() => subnetsStore.subnets || [])
const loading = computed(() => subnetsStore.isLoading)
const error = computed(() => subnetsStore.error)

// Methods
const fetchSubnets = () => subnetsStore.loadSubnets()

// Build hierarchical tree structure
const subnetTree = computed(() => {
  if (!Array.isArray(subnets.value) || subnets.value.length === 0) return []

  const nodeMap = new Map<string, SubnetNode>()
  const roots: SubnetNode[] = []

  // Create nodes for all subnets
  subnets.value.forEach(subnet => {
    nodeMap.set(subnet.id, {
      subnet,
      children: [],
      depth: 0
    })
  })

  // Build parent-child relationships
  subnets.value.forEach(subnet => {
    const node = nodeMap.get(subnet.id)!
    const parentId = subnet.parent

    if (parentId && nodeMap.has(parentId)) {
      // This is a child subnet
      const parentNode = nodeMap.get(parentId)!
      parentNode.children.push(node)
      node.depth = parentNode.depth + 1
    } else {
      // This is a root subnet or parent not found
      roots.push(node)
    }
  })

  // Sort children by creation date (with safe date handling)
  const sortChildren = (node: SubnetNode) => {
    node.children.sort((a, b) => {
      const dateA = a.subnet.created_at ? new Date(a.subnet.created_at).getTime() : 0
      const dateB = b.subnet.created_at ? new Date(b.subnet.created_at).getTime() : 0
      return dateB - dateA
    })
    node.children.forEach(sortChildren)
  }

  roots.forEach(sortChildren)
  return roots.sort((a, b) => {
    const dateA = a.subnet.created_at ? new Date(a.subnet.created_at).getTime() : 0
    const dateB = b.subnet.created_at ? new Date(b.subnet.created_at).getTime() : 0
    return dateB - dateA
  })
})

// Helper functions
const toggleNode = (subnetId: string) => {
  if (expandedNodes.value.has(subnetId)) {
    expandedNodes.value.delete(subnetId)
  } else {
    expandedNodes.value.add(subnetId)
  }
}

const isExpanded = (subnetId: string) => {
  return expandedNodes.value.has(subnetId)
}

const getStatusColor = (status: string) => {
  switch (status.toLowerCase()) {
    case 'active': return 'text-green-600 bg-green-50'
    case 'paused': return 'text-yellow-600 bg-yellow-50'
    case 'deploying': return 'text-blue-600 bg-blue-50'
    case 'failed': return 'text-red-600 bg-red-50'
    case 'pending approval': return 'text-orange-600 bg-orange-50'
    case 'approved - no validators': return 'text-blue-600 bg-blue-50'
    case 'inactive': return 'text-gray-600 bg-gray-50'
    default: return 'text-gray-600 bg-gray-50'
  }
}

const formatAddress = (address: any) => {
  if (!address) return 'N/A'

  let addressStr = ''
  if (typeof address === 'string') {
    addressStr = address
  } else if (Array.isArray(address) && address.length >= 20) {
    const addressBytes = address.slice(0, 20)
    addressStr = '0x' + addressBytes.map(b => b.toString(16).padStart(2, '0')).join('')
  } else if (typeof address === 'object' && address.route && Array.isArray(address.route)) {
    const lastRoute = address.route[address.route.length - 1]
    if (lastRoute && Array.isArray(lastRoute) && lastRoute.length === 20) {
      addressStr = '0x' + lastRoute.map(b => b.toString(16).padStart(2, '0')).join('')
    } else {
      return 'N/A'
    }
  } else {
    return 'N/A'
  }

  if (addressStr && !addressStr.startsWith('0x') && addressStr.length === 40) {
    addressStr = '0x' + addressStr
  }

  if (addressStr.startsWith('0x') && addressStr.length === 42) {
    return `${addressStr.slice(0, 8)}...${addressStr.slice(-6)}`
  }

  return 'N/A'
}

const formatDate = (dateStr?: string) => {
  if (!dateStr) return 'N/A'
  try {
    return new Date(dateStr).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return 'Unknown'
  }
}

const getIndentStyle = (depth: number) => {
  return {
    paddingLeft: `${depth * 24 + 16}px`
  }
}

// Lifecycle
// Data fetching is now handled by the centralized app store
// onMounted(() => {
//   fetchSubnets() // Removed - handled by app store
// })
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Header -->
    <div class="bg-white shadow-sm border-b">
      <div class="max-w-7xl mx-auto px-6 py-6">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-3xl font-bold text-gray-900">Subnets</h1>
            <p class="text-gray-600 mt-2">Manage and monitor your deployed subnets</p>
          </div>

          <div class="flex items-center space-x-4">
            <!-- View Mode Toggle -->
            <div class="flex bg-gray-100 rounded-lg p-1">
              <button
                @click="viewMode = 'hierarchy'"
                :class="[
                  'px-3 py-2 text-sm font-medium rounded-md transition-colors',
                  viewMode === 'hierarchy'
                    ? 'bg-white text-gray-900 shadow-sm'
                    : 'text-gray-600 hover:text-gray-900'
                ]"
              >
                <svg class="w-4 h-4 mr-2 inline-block" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M19 11H5m14-7H3m14 14H9m6-7l-6 6-4-4" />
                </svg>
                Hierarchy
              </button>
              <button
                @click="viewMode = 'list'"
                :class="[
                  'px-3 py-2 text-sm font-medium rounded-md transition-colors',
                  viewMode === 'list'
                    ? 'bg-white text-gray-900 shadow-sm'
                    : 'text-gray-600 hover:text-gray-900'
                ]"
              >
                <svg class="w-4 h-4 mr-2 inline-block" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M4 6h16M4 10h16M4 14h16M4 18h16" />
                </svg>
                List
              </button>
            </div>

            <!-- Deploy New Subnet Button -->
            <RouterLink to="/wizard" class="btn-primary">
              <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              Deploy New Subnet
            </RouterLink>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="max-w-7xl mx-auto px-6 py-8">
      <!-- Loading State -->
      <div v-if="loading" class="text-center py-12">
        <div class="animate-spin inline-block w-8 h-8 border-4 border-primary-600 border-t-transparent rounded-full"></div>
        <p class="mt-4 text-gray-600">Loading subnets...</p>
      </div>

      <!-- Error State -->
      <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-lg p-6">
        <div class="flex items-start space-x-3">
          <svg class="w-6 h-6 text-red-600 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          <div>
            <h3 class="font-semibold text-red-800 mb-1">Error Loading Subnets</h3>
            <p class="text-red-700">{{ error }}</p>
            <button
              @click="fetchSubnets"
              class="mt-3 text-red-600 hover:text-red-700 font-medium text-sm"
            >
              Try Again
            </button>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else-if="(subnets?.length || 0) === 0" class="text-center py-12 bg-white rounded-lg shadow-sm">
        <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 9a2 2 0 00-2 2v2a2 2 0 002 2m0 0h14m-14 0a2 2 0 002 2v2a2 2 0 01-2 2M5 9V7a2 2 0 012-2h10a2 2 0 012 2v2M7 7V5a2 2 0 012-2h6a2 2 0 012 2v2" />
        </svg>
        <h3 class="text-lg font-medium text-gray-900 mb-2">No Subnets Found</h3>
        <p class="text-gray-600 mb-6">No subnets are currently deployed in this network.</p>
        <RouterLink to="/wizard" class="btn-primary">Deploy Your First Subnet</RouterLink>
      </div>

      <!-- Hierarchy View -->
      <div v-else-if="viewMode === 'hierarchy'" class="space-y-2">
        <template v-for="node in subnetTree" :key="`hierarchy-${node.subnet.id}`">
          <!-- Render subnet node -->
          <div
            class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden"
            :style="getIndentStyle(node.depth)"
          >
            <div class="p-6">
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center space-x-3 mb-3">
                    <!-- Expand/Collapse Button -->
                    <button
                      v-if="(node.children?.length || 0) > 0"
                      @click="toggleNode(node.subnet.id)"
                      class="p-1 hover:bg-gray-100 rounded transition-colors"
                    >
                      <svg
                        class="w-5 h-5 text-gray-500 transition-transform duration-200"
                        :class="{ 'rotate-90': isExpanded(node.subnet.id) }"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                      </svg>
                    </button>
                    <div v-else class="w-7"></div> <!-- Spacer for alignment -->

                    <!-- Subnet Icon -->
                    <div
                      :class="[
                        'w-10 h-10 rounded-lg flex items-center justify-center',
                        node.depth === 0 ? 'bg-primary-100' : 'bg-blue-100'
                      ]"
                    >
                      <svg
                        :class="[
                          'w-5 h-5',
                          node.depth === 0 ? 'text-primary-600' : 'text-blue-600'
                        ]"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M19 11H5m14-7H3m14 14H9m6-7l-6 6-4-4" />
                      </svg>
                    </div>

                    <div>
                      <h3 class="text-lg font-semibold text-gray-900">{{ node.subnet.name }}</h3>
                      <p class="text-sm text-gray-600 font-mono">{{ node.subnet.id }}</p>
                      <div class="flex items-center space-x-4 mt-1">
                        <span :class="['inline-flex items-center px-2 py-1 rounded-full text-xs font-medium', getStatusColor(node.subnet.status)]">
                          {{ node.subnet.status }}
                        </span>
                        <span v-if="node.depth > 0" class="text-xs text-gray-500">
                          Child of {{ node.subnet.parent }}
                        </span>
                        <span v-else class="text-xs text-gray-500">
                          Root Network
                        </span>
                      </div>
                    </div>
                  </div>

                  <!-- Subnet metrics -->
                  <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-4">
                    <div>
                      <p class="text-sm text-gray-500">Validators</p>
                      <p class="font-semibold text-gray-900">
                        <span v-if="node.subnet.isLoading">
                          <ProgressiveLoader :show-text="false" :inline="true" />
                        </span>
                        <span v-else>{{ node.subnet.validators?.length || 0 }}</span>
                      </p>
                    </div>
                    <div>
                      <p class="text-sm text-gray-500">Total Stake</p>
                      <p class="font-semibold text-gray-900">
                        <span v-if="node.subnet.isLoading">
                          <ProgressiveLoader :show-text="false" :inline="true" />
                        </span>
                        <span v-else>
                          {{ (node.subnet.validators || []).reduce((s: number, v: any) => s + parseFloat(v.stake || '0'), 0).toFixed(1) }} FIL
                        </span>
                      </p>
                    </div>
                    <div>
                      <p class="text-sm text-gray-500">Permission Mode</p>
                      <p class="font-semibold text-gray-900 capitalize">
                        <span v-if="node.subnet.isLoading">
                          <ProgressiveLoader :show-text="false" :inline="true" />
                        </span>
                        <span v-else>{{ node.subnet.status_info.permission_mode || 'Unknown' }}</span>
                      </p>
                    </div>
                    <div>
                      <p class="text-sm text-gray-500">Created</p>
                      <p class="font-semibold text-gray-900">{{ formatDate(node.subnet.created_at) }}</p>
                    </div>
                  </div>

                  <!-- Child subnets count -->
                  <div v-if="(node.children?.length || 0) > 0" class="mt-4 pt-4 border-t border-gray-200">
                    <p class="text-sm text-gray-600">
                      <svg class="w-4 h-4 inline-block mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M19 11H5m14-7H3m14 14H9m6-7l-6 6-4-4" />
                      </svg>
                      {{ node.children?.length || 0 }} child subnet{{ (node.children?.length || 0) !== 1 ? 's' : '' }}
                    </p>
                  </div>
                </div>

                <!-- Actions -->
                <div class="flex space-x-2">
                  <RouterLink
                    :to="`/instance/${encodeURIComponent(node.subnet.id)}`"
                    class="btn-secondary text-sm"
                  >
                    Manage
                  </RouterLink>
                </div>
              </div>
            </div>
          </div>

          <!-- Render children if expanded -->
          <template v-if="isExpanded(node.subnet.id)">
            <div v-for="childNode in node.children" :key="`child-${childNode.subnet.id}`" class="ml-6">
              <!-- Recursive rendering would be done with a separate component in a real implementation -->
              <!-- For now, we'll render one level of children -->
              <div
                class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden"
                :style="getIndentStyle(childNode.depth)"
              >
                <div class="p-6">
                  <div class="flex items-start justify-between">
                    <div class="flex-1">
                      <div class="flex items-center space-x-3 mb-3">
                        <div class="w-7"></div> <!-- Spacer -->

                        <!-- Child subnet icon -->
                        <div class="w-8 h-8 bg-blue-50 rounded-lg flex items-center justify-center">
                          <svg class="w-4 h-4 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M13 7l5 5m0 0l-5 5m5-5H6" />
                          </svg>
                        </div>

                        <div>
                          <h4 class="text-md font-semibold text-gray-900">{{ childNode.subnet.name }}</h4>
                          <p class="text-sm text-gray-600 font-mono">{{ childNode.subnet.id }}</p>
                          <span :class="['inline-flex items-center px-2 py-1 rounded-full text-xs font-medium mt-1', getStatusColor(childNode.subnet.status)]">
                            {{ childNode.subnet.status }}
                          </span>
                        </div>
                      </div>

                      <!-- Child subnet metrics -->
                      <div class="grid grid-cols-4 gap-4 text-sm">
                        <div>
                          <p class="text-gray-500">Validators</p>
                          <p class="font-semibold">
                            <span v-if="childNode.subnet.isLoading">
                              <ProgressiveLoader :show-text="false" :inline="true" />
                            </span>
                            <span v-else>{{ childNode.subnet.validators?.length || 0 }}</span>
                          </p>
                        </div>
                        <div>
                          <p class="text-gray-500">Stake</p>
                          <p class="font-semibold">
                            <span v-if="childNode.subnet.isLoading">
                              <ProgressiveLoader :show-text="false" :inline="true" />
                            </span>
                            <span v-else>
                              {{ (childNode.subnet.validators || []).reduce((s: number, v: any) => s + parseFloat(v.stake || '0'), 0).toFixed(1) }}
                            </span>
                          </p>
                        </div>
                        <div>
                          <p class="text-gray-500">Permission Mode</p>
                          <p class="font-semibold capitalize">
                            <span v-if="childNode.subnet.isLoading">
                              <ProgressiveLoader :show-text="false" :inline="true" />
                            </span>
                            <span v-else>{{ childNode.subnet.status_info.permission_mode || 'Unknown' }}</span>
                          </p>
                        </div>
                        <div>
                          <p class="text-gray-500">Created</p>
                          <p class="font-semibold">{{ formatDate(childNode.subnet.created_at) }}</p>
                        </div>
                      </div>
                    </div>

                    <!-- Child actions -->
                    <div class="flex space-x-2">
                      <RouterLink
                        :to="`/instance/${encodeURIComponent(childNode.subnet.id)}`"
                        class="btn-secondary text-sm"
                      >
                        Manage
                      </RouterLink>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </template>
        </template>
      </div>

      <!-- List View -->
      <div v-else class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
        <div class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-200">
            <thead class="bg-gray-50">
              <tr>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Subnet
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Parent
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Validators
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Total Stake
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Created
                </th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
              <tr v-for="subnet in subnets" :key="subnet.id" class="hover:bg-gray-50">
                <td class="px-6 py-4 whitespace-nowrap">
                  <div>
                    <div class="text-sm font-medium text-gray-900">{{ subnet.name }}</div>
                    <div class="text-sm text-gray-500 font-mono">{{ subnet.id.slice(0, 20) }}...</div>
                  </div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <span :class="['inline-flex items-center px-2 py-1 rounded-full text-xs font-medium', getStatusColor(subnet.status)]">
                    {{ subnet.status }}
                  </span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900 font-mono">
                  {{ subnet.parent }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  <span v-if="subnet.isLoading">
                    <ProgressiveLoader :show-text="false" :inline="true" />
                  </span>
                  <span v-else>{{ subnet.validators?.length || 0 }}</span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  <span v-if="subnet.isLoading">
                    <ProgressiveLoader :show-text="false" :inline="true" />
                  </span>
                  <span v-else>{{ (subnet.validators || []).reduce((s: number, v: any) => s + parseFloat(v.stake || '0'), 0).toFixed(1) }} FIL</span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {{ formatDate(subnet.created_at) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                  <RouterLink
                    :to="`/instance/${encodeURIComponent(subnet.id)}`"
                    class="text-primary-600 hover:text-primary-900"
                  >
                    Manage
                  </RouterLink>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.btn-primary {
  @apply inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}

.btn-secondary {
  @apply inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md shadow-sm text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}
</style>