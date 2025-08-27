<template>
  <div class="address-selector">
    <label v-if="label" :for="inputId" class="block text-sm font-medium text-gray-700 mb-1">
      {{ label }}
      <span v-if="required" class="text-red-500">*</span>
    </label>

    <div class="relative">
      <!-- Input field with dropdown toggle -->
      <div class="relative">
        <input
          :id="inputId"
          v-model="inputValue"
          type="text"
          :placeholder="placeholder || 'Enter address or select from wallet'"
          :required="required"
          :disabled="disabled"
          class="block w-full px-3 py-2 pr-10 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm"
          :class="{
            'border-red-300 text-red-900 placeholder-red-300 focus:ring-red-500 focus:border-red-500': error,
            'bg-gray-50 text-gray-500': disabled
          }"
          @input="handleInput"
          @focus="handleFocus"
          @blur="handleBlur"
        />

        <!-- Dropdown toggle button -->
        <button
          type="button"
          :disabled="disabled || walletStore.isLoading"
          class="absolute inset-y-0 right-0 flex items-center px-2 text-gray-400 hover:text-gray-600 focus:outline-none"
          @click="toggleDropdown"
        >
          <svg v-if="walletStore.isLoading" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
        </button>
      </div>

      <!-- Dropdown menu -->
      <div
        v-if="showDropdown"
        class="absolute z-50 mt-1 w-full bg-white border border-gray-300 rounded-md shadow-lg max-h-60 overflow-auto"
      >
        <!-- Loading state -->
        <div v-if="walletStore.isLoading" class="px-3 py-2 text-sm text-gray-500">
          Loading wallet addresses...
        </div>

        <!-- Error state -->
        <div v-else-if="walletStore.error" class="px-3 py-2 text-sm text-red-600">
          {{ walletStore.error }}
          <button @click="refreshAddresses" class="ml-2 text-primary-600 hover:text-primary-700">
            Retry
          </button>
        </div>

        <!-- Address list -->
        <div v-else-if="filteredAddresses.length > 0">
          <!-- Wallet type filter -->
          <div v-if="showWalletTypeFilter" class="border-b border-gray-200 px-3 py-2">
            <div class="flex space-x-2">
              <button
                v-for="type in ['all', 'evm', 'fvm']"
                :key="type"
                @click="walletTypeFilter = type as any"
                class="px-2 py-1 text-xs rounded"
                :class="{
                  'bg-primary-100 text-primary-700': walletTypeFilter === type,
                  'bg-gray-100 text-gray-600 hover:bg-gray-200': walletTypeFilter !== type
                }"
              >
                {{ type.toUpperCase() }}
              </button>
            </div>
          </div>

          <div class="py-1">
            <button
              v-for="address in filteredAddresses"
              :key="address.address"
              @click="selectAddress(address)"
              class="w-full px-3 py-2 text-left hover:bg-gray-50 focus:outline-none focus:bg-gray-50"
              :class="{
                'opacity-50': address.is_compatible === false,
                'border-l-4 border-primary-500': address.address === walletStore.defaultAddress
              }"
            >
              <div class="flex items-center justify-between">
                <div class="flex-1 min-w-0">
                  <!-- Custom label or formatted address -->
                  <div class="flex items-center space-x-2">
                    <span class="text-sm font-medium text-gray-900 truncate">
                      {{ address.custom_label || formatAddress(address.address) }}
                    </span>

                    <!-- Wallet type badge -->
                    <span
                      class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium"
                      :class="{
                        'bg-blue-100 text-blue-800': address.wallet_type === 'evm',
                        'bg-purple-100 text-purple-800': address.wallet_type === 'fvm'
                      }"
                    >
                      {{ address.wallet_type.toUpperCase() }}
                    </span>

                    <!-- Default indicator -->
                    <span
                      v-if="address.address === walletStore.defaultAddress"
                      class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800"
                    >
                      Default
                    </span>

                    <!-- Incompatible indicator -->
                    <span
                      v-if="address.is_compatible === false"
                      class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600"
                    >
                      Incompatible
                    </span>
                  </div>

                  <!-- Full address -->
                  <div class="text-xs text-gray-500 font-mono">
                    {{ address.address }}
                  </div>

                  <!-- Balance and pubkey info -->
                  <div v-if="address.balance || (address.pubkey && showPubkey)" class="text-xs text-gray-400 mt-1">
                    <span v-if="address.balance">Balance: {{ address.balance }}</span>
                    <span v-if="address.balance && address.pubkey && showPubkey"> • </span>
                    <span v-if="address.pubkey && showPubkey" class="break-all">PubKey: {{ formatAddress(address.pubkey) }}</span>
                  </div>
                </div>
              </div>
            </button>
          </div>
        </div>

        <!-- Empty state -->
        <div v-else class="px-3 py-2 text-sm text-gray-500">
          No wallet addresses found. Make sure you have configured wallets using the CLI.
        </div>

        <!-- Manual entry option -->
        <div class="border-t border-gray-200 px-3 py-2">
          <button
            @click="clearSelection"
            class="text-sm text-primary-600 hover:text-primary-700"
          >
            Clear and enter manually
          </button>
        </div>
      </div>
    </div>

    <!-- Help text -->
    <p v-if="helpText" class="mt-2 text-sm text-gray-600">
      {{ helpText }}
    </p>

    <!-- Error message -->
    <p v-if="error" class="mt-2 text-sm text-red-600">
      {{ error }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useWalletStore, type WalletAddress } from '../../stores/wallet'

interface Props {
  modelValue?: string
  label?: string
  placeholder?: string
  required?: boolean
  disabled?: boolean
  error?: string
  helpText?: string
  fieldName?: string // For per-field default storage
  networkType?: 'evm' | 'fvm' | 'both' // For compatibility filtering
  showPubkey?: boolean // Whether to show pubkey info (for validator addresses)
  showWalletTypeFilter?: boolean // Whether to show wallet type filter buttons
}

const props = withDefaults(defineProps<Props>(), {
  networkType: 'both',
  showPubkey: false,
  showWalletTypeFilter: true
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'address-selected': [address: WalletAddress]
}>()

const walletStore = useWalletStore()
const inputValue = ref('')
const showDropdown = ref(false)
const walletTypeFilter = ref<'all' | 'evm' | 'fvm'>('all')
const inputId = `address-input-${Math.random().toString(36).substr(2, 9)}`

// Computed
const filteredAddresses = computed(() => {
  let addresses = walletStore.compatibleAddresses

  // Apply wallet type filter
  if (walletTypeFilter.value !== 'all') {
    addresses = addresses.filter(addr => addr.wallet_type === walletTypeFilter.value)
  }

  return addresses
})

// Methods
const formatAddress = (address: string): string => {
  if (!address) return ''
  if (address.length <= 12) return address
  return `${address.slice(0, 6)}...${address.slice(-6)}`
}

const handleInput = (event: Event) => {
  const target = event.target as HTMLInputElement
  inputValue.value = target.value
  emit('update:modelValue', target.value)
}

const handleFocus = () => {
  showDropdown.value = true

  // Load addresses if not already loaded
  if (walletStore.addresses.length === 0 && !walletStore.isLoading) {
    refreshAddresses()
  }
}

const handleBlur = (event: FocusEvent) => {
  // Delay hiding dropdown to allow for clicks
  setTimeout(() => {
    if (!event.relatedTarget || !(event.relatedTarget as Element).closest('.address-selector')) {
      showDropdown.value = false
    }
  }, 150)
}

const toggleDropdown = () => {
  showDropdown.value = !showDropdown.value

  if (showDropdown.value && walletStore.addresses.length === 0 && !walletStore.isLoading) {
    refreshAddresses()
  }
}

const selectAddress = (address: WalletAddress) => {
  inputValue.value = address.address
  emit('update:modelValue', address.address)
  emit('address-selected', address)
  showDropdown.value = false

  // Save as field default if fieldName provided
  if (props.fieldName) {
    walletStore.saveFieldDefault(props.fieldName, address.address)
  }
}

const clearSelection = () => {
  inputValue.value = ''
  emit('update:modelValue', '')
  showDropdown.value = false
}

const refreshAddresses = async () => {
  await walletStore.fetchAddresses(undefined, true)
}

// Initialize component
onMounted(() => {
  // Set network compatibility
  if (props.networkType !== 'both') {
    walletStore.updateNetworkCompatibility(props.networkType)
  }

  // Set initial value from modelValue
  if (props.modelValue) {
    inputValue.value = props.modelValue
  } else if (props.fieldName) {
    // Try to load field default
    const fieldDefault = walletStore.getFieldDefault(props.fieldName)
    if (fieldDefault) {
      inputValue.value = fieldDefault
      emit('update:modelValue', fieldDefault)
    }
  }

  // Load addresses
  walletStore.fetchAddresses()
})

// Watch for modelValue changes from parent
watch(
  () => props.modelValue,
  (newValue) => {
    if (newValue !== inputValue.value) {
      inputValue.value = newValue || ''
    }
  }
)

// Handle clicks outside to close dropdown
const handleClickOutside = (event: Event) => {
  const target = event.target as Element
  if (!target.closest('.address-selector')) {
    showDropdown.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.address-selector {
  @apply relative;
}
</style>