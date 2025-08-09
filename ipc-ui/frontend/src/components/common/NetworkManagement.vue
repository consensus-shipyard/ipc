<template>
  <div class="fixed inset-0 z-50 overflow-y-auto">
    <!-- Backdrop -->
    <div class="fixed inset-0 bg-black bg-opacity-50 transition-opacity" @click="$emit('close')"></div>

    <!-- Modal -->
    <div class="relative min-h-screen flex items-center justify-center p-4">
      <div class="relative bg-white rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden">
        <!-- Header -->
        <div class="flex items-center justify-between p-6 border-b border-gray-200">
          <h2 class="text-xl font-semibold text-gray-900">Manage Networks</h2>
          <button
            @click="$emit('close')"
            class="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Content -->
        <div class="flex h-[600px]">
          <!-- Network List -->
          <div class="w-1/2 border-r border-gray-200 overflow-y-auto">
            <div class="p-6">
              <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-medium text-gray-900">Networks</h3>
                <button
                  @click="startAddNetwork"
                  class="btn-primary text-sm"
                >
                  <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                  </svg>
                  Add Network
                </button>
              </div>

              <div class="space-y-2">
                <div
                  v-for="network in networkStore.networks"
                  :key="network.id"
                  class="p-3 border border-gray-200 rounded-lg hover:border-gray-300 transition-colors cursor-pointer"
                  :class="{ 'border-blue-500 bg-blue-50': selectedNetworkForEdit?.id === network.id }"
                  @click="selectNetworkForEdit(network)"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-3">
                      <!-- Status indicator -->
                      <div
                        class="w-2 h-2 rounded-full"
                        :class="{
                          'bg-green-500': network.type === 'mainnet',
                          'bg-yellow-500': network.type === 'testnet',
                          'bg-blue-500': network.type === 'local',
                          'bg-gray-500': network.type === 'custom'
                        }"
                      ></div>

                      <div>
                        <div class="font-medium text-gray-900">{{ network.name }}</div>
                        <div class="text-sm text-gray-500 truncate max-w-48">{{ network.rpcUrl }}</div>
                      </div>
                    </div>

                    <div class="flex items-center space-x-2">
                      <!-- Current network indicator -->
                      <span
                        v-if="network.id === networkStore.selectedNetworkId"
                        class="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-blue-100 text-blue-700"
                      >
                        Current
                      </span>

                      <!-- Default network indicator -->
                      <span
                        v-if="network.isDefault"
                        class="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-gray-100 text-gray-700"
                      >
                        Default
                      </span>

                      <!-- Delete button for custom networks -->
                      <button
                        v-if="!network.isDefault"
                        @click.stop="confirmDeleteNetwork(network)"
                        class="text-red-400 hover:text-red-600 transition-colors"
                        title="Delete Network"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Edit Form -->
          <div class="w-1/2 overflow-y-auto">
            <div class="p-6">
              <h3 class="text-lg font-medium text-gray-900 mb-4">
                {{ isAddMode ? 'Add New Network' : selectedNetworkForEdit ? 'Edit Network' : 'Select a network to edit' }}
              </h3>

              <form v-if="isAddMode || selectedNetworkForEdit" @submit.prevent="saveNetwork" class="space-y-4">
                <!-- Network Name -->
                <div>
                  <label for="name" class="block text-sm font-medium text-gray-700 mb-1">
                    Network Name *
                  </label>
                  <input
                    id="name"
                    v-model="formData.name"
                    type="text"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                    :class="{ 'border-red-300': errors.name }"
                    placeholder="Enter network name"
                    required
                  />
                  <p v-if="errors.name" class="mt-1 text-sm text-red-600">{{ errors.name }}</p>
                </div>

                <!-- RPC URL -->
                <div>
                  <label for="rpcUrl" class="block text-sm font-medium text-gray-700 mb-1">
                    RPC URL *
                  </label>
                  <input
                    id="rpcUrl"
                    v-model="formData.rpcUrl"
                    type="url"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                    :class="{ 'border-red-300': errors.rpcUrl }"
                    placeholder="https://example.com/rpc"
                    required
                    :disabled="selectedNetworkForEdit?.isDefault && !isAddMode"
                  />
                  <p v-if="errors.rpcUrl" class="mt-1 text-sm text-red-600">{{ errors.rpcUrl }}</p>
                </div>

                <!-- WebSocket URL -->
                <div>
                  <label for="wsUrl" class="block text-sm font-medium text-gray-700 mb-1">
                    WebSocket URL (Optional)
                  </label>
                  <input
                    id="wsUrl"
                    v-model="formData.wsUrl"
                    type="url"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                    :class="{ 'border-red-300': errors.wsUrl }"
                    placeholder="wss://example.com/ws"
                    :disabled="selectedNetworkForEdit?.isDefault && !isAddMode"
                  />
                  <p v-if="errors.wsUrl" class="mt-1 text-sm text-red-600">{{ errors.wsUrl }}</p>
                </div>

                <!-- Chain ID -->
                <div>
                  <label for="chainId" class="block text-sm font-medium text-gray-700 mb-1">
                    Chain ID (Optional)
                  </label>
                  <input
                    id="chainId"
                    v-model.number="formData.chainId"
                    type="number"
                    min="1"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                    :class="{ 'border-red-300': errors.chainId }"
                    placeholder="1"
                    :disabled="selectedNetworkForEdit?.isDefault && !isAddMode"
                  />
                  <p v-if="errors.chainId" class="mt-1 text-sm text-red-600">{{ errors.chainId }}</p>
                </div>

                <!-- Network Type -->
                <div>
                  <label for="type" class="block text-sm font-medium text-gray-700 mb-1">
                    Network Type *
                  </label>
                  <select
                    id="type"
                    v-model="formData.type"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                    required
                    :disabled="selectedNetworkForEdit?.isDefault && !isAddMode"
                  >
                    <option value="mainnet">Mainnet</option>
                    <option value="testnet">Testnet</option>
                    <option value="local">Local</option>
                    <option value="custom">Custom</option>
                  </select>
                </div>

                <!-- Default network notice -->
                <div v-if="selectedNetworkForEdit?.isDefault && !isAddMode" class="p-3 bg-yellow-50 border border-yellow-200 rounded-md">
                  <div class="flex">
                    <svg class="w-5 h-5 text-yellow-400 mr-2" fill="currentColor" viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                    </svg>
                    <p class="text-sm text-yellow-700">
                      This is a default network. Only the name can be modified.
                    </p>
                  </div>
                </div>

                <!-- Action buttons -->
                <div class="flex space-x-3 pt-4">
                  <button
                    type="submit"
                    class="btn-primary"
                    :disabled="saving"
                  >
                    {{ saving ? 'Saving...' : (isAddMode ? 'Add Network' : 'Save Changes') }}
                  </button>

                  <button
                    type="button"
                    @click="cancelEdit"
                    class="btn-secondary"
                  >
                    Cancel
                  </button>
                </div>
              </form>

              <!-- Placeholder when no network selected -->
              <div v-else class="text-center py-12">
                <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0" />
                </svg>
                <h3 class="mt-2 text-sm font-medium text-gray-900">No network selected</h3>
                <p class="mt-1 text-sm text-gray-500">Select a network from the list to edit its settings.</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="border-t border-gray-200 px-6 py-4 bg-gray-50">
          <div class="flex justify-between items-center">
            <button
              @click="resetToDefaults"
              class="text-sm text-gray-600 hover:text-gray-800 transition-colors"
            >
              Reset to Defaults
            </button>

            <button
              @click="$emit('close')"
              class="btn-secondary"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div v-if="networkToDelete" class="fixed inset-0 z-60 overflow-y-auto">
      <div class="fixed inset-0 bg-black bg-opacity-50 transition-opacity"></div>
      <div class="relative min-h-screen flex items-center justify-center p-4">
        <div class="relative bg-white rounded-lg shadow-xl max-w-md w-full">
          <div class="p-6">
            <div class="flex items-center mb-4">
              <svg class="w-6 h-6 text-red-600 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
              <h3 class="text-lg font-medium text-gray-900">Delete Network</h3>
            </div>

            <p class="text-sm text-gray-500 mb-6">
              Are you sure you want to delete "{{ networkToDelete.name }}"? This action cannot be undone.
            </p>

            <div class="flex space-x-3">
              <button
                @click="deleteNetwork"
                class="btn-danger flex-1"
                :disabled="deleting"
              >
                {{ deleting ? 'Deleting...' : 'Delete' }}
              </button>

              <button
                @click="networkToDelete = null"
                class="btn-secondary flex-1"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useNetworkStore, type Network } from '@/stores/network';
import { reactive, ref } from 'vue';

// Emits
defineEmits<{
  close: []
}>()

const networkStore = useNetworkStore()

// Component state
const selectedNetworkForEdit = ref<Network | null>(null)
const isAddMode = ref(false)
const saving = ref(false)
const deleting = ref(false)
const networkToDelete = ref<Network | null>(null)

// Form data
const formData = reactive({
  name: '',
  rpcUrl: '',
  wsUrl: '',
  chainId: undefined as number | undefined,
  type: 'custom' as Network['type']
})

// Form errors
const errors = reactive({
  name: '',
  rpcUrl: '',
  wsUrl: '',
  chainId: ''
})

// Actions
const selectNetworkForEdit = (network: Network) => {
  selectedNetworkForEdit.value = network
  isAddMode.value = false
  populateForm(network)
}

const startAddNetwork = () => {
  isAddMode.value = true
  selectedNetworkForEdit.value = null
  resetForm()
}

const populateForm = (network: Network) => {
  formData.name = network.name
  formData.rpcUrl = network.rpcUrl
  formData.wsUrl = network.wsUrl || ''
  formData.chainId = network.chainId
  formData.type = network.type
  clearErrors()
}

const resetForm = () => {
  formData.name = ''
  formData.rpcUrl = ''
  formData.wsUrl = ''
  formData.chainId = undefined
  formData.type = 'custom'
  clearErrors()
}

const clearErrors = () => {
  errors.name = ''
  errors.rpcUrl = ''
  errors.wsUrl = ''
  errors.chainId = ''
}

const validateForm = (): boolean => {
  clearErrors()

  const validationErrors = networkStore.validateNetwork({
    name: formData.name,
    rpcUrl: formData.rpcUrl,
    wsUrl: formData.wsUrl || undefined,
    chainId: formData.chainId,
    type: formData.type
  })

  // Map validation errors to form errors
  validationErrors.forEach(error => {
    if (error.includes('name')) errors.name = error
    else if (error.includes('RPC')) errors.rpcUrl = error
    else if (error.includes('WebSocket')) errors.wsUrl = error
    else if (error.includes('Chain ID')) errors.chainId = error
  })

  // Check name uniqueness
  const isNameUnique = networkStore.isNetworkNameUnique(
    formData.name,
    isAddMode.value ? undefined : selectedNetworkForEdit.value?.id
  )

  if (!isNameUnique) {
    errors.name = 'Network name must be unique'
  }

  return validationErrors.length === 0 && isNameUnique
}

const saveNetwork = async () => {
  if (!validateForm()) {
    return
  }

  saving.value = true

  try {
    if (isAddMode.value) {
      networkStore.addNetwork({
        name: formData.name,
        rpcUrl: formData.rpcUrl,
        wsUrl: formData.wsUrl || undefined,
        chainId: formData.chainId,
        type: formData.type
      })
    } else if (selectedNetworkForEdit.value) {
      networkStore.updateNetwork(selectedNetworkForEdit.value.id, {
        name: formData.name,
        rpcUrl: formData.rpcUrl,
        wsUrl: formData.wsUrl || undefined,
        chainId: formData.chainId,
        type: formData.type
      })
    }

    cancelEdit()
  } catch (error) {
    console.error('Failed to save network:', error)
    // Handle error (could show a toast notification)
  } finally {
    saving.value = false
  }
}

const cancelEdit = () => {
  selectedNetworkForEdit.value = null
  isAddMode.value = false
  resetForm()
}

const confirmDeleteNetwork = (network: Network) => {
  networkToDelete.value = network
}

const deleteNetwork = async () => {
  if (!networkToDelete.value) return

  deleting.value = true

  try {
    const success = networkStore.removeNetwork(networkToDelete.value.id)

    if (success) {
      // If we were editing this network, cancel the edit
      if (selectedNetworkForEdit.value?.id === networkToDelete.value.id) {
        cancelEdit()
      }
    }

    networkToDelete.value = null
  } catch (error) {
    console.error('Failed to delete network:', error)
  } finally {
    deleting.value = false
  }
}

const resetToDefaults = () => {
  if (confirm('This will reset all networks to defaults and remove any custom networks. Continue?')) {
    networkStore.resetToDefaults()
    cancelEdit()
  }
}
</script>