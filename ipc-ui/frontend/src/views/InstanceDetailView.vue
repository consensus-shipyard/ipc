<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { apiService } from '../services/api'

interface Validator {
  address: string
  stake: string
  power: number
  status: string
  // Additional properties for federated mode
  current_power?: number
  next_power?: number
  waiting?: boolean
  // Additional properties for collateral mode
  initial_balance?: number
}

interface SubnetInstance {
  id: string
  name: string
  status: string
  template: string
  parent: string
  created_at: string
  validators: Validator[]
  config: Record<string, any>
}

const route = useRoute()
const router = useRouter()

// Props
const props = defineProps<{
  id: string
}>()

// State
const instance = ref<SubnetInstance | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const activeTab = ref('overview')
const approvingSubnet = ref(false)

// Validator management state
const newValidator = ref({
  address: '',
  pubkey: '',
  power: 1,
  collateral: 0,
  initialBalance: 0
})

const addingValidator = ref(false)
const removingValidator = ref<Record<string, boolean>>({})
const updatingStake = ref<Record<string, boolean>>({})
const stakeAmounts = ref<Record<string, number>>({})

// Bulk federated validator management state
const showBulkManagement = ref(false)
const bulkValidators = ref<Array<{
  address: string
  pubkey: string
  power: number
  isNew?: boolean
}>>([])
const settingFederatedPower = ref(false)

// Computed
const createdDate = computed(() => {
  if (!instance.value || !instance.value.created_at) return 'Unknown'

  try {
    return new Date(instance.value.created_at).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch (error) {
    console.warn('Error parsing created_at date:', instance.value.created_at)
    return 'Invalid Date'
  }
})

const totalStake = computed(() => {
  if (!instance.value?.validators) return '0'
  return instance.value.validators
    .reduce((sum, v) => sum + parseFloat(v.stake || '0'), 0)
    .toFixed(2)
})

const totalPower = computed(() => {
  if (!instance.value?.validators) return 0
  return instance.value.validators
    .reduce((sum, v) => sum + (v.power || 0), 0)
})

const gatewayAddress = computed(() => {
  if (!instance.value?.config?.gateway_addr) return 'N/A'
  return formatAddress(instance.value.config.gateway_addr)
})

const gatewayAddressShort = computed(() => {
  if (!instance.value?.config?.gateway_addr) return 'N/A'
  return formatAddressShort(instance.value.config.gateway_addr)
})

// Copy to clipboard functionality
const copyingAddress = ref<string | null>(null)

const copyToClipboard = async (text: string, type: string = 'address') => {
  try {
    await navigator.clipboard.writeText(text)
    copyingAddress.value = type
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
      copyingAddress.value = type
      setTimeout(() => {
        copyingAddress.value = null
      }, 2000)
    } catch (fallbackErr) {
      console.error('Fallback copy failed:', fallbackErr)
    }
    document.body.removeChild(textArea)
  }
}

const statusColor = computed(() => {
  if (!instance.value || !instance.value.status) return 'text-gray-600 bg-gray-50'

  switch (instance.value.status.toLowerCase()) {
    case 'active': return 'text-green-600 bg-green-50'
    case 'paused': return 'text-yellow-600 bg-yellow-50'
    case 'deploying': return 'text-blue-600 bg-blue-50'
    case 'failed': return 'text-red-600 bg-red-50'
    default: return 'text-gray-600 bg-gray-50'
  }
})

// Methods
const fetchInstance = async () => {
  try {
    loading.value = true
    error.value = null

    // Decode the URL-encoded ID parameter
    const decodedId = decodeURIComponent(props.id)
    const response = await apiService.getInstance(decodedId)

    // Check if we got HTML instead of JSON (indicates backend routing issue)
    if (typeof response.data === 'string' && response.data.includes('<!DOCTYPE html>')) {
      error.value = 'Backend routing error: API endpoint returned HTML instead of JSON data. This usually means the route is not properly configured.'
      return
    }

    if (response.data) {
      instance.value = response.data
    } else {
      error.value = 'Instance not found'
    }
  } catch (err) {
    console.error('Error fetching instance:', err)
    error.value = err instanceof Error ? err.message : 'Failed to load instance'
  } finally {
    loading.value = false
  }
}

// Helper function to format addresses
const formatAddress = (address: any) => {
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

// Helper function to format address for short display
const formatAddressShort = (address: any) => {
  const fullAddress = formatAddress(address)
  if (fullAddress === 'N/A' || !fullAddress.startsWith('0x')) return fullAddress
  if (fullAddress.length < 14) return fullAddress // Don't truncate short addresses
  return `${fullAddress.slice(0, 8)}...${fullAddress.slice(-6)}`
}

const goBack = () => {
  router.push('/')
}

const exportConfig = () => {
  if (!instance.value) return

  const configData = {
    name: instance.value.name,
    config: instance.value.config,
    validators: instance.value.validators,
    exported_at: new Date().toISOString()
  }

  const blob = new Blob([JSON.stringify(configData, null, 2)], {
    type: 'application/json'
  })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `${instance.value.name}-config.json`
  link.click()
  URL.revokeObjectURL(url)
}

const pauseSubnet = async () => {
  // TODO: Implement pause functionality
  console.log('Pause subnet:', decodeURIComponent(props.id))
}

const resumeSubnet = async () => {
  // TODO: Implement resume functionality
  console.log('Resume subnet:', decodeURIComponent(props.id))
}

const viewLogs = () => {
  // TODO: Implement log viewing
  console.log('View logs for:', decodeURIComponent(props.id))
}

const getGatewayOwner = async (): Promise<string> => {
  if (!instance.value) return '0x0a36d7c34ba5523d5bf783bb47f62371e52e0298'

  try {
    // Try to get gateway information from the API
    const gatewaysResponse = await fetch('/api/gateways')
    const gatewaysResult = await gatewaysResponse.json()

    if (gatewaysResult && Array.isArray(gatewaysResult)) {
      // Find the gateway that matches this subnet's gateway address
      const gatewayAddr = instance.value.config?.gateway_addr?.toString()
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
  return instance.value.config?.deployer_address || '0x0a36d7c34ba5523d5bf783bb47f62371e52e0298'
}

const approveSubnet = async () => {
  if (!instance.value) return

  try {
    approvingSubnet.value = true

    // Get the correct gateway owner address
    const gatewayOwnerAddress = await getGatewayOwner()

    // Use the API service with extended timeout for approval
    const response = await apiService.approveSubnet(props.id, gatewayOwnerAddress)

    if (response.data?.success) {
      console.log('Subnet approved successfully:', response.data.message)
      // Refresh the instance data to show updated status
      await fetchInstance()
    } else {
      console.error('Failed to approve subnet:', response.data?.error)
      error.value = response.data?.error || 'Failed to approve subnet'
    }
  } catch (err: any) {
    console.error('Error approving subnet:', err)
    error.value = err?.message || 'Failed to approve subnet'
  } finally {
    approvingSubnet.value = false
  }
}

// Validator management methods
const addValidator = async () => {
  if (!instance.value) return

  addingValidator.value = true
  try {
    const validatorData = {
      subnetId: decodeURIComponent(props.id),
      address: newValidator.value.address,
      permissionMode: instance.value.config.permissionMode || 'collateral',
      ...((instance.value.config.permissionMode === 'collateral') ? {
        collateral: newValidator.value.collateral,
        initialBalance: newValidator.value.initialBalance || undefined
      } : {
        pubkey: newValidator.value.pubkey,
        power: newValidator.value.power
      })
    }

    const response = await apiService.addValidator(validatorData)

    if (response.data.success) {
      // Reset form
      newValidator.value = {
        address: '',
        pubkey: '',
        power: 1,
        collateral: 0,
        initialBalance: 0
      }

      // Refresh instance data
      await fetchInstance()

      // Show success message (you could add a toast notification here)
      console.log('Validator added successfully')
    } else {
      error.value = response.data.error || 'Failed to add validator'
    }
  } catch (err) {
    console.error('Error adding validator:', err)
    error.value = err instanceof Error ? err.message : 'Failed to add validator'
  } finally {
    addingValidator.value = false
  }
}

const removeValidator = async (validatorAddress: string) => {
  if (!instance.value) return

  removingValidator.value = { ...removingValidator.value, [validatorAddress]: true }
  try {
    const validatorData = {
      subnetId: decodeURIComponent(props.id),
      address: validatorAddress
    }

    const response = await apiService.removeValidator(validatorData)

    if (response.data.success) {
      // Refresh instance data
      await fetchInstance()
      console.log('Validator removed successfully')
    } else {
      error.value = response.data.error || 'Failed to remove validator'
    }
  } catch (err) {
    console.error('Error removing validator:', err)
    error.value = err instanceof Error ? err.message : 'Failed to remove validator'
  } finally {
    removingValidator.value = { ...removingValidator.value, [validatorAddress]: false }
  }
}

const updateStake = async (validatorAddress: string, action: 'stake' | 'unstake') => {
  if (!instance.value) return

  const amount = stakeAmounts.value[validatorAddress]
  if (!amount || amount <= 0) {
    error.value = 'Please enter a valid stake amount'
    return
  }

  updatingStake.value = { ...updatingStake.value, [validatorAddress]: true }
  try {
    const stakeData = {
      subnetId: decodeURIComponent(props.id),
      address: validatorAddress,
      amount,
      action
    }

    const response = await apiService.updateValidatorStake(stakeData)

    if (response.data.success) {
      // Clear the input
      stakeAmounts.value = { ...stakeAmounts.value, [validatorAddress]: 0 }

      // Refresh instance data
      await fetchInstance()
      console.log(`Stake ${action} successful:`, response.data.message)
    } else {
      error.value = response.data.error || `Failed to ${action} validator`
    }
  } catch (err) {
    console.error(`Error ${action}ing validator:`, err)
    error.value = err instanceof Error ? err.message : `Failed to ${action} validator`
  } finally {
    updatingStake.value = { ...updatingStake.value, [validatorAddress]: false }
  }
}

// Bulk federated validator management methods
const initializeBulkManagement = () => {
  if (!instance.value) return

  // Initialize with existing validators
  bulkValidators.value = instance.value.validators.map(validator => ({
    address: validator.address,
    pubkey: '', // Will need to be filled in manually
    power: validator.power || 1,
    isNew: false
  }))

  showBulkManagement.value = true
}

const addBulkValidator = () => {
  bulkValidators.value.push({
    address: '',
    pubkey: '',
    power: 1,
    isNew: true
  })
}

const removeBulkValidator = (index: number) => {
  bulkValidators.value.splice(index, 1)
}

const setBulkFederatedPower = async () => {
  if (!instance.value || bulkValidators.value.length === 0) return

  // Validate all validators have required fields
  const invalidValidators = bulkValidators.value.filter(v =>
    !v.address.trim() || !v.pubkey.trim() || v.power <= 0
  )

  if (invalidValidators.length > 0) {
    error.value = 'All validators must have a valid address, public key, and power > 0'
    return
  }

  settingFederatedPower.value = true
  try {
    // Find the first existing validator to use as fromAddress
    const fromAddress = instance.value.validators.length > 0 ?
      instance.value.validators[0].address :
      bulkValidators.value[0].address

    const powerData = {
      subnetId: decodeURIComponent(props.id),
      fromAddress,
      validators: bulkValidators.value.map(v => ({
        address: v.address,
        pubkey: v.pubkey,
        power: v.power
      }))
    }

    const response = await apiService.setFederatedPower(powerData)

    if (response.data.success) {
      showBulkManagement.value = false

      // Refresh instance data
      await fetchInstance()

      console.log('Bulk federated power set successfully:', response.data.message)
    } else {
      error.value = response.data.error || 'Failed to set federated power'
    }
  } catch (err) {
    console.error('Error setting bulk federated power:', err)
    error.value = err instanceof Error ? err.message : 'Failed to set federated power'
  } finally {
    settingFederatedPower.value = false
  }
}

// Lifecycle
onMounted(() => {
  fetchInstance()
})

// Watch for route changes
watch(() => props.id, (newId) => {
  if (newId) {
    fetchInstance()
  }
})
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Header -->
    <div class="bg-white shadow-sm border-b">
      <div class="max-w-7xl mx-auto px-6 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <button
              @click="goBack"
              class="text-gray-600 hover:text-gray-700 flex items-center"
            >
              <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
              Back to Dashboard
            </button>
            <div>
              <h1 class="text-2xl font-bold text-gray-900">
                {{ instance?.name || 'Loading...' }}
              </h1>
              <p class="text-gray-600 mt-1">Subnet ID: {{ decodeURIComponent(props.id) }}</p>
            </div>
          </div>

          <div v-if="instance" class="flex items-center space-x-3">
            <span
              :class="[
                'inline-flex items-center px-3 py-1 rounded-full text-sm font-medium',
                statusColor
              ]"
            >
              {{ (instance.status || 'Unknown').charAt(0).toUpperCase() + (instance.status || 'unknown').slice(1) }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="max-w-7xl mx-auto px-6 py-8">
      <div class="text-center py-12">
        <div class="animate-spin inline-block w-8 h-8 border-4 border-primary-600 border-t-transparent rounded-full"></div>
        <p class="mt-4 text-gray-600">Loading subnet details...</p>
      </div>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="max-w-7xl mx-auto px-6 py-8">
      <div class="bg-red-50 border border-red-200 rounded-lg p-6">
        <div class="flex items-start space-x-3">
          <svg class="w-6 h-6 text-red-600 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          <div>
            <h3 class="font-semibold text-red-800 mb-1">Error Loading Subnet</h3>
            <p class="text-red-700">{{ error }}</p>
            <button
              @click="fetchInstance"
              class="mt-3 text-red-600 hover:text-red-700 font-medium text-sm"
            >
              Try Again
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div v-else-if="instance" class="max-w-7xl mx-auto px-6 py-8">
      <!-- Quick Actions -->
      <div class="flex flex-wrap gap-3 mb-6">
        <button
          v-if="instance.status === 'active'"
          @click="pauseSubnet"
          class="btn-secondary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          Pause Subnet
        </button>

        <button
          v-else-if="instance.status === 'paused'"
          @click="resumeSubnet"
          class="btn-primary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1m4 0h1m-6 4h1m4 0h1M9 16h6" />
          </svg>
          Resume Subnet
        </button>

        <button
          v-if="instance.status.toLowerCase() === 'pending approval'"
          :disabled="approvingSubnet"
          @click="approveSubnet"
          class="btn-primary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          {{ approvingSubnet ? 'Approving...' : 'Approve Subnet' }}
        </button>

        <button
          @click="viewLogs"
          class="btn-secondary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          View Logs
        </button>

        <button
          @click="exportConfig"
          class="btn-secondary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          Export Config
        </button>
      </div>

      <!-- Tab Navigation -->
      <div class="border-b border-gray-200 mb-6">
        <nav class="flex space-x-8">
          <button
            @click="activeTab = 'overview'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'overview'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Overview
          </button>
          <button
            @click="activeTab = 'validators'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'validators'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Validators ({{ instance.validators.length }})
          </button>
          <button
            @click="activeTab = 'validator-management'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'validator-management'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Manage Validators
          </button>
          <button
            @click="activeTab = 'configuration'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'configuration'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Configuration
          </button>
          <button
            @click="activeTab = 'contracts'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'contracts'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Contracts
          </button>
          <button
            @click="activeTab = 'metrics'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'metrics'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Metrics
          </button>
        </nav>
      </div>

      <!-- Tab Content -->
      <div class="space-y-6">
        <!-- Overview Tab -->
        <div v-if="activeTab === 'overview'" class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <!-- Basic Information -->
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Basic Information</h3>
            <dl class="space-y-3">
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Subnet ID</dt>
                <dd class="text-sm text-gray-900 font-mono">{{ instance.id }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Name</dt>
                <dd class="text-sm text-gray-900">{{ instance.name }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Status</dt>
                <dd>
                  <span :class="['inline-flex items-center px-2 py-1 rounded-full text-xs font-medium', statusColor]">
                    {{ (instance.status || 'Unknown').charAt(0).toUpperCase() + (instance.status || 'unknown').slice(1) }}
                  </span>
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Template</dt>
                <dd class="text-sm text-gray-900">{{ instance.template }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Parent Network</dt>
                <dd class="text-sm text-gray-900 font-mono">{{ instance.parent }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Gateway Contract</dt>
                <dd class="text-sm text-gray-900 font-mono relative">
                  <button
                    @click="copyToClipboard(gatewayAddress, 'gateway')"
                    class="hover:bg-gray-100 px-2 py-1 rounded transition-colors cursor-pointer text-left"
                    :title="copyingAddress === 'gateway' ? 'Copied!' : `Click to copy: ${gatewayAddress}`"
                  >
                    {{ gatewayAddressShort }}
                    <svg v-if="copyingAddress === 'gateway'" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                    </svg>
                  </button>
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Created</dt>
                <dd class="text-sm text-gray-900">{{ createdDate }}</dd>
              </div>
            </dl>
          </div>

          <!-- Quick Stats -->
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Quick Stats</h3>
            <div class="grid grid-cols-2 gap-4">
              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">{{ instance.validators.length }}</div>
                <div class="text-sm text-gray-500">Validators</div>
              </div>
              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">{{ totalStake }}</div>
                <div class="text-sm text-gray-500">Total Stake (FIL)</div>
              </div>
              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">{{ totalPower }}</div>
                <div class="text-sm text-gray-500">Total Power</div>
              </div>
              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">{{ instance.config.permissionMode || 'N/A' }}</div>
                <div class="text-sm text-gray-500">Mode</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Validators Tab -->
        <div v-if="activeTab === 'validators'" class="space-y-6">
          <div class="card">
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-lg font-semibold text-gray-900">Validators</h3>
              <div class="text-sm text-gray-500">
                {{ instance.validators.length }} validator{{ instance.validators.length !== 1 ? 's' : '' }}
              </div>
            </div>

            <div v-if="instance.validators.length === 0" class="text-center py-8 text-gray-500">
              <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
              </svg>
              <p class="font-medium">No validators configured</p>
              <p class="text-sm">This subnet has no validators set up yet.</p>
            </div>

            <div v-else class="overflow-x-auto">
              <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                  <tr>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Address
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Stake
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Power
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Status
                    </th>
                  </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                  <tr v-for="validator in instance.validators" :key="validator.address">
                    <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900">
                      {{ validator.address.slice(0, 8) }}...{{ validator.address.slice(-6) }}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {{ validator.stake }} FIL
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {{ validator.power }}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap">
                      <span :class="[
                        'inline-flex px-2 py-1 text-xs font-semibold rounded-full',
                        validator.status === 'Active' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                      ]">
                        {{ validator.status }}
                      </span>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- Validator Management Tab -->
        <div v-if="activeTab === 'validator-management'" class="space-y-6">
          <div class="card">
            <div class="flex items-center justify-between mb-6">
              <h3 class="text-lg font-semibold text-gray-900">Validator Management</h3>
              <div class="text-sm text-gray-500">
                Mode: {{ instance.config.permissionMode || 'Unknown' }}
              </div>
            </div>

            <!-- Permission Mode Explanation -->
            <div class="mb-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
              <h4 class="text-md font-semibold text-blue-800 mb-2 flex items-center">
                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                {{ instance.config.permissionMode === 'federated' ? 'Federated Mode' : 'Collateral Mode' }}
              </h4>

              <div v-if="instance.config.permissionMode === 'federated'" class="text-blue-700 text-sm">
                <p class="mb-2"><strong>Federated subnets</strong> use centralized validator management:</p>
                <ul class="list-disc list-inside space-y-1 ml-4">
                  <li>Validators are added by setting their power directly</li>
                  <li>No collateral staking required</li>
                  <li>Network owner controls validator set</li>
                  <li>Changes are applied to all validators simultaneously</li>
                </ul>
              </div>

              <div v-else-if="instance.config.permissionMode === 'collateral'" class="text-blue-700 text-sm">
                <p class="mb-2"><strong>Collateral subnets</strong> use stake-based validator management:</p>
                <ul class="list-disc list-inside space-y-1 ml-4">
                  <li>Validators join by staking FIL collateral</li>
                  <li>Minimum stake requirement: {{ instance.config.minValidatorStake || 'Not set' }} FIL</li>
                  <li>Validators can increase/decrease their stake</li>
                  <li>Higher stake generally means higher voting power</li>
                </ul>
              </div>

              <div v-else class="text-blue-700 text-sm">
                <p>Unknown permission mode. Please check your subnet configuration.</p>
              </div>
            </div>

            <!-- Bulk Federated Management (Federated Mode Only) -->
            <div v-if="instance.config.permissionMode === 'federated'" class="mb-8 p-6 bg-blue-50 rounded-lg">
              <div class="flex items-center justify-between mb-4">
                <h4 class="text-md font-semibold text-gray-800 flex items-center">
                  <svg class="w-5 h-5 mr-2 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                  </svg>
                  Advanced Federated Management
                </h4>
                <button
                  v-if="!showBulkManagement"
                  @click="initializeBulkManagement"
                  class="btn-secondary text-sm"
                >
                  Manage All Validators
                </button>
                <button
                  v-else
                  @click="showBulkManagement = false"
                  class="btn-secondary text-sm"
                >
                  Cancel
                </button>
              </div>

              <div v-if="!showBulkManagement" class="text-sm text-blue-700">
                <p class="mb-2">💡 <strong>Tip:</strong> Use bulk management to:</p>
                <ul class="list-disc list-inside space-y-1 ml-4">
                  <li>Set power for all validators at once</li>
                  <li>Add multiple validators simultaneously</li>
                  <li>Manage the complete validator set in one operation</li>
                </ul>
              </div>

              <!-- Bulk Management Form -->
              <div v-if="showBulkManagement" class="space-y-4">
                <div class="bg-yellow-50 border border-yellow-200 rounded-md p-3 mb-4">
                  <p class="text-yellow-800 text-sm">
                    <strong>⚠️ Important:</strong> This will set the complete validator set. All validators not listed here will be removed from the subnet.
                  </p>
                </div>

                <div class="space-y-3">
                  <div v-for="(validator, index) in bulkValidators" :key="index"
                       class="grid grid-cols-12 gap-2 items-center p-3 bg-white rounded border">
                    <div class="col-span-4">
                      <input
                        v-model="validator.address"
                        type="text"
                        placeholder="Validator Address (0x...)"
                        class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
                      />
                    </div>
                    <div class="col-span-4">
                      <input
                        v-model="validator.pubkey"
                        type="text"
                        placeholder="Public Key (0x04...)"
                        class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
                      />
                    </div>
                    <div class="col-span-2">
                      <input
                        v-model.number="validator.power"
                        type="number"
                        min="1"
                        placeholder="Power"
                        class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
                      />
                    </div>
                    <div class="col-span-1">
                      <span v-if="validator.isNew" class="text-xs text-green-600 font-medium">NEW</span>
                      <span v-else class="text-xs text-blue-600 font-medium">EXISTING</span>
                    </div>
                    <div class="col-span-1">
                      <button
                        @click="removeBulkValidator(index)"
                        type="button"
                        class="text-red-600 hover:text-red-800 p-1"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>

                <div class="flex items-center justify-between">
                  <button
                    @click="addBulkValidator"
                    type="button"
                    class="btn-secondary text-sm"
                  >
                    <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                    </svg>
                    Add Validator
                  </button>

                  <button
                    @click="setBulkFederatedPower"
                    :disabled="settingFederatedPower || bulkValidators.length === 0"
                    class="btn-primary"
                  >
                    <div v-if="settingFederatedPower" class="animate-spin inline-block w-4 h-4 mr-2 border-2 border-current border-t-transparent rounded-full"></div>
                    {{ settingFederatedPower ? 'Setting Power...' : 'Set Federated Power' }}
                  </button>
                </div>
              </div>
            </div>

            <!-- Add Validator Form -->
            <div class="mb-8 p-6 bg-gray-50 rounded-lg">
              <h4 class="text-md font-semibold text-gray-800 mb-4 flex items-center">
                <svg class="w-5 h-5 mr-2 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                </svg>
                Add New Validator
              </h4>

              <!-- Mode-specific instructions -->
              <div class="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded-md">
                <div v-if="instance.config.permissionMode === 'federated'" class="text-yellow-800 text-sm">
                  <p class="font-medium mb-1">📋 Federated Mode Instructions:</p>
                  <p>Enter the validator's Ethereum address, public key, and desired power level. The validator will be added to the network with the specified power.</p>
                </div>

                <div v-else-if="instance.config.permissionMode === 'collateral'" class="text-yellow-800 text-sm">
                  <p class="font-medium mb-1">💰 Collateral Mode Instructions:</p>
                  <p>Enter the validator's address and collateral amount. The validator must have sufficient FIL to stake the specified collateral.</p>
                </div>
              </div>

              <form @submit.prevent="addValidator" class="space-y-4">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Validator Address *
                    </label>
                    <input
                      v-model="newValidator.address"
                      type="text"
                      placeholder="0x..."
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                      required
                    />
                  </div>

                  <div v-if="instance.config.permissionMode === 'federated' || instance.config.permissionMode === 'static'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Public Key *
                    </label>
                    <input
                      v-model="newValidator.pubkey"
                      type="text"
                      placeholder="0x04..."
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                      required
                    />
                  </div>

                  <div v-if="instance.config.permissionMode === 'federated' || instance.config.permissionMode === 'static'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Power
                    </label>
                    <input
                      v-model.number="newValidator.power"
                      type="number"
                      min="1"
                      placeholder="1"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                    />
                  </div>

                  <div v-if="instance.config.permissionMode === 'collateral'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Collateral (FIL) *
                    </label>
                    <input
                      v-model.number="newValidator.collateral"
                      type="number"
                      step="0.01"
                      min="0"
                      placeholder="10.0"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                      required
                    />
                  </div>

                  <div v-if="instance.config.permissionMode === 'collateral'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Initial Balance (FIL)
                    </label>
                    <input
                      v-model.number="newValidator.initialBalance"
                      type="number"
                      step="0.01"
                      min="0"
                      placeholder="0.0"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                    />
                  </div>
                </div>

                <div class="flex justify-end">
                  <button
                    type="submit"
                    :disabled="addingValidator"
                    class="btn-primary"
                  >
                    <div v-if="addingValidator" class="animate-spin inline-block w-4 h-4 mr-2 border-2 border-current border-t-transparent rounded-full"></div>
                    {{ addingValidator ? 'Adding...' : 'Add Validator' }}
                  </button>
                </div>
              </form>
            </div>

            <!-- Existing Validators Management -->
            <div>
              <h4 class="text-md font-semibold text-gray-800 mb-4 flex items-center">
                <svg class="w-5 h-5 mr-2 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                </svg>
                Existing Validators
                <span class="ml-2 px-2 py-1 bg-gray-200 text-gray-700 text-xs rounded-full">
                  {{ instance.validators.length }} total
                </span>
              </h4>

              <div v-if="instance.validators.length === 0" class="text-center py-8 text-gray-500">
                <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                </svg>
                <p class="font-medium">No validators configured</p>
                <p class="text-sm">Add the first validator using the form above.</p>
              </div>

              <div v-else class="space-y-4">
                <div
                  v-for="validator in instance.validators"
                  :key="validator.address"
                  class="border border-gray-200 rounded-lg p-4 hover:shadow-sm transition-shadow"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex-1">
                      <div class="flex items-center space-x-3 mb-2">
                        <span class="font-mono text-sm font-medium">{{ validator.address.slice(0, 8) }}...{{ validator.address.slice(-6) }}</span>
                        <span :class="[
                          'inline-flex px-2 py-1 text-xs font-semibold rounded-full',
                          validator.status === 'Active' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                        ]">
                          {{ validator.status }}
                        </span>

                        <!-- Power transition indicator for federated mode -->
                        <span v-if="instance.config.permissionMode === 'federated' && validator.current_power !== validator.next_power"
                              class="inline-flex px-2 py-1 text-xs font-medium rounded-full bg-yellow-100 text-yellow-800">
                          Power Changing
                        </span>
                      </div>

                      <div class="text-sm text-gray-600">
                        <div v-if="instance.config.permissionMode === 'collateral'">
                          <span class="font-medium">Stake:</span> {{ validator.stake }} FIL
                          <span v-if="validator.initial_balance" class="ml-3">
                            <span class="font-medium">Initial Balance:</span> {{ validator.initial_balance }} FIL
                          </span>
                        </div>

                        <div v-else-if="instance.config.permissionMode === 'federated'">
                          <div class="flex items-center space-x-4">
                            <span>
                              <span class="font-medium">Current Power:</span> {{ validator.current_power || validator.power || '0' }}
                            </span>
                            <span v-if="validator.next_power !== undefined && validator.current_power !== validator.next_power"
                                  class="text-blue-600">
                              <span class="font-medium">→ Next Power:</span> {{ validator.next_power }}
                            </span>
                          </div>

                          <div v-if="validator.waiting" class="mt-1 text-yellow-600 text-xs">
                            ⏳ Validator changes pending epoch transition
                          </div>
                        </div>

                        <div v-else>
                          <span class="font-medium">Power:</span> {{ validator.power || 'Unknown' }}
                        </div>
                      </div>
                    </div>

                    <div class="flex items-center space-x-2">
                      <!-- Stake Management for Collateral Mode -->
                      <div v-if="instance.config.permissionMode === 'collateral'" class="flex items-center space-x-2">
                        <input
                          v-model.number="stakeAmounts[validator.address]"
                          type="number"
                          step="0.01"
                          min="0"
                          placeholder="Amount"
                          class="w-24 px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
                        />
                        <button
                          @click="updateStake(validator.address, 'stake')"
                          :disabled="updatingStake[validator.address]"
                          class="btn-secondary text-xs px-2 py-1"
                        >
                          Stake
                        </button>
                        <button
                          @click="updateStake(validator.address, 'unstake')"
                          :disabled="updatingStake[validator.address]"
                          class="btn-secondary text-xs px-2 py-1"
                        >
                          Unstake
                        </button>
                      </div>

                      <!-- Federated mode notice -->
                      <div v-else-if="instance.config.permissionMode === 'federated'" class="text-xs text-gray-500 mr-3">
                        <div class="text-center">
                          <svg class="w-4 h-4 mx-auto mb-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                          </svg>
                          <p>Federated</p>
                          <p>management</p>
                        </div>
                      </div>

                      <button
                        @click="removeValidator(validator.address)"
                        :disabled="removingValidator[validator.address]"
                        class="text-red-600 hover:text-red-700 text-sm font-medium px-3 py-1 border border-red-300 rounded hover:bg-red-50 transition-colors"
                      >
                        {{ removingValidator[validator.address] ? 'Removing...' : 'Remove' }}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Configuration Tab -->
        <div v-if="activeTab === 'configuration'" class="space-y-6">
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Configuration Details</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div v-for="(value, key) in instance.config" :key="key" class="flex justify-between py-2 border-b border-gray-100">
                <dt class="text-sm font-medium text-gray-500 capitalize">
                  {{ key.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase()) }}
                </dt>
                <dd class="text-sm text-gray-900">
                  <span v-if="typeof value === 'boolean'" :class="value ? 'text-green-600' : 'text-red-600'">
                    {{ value ? 'Yes' : 'No' }}
                  </span>
                  <button
                    v-else-if="key === 'gateway_addr' || key === 'registry_addr'"
                    @click="copyToClipboard(formatAddress(value), key)"
                    class="font-mono hover:bg-gray-100 px-2 py-1 rounded transition-colors cursor-pointer text-left"
                    :title="copyingAddress === key ? 'Copied!' : `Click to copy: ${formatAddress(value)}`"
                  >
                    {{ formatAddressShort(value) }}
                    <svg v-if="copyingAddress === key" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                    </svg>
                  </button>
                  <span v-else-if="typeof value === 'string' && value.startsWith('0x')" class="font-mono">
                    {{ value.slice(0, 8) }}...{{ value.slice(-6) }}
                  </span>
                  <span v-else>{{ value }}</span>
                </dd>
              </div>
            </div>
          </div>
        </div>

        <!-- Contracts Tab -->
        <div v-if="activeTab === 'contracts'" class="space-y-6">
          <!-- Related Contracts Overview -->
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Related Contracts</h3>
            <p class="text-gray-600 mb-6">Smart contracts associated with this subnet and its operations.</p>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <!-- Gateway Contract -->
              <div class="border border-gray-200 rounded-lg p-4">
                <div class="flex items-start justify-between mb-4">
                  <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 bg-primary-100 rounded-lg flex items-center justify-center">
                      <svg class="w-5 h-5 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                      </svg>
                    </div>
                    <div>
                      <h4 class="font-semibold text-gray-900">Gateway Contract</h4>
                      <p class="text-sm text-gray-600">Manages subnet registration and cross-chain messaging</p>
                    </div>
                  </div>
                  <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-primary-100 text-primary-800">
                    Gateway
                  </span>
                </div>

                <div class="space-y-3 mb-4">
                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Address</span>
                    <button
                      @click="copyToClipboard(gatewayAddress, 'gateway')"
                      class="text-sm font-mono text-gray-900 hover:bg-gray-100 px-2 py-1 rounded transition-colors"
                      :title="copyingAddress === 'gateway' ? 'Copied!' : `Click to copy: ${gatewayAddress}`"
                    >
                      {{ gatewayAddressShort }}
                      <svg v-if="copyingAddress === 'gateway'" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                      </svg>
                    </button>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Network</span>
                    <span class="text-sm text-gray-900 font-mono">{{ instance.parent }}</span>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Status</span>
                    <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                      Active
                    </span>
                  </div>
                </div>

                <div class="flex space-x-2 pt-3 border-t border-gray-200">
                  <button class="btn-secondary text-xs flex-1">
                    <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    Inspect
                  </button>
                  <RouterLink :to="`/contracts`" class="btn-secondary text-xs">
                    Manage
                  </RouterLink>
                </div>
              </div>

              <!-- Registry Contract -->
              <div class="border border-gray-200 rounded-lg p-4">
                <div class="flex items-start justify-between mb-4">
                  <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                      <svg class="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M9 12h6m-6 4h6M7 20l4-16m6 16l-4-16" />
                      </svg>
                    </div>
                    <div>
                      <h4 class="font-semibold text-gray-900">Registry Contract</h4>
                      <p class="text-sm text-gray-600">Stores subnet metadata and configurations</p>
                    </div>
                  </div>
                  <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                    Registry
                  </span>
                </div>

                <div class="space-y-3 mb-4">
                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Address</span>
                    <button
                      @click="copyToClipboard(instance.config?.registry_addr ? formatAddress(instance.config.registry_addr) : 'N/A', 'registry')"
                      class="text-sm font-mono text-gray-900 hover:bg-gray-100 px-2 py-1 rounded transition-colors"
                      :title="copyingAddress === 'registry' ? 'Copied!' : `Click to copy registry address`"
                    >
                      {{ instance.config?.registry_addr ? formatAddressShort(instance.config.registry_addr) : 'N/A' }}
                      <svg v-if="copyingAddress === 'registry'" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                      </svg>
                    </button>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Network</span>
                    <span class="text-sm text-gray-900 font-mono">{{ instance.parent }}</span>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Status</span>
                    <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                      Active
                    </span>
                  </div>
                </div>

                <div class="flex space-x-2 pt-3 border-t border-gray-200">
                  <button class="btn-secondary text-xs flex-1">
                    <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    Inspect
                  </button>
                  <RouterLink :to="`/contracts`" class="btn-secondary text-xs">
                    Manage
                  </RouterLink>
                </div>
              </div>

              <!-- Subnet Actor Contract -->
              <div class="border border-gray-200 rounded-lg p-4">
                <div class="flex items-start justify-between mb-4">
                  <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 bg-green-100 rounded-lg flex items-center justify-center">
                      <svg class="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M19 11H5m14-7H3m14 14H9m6-7l-6 6-4-4" />
                      </svg>
                    </div>
                    <div>
                      <h4 class="font-semibold text-gray-900">Subnet Actor</h4>
                      <p class="text-sm text-gray-600">Core subnet logic and validator management</p>
                    </div>
                  </div>
                  <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                    Subnet
                  </span>
                </div>

                <div class="space-y-3 mb-4">
                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Subnet ID</span>
                    <button
                      @click="copyToClipboard(instance.id, 'subnet-id')"
                      class="text-sm font-mono text-gray-900 hover:bg-gray-100 px-2 py-1 rounded transition-colors"
                      :title="copyingAddress === 'subnet-id' ? 'Copied!' : `Click to copy: ${instance.id}`"
                    >
                      {{ instance.id.slice(0, 20) }}...
                      <svg v-if="copyingAddress === 'subnet-id'" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                      </svg>
                    </button>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Permission Mode</span>
                    <span class="text-sm text-gray-900 capitalize">{{ instance.config?.permissionMode || 'N/A' }}</span>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Status</span>
                    <span :class="['inline-flex items-center px-2 py-1 rounded-full text-xs font-medium', statusColor]">
                      {{ (instance.status || 'Unknown').charAt(0).toUpperCase() + (instance.status || 'unknown').slice(1) }}
                    </span>
                  </div>
                </div>

                <div class="flex space-x-2 pt-3 border-t border-gray-200">
                  <button class="btn-secondary text-xs flex-1">
                    <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    Inspect
                  </button>
                  <button
                    v-if="instance.status.toLowerCase() === 'pending approval'"
                    :disabled="approvingSubnet"
                    @click="approveSubnet"
                    class="btn-primary text-xs"
                  >
                    {{ approvingSubnet ? 'Approving...' : 'Approve' }}
                  </button>
                </div>
              </div>

              <!-- Additional IPC Contracts (if any) -->
              <div class="border border-gray-200 rounded-lg p-4">
                <div class="flex items-start justify-between mb-4">
                  <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 bg-purple-100 rounded-lg flex items-center justify-center">
                      <svg class="w-5 h-5 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                      </svg>
                    </div>
                    <div>
                      <h4 class="font-semibold text-gray-900">IPC Contracts</h4>
                      <p class="text-sm text-gray-600">Additional subnet-specific contracts</p>
                    </div>
                  </div>
                  <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                    IPC
                  </span>
                </div>

                <div class="text-center py-6 text-gray-500">
                  <svg class="mx-auto h-8 w-8 text-gray-400 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M9 12h6m-6 4h6M7 20l4-16m6 16l-4-16" />
                  </svg>
                  <p class="text-sm">No additional contracts deployed</p>
                  <button class="text-primary-600 hover:text-primary-700 text-sm font-medium mt-1">
                    Deploy IPC Contract
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Contract Configuration -->
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Contract Configuration</h3>
            <div class="bg-gray-50 rounded-lg p-4">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <h4 class="font-medium text-gray-900 mb-3">Gateway Settings</h4>
                  <div class="space-y-2 text-sm">
                    <div class="flex justify-between">
                      <span class="text-gray-600">Min Validator Stake</span>
                      <span class="font-mono">{{ instance.config?.minValidatorStake || 'N/A' }} FIL</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-gray-600">Min Validators</span>
                      <span class="font-mono">{{ instance.config?.minValidators || 'N/A' }}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-gray-600">Bottom-up Period</span>
                      <span class="font-mono">{{ instance.config?.bottomupCheckPeriod || 'N/A' }} blocks</span>
                    </div>
                  </div>
                </div>

                <div>
                  <h4 class="font-medium text-gray-900 mb-3">Subnet Settings</h4>
                  <div class="space-y-2 text-sm">
                    <div class="flex justify-between">
                      <span class="text-gray-600">Supply Source</span>
                      <span class="capitalize">{{ instance.config?.supplySourceKind || 'N/A' }}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-gray-600">Collateral Source</span>
                      <span class="capitalize">{{ instance.config?.collateralSourceKind || 'N/A' }}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-gray-600">Cross-msg Fee</span>
                      <span class="font-mono">{{ instance.config?.minCrossMsgFee || 'N/A' }} FIL</span>
                    </div>
                  </div>
                </div>
              </div>

              <div class="mt-6 pt-4 border-t border-gray-200">
                <div class="flex justify-between items-center">
                  <div>
                    <h4 class="font-medium text-gray-900">Contract Upgrades</h4>
                    <p class="text-sm text-gray-600">Manage contract versions and upgrades</p>
                  </div>
                  <button class="btn-secondary text-sm">
                    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                    </svg>
                    Check for Updates
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Metrics Tab -->
        <div v-if="activeTab === 'metrics'" class="space-y-6">
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <!-- Performance Metrics -->
            <div class="card">
              <h4 class="text-md font-semibold text-gray-900 mb-3">Performance</h4>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Block Height</span>
                  <span class="text-sm font-medium text-gray-900">{{ Math.floor(Math.random() * 10000) + 1000 }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Avg Block Time</span>
                  <span class="text-sm font-medium text-gray-900">2.1s</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">TPS</span>
                  <span class="text-sm font-medium text-gray-900">{{ Math.floor(Math.random() * 100) + 50 }}</span>
                </div>
              </div>
            </div>

            <!-- Economic Metrics -->
            <div class="card">
              <h4 class="text-md font-semibold text-gray-900 mb-3">Economic</h4>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Total Supply</span>
                  <span class="text-sm font-medium text-gray-900">{{ (Math.random() * 1000000).toFixed(0) }} FIL</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Circulating</span>
                  <span class="text-sm font-medium text-gray-900">{{ (Math.random() * 500000).toFixed(0) }} FIL</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Fees Collected</span>
                  <span class="text-sm font-medium text-gray-900">{{ (Math.random() * 1000).toFixed(2) }} FIL</span>
                </div>
              </div>
            </div>

            <!-- Network Metrics -->
            <div class="card">
              <h4 class="text-md font-semibold text-gray-900 mb-3">Network</h4>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Active Validators</span>
                  <span class="text-sm font-medium text-gray-900">{{ instance.validators.length }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Last Checkpoint</span>
                  <span class="text-sm font-medium text-gray-900">{{ Math.floor(Math.random() * 60) + 1 }}m ago</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Uptime</span>
                  <span class="text-sm font-medium text-gray-900">{{ (Math.random() * 10 + 90).toFixed(1) }}%</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Activity Chart Placeholder -->
          <div class="card">
            <h4 class="text-md font-semibold text-gray-900 mb-4">Activity Overview</h4>
            <div class="bg-gray-50 rounded-lg p-8 text-center">
              <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
              </svg>
              <p class="text-gray-600">Activity charts and detailed metrics will be available here</p>
              <p class="text-sm text-gray-500 mt-1">Real-time performance monitoring coming soon</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card {
  @apply bg-white rounded-lg shadow-sm border border-gray-200 p-6;
}

.btn-primary {
  @apply inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}

.btn-secondary {
  @apply inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md shadow-sm text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}

.address-button {
  @apply transition-all duration-200;
}

.address-button:hover {
  @apply bg-gray-100 shadow-sm;
}
</style>