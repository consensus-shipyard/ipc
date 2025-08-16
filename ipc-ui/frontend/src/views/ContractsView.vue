<script setup lang="ts">
import SubnetApprovalsModal from '@/components/SubnetApprovalsModal.vue'
import ProgressiveLoader from '@/components/common/ProgressiveLoader.vue'
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useGatewaysStore, type ContractInfo } from '../stores/gateways'
import { useNetworkStore } from '../stores/network'

// Stores
const gatewaysStore = useGatewaysStore()
const networkStore = useNetworkStore()

// State
const selectedContract = ref<ContractInfo | null>(null)
const showInspectModal = ref(false)
const filterType = ref<string>('all')
const filterNetwork = ref<string>('all')
const searchQuery = ref<string>('')

// Use store data instead of local state
const allContracts = computed(() => gatewaysStore.contracts || [])
const loading = computed(() => gatewaysStore.loading)
const error = computed(() => gatewaysStore.error)

// Methods
const fetchContracts = () => gatewaysStore.fetchGateways()

// Computed properties
const uniqueNetworks = computed(() => {
  if (!Array.isArray(allContracts.value)) return []
  const networks = new Set(allContracts.value.map(c => c.network))
  return Array.from(networks).sort()
})

const filteredContracts = computed(() => {
  let filtered = allContracts.value

  // Filter by type
  if (filterType.value !== 'all') {
    filtered = filtered.filter(c => c.type === filterType.value)
  }

  // Filter by network (this is redundant now since store already filters by network)
  if (filterNetwork.value !== 'all') {
    filtered = filtered.filter(c => c.network === filterNetwork.value)
  }

  // Filter by search query
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(c =>
      c.name.toLowerCase().includes(query) ||
      c.address.toLowerCase().includes(query) ||
      c.deployer.toLowerCase().includes(query) ||
      c.network.toLowerCase().includes(query)
    )
  }

  return filtered.sort((a, b) => new Date(b.deployed_at).getTime() - new Date(a.deployed_at).getTime())
})

const contractTypeStats = computed(() => {
  const stats = allContracts.value.reduce((acc, contract) => {
    acc[contract.type] = (acc[contract.type] || 0) + 1
    return acc
  }, {} as Record<string, number>)

  return {
    total: allContracts.value.length,
    gateway: stats.gateway || 0,
    registry: stats.registry || 0,
    subnet: stats.subnet || 0,
    other: stats.other || 0,
    active: allContracts.value.filter((c: ContractInfo) => c.status === 'active').length
  }
})

// Helper functions
const getContractTypeIcon = (type: string) => {
  switch (type) {
    case 'gateway':
      return 'M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z'
    case 'registry':
      return 'M9 12h6m-6 4h6M7 20l4-16m6 16l-4-16'
    case 'subnet':
      return 'M19 11H5m14-7H3m14 14H9m6-7l-6 6-4-4'
    default:
      return 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4'
  }
}

const getContractTypeColor = (type: string) => {
  switch (type) {
    case 'gateway':
      return 'bg-primary-100 text-primary-800'
    case 'registry':
      return 'bg-blue-100 text-blue-800'
    case 'subnet':
      return 'bg-green-100 text-green-800'
    default:
      return 'bg-gray-100 text-gray-800'
  }
}

const getStatusColor = (status: string) => {
  switch (status) {
    case 'active':
      return 'text-green-600 bg-green-50'
    case 'inactive':
      return 'text-gray-600 bg-gray-50'
    default:
      return 'text-yellow-600 bg-yellow-50'
  }
}

const formatAddress = (address: string) => {
  if (!address || !address.startsWith('0x')) return 'N/A'
  return `${address.slice(0, 8)}...${address.slice(-6)}`
}

const formatDate = (dateStr: string) => {
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

const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
  } catch (err) {
    console.error('Failed to copy to clipboard:', err)
  }
}

const inspectContract = (contract: ContractInfo) => {
  selectedContract.value = contract
  showInspectModal.value = true
}

const configureContract = (contract: ContractInfo) => {
  // TODO: Implement contract configuration
  console.log('Configure contract:', contract)
}

const showApprovalsModal = ref(false)
const selectedGatewayAddress = ref('')

const approveSubnets = (contract: ContractInfo) => {
  selectedGatewayAddress.value = contract.address
  showApprovalsModal.value = true
}

const upgradeContract = (contract: ContractInfo) => {
  // TODO: Implement contract upgrade
  console.log('Upgrade contract:', contract)
}

// Lifecycle
// Data fetching is now handled by the centralized app store
// onMounted(() => {
//   fetchContracts() // Removed - handled by app store
// })
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Header -->
    <div class="bg-white shadow-sm border-b">
      <div class="max-w-7xl mx-auto px-6 py-6">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-3xl font-bold text-gray-900">Contracts</h1>
            <p class="text-gray-600 mt-2">Manage and monitor your deployed smart contracts</p>
          </div>

          <div class="flex items-center space-x-4">
            <!-- Deploy New Contract Button -->
            <RouterLink to="/wizard" class="btn-primary">
              <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              Deploy Gateway
            </RouterLink>
          </div>
        </div>
      </div>
    </div>

    <!-- Stats Cards -->
    <div class="max-w-7xl mx-auto px-6 py-6">
      <div class="grid grid-cols-1 md:grid-cols-5 gap-6 mb-8">
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
          <div class="flex items-center">
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-600">Total Contracts</p>
              <p class="text-2xl font-bold text-gray-900">{{ contractTypeStats.total }}</p>
            </div>
            <div class="w-12 h-12 bg-gray-100 rounded-lg flex items-center justify-center">
              <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M9 12h6m-6 4h6M7 20l4-16m6 16l-4-16" />
              </svg>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
          <div class="flex items-center">
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-600">Gateways</p>
              <p class="text-2xl font-bold text-gray-900">{{ contractTypeStats.gateway }}</p>
            </div>
            <div class="w-12 h-12 bg-primary-100 rounded-lg flex items-center justify-center">
              <svg class="w-6 h-6 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
          <div class="flex items-center">
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-600">Registries</p>
              <p class="text-2xl font-bold text-gray-900">{{ contractTypeStats.registry }}</p>
            </div>
            <div class="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
              <svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M9 12h6m-6 4h6M7 20l4-16m6 16l-4-16" />
              </svg>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
          <div class="flex items-center">
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-600">Active</p>
              <p class="text-2xl font-bold text-gray-900">{{ contractTypeStats.active }}</p>
            </div>
            <div class="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
              <svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
          <div class="flex items-center">
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-600">Networks</p>
              <p class="text-2xl font-bold text-gray-900">{{ uniqueNetworks?.length || 0 }}</p>
            </div>
            <div class="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center">
              <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
              </svg>
            </div>
          </div>
        </div>
      </div>

      <!-- Filters and Search -->
      <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6">
        <div class="flex flex-col md:flex-row md:items-center md:justify-between space-y-4 md:space-y-0 md:space-x-4">
          <!-- Search -->
          <div class="flex-1 max-w-md">
            <div class="relative">
              <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg class="h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>
              <input
                v-model="searchQuery"
                type="text"
                placeholder="Search contracts..."
                class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              />
            </div>
          </div>

          <!-- Filters -->
          <div class="flex space-x-4">
            <select
              v-model="filterType"
              class="border border-gray-300 rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            >
              <option value="all">All Types</option>
              <option value="gateway">Gateways</option>
              <option value="registry">Registries</option>
              <option value="subnet">Subnets</option>
              <option value="other">Other</option>
            </select>

            <select
              v-model="filterNetwork"
              class="border border-gray-300 rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            >
              <option value="all">All Networks</option>
              <option v-for="network in uniqueNetworks" :key="network" :value="network">
                {{ network }}
              </option>
            </select>
          </div>
        </div>
      </div>

      <!-- Loading State (only show if no contracts at all) -->
      <div v-if="loading && allContracts.length === 0" class="text-center py-12">
        <div class="animate-spin inline-block w-8 h-8 border-4 border-primary-600 border-t-transparent rounded-full"></div>
        <p class="mt-4 text-gray-600">Loading contracts...</p>
      </div>

      <!-- Error State -->
      <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-lg p-6">
        <div class="flex items-start space-x-3">
          <svg class="w-6 h-6 text-red-600 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          <div>
            <h3 class="font-semibold text-red-800 mb-1">Error Loading Contracts</h3>
            <p class="text-red-700">{{ error }}</p>
            <button
              @click="fetchContracts"
              class="mt-3 text-red-600 hover:text-red-700 font-medium text-sm"
            >
              Try Again
            </button>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else-if="(filteredContracts?.length || 0) === 0" class="text-center py-12 bg-white rounded-lg shadow-sm">
        <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <h3 class="text-lg font-medium text-gray-900 mb-2">No Contracts Found</h3>
        <p class="text-gray-600 mb-4">
          {{ (allContracts?.length || 0) === 0 ? 'No contracts found' : 'No contracts match your filters' }}
        </p>
        <p class="text-gray-500 text-sm">
          {{ (allContracts?.length || 0) === 0 ? 'Deploy your first gateway to get started' : 'Try adjusting your search or filters' }}
        </p>
        <RouterLink v-if="(allContracts?.length || 0) === 0" to="/wizard" class="btn-primary">
          Deploy Gateway
        </RouterLink>
      </div>

      <!-- Contracts Grid -->
      <div v-else-if="filteredContracts.length > 0 || loading" class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div
          v-for="contract in filteredContracts"
          :key="contract.id"
          class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden hover:shadow-md transition-shadow"
        >
          <div class="p-6">
            <div class="flex items-start justify-between mb-4">
              <div class="flex items-center space-x-3">
                <!-- Contract Type Icon -->
                <div :class="['w-10 h-10 rounded-lg flex items-center justify-center', getContractTypeColor(contract.type)]">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="getContractTypeIcon(contract.type)" />
                  </svg>
                </div>

                <div>
                  <h3 class="text-lg font-semibold text-gray-900">{{ contract.name }}</h3>
                  <div class="flex items-center space-x-2 mt-1">
                    <span :class="['inline-flex items-center px-2 py-1 rounded-full text-xs font-medium', getContractTypeColor(contract.type)]">
                      {{ contract.type.charAt(0).toUpperCase() + contract.type.slice(1) }}
                    </span>
                    <span :class="['inline-flex items-center px-2 py-1 rounded-full text-xs font-medium', getStatusColor(contract.status)]">
                      {{ contract.status.charAt(0).toUpperCase() + contract.status.slice(1) }}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Contract Details -->
            <div class="space-y-3 mb-4">
              <div class="flex justify-between items-center">
                <span class="text-sm font-medium text-gray-500">Address</span>
                <button
                  @click="copyToClipboard(contract.address)"
                  class="text-sm font-mono text-gray-900 hover:bg-gray-100 px-2 py-1 rounded transition-colors"
                  :title="`Click to copy: ${contract.address}`"
                >
                  {{ formatAddress(contract.address) }}
                </button>
              </div>

              <div class="flex justify-between items-center">
                <span class="text-sm font-medium text-gray-500">Deployer</span>
                <button
                  @click="copyToClipboard(contract.deployer)"
                  class="text-sm font-mono text-gray-900 hover:bg-gray-100 px-2 py-1 rounded transition-colors"
                  :title="`Click to copy: ${contract.deployer}`"
                >
                  {{ formatAddress(contract.deployer) }}
                </button>
              </div>

              <div class="flex justify-between items-center">
                <span class="text-sm font-medium text-gray-500">Network</span>
                <span class="text-sm text-gray-900 font-mono">{{ contract.network }}</span>
              </div>

              <div class="flex justify-between items-center">
                <span class="text-sm font-medium text-gray-500">Deployed</span>
                <span class="text-sm text-gray-900">{{ formatDate(contract.deployed_at) }}</span>
              </div>

              <div v-if="contract.type === 'gateway'" class="flex justify-between items-center">
                <span class="text-sm font-medium text-gray-500">Subnets Created</span>
                <span class="text-sm text-gray-900 font-semibold">
                  <span v-if="loading && contract.subnets_created === undefined">
                    <ProgressiveLoader :show-text="false" :inline="true" />
                  </span>
                  <span v-else>{{ contract.subnets_created ?? 0 }}</span>
                </span>
              </div>
            </div>

            <!-- Description -->
            <div v-if="contract.description" class="mb-4">
              <p class="text-sm text-gray-600">{{ contract.description }}</p>
            </div>

            <!-- Actions -->
            <div class="flex space-x-2 pt-4 border-t border-gray-200">
              <button
                @click="inspectContract(contract)"
                class="btn-secondary text-sm flex-1"
              >
                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
                Inspect
              </button>

              <button
                @click="configureContract(contract)"
                class="btn-secondary text-sm flex-1"
              >
                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
                Configure
              </button>

              <button
                v-if="contract.type === 'gateway'"
                @click="approveSubnets(contract)"
                class="btn-secondary text-sm"
                title="Manage Subnet Approvals"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Contract Inspection Modal -->
    <div v-if="showInspectModal && selectedContract" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div class="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-screen overflow-y-auto">
        <div class="p-6">
          <div class="flex items-center justify-between mb-6">
            <h2 class="text-xl font-bold text-gray-900">Contract Inspector</h2>
            <button
              @click="showInspectModal = false"
              class="text-gray-400 hover:text-gray-600 transition-colors"
            >
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div class="space-y-6">
            <!-- Contract Overview -->
            <div class="bg-gray-50 rounded-lg p-4">
              <h3 class="font-semibold text-gray-900 mb-3">Contract Overview</h3>
              <div class="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <p class="text-gray-500">Name</p>
                  <p class="font-medium">{{ selectedContract.name }}</p>
                </div>
                <div>
                  <p class="text-gray-500">Type</p>
                  <p class="font-medium capitalize">{{ selectedContract.type }}</p>
                </div>
                <div class="col-span-2">
                  <p class="text-gray-500">Address</p>
                  <p class="font-mono text-sm">{{ selectedContract.address }}</p>
                </div>
                <div class="col-span-2">
                  <p class="text-gray-500">Deployer</p>
                  <p class="font-mono text-sm">{{ selectedContract.deployer }}</p>
                </div>
              </div>
            </div>

            <!-- Contract Details -->
            <div class="bg-gray-50 rounded-lg p-4">
              <h3 class="font-semibold text-gray-900 mb-3">Contract Details</h3>
              <div class="text-sm text-gray-600">
                <p class="mb-2">This is a {{ selectedContract.type }} contract deployed on {{ selectedContract.network }}.</p>
                <p v-if="selectedContract.description">{{ selectedContract.description }}</p>
                <p v-if="selectedContract.type === 'gateway'" class="mt-2">
                  Gateway contracts manage subnet registration and cross-chain messaging.
                  This gateway has been used to create {{ selectedContract.subnets_created || 0 }} subnet(s).
                </p>
                <p v-else-if="selectedContract.type === 'registry'" class="mt-2">
                  Registry contracts store subnet metadata and facilitate subnet discovery.
                </p>
              </div>
            </div>

            <!-- Available Actions -->
            <div class="bg-gray-50 rounded-lg p-4">
              <h3 class="font-semibold text-gray-900 mb-3">Available Actions</h3>
              <div class="space-y-2">
                <button class="w-full text-left p-3 bg-white rounded border hover:bg-gray-50 transition-colors">
                  <div class="font-medium text-gray-900">View on Block Explorer</div>
                  <div class="text-sm text-gray-600">Inspect contract on blockchain explorer</div>
                </button>
                <button class="w-full text-left p-3 bg-white rounded border hover:bg-gray-50 transition-colors">
                  <div class="font-medium text-gray-900">Export ABI</div>
                  <div class="text-sm text-gray-600">Download contract interface definition</div>
                </button>
                <button
                  v-if="selectedContract.type === 'gateway'"
                  @click="approveSubnets(selectedContract); showInspectModal = false"
                  class="w-full text-left p-3 bg-white rounded border hover:bg-gray-50 transition-colors"
                >
                  <div class="font-medium text-gray-900">Manage Subnet Approvals</div>
                  <div class="text-sm text-gray-600">Approve or reject subnet registration requests</div>
                </button>
              </div>
            </div>
          </div>

          <div class="flex justify-end space-x-3 pt-6 border-t">
            <button
              @click="showInspectModal = false"
              class="btn-secondary"
            >
              Close
            </button>
            <button
              @click="configureContract(selectedContract); showInspectModal = false"
              class="btn-primary"
            >
              Configure Contract
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Subnet Approvals Modal -->
    <SubnetApprovalsModal
      :show="showApprovalsModal"
      :gateway-address="selectedGatewayAddress"
      @close="showApprovalsModal = false"
      @approved="(subnetId) => {
        console.log('Subnet approved:', subnetId)
        // Optionally refresh data or show success message
      }"
    />
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