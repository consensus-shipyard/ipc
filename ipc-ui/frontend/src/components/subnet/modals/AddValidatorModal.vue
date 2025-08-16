<script setup lang="ts">
import { ValidatorService } from '@/services/subnet/validator.service'
import type { NewValidator, SubnetInstance } from '@/types/subnet'
import { computed, ref } from 'vue'

interface Props {
  modelValue: boolean
  instance: SubnetInstance | null
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void
  (e: 'validatorAdded'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const show = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const newValidator = ref<NewValidator>({
  address: '',
  pubkey: '',
  power: 1,
  collateral: 0,
  initialBalance: 0
})

const adding = ref(false)
const error = ref<string | null>(null)

const permissionMode = computed(() => {
  const mode = props.instance?.data?.config?.permissionMode || props.instance?.config?.permissionMode
  // Filter to only valid API permission modes
  if (mode === 'federated' || mode === 'collateral' || mode === 'static') {
    return mode
  }
  return 'collateral' // Default fallback
})

const close = () => {
  show.value = false
  // Reset form
  newValidator.value = {
    address: '',
    pubkey: '',
    power: 1,
    collateral: 0,
    initialBalance: 0
  }
  error.value = null
}

const addValidator = async () => {
  if (!props.instance) return

  adding.value = true
  error.value = null

  try {
    const validatorData = {
      subnetId: props.instance.data?.id || props.instance.id,
      address: newValidator.value.address,
      permissionMode: permissionMode.value,
      ...(permissionMode.value === 'collateral' ? {
        collateral: newValidator.value.collateral,
        initialBalance: newValidator.value.initialBalance || undefined
      } : {
        pubkey: newValidator.value.pubkey,
        power: newValidator.value.power
      })
    }

    const response = await ValidatorService.addValidator(validatorData)

    if (response.data.success) {
      emit('validatorAdded')
      close()
    } else {
      error.value = response.data.error || 'Failed to add validator'
    }
  } catch (err) {
    console.error('Error adding validator:', err)
    error.value = err instanceof Error ? err.message : 'Failed to add validator'
  } finally {
    adding.value = false
  }
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
    <div class="relative top-20 mx-auto p-5 border w-11/12 max-w-md shadow-lg rounded-md bg-white">
      <div class="mt-3">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-medium text-gray-900">Add New Validator</h3>
          <button
            @click="close"
            class="text-gray-400 hover:text-gray-600"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Mode-specific instructions -->
        <div class="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded-md">
          <div v-if="permissionMode === 'federated'" class="text-yellow-800 text-sm">
            <p class="font-medium mb-1">📋 Federated Mode Instructions:</p>
            <p>Enter the validator's Ethereum address, public key, and desired power level. The validator will be added to the network with the specified power.</p>
          </div>

          <div v-else-if="permissionMode === 'collateral'" class="text-yellow-800 text-sm">
            <p class="font-medium mb-1">💰 Collateral Mode Instructions:</p>
            <p>Enter the validator's address and collateral amount. The validator must have sufficient FIL to stake the specified collateral.</p>
          </div>
        </div>

        <!-- Error display -->
        <div v-if="error" class="mb-4 p-3 bg-red-50 border border-red-200 rounded-md">
          <p class="text-red-800 text-sm">{{ error }}</p>
        </div>

        <form @submit.prevent="addValidator" class="space-y-4">
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

          <div v-if="permissionMode === 'federated' || permissionMode === 'static'">
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

          <div v-if="permissionMode === 'federated' || permissionMode === 'static'">
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

          <div v-if="permissionMode === 'collateral'">
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

          <div v-if="permissionMode === 'collateral'">
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

          <div class="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              @click="close"
              class="btn-secondary"
            >
              Cancel
            </button>
            <button
              type="submit"
              :disabled="adding"
              class="btn-primary"
            >
              <div v-if="adding" class="animate-spin inline-block w-4 h-4 mr-2 border-2 border-current border-t-transparent rounded-full"></div>
              {{ adding ? 'Adding...' : 'Add Validator' }}
            </button>
          </div>
        </form>
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
