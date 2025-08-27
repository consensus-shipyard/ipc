<template>
  <div v-if="show" class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50" @click="handleBackdropClick">
    <div class="relative top-20 mx-auto p-5 border w-full max-w-lg shadow-lg rounded-md bg-white">
      <div class="mt-3">
        <!-- Header -->
        <div class="flex items-center justify-between mb-6">
          <div>
            <h3 class="text-lg font-semibold text-gray-900">Approve Subnet</h3>
            <p class="text-sm text-gray-600 mt-1">Register the subnet in the gateway</p>
          </div>
          <button @click="$emit('close')" class="text-gray-400 hover:text-gray-600">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>

        <!-- Subnet Info -->
        <div class="bg-blue-50 rounded-lg p-4 mb-6">
          <h4 class="text-sm font-medium text-blue-900 mb-2">Subnet Information</h4>
          <div class="text-sm text-blue-800 space-y-1">
            <div><span class="font-medium">Subnet ID:</span> {{ subnetId }}</div>
            <div><span class="font-medium">Action:</span> Register subnet in gateway</div>
          </div>
        </div>

        <!-- Warning/Info -->
        <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
          <div class="flex">
            <svg class="w-5 h-5 text-yellow-400 mt-0.5 mr-3" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
            </svg>
            <div>
              <h4 class="text-sm font-medium text-yellow-800">Prerequisites</h4>
              <div class="text-sm text-yellow-700 mt-1">
                <p>Before approving, ensure:</p>
                <ul class="list-disc list-inside mt-1 space-y-1">
                  <li>Subnet is bootstrapped with validators</li>
                  <li>You have the authority to approve (gateway owner/admin)</li>
                  <li>Transaction signer has sufficient gas funds</li>
                </ul>
              </div>
            </div>
          </div>
        </div>

        <!-- From Address -->
        <div class="mb-6">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            From Address (Transaction Signer)
          </label>
          <input
            v-model="fromAddress"
            type="text"
            placeholder="0x..."
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            :class="{ 'border-red-300': validationErrors.fromAddress }"
          />
          <p v-if="validationErrors.fromAddress" class="mt-1 text-sm text-red-600">
            {{ validationErrors.fromAddress }}
          </p>
          <p class="mt-1 text-sm text-gray-500">
            Address that will sign the approval transaction (must have gateway approval permissions)
          </p>
        </div>

        <!-- Advanced Options -->
        <div class="mb-6">
          <button @click="showAdvanced = !showAdvanced"
                  class="flex items-center text-sm text-gray-600 hover:text-gray-800">
            <svg class="w-4 h-4 mr-1 transition-transform"
                 :class="{ 'rotate-90': showAdvanced }"
                 fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
            </svg>
            Advanced Options
          </button>

          <div v-if="showAdvanced" class="mt-3 space-y-4 bg-gray-50 rounded-lg p-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Gas Limit (optional)
              </label>
              <input
                v-model.number="gasLimit"
                type="number"
                placeholder="Auto-estimate"
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <p class="mt-1 text-sm text-gray-500">
                Leave empty to auto-estimate gas limit
              </p>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Gas Price (gwei, optional)
              </label>
              <input
                v-model.number="gasPrice"
                type="number"
                step="0.1"
                placeholder="Auto-estimate"
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <p class="mt-1 text-sm text-gray-500">
                Leave empty to use network gas price
              </p>
            </div>
          </div>
        </div>

        <!-- Error Message -->
        <div v-if="error" class="mb-6 bg-red-50 border border-red-200 rounded-md p-4">
          <div class="flex">
            <svg class="w-5 h-5 text-red-400 mt-0.5 mr-3" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
            </svg>
            <div>
              <h4 class="text-sm font-medium text-red-800">Error</h4>
              <p class="text-sm text-red-700 mt-1">{{ error }}</p>
            </div>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex justify-end space-x-3">
          <button @click="$emit('close')"
                  class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
            Cancel
          </button>
          <button @click="handleSubmit"
                  :disabled="submitting || !isValid"
                  class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50 disabled:cursor-not-allowed">
            <svg v-if="submitting" class="w-4 h-4 mr-2 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {{ submitting ? 'Approving...' : 'Approve Subnet' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { apiService } from '@/services/api'
import { computed, ref, watch } from 'vue'

interface Props {
  show: boolean
  subnetId: string
  initialData?: any
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
  success: []
}>()

// Form state
const fromAddress = ref('')
const gasLimit = ref<number | null>(null)
const gasPrice = ref<number | null>(null)
const showAdvanced = ref(false)
const submitting = ref(false)
const error = ref<string | null>(null)
const validationErrors = ref<Record<string, string>>({})

// Initialize with any provided data
watch(() => props.initialData, (newData) => {
  if (newData) {
    // Pre-fill any available data
    console.log('Initial data for approve modal:', newData)
  }
}, { immediate: true })

// Validation
const isValid = computed(() => {
  return fromAddress.value.trim() && Object.keys(validationErrors.value).length === 0
})

const validateForm = () => {
  validationErrors.value = {}

  // Validate from address
  if (!fromAddress.value.trim()) {
    validationErrors.value.fromAddress = 'From address is required'
  } else if (!fromAddress.value.match(/^0x[a-fA-F0-9]{40}$/)) {
    validationErrors.value.fromAddress = 'Invalid Ethereum address format'
  }

  return Object.keys(validationErrors.value).length === 0
}

const handleSubmit = async () => {
  if (!validateForm()) return

  submitting.value = true
  error.value = null

  try {
    // Prepare approval data
    const approvalData: any = {
      from: fromAddress.value
    }

    // Add advanced options if provided
    if (gasLimit.value) {
      approvalData.gasLimit = gasLimit.value
    }
    if (gasPrice.value) {
      approvalData.gasPrice = gasPrice.value
    }

    // Call the API to approve subnet
    const response = await apiService.approveSubnet(props.subnetId, fromAddress.value)

    if (response.data.success) {
      emit('success')
    } else {
      error.value = response.data.error || 'Failed to approve subnet'
    }
  } catch (err: any) {
    console.error('Failed to approve subnet:', err)
    error.value = err.message || 'Failed to approve subnet'
  } finally {
    submitting.value = false
  }
}

const handleBackdropClick = (event: MouseEvent) => {
  if (event.target === event.currentTarget) {
    emit('close')
  }
}
</script>
