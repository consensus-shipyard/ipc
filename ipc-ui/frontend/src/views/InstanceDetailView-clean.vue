<script setup lang="ts">
import { computed, onMounted, ref, toRef, watch } from 'vue'
import { useRouter } from 'vue-router'

// Components
import SubnetHeader from '@/components/subnet/SubnetHeader.vue'
import SubnetQuickActions from '@/components/subnet/SubnetQuickActions.vue'
import SubnetTabNavigation from '@/components/subnet/tabs/SubnetTabNavigation.vue'

// Modals
import AddValidatorModal from '@/components/subnet/modals/AddValidatorModal.vue'
import NodeConfigModal from '@/components/subnet/modals/NodeConfigModal.vue'
import TestTransactionModal from '@/components/subnet/modals/TestTransactionModal.vue'

// Composables
import { useChainStats } from '@/composables/useChainStats'
import { useSubnetInstance } from '@/composables/useSubnetInstance'
import { useClipboard } from '@/utils/clipboard'

// Types
import type {
    BulkValidator,
    NodeConfigData
} from '@/types/subnet'

// Services
import { ValidatorService } from '@/services/subnet/validator.service'

// Utils

const router = useRouter()

// Props
const props = defineProps<{
  id: string
}>()

// Core state
const activeTab = ref('overview')
const approvingSubnet = ref(false)

// Use composables
const subnetId = computed(() => decodeURIComponent(props.id))
const {
  instance,
  loading,
  error,
  createdDate,
  totalStake,
  gatewayAddress,
  gatewayAddressShort,
  subnetActorAddress,
  subnetActorAddressShort,
  statusColor,
  fetchInstance,
  approveSubnet: approveSubnetAction,
  exportConfig
} = useSubnetInstance(subnetId.value)

const subnetIdRef = toRef(() => instance.value?.data?.id || instance.value?.id)
const {
  chainStats,
  subnetStatus,
  loading: loadingStats,
  error: statsError,
  startAutoRefresh: startStatsRefresh
} = useChainStats(subnetIdRef)

const { copy, isCopying } = useClipboard()

// For backward compatibility with existing template
const loadingBasicInfo = loading
const basicInfoError = error
const loadingChainStats = loadingStats

// Modal states
const showTestTxModal = ref(false)
const showAddValidatorModal = ref(false)
const showNodeConfigModal = ref(false)
const showBulkManagement = ref(false)

// Validator management state
const removingValidator = ref<Record<string, boolean>>({})
const updatingStake = ref<Record<string, boolean>>({})
const stakeAmounts = ref<Record<string, number>>({})
const bulkValidators = ref<BulkValidator[]>([])
const settingFederatedPower = ref(false)

// Node config state
const nodeConfigData = ref<NodeConfigData | null>(null)
const loadingNodeConfig = ref(false)

// Additional computed properties needed for backward compatibility
const copyingAddress = computed(() => {
  // Return the current copying item from the clipboard composable
  return isCopying
})

// Copy to clipboard handler wrapper
const copyToClipboard = async (text: string, type: string = 'address') => {
  await copy(text, type)
}

// Methods
const goBack = () => {
  router.push('/')
}

// Event handlers
const pauseSubnet = async () => {
  // TODO: Implement pause functionality
  console.log('Pause subnet:', subnetId.value)
}

const resumeSubnet = async () => {
  // TODO: Implement resume functionality
  console.log('Resume subnet:', subnetId.value)
}

const viewLogs = () => {
  // TODO: Implement log viewing
  console.log('View logs for:', subnetId.value)
}

const approveSubnet = async () => {
  approvingSubnet.value = true
  try {
    const result = await approveSubnetAction()
    if (!result.success) {
      console.error('Failed to approve subnet:', result.error)
    } else {
      console.log('Subnet approved successfully:', result.message)
    }
  } finally {
    approvingSubnet.value = false
  }
}

// Validator management methods
const handleValidatorAdded = async () => {
  await fetchInstance()
}

const removeValidator = async (validatorAddress: string) => {
  if (!instance.value) return

  removingValidator.value = { ...removingValidator.value, [validatorAddress]: true }
  try {
    const validatorData = {
      subnetId: subnetId.value,
      address: validatorAddress
    }

    const response = await ValidatorService.removeValidator(validatorData)

    if (response.data.success) {
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
      subnetId: subnetId.value,
      address: validatorAddress,
      amount,
      action
    }

    const response = await ValidatorService.updateStake(stakeData)

    if (response.data.success) {
      stakeAmounts.value = { ...stakeAmounts.value, [validatorAddress]: 0 }
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

// Node configuration methods
const showNodeConfig = async (validatorAddress: string) => {
  if (!instance.value) return

  loadingNodeConfig.value = true
  showNodeConfigModal.value = true

  try {
    const result = await ValidatorService.getNodeConfigAndCommands(
      instance.value.data?.id || instance.value.id || '',
      validatorAddress
    )

    if (result.success && result.data) {
      nodeConfigData.value = result.data
    } else {
      error.value = result.error || 'Failed to generate node configuration'
      showNodeConfigModal.value = false
    }
  } catch (err) {
    console.error('Error fetching node config:', err)
    error.value = err instanceof Error ? err.message : 'Failed to generate node configuration'
    showNodeConfigModal.value = false
  } finally {
    loadingNodeConfig.value = false
  }
}

// Bulk federated validator management methods
const initializeBulkManagement = () => {
  if (!instance.value) return

  const validatorList = instance.value.data?.validators || instance.value.validators || []
  bulkValidators.value = validatorList.map(validator => ({
    address: validator.address,
    pubkey: '',
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

  const invalidValidators = bulkValidators.value.filter(v =>
    !v.address.trim() || !v.pubkey.trim() || v.power <= 0
  )

  if (invalidValidators.length > 0) {
    error.value = 'All validators must have a valid address, public key, and power > 0'
    return
  }

  settingFederatedPower.value = true
  try {
    const validatorList = instance.value.data?.validators || instance.value.validators || []
    const fromAddress = validatorList.length > 0 ?
      validatorList[0].address :
      bulkValidators.value[0].address

    const powerData = {
      subnetId: subnetId.value,
      fromAddress,
      validators: bulkValidators.value.map(v => ({
        address: v.address,
        pubkey: v.pubkey,
        power: v.power
      }))
    }

    const response = await ValidatorService.setFederatedPower(powerData)

    if (response.data.success) {
      showBulkManagement.value = false
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

// Event handlers for test transactions
const handleTransactionSent = () => {
  // Refresh stats after successful transaction
  setTimeout(() => {
    startStatsRefresh()
  }, 2000)
}

// Lifecycle
onMounted(async () => {
  await fetchInstance()
  if (instance.value) {
    startStatsRefresh()
  }
})

// Watch for route changes
watch(() => props.id, async (newId) => {
  if (newId) {
    await fetchInstance()
    if (instance.value) {
      startStatsRefresh()
    }
  }
})
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Header -->
    <SubnetHeader
      :instance="instance"
      :subnet-id="props.id"
      :status-color="statusColor"
    />

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
    <div v-else-if="instance || loading" class="max-w-7xl mx-auto px-6 py-8">
      <!-- Quick Actions -->
      <SubnetQuickActions
        :instance="instance"
        :approving-subnet="approvingSubnet"
        @approve="approveSubnet"
        @pause="pauseSubnet"
        @resume="resumeSubnet"
        @test-tx="showTestTxModal = true"
        @view-logs="viewLogs"
        @export-config="exportConfig"
      />

      <!-- Tab Navigation -->
      <SubnetTabNavigation
        v-model="activeTab"
        :instance="instance"
      />

      <!-- Tab Content would go here -->
      <!-- For now, showing a placeholder -->
      <div class="bg-white rounded-lg shadow p-6">
        <p class="text-gray-600">Tab content for "{{ activeTab }}" would be displayed here.</p>
        <p class="text-sm text-gray-500 mt-2">The tab components need to be created separately to complete the refactoring.</p>
      </div>

      <!-- Modals -->
      <AddValidatorModal
        v-model="showAddValidatorModal"
        :instance="instance"
        @validator-added="handleValidatorAdded"
      />

      <NodeConfigModal
        v-model="showNodeConfigModal"
        :config-data="nodeConfigData"
        :loading="loadingNodeConfig"
      />

      <TestTransactionModal
        v-model="showTestTxModal"
        :instance="instance"
        @transaction-sent="handleTransactionSent"
      />
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
