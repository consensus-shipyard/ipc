<template>
  <div class="bg-white rounded-lg border border-gray-200 p-6">
    <div class="flex items-center justify-between mb-6">
      <div class="flex-1">
        <div class="flex items-center">
          <h3 class="text-lg font-semibold text-gray-900">Subnet Setup Status</h3>
          <!-- Expand/Collapse button -->
          <button
            v-if="checklist?.steps?.length > 0"
            @click="toggleExpanded"
            class="ml-3 p-1 text-gray-400 hover:text-gray-600 transition-colors"
            :title="isExpanded ? 'Collapse steps' : 'Expand steps'"
          >
            <svg class="w-5 h-5 transform transition-transform" :class="{ 'rotate-180': isExpanded }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
            </svg>
          </button>
        </div>
        <p class="text-sm text-gray-600 mt-1">
          Track configuration steps and complete missing requirements
        </p>
      </div>

      <!-- Overall status badge -->
      <div class="flex items-center space-x-3">
        <div v-if="checklist?.all_complete" class="flex items-center text-green-600">
          <svg class="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
          </svg>
          <span class="font-medium">Setup Complete</span>
        </div>
        <div v-else class="flex items-center text-yellow-600">
          <svg class="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
          </svg>
          <span class="font-medium">Setup Required</span>
        </div>

        <!-- Permission mode badge -->
        <span v-if="checklist?.permission_mode"
              class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-blue-100 text-blue-800 capitalize">
          {{ checklist.permission_mode }} Mode
        </span>
      </div>
    </div>

    <!-- Next action alert -->
    <div v-if="checklist?.next_required_action && !checklist.all_complete"
         class="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
      <div class="flex">
        <svg class="w-5 h-5 text-yellow-400 mt-0.5 mr-3" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
        </svg>
        <div>
          <h4 class="text-sm font-medium text-yellow-800">Next Action Required</h4>
          <p class="text-sm text-yellow-700 mt-1">{{ checklist.next_required_action }}</p>
        </div>
      </div>
    </div>

    <!-- Setup steps checklist (collapsible) -->
    <div v-if="checklist?.steps?.length > 0" class="transition-all duration-300">
      <div v-show="isExpanded" class="space-y-2">
        <div v-for="(step, index) in checklist.steps"
             :key="step.id"
             class="flex items-center space-x-3 p-3 rounded-md border transition-colors"
             :class="getStepBorderClass(step.status)">

          <!-- Step status icon -->
          <div class="flex-shrink-0">
            <div class="flex items-center justify-center w-6 h-6 rounded-full"
                 :class="getStepIconClass(step.status)">
              <!-- Completed -->
              <svg v-if="step.status === 'completed'" class="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
              </svg>
              <!-- In Progress -->
              <svg v-else-if="step.status === 'in_progress'" class="w-3 h-3 text-white animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              <!-- Pending or Failed -->
              <span v-else class="text-xs font-semibold">{{ index + 1 }}</span>
            </div>
          </div>

          <!-- Step content -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between">
              <div class="flex-1 min-w-0">
                <div class="flex items-center">
                  <h4 class="text-sm font-medium text-gray-900 truncate">{{ step.title }}</h4>
                  <span class="ml-2 inline-flex items-center px-1.5 py-0.5 rounded-full text-xs font-medium"
                        :class="getStatusBadgeClass(step.status)">
                    {{ formatStepStatus(step.status) }}
                  </span>
                </div>
                <p class="text-xs text-gray-600 mt-0.5 line-clamp-1">{{ step.description }}</p>
              </div>

              <!-- Action button -->
              <div v-if="step.action_available && step.action_button_text" class="flex-shrink-0 ml-3">
                <button @click="handleStepAction(step)"
                        :disabled="actionInProgress === step.id"
                        class="inline-flex items-center px-2 py-1 border border-transparent text-xs font-medium rounded text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-1 focus:ring-offset-1 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed">
                  <svg v-if="actionInProgress === step.id" class="w-3 h-3 mr-1 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  {{ actionInProgress === step.id ? 'Processing...' : step.action_button_text }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Success message when all complete -->
    <div v-if="checklist?.all_complete"
         class="mt-6 bg-green-50 border border-green-200 rounded-lg p-4">
      <div class="flex">
        <svg class="w-5 h-5 text-green-400 mt-0.5 mr-3" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
        </svg>
        <div>
          <h4 class="text-sm font-medium text-green-800">Setup Complete!</h4>
          <p class="text-sm text-green-700 mt-1">Your subnet is fully configured and ready for use.</p>
        </div>
      </div>
    </div>

    <!-- Modals for actions -->
    <SetFederatedPowerModal
      v-if="showFederatedPowerModal"
      :show="showFederatedPowerModal"
      :subnet-id="subnetId"
      :initial-data="modalInitialData"
      @close="showFederatedPowerModal = false"
      @success="handleActionSuccess"
    />

    <ApproveSubnetModal
      v-if="showApproveModal"
      :show="showApproveModal"
      :subnet-id="subnetId"
      :initial-data="modalInitialData"
      @close="showApproveModal = false"
      @success="handleActionSuccess"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import ApproveSubnetModal from './modals/ApproveSubnetModal.vue'
import SetFederatedPowerModal from './modals/SetFederatedPowerModal.vue'

interface SetupStep {
  id: string
  title: string
  description: string
  status: 'completed' | 'pending' | 'in_progress' | 'failed' | 'not_applicable'
  required: boolean
  action_available: boolean
  action_button_text?: string
  action_type?: string
  details?: any
}

interface SubnetSetupChecklist {
  permission_mode: string
  steps: SetupStep[]
  next_required_action?: string
  all_complete: boolean
}

interface Props {
  subnetId: string
  checklist?: SubnetSetupChecklist
  loading?: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{
  refresh: []
}>()

// Modal state
const showFederatedPowerModal = ref(false)
const showApproveModal = ref(false)
const modalInitialData = ref<any>(null)
const actionInProgress = ref<string | null>(null)

// Collapsible state
const isExpanded = ref(false)

// Compute default expanded state: expand if any steps failed or are in progress
const shouldBeExpanded = computed(() => {
  if (!props.checklist?.steps?.length) return false

  // Always expand if there are failed or in-progress steps
  const hasFailuresOrProgress = props.checklist.steps.some(step =>
    step.status === 'failed' || step.status === 'in_progress'
  )

  if (hasFailuresOrProgress) return true

  // Collapse by default if all steps are completed
  return !props.checklist.all_complete
})

// Watch for changes in checklist and update expanded state
watch(shouldBeExpanded, (newValue) => {
  isExpanded.value = newValue
}, { immediate: true })

// Toggle function
const toggleExpanded = () => {
  isExpanded.value = !isExpanded.value
}

// Handle step action button clicks
const handleStepAction = (step: SetupStep) => {
  if (!step.action_type || actionInProgress.value) return

  actionInProgress.value = step.id
  modalInitialData.value = step.details

  switch (step.action_type) {
    case 'set_federated_power':
      showFederatedPowerModal.value = true
      break
    case 'approve_subnet':
      showApproveModal.value = true
      break
    case 'join_subnet':
      // TODO: Implement join subnet modal
      console.log('Join subnet action not yet implemented')
      actionInProgress.value = null
      break
    default:
      console.warn('Unknown action type:', step.action_type)
      actionInProgress.value = null
  }
}

// Handle successful actions
const handleActionSuccess = () => {
  actionInProgress.value = null
  showFederatedPowerModal.value = false
  showApproveModal.value = false

  // Refresh the parent component to get updated status
  emit('refresh')
}

// Utility functions for styling
const getStepBorderClass = (status: string) => {
  switch (status) {
    case 'completed': return 'border-green-200 bg-green-50'
    case 'in_progress': return 'border-blue-200 bg-blue-50'
    case 'failed': return 'border-red-200 bg-red-50'
    case 'pending': return 'border-yellow-200 bg-yellow-50'
    default: return 'border-gray-200 bg-gray-50'
  }
}

const getStepIconClass = (status: string) => {
  switch (status) {
    case 'completed': return 'bg-green-500 text-white'
    case 'in_progress': return 'bg-blue-500 text-white'
    case 'failed': return 'bg-red-500 text-white'
    case 'pending': return 'bg-yellow-500 text-white'
    default: return 'bg-gray-300 text-gray-600'
  }
}

const getStatusBadgeClass = (status: string) => {
  switch (status) {
    case 'completed': return 'bg-green-100 text-green-800'
    case 'in_progress': return 'bg-blue-100 text-blue-800'
    case 'failed': return 'bg-red-100 text-red-800'
    case 'pending': return 'bg-yellow-100 text-yellow-800'
    default: return 'bg-gray-100 text-gray-800'
  }
}

const formatStepStatus = (status: string) => {
  switch (status) {
    case 'completed': return 'Completed'
    case 'in_progress': return 'In Progress'
    case 'failed': return 'Failed'
    case 'pending': return 'Pending'
    case 'not_applicable': return 'N/A'
    default: return status
  }
}
</script>

<style scoped>
/* Optional: Add any custom styles here */
</style>
