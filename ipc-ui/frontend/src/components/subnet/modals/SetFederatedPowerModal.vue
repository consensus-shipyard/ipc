<template>
  <div v-if="show" class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50" @click="handleBackdropClick">
    <div class="relative top-20 mx-auto p-5 border w-full max-w-2xl shadow-lg rounded-md bg-white">
      <div class="mt-3">
        <!-- Header -->
        <div class="flex items-center justify-between mb-6">
          <div>
            <h3 class="text-lg font-semibold text-gray-900">Set Federated Power</h3>
            <p class="text-sm text-gray-600 mt-1">Configure validators and their power for this federated subnet</p>
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
          <div class="text-sm text-blue-800">
            <div><span class="font-medium">Subnet ID:</span> {{ subnetId }}</div>
            <div class="mt-1"><span class="font-medium">Mode:</span> Federated</div>
          </div>
        </div>

        <!-- Validators Configuration -->
        <div class="space-y-6">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">
              Validators Configuration
            </label>

            <div class="space-y-4">
              <div v-for="(validator, index) in validators" :key="index"
                   class="border border-gray-200 rounded-lg p-4">
                <div class="flex items-center justify-between mb-4">
                  <h5 class="text-sm font-medium text-gray-900">Validator {{ index + 1 }}</h5>
                  <button v-if="validators.length > 1"
                          @click="removeValidator(index)"
                          class="text-red-600 hover:text-red-800">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                    </svg>
                  </button>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                      Validator Address
                    </label>
                    <input
                      v-model="validator.address"
                      type="text"
                      placeholder="0x..."
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      :class="{ 'border-red-300': validationErrors[`validator_${index}_address`] }"
                    />
                    <p v-if="validationErrors[`validator_${index}_address`]"
                       class="mt-1 text-sm text-red-600">
                      {{ validationErrors[`validator_${index}_address`] }}
                    </p>
                  </div>

                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                      Power (voting weight)
                    </label>
                    <input
                      v-model.number="validator.power"
                      type="number"
                      min="1"
                      placeholder="1"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      :class="{ 'border-red-300': validationErrors[`validator_${index}_power`] }"
                    />
                    <p v-if="validationErrors[`validator_${index}_power`]"
                       class="mt-1 text-sm text-red-600">
                      {{ validationErrors[`validator_${index}_power`] }}
                    </p>
                  </div>
                </div>

                <div class="mt-4">
                  <label class="block text-sm font-medium text-gray-700 mb-1">
                    Public Key (65 bytes, hex format)
                  </label>
                  <input
                    v-model="validator.publicKey"
                    type="text"
                    placeholder="04..."
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
                    :class="{ 'border-red-300': validationErrors[`validator_${index}_publicKey`] }"
                  />
                  <p v-if="validationErrors[`validator_${index}_publicKey`]"
                     class="mt-1 text-sm text-red-600">
                    {{ validationErrors[`validator_${index}_publicKey`] }}
                  </p>
                  <p class="mt-1 text-xs text-gray-500">
                    65-byte secp256k1 public key in hex format (130 characters starting with '04')
                  </p>
                </div>
              </div>
            </div>

            <button @click="addValidator"
                    class="mt-4 inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
              <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"/>
              </svg>
              Add Validator
            </button>
          </div>

          <!-- From Address -->
          <div>
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
              Address that will sign the transaction (must be the subnet owner)
            </p>
          </div>
        </div>

        <!-- Error Message -->
        <div v-if="error" class="mt-4 bg-red-50 border border-red-200 rounded-md p-4">
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
        <div class="mt-8 flex justify-end space-x-3">
          <button @click="$emit('close')"
                  class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
            Cancel
          </button>
          <button @click="handleSubmit"
                  :disabled="submitting || !isValid"
                  class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed">
            <svg v-if="submitting" class="w-4 h-4 mr-2 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {{ submitting ? 'Setting Power...' : 'Set Federated Power' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { apiService } from '@/services/api'
import { computed, ref, watch } from 'vue'

interface Validator {
  address: string
  publicKey: string
  power: number
}

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
const validators = ref<Validator[]>([
  { address: '', publicKey: '', power: 1 }
])
const fromAddress = ref('')
const submitting = ref(false)
const error = ref<string | null>(null)
const validationErrors = ref<Record<string, string>>({})

// Initialize with any provided data
watch(() => props.initialData, (newData) => {
  if (newData) {
    // Pre-fill any available data from the initial data
    if (newData.min_validators && validators.value.length < newData.min_validators) {
      // Add more validators if needed
      while (validators.value.length < newData.min_validators) {
        validators.value.push({ address: '', publicKey: '', power: 1 })
      }
    }
  }
}, { immediate: true })

// Validation
const isValid = computed(() => {
  return validators.value.every(v =>
    v.address.trim() &&
    v.publicKey.trim() &&
    v.power > 0
  ) && fromAddress.value.trim() && Object.keys(validationErrors.value).length === 0
})

const addValidator = () => {
  validators.value.push({ address: '', publicKey: '', power: 1 })
}

const removeValidator = (index: number) => {
  if (validators.value.length > 1) {
    validators.value.splice(index, 1)
  }
}

const validateForm = () => {
  validationErrors.value = {}

  validators.value.forEach((validator, index) => {
    // Validate address
    if (!validator.address.trim()) {
      validationErrors.value[`validator_${index}_address`] = 'Address is required'
    } else if (!validator.address.match(/^0x[a-fA-F0-9]{40}$/)) {
      validationErrors.value[`validator_${index}_address`] = 'Invalid Ethereum address format'
    }

    // Validate power
    if (!validator.power || validator.power <= 0) {
      validationErrors.value[`validator_${index}_power`] = 'Power must be greater than 0'
    }

    // Validate public key
    if (!validator.publicKey.trim()) {
      validationErrors.value[`validator_${index}_publicKey`] = 'Public key is required'
    } else if (!validator.publicKey.match(/^04[a-fA-F0-9]{128}$/)) {
      validationErrors.value[`validator_${index}_publicKey`] = 'Invalid public key format (must be 65 bytes in hex starting with 04)'
    }
  })

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
    // Call the API to set federated power
    const response = await apiService.setFederatedPower(
      props.subnetId,
      validators.value.map(v => v.address),
      validators.value.map(v => v.publicKey),
      validators.value.map(v => v.power),
      fromAddress.value
    )

    if (response.data.success) {
      emit('success')
    } else {
      error.value = response.data.error || 'Failed to set federated power'
    }
  } catch (err: any) {
    console.error('Failed to set federated power:', err)
    error.value = err.message || 'Failed to set federated power'
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
