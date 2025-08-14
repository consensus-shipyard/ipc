<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import SubnetStatusIndicator from '../components/common/SubnetStatusIndicator.vue'
import { apiService } from '../services/api'
import { useSubnetsStore, type SubnetInstance } from '../stores/subnets'

// Stores
const subnetsStore = useSubnetsStore()

// State
const approvingSubnets = ref<Set<string>>(new Set())
const approvalError = ref<string | null>(null)

// Use store data instead of local state
const subnets = computed(() => subnetsStore.subnets || [])
const loading = computed(() => subnetsStore.isLoading)
const error = computed(() => subnetsStore.error)

// Methods
const fetchSubnets = () => subnetsStore.loadSubnets()

interface Gateway {
  gateway_address: string
  deployer_address: string
}

const getGatewayOwner = async (subnet: SubnetInstance): Promise<string> => {
  try {
    console.log('[DashboardView] Getting gateway owner for subnet:', subnet.id)
    // Use the proper API service instead of direct fetch
    const gatewaysResponse = await apiService.getGateways()
    const gatewaysResult = gatewaysResponse.data

    if (gatewaysResult && Array.isArray(gatewaysResult)) {
      console.log(`[DashboardView] Found ${gatewaysResult.length} gateways from API`)
      // Find the gateway that matches this subnet's gateway address
      const gatewayAddr = subnet.config?.gateway_addr?.toString()
      if (gatewayAddr) {
        console.log(`[DashboardView] Looking for gateway with address: ${gatewayAddr}`)
        const matchingGateway = gatewaysResult.find((gw: Gateway) =>
          gw.gateway_address === gatewayAddr
        )
        if (matchingGateway) {
          console.log(`[DashboardView] Found matching gateway, owner: ${matchingGateway.deployer_address}`)
          return matchingGateway.deployer_address
        } else {
          console.log('[DashboardView] No matching gateway found')
        }
      } else {
        console.log('[DashboardView] No gateway address found in subnet config')
      }
    } else {
      console.warn('[DashboardView] Invalid gateway API response')
    }
  } catch (err) {
    console.warn('[DashboardView] Failed to fetch gateway information:', err)
  }

  // Fallback to config or default address
  const fallbackAddress = subnet.config?.deployer_address || '0x0a36d7c34ba5523d5bf783bb47f62371e52e0298'
  console.log(`[DashboardView] Using fallback address: ${fallbackAddress}`)
  return fallbackAddress
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
      approvalError.value = response.data?.error || 'Failed to approve subnet'
    }
  } catch (err: unknown) {
    console.error('Error approving subnet:', err)
    const errorMessage = err instanceof Error ? err.message : 'Failed to approve subnet'
    approvalError.value = errorMessage
  } finally {
    approvingSubnets.value.delete(subnet.id)
  }
}

// Helper function to format addresses
type AddressInput = string | number[] | { route: number[][] } | number | null | undefined

const formatAddress = (address: AddressInput): string => {
  if (!address) return 'N/A'

  // Handle different address formats
  let addressStr = ''

  if (typeof address === 'string') {
    // Already a string, check if it needs 0x prefix
    addressStr = address
  } else if (Array.isArray(address)) {
    // Handle byte arrays - convert to hex string
    if (address.length >= 20 && address.every(b => typeof b === 'number' && b >= 0 && b <= 255)) {
      // This is a 20-byte (or longer) Ethereum address as array of numbers
      // Take only the first 20 bytes for the address
      const addressBytes = address.slice(0, 20)
      addressStr = '0x' + addressBytes.map(b => b.toString(16).padStart(2, '0')).join('')
    } else {
      return 'N/A (invalid array)'
    }
  } else if (typeof address === 'object') {
    // Handle object format
    if (address.route && Array.isArray(address.route)) {
      // Subnet ID format - extract the address from route
      const lastRoute = address.route[address.route.length - 1]
      if (lastRoute && Array.isArray(lastRoute) && lastRoute.length === 20) {
        addressStr = '0x' + lastRoute.map(b => b.toString(16).padStart(2, '0')).join('')
      } else {
        return 'N/A (invalid route)'
      }
    } else {
      return 'N/A (invalid object)'
    }
  } else if (typeof address === 'number') {
    return 'N/A (single number)'
  } else {
    return 'N/A (unknown format)'
  }

  // Ensure we have a valid hex address format
  if (addressStr && !addressStr.startsWith('0x') && addressStr.length === 40) {
    addressStr = '0x' + addressStr
  }

  // Validate the address length
  if (addressStr.startsWith('0x') && addressStr.length !== 42) {
    return 'N/A (invalid length)'
  }

  return addressStr
}



// Helper functions to safely calculate metrics and avoid NaN values
const safeParseStake = (stake: string | number | null | undefined): number => {
  if (stake === null || stake === undefined || stake === '') return 0
  const parsed = parseFloat(stake.toString())
  return isNaN(parsed) ? 0 : parsed
}

const safeCalculateSubnetStake = (subnet: SubnetInstance): number => {
  if (!subnet.validators || !Array.isArray(subnet.validators)) return 0
  return subnet.validators.reduce((sum, validator) => sum + safeParseStake(validator.stake), 0)
}

const safeGetValidatorCount = (subnet: SubnetInstance): number => {
  return subnet.validators?.length || 0
}

const getGatewayAddressFull = (subnet: SubnetInstance): string => {
  if (!subnet?.config?.gateway_addr) return 'N/A'
  return formatAddress(subnet.config.gateway_addr)
}

// Gateway grouping and collapsible sections
const collapsedGateways = ref<Set<string>>(new Set())

const groupedSubnets = computed(() => {
  const groups = new Map<string, SubnetInstance[]>()

  subnets.value.forEach(subnet => {
    const gatewayAddr = getGatewayAddressFull(subnet)
    if (!groups.has(gatewayAddr)) {
      groups.set(gatewayAddr, [])
    }
    groups.get(gatewayAddr)!.push(subnet)
  })

  return Array.from(groups.entries()).map(([gateway, subnets]) => ({
    gateway,
    subnets,
    count: subnets.length,
    activeCount: subnets.filter(s => s.status === 'active').length,
    totalValidators: subnets.reduce((sum, s) => sum + safeGetValidatorCount(s), 0),
    totalStake: subnets.reduce((sum, s) => sum + safeCalculateSubnetStake(s), 0)
  }))
})

const toggleGateway = (gateway: string) => {
  if (collapsedGateways.value.has(gateway)) {
    collapsedGateways.value.delete(gateway)
  } else {
    collapsedGateways.value.add(gateway)
  }
}

const isGatewayCollapsed = (gateway: string) => {
  return collapsedGateways.value.has(gateway)
}

// Copy to clipboard functionality
const copyingAddress = ref<string | null>(null)

const copyToClipboard = async (text: string, subnetId: string) => {
  try {
    await navigator.clipboard.writeText(text)
    copyingAddress.value = subnetId
    setTimeout(() => {
      copyingAddress.value = null
    }, 2000)
  } catch (err) {
    console.error('Failed to copy to clipboard:', err)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = text
    document.body.appendChild(textArea)
    textArea.focus()
    textArea.select()
    try {
      document.execCommand('copy')
      copyingAddress.value = subnetId
      setTimeout(() => {
        copyingAddress.value = null
      }, 2000)
    } catch (fallbackErr) {
      console.error('Fallback copy failed:', fallbackErr)
    }
    document.body.removeChild(textArea)
  }
}

// Enhanced subnet status event handlers
const handleStartValidators = (subnet: SubnetInstance) => {
  console.log('Start validators for subnet:', subnet.id)
  // TODO: Implement validator setup workflow
  // Could open a modal with step-by-step validator setup instructions
}

const handleTroubleshoot = (subnet: SubnetInstance) => {
  console.log('Troubleshoot subnet:', subnet.id)
  // TODO: Implement troubleshooting workflow
  // Could show detailed error information and suggested solutions
}

const handleSubnetRetry = (subnetId: string) => {
  console.log('Retrying load for subnet:', subnetId)
  subnetsStore.loadSubnetDetails(subnetId)
}

// Lifecycle - Data fetching is now handled by the centralized app store
// onMounted(() => {
//   fetchSubnets() // Removed - handled by app store
// })
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
            <p class="text-3xl font-bold text-gray-900">{{ subnets?.length || 0 }}</p>
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
            <p class="text-2xl font-bold text-green-600">
              {{ subnets?.filter(s => s.status === 'active')?.length || 0 }}
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
            <p class="text-2xl font-bold text-blue-600">
              {{ subnets?.reduce((sum, subnet) => sum + (subnet.validators?.length || 0), 0) || 0 }}
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
              {{ subnets.reduce((sum, subnet) => sum + safeCalculateSubnetStake(subnet), 0).toFixed(1) }} FIL
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

      <!-- Global Error State (only for initial load failures) -->
      <div v-if="error && (subnets?.length || 0) === 0" class="bg-red-50 border border-red-200 rounded-lg p-6 mb-6">
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

      <!-- Initial Loading State (only when no subnets exist) -->
      <div v-if="loading && (subnets?.length || 0) === 0" class="text-center py-8">
        <div class="animate-spin inline-block w-8 h-8 border-4 border-primary-600 border-t-transparent rounded-full"></div>
        <p class="mt-4 text-gray-600">Loading subnets...</p>
      </div>

      <!-- Empty State -->
      <div v-else-if="!loading && (subnets?.length || 0) === 0" class="text-center py-12 text-gray-500">
        <div class="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center">
          <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01"/>
          </svg>
        </div>
        <h3 class="text-lg font-medium text-gray-900 mb-2">No Subnets Found</h3>
        <p class="text-gray-600 mb-6">No subnets are currently deployed in this network.</p>
        <RouterLink to="/wizard" class="btn-primary">Deploy Your First Subnet</RouterLink>
      </div>

      <!-- Gateway-Grouped Subnets -->
      <div v-if="(subnets?.length || 0) > 0" class="space-y-6">
        <div
          v-for="group in groupedSubnets"
          :key="group.gateway"
          class="border border-gray-200 rounded-lg overflow-hidden"
        >
          <!-- Gateway Header -->
          <div
            @click="toggleGateway(group.gateway)"
            class="gateway-header bg-gray-50 border-b border-gray-200 p-4 cursor-pointer"
            :class="{
              'rounded-b-lg border-b-0': isGatewayCollapsed(group.gateway),
              'hover:shadow-sm': true
            }"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center space-x-3">
                <!-- Expand/Collapse Icon -->
                <svg
                  class="w-5 h-5 text-gray-500 transition-transform duration-200"
                  :class="{ 'rotate-90': !isGatewayCollapsed(group.gateway) }"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>

                <!-- Gateway Icon -->
                <div class="w-8 h-8 bg-primary-100 rounded-lg flex items-center justify-center">
                  <svg class="w-4 h-4 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                  </svg>
                </div>

                <div>
                  <h3 class="text-lg font-semibold text-gray-900">Gateway</h3>
                  <button
                    @click.stop="copyToClipboard(group.gateway, `gateway-${group.gateway}`)"
                    class="text-sm text-gray-600 font-mono hover:bg-gray-200 px-2 py-1 rounded transition-colors"
                    :title="copyingAddress === `gateway-${group.gateway}` ? 'Copied!' : `Click to copy: ${group.gateway}`"
                  >
                    {{ (group.gateway?.length || 0) > 20 ? `${group.gateway?.slice(0, 8)}...${group.gateway?.slice(-6)}` : group.gateway || 'N/A' }}
                    <svg v-if="copyingAddress === `gateway-${group.gateway}`" class="inline-block w-3 h-3 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                    </svg>
                  </button>
                </div>
              </div>

              <!-- Gateway Summary -->
              <div class="flex items-center space-x-6 text-sm text-gray-600">
                <div class="text-center">
                  <p class="font-semibold text-gray-900">{{ group.count }}</p>
                  <p>{{ group.count === 1 ? 'Subnet' : 'Subnets' }}</p>
                </div>
                <div class="text-center">
                  <p class="font-semibold text-green-600">{{ group.activeCount }}</p>
                  <p>Active</p>
                </div>
                <div class="text-center">
                  <p class="font-semibold text-gray-900">{{ group.totalValidators }}</p>
                  <p>Validators</p>
                </div>
                <div class="text-center">
                  <p class="font-semibold text-purple-600">{{ group.totalStake.toFixed(1) }}</p>
                  <p>FIL Stake</p>
                                </div>
              </div>
            </div>
          </div>

          <!-- Subnets in Gateway -->
          <Transition
            name="gateway-collapse"
            enter-active-class="gateway-collapse-enter-active"
            leave-active-class="gateway-collapse-leave-active"
            enter-from-class="gateway-collapse-enter-from"
            leave-to-class="gateway-collapse-leave-to"
          >
            <div
              v-if="!isGatewayCollapsed(group.gateway)"
              class="divide-y divide-gray-200"
            >
            <div
              v-for="subnet in group.subnets"
              :key="subnet.id"
              class="relative p-6 hover:bg-gray-50 transition-colors"
            >
              <!-- Per-subnet loading indicator -->
              <SubnetLoadingIndicator
                :is-loading="subnet.isLoading"
                :has-error="!!subnet.loadError"
                :error-message="subnet.loadError || 'Failed to load subnet data'"
                :loading-text="'Loading subnet data...'"
                @retry="handleSubnetRetry(subnet.id)"
              />

              <div class="flex items-start justify-between mb-4">
                <div class="flex-1">
                  <div class="flex items-center space-x-3 mb-2">
                    <h4 class="text-lg font-semibold text-gray-900">{{ subnet.name }}</h4>
                  </div>

                  <!-- Enhanced status with new component -->
                  <SubnetStatusIndicator
                    :subnet="subnet"
                    :show-details="false"
                    @start-validators="handleStartValidators(subnet)"
                    @troubleshoot="handleTroubleshoot(subnet)"
                  />
                  <p class="text-gray-600 text-sm mb-1">{{ subnet.id }}</p>
                  <p class="text-gray-500 text-sm">Parent: {{ subnet.parent }}</p>
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
              <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div>
                  <p class="text-sm text-gray-500">Validators</p>
                  <p class="font-semibold text-gray-900">{{ safeGetValidatorCount(subnet) }}</p>
                </div>
                <div>
                  <p class="text-sm text-gray-500">Total Stake</p>
                  <p class="font-semibold text-gray-900">{{ safeCalculateSubnetStake(subnet).toFixed(1) }} FIL</p>
                </div>
                <div>
                  <p class="text-sm text-gray-500">Permission Mode</p>
                  <p class="font-semibold text-gray-900 capitalize">{{ subnet.config?.permissionMode || 'Unknown' }}</p>
                </div>
                <div v-if="subnet.created_at">
                  <p class="text-sm text-gray-500">Created</p>
                  <p class="font-semibold text-gray-900">{{ new Date(subnet.created_at).toLocaleDateString() }}</p>
                </div>
              </div>
            </div>
          </div>
          </Transition>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.address-button {
  @apply transition-all duration-200;
}

.address-button:hover {
  @apply bg-gray-100 shadow-sm;
}

.btn-primary {
  @apply inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}

.btn-secondary {
  @apply inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md shadow-sm text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}

.gateway-header {
  transition: all 0.2s ease-in-out;
}

.gateway-header:hover {
  background-color: #f9fafb;
}

.rotate-90 {
  transform: rotate(90deg);
}

.gateway-collapse-enter-active,
.gateway-collapse-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.gateway-collapse-enter-from,
.gateway-collapse-leave-to {
  max-height: 0;
  opacity: 0;
}
</style>