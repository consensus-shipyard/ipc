<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { apiService } from '../services/api'

interface SubnetInstance {
  id: string
  name: string
  status: string
  template: string
  parent: string
  created_at: string
  validators: Array<{
    address: string
    stake: string
    power: number
    status: string
  }>
  config: Record<string, any>
}

// State
const subnets = ref<SubnetInstance[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const approvingSubnets = ref<Set<string>>(new Set())

// Methods
const fetchSubnets = async () => {
  try {
    loading.value = true
    error.value = null

    const response = await apiService.getInstances()

    if (response.data) {
      subnets.value = response.data
    }
  } catch (err: any) {
    console.error('Error fetching subnets:', err)
    error.value = err?.message || 'Failed to load subnets'
  } finally {
    loading.value = false
  }
}

const getGatewayOwner = async (subnet: SubnetInstance): Promise<string> => {
  try {
    // Try to get gateway information from the API
    const gatewaysResponse = await fetch('/api/gateways')
    const gatewaysResult = await gatewaysResponse.json()

    if (gatewaysResult && Array.isArray(gatewaysResult)) {
      // Find the gateway that matches this subnet's gateway address
      const gatewayAddr = subnet.config?.gateway_addr?.toString()
      if (gatewayAddr) {
        const matchingGateway = gatewaysResult.find((gw: any) =>
          gw.gateway_address === gatewayAddr
        )
        if (matchingGateway) {
          return matchingGateway.deployer_address
        }
      }
    }
  } catch (err) {
    console.warn('Failed to fetch gateway information:', err)
  }

  // Fallback to config or default address
  return subnet.config?.deployer_address || '0x0a36d7c34ba5523d5bf783bb47f62371e52e0298'
}

const approveSubnet = async (subnet: SubnetInstance) => {
  try {
    approvingSubnets.value.add(subnet.id)

    // Get the correct gateway owner address
    const gatewayOwnerAddress = await getGatewayOwner(subnet)

    // Use the API service with extended timeout for approval
    const response = await apiService.approveSubnet(subnet.id, gatewayOwnerAddress)

    if (response.data?.success) {
      console.log('Subnet approved successfully:', response.data.message)
      // Refresh the subnet list to show updated status
      await fetchSubnets()
    } else {
      console.error('Failed to approve subnet:', response.data?.error)
      error.value = response.data?.error || 'Failed to approve subnet'
    }
  } catch (err: any) {
    console.error('Error approving subnet:', err)
    error.value = err?.message || 'Failed to approve subnet'
  } finally {
    approvingSubnets.value.delete(subnet.id)
  }
}

// Computed
const getStatusColor = (status: string) => {
  switch (status.toLowerCase()) {
    case 'active': return 'text-green-600 bg-green-50'
    case 'paused': return 'text-yellow-600 bg-yellow-50'
    case 'deploying': return 'text-blue-600 bg-blue-50'
    case 'failed': return 'text-red-600 bg-red-50'
    case 'pending approval': return 'text-orange-600 bg-orange-50'
    case 'approved - no validators': return 'text-blue-600 bg-blue-50'
    default: return 'text-gray-600 bg-gray-50'
  }
}

const getStatusIcon = (status: string) => {
  switch (status.toLowerCase()) {
    case 'active':
      return 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z'
    case 'paused':
      return 'M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z'
    case 'deploying':
      return 'M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15'
    case 'failed':
      return 'M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z'
    case 'pending approval':
      return 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z'
    case 'approved - no validators':
      return 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z'
    default:
      return 'M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M12 21a9 9 0 100-18 9 9 0 000 18z'
  }
}

const formatAddress = (address: string) => {
  if (!address || address === 'N/A') return address
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

// Lifecycle
onMounted(() => {
  fetchSubnets()
})
</script>

<template>
  <div class="p-6">
    <!-- Dashboard Header -->
    <div class="mb-8">
      <h1 class="text-3xl font-bold text-gray-900 mb-2">Dashboard</h1>
      <p class="text-gray-600">Manage and monitor your IPC subnet deployments</p>
    </div>

    <!-- Quick Stats -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-600">Total Subnets</p>
            <p class="text-3xl font-bold text-gray-900">{{ subnets.length }}</p>
          </div>
          <div class="w-12 h-12 bg-primary-50 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14-7l2 2m0 0l2 2m-2-2v6m-2-2H5m14-7v2a2 2 0 01-2 2H5a2 2 0 01-2-2V4"/>
            </svg>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-600">Active Subnets</p>
            <p class="text-3xl font-bold text-green-600">
              {{ subnets.filter(s => s.status === 'active').length }}
            </p>
          </div>
          <div class="w-12 h-12 bg-green-50 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
            </svg>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-600">Total Validators</p>
            <p class="text-3xl font-bold text-gray-900">
              {{ subnets.reduce((sum, subnet) => sum + subnet.validators.length, 0) }}
            </p>
          </div>
          <div class="w-12 h-12 bg-blue-50 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>
            </svg>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-600">Total Stake</p>
            <p class="text-3xl font-bold text-purple-600">
              {{ subnets.reduce((sum, subnet) => sum + subnet.validators.reduce((s, v) => s + parseFloat(v.stake || '0'), 0), 0).toFixed(1) }} FIL
            </p>
          </div>
          <div class="w-12 h-12 bg-purple-50 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1"/>
            </svg>
          </div>
        </div>
      </div>
    </div>

    <!-- Your Subnets -->
    <div class="card">
      <div class="flex items-center justify-between mb-6">
        <h2 class="text-xl font-semibold text-gray-900">Your Subnets</h2>
        <RouterLink to="/wizard" class="btn-primary">
          Deploy New Subnet
        </RouterLink>
      </div>

      <!-- Loading State -->
      <div v-if="loading" class="text-center py-8">
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
      <div v-else-if="subnets.length === 0" class="text-center py-12 text-gray-500">
        <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 9a2 2 0 00-2 2v2a2 2 0 002 2m0 0h14m-14 0a2 2 0 002 2v2a2 2 0 01-2 2M5 9V7a2 2 0 012-2h10a2 2 0 012 2v2M7 7V5a2 2 0 012-2h6a2 2 0 012 2v2" />
        </svg>
        <p class="text-lg font-medium mb-2">No Subnets Found</p>
        <p class="mb-4">You haven't deployed any subnets yet.</p>
        <RouterLink to="/wizard" class="btn-primary">
          Deploy Your First Subnet
        </RouterLink>
      </div>

      <!-- Subnets List -->
      <div v-else class="space-y-4">
        <div
          v-for="subnet in subnets"
          :key="subnet.id"
          class="border border-gray-200 rounded-lg p-6 hover:shadow-md transition-shadow"
        >
          <div class="flex items-start justify-between mb-4">
            <div class="flex-1">
              <div class="flex items-center space-x-3 mb-2">
                <h3 class="text-lg font-semibold text-gray-900">{{ subnet.name }}</h3>
                <span
                  :class="[
                    'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium',
                    getStatusColor(subnet.status)
                  ]"
                >
                  <svg
                    :class="['w-3 h-3 mr-1']"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      :d="getStatusIcon(subnet.status)"
                    />
                  </svg>
                  {{ subnet.status.charAt(0).toUpperCase() + subnet.status.slice(1) }}
                </span>
              </div>
              <p class="text-gray-600 text-sm mb-1">{{ subnet.id }}</p>
              <p class="text-gray-500 text-sm">Parent: {{ subnet.parent }}</p>
              <!-- Gateway Information -->
              <p v-if="subnet.config?.gateway_addr" class="text-gray-500 text-sm">
                Gateway: {{ formatAddress(subnet.config.gateway_addr.toString()) }}
              </p>
            </div>

            <div class="flex space-x-2">
              <RouterLink
                :to="`/instance/${encodeURIComponent(subnet.id)}`"
                class="btn-secondary text-sm"
              >
                View Details
              </RouterLink>
              <button
                v-if="subnet.status.toLowerCase() === 'pending approval'"
                :disabled="approvingSubnets.has(subnet.id)"
                @click="approveSubnet(subnet)"
                class="btn-primary text-sm"
              >
                {{ approvingSubnets.has(subnet.id) ? 'Approving...' : 'Approve' }}
              </button>
            </div>
          </div>

          <!-- Subnet Metrics -->
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
            <div>
              <p class="text-sm text-gray-500">Validators</p>
              <p class="font-semibold text-gray-900">{{ subnet.validators.length }}</p>
            </div>
            <div>
              <p class="text-sm text-gray-500">Total Stake</p>
              <p class="font-semibold text-gray-900">{{ subnet.validators.reduce((s, v) => s + parseFloat(v.stake || '0'), 0).toFixed(1) }} FIL</p>
            </div>
            <div>
              <p class="text-sm text-gray-500">Template</p>
              <p class="font-semibold text-gray-900">{{ subnet.template }}</p>
            </div>
            <div>
              <p class="text-sm text-gray-500">Created</p>
              <p class="font-semibold text-gray-900">{{ new Date(subnet.created_at).toLocaleDateString() }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>