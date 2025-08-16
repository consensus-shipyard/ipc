<template>
  <div class="wallet-selector relative">
    <!-- Wallet selector button -->
    <button
      @click="toggleDropdown"
      class="flex items-center space-x-2 px-3 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
      :class="{ 'bg-gray-50': showDropdown }"
    >
      <!-- Wallet icon -->
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
      </svg>

      <!-- Address or placeholder -->
      <span class="max-w-32 truncate">
        <template v-if="walletStore.defaultAddressDetails">
          {{ formatAddress(walletStore.defaultAddressDetails.address) }}
        </template>
        <template v-else-if="walletStore.isLoading">
          Loading...
        </template>
        <template v-else>
          No Wallet
        </template>
      </span>

      <!-- Wallet type badge -->
      <span
        v-if="walletStore.defaultAddressDetails"
        class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium"
        :class="{
          'bg-blue-100 text-blue-700': walletStore.defaultAddressDetails.wallet_type === 'evm',
          'bg-purple-100 text-purple-700': walletStore.defaultAddressDetails.wallet_type === 'fvm'
        }"
      >
        {{ walletStore.defaultAddressDetails.wallet_type.toUpperCase() }}
      </span>

      <!-- Balance -->
      <span
        v-if="walletStore.defaultAddressDetails?.balance"
        class="text-xs text-gray-500"
      >
        {{ formatBalance(walletStore.defaultAddressDetails.balance) }}
      </span>

      <!-- Dropdown arrow -->
      <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <!-- Dropdown menu -->
    <div
      v-if="showDropdown"
      class="absolute top-full right-0 mt-1 w-80 bg-white border border-gray-300 rounded-md shadow-lg z-50 max-h-96 overflow-auto"
    >
      <!-- Header -->
      <div class="px-4 py-3 border-b border-gray-200">
        <h3 class="text-sm font-medium text-gray-900">Select Default Wallet</h3>
        <p class="text-xs text-gray-500 mt-1">
          This address will be used as the default for all owner fields
        </p>
      </div>

      <!-- Wallet type filter -->
      <div class="px-4 py-2 border-b border-gray-200">
        <div class="flex space-x-2">
          <button
            v-for="type in walletTypeOptions"
            :key="type.value"
            @click="walletTypeFilter = type.value"
            class="px-3 py-1 text-xs rounded-full font-medium transition-colors"
            :class="{
              'bg-primary-100 text-primary-700': walletTypeFilter === type.value,
              'bg-gray-100 text-gray-600 hover:bg-gray-200': walletTypeFilter !== type.value
            }"
          >
            {{ type.label }}
          </button>
        </div>
      </div>

      <!-- Loading state -->
      <div v-if="walletStore.isLoading" class="px-4 py-3 text-center">
        <div class="flex items-center justify-center space-x-2 text-sm text-gray-500">
          <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span>Loading wallet addresses...</span>
        </div>
      </div>

      <!-- Error state -->
      <div v-else-if="walletStore.error" class="px-4 py-3">
        <div class="text-sm text-red-600 mb-2">{{ walletStore.error }}</div>
        <button
          @click="refreshWallets"
          class="text-xs text-primary-600 hover:text-primary-700 font-medium"
        >
          Retry
        </button>
      </div>

      <!-- Address list -->
      <div v-else-if="filteredAddresses.length > 0" class="py-1">
        <button
          v-for="address in filteredAddresses"
          :key="address.address"
          @click="selectAddress(address)"
          class="w-full px-4 py-3 text-left hover:bg-gray-50 focus:outline-none focus:bg-gray-50 transition-colors"
          :class="{
            'bg-primary-50 border-l-4 border-primary-500': address.address === walletStore.defaultAddress
          }"
        >
          <div class="flex items-center justify-between">
            <div class="flex-1 min-w-0">
              <!-- Address with custom label -->
              <div class="flex items-center space-x-2 mb-1">
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

                <!-- Current default indicator -->
                <span
                  v-if="address.address === walletStore.defaultAddress"
                  class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800"
                >
                  Current
                </span>
              </div>

              <!-- Full address -->
              <div class="text-xs text-gray-500 font-mono mb-1">
                {{ address.address }}
              </div>

              <!-- Balance and pubkey -->
              <div class="flex items-center space-x-3 text-xs text-gray-400">
                <span v-if="address.balance">
                  Balance: {{ formatBalance(address.balance) }}
                </span>
                <span v-if="address.pubkey">
                  PubKey: {{ formatAddress(address.pubkey) }}
                </span>
              </div>
            </div>

            <!-- Check icon for current default -->
            <svg
              v-if="address.address === walletStore.defaultAddress"
              class="w-5 h-5 text-primary-600"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
            </svg>
          </div>
        </button>
      </div>

      <!-- Empty state -->
      <div v-else class="px-4 py-6 text-center">
        <svg class="w-8 h-8 text-gray-400 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
        </svg>
        <p class="text-sm text-gray-500 mb-2">No wallet addresses found</p>
        <p class="text-xs text-gray-400">
          Configure wallets using the IPC CLI to get started
        </p>
      </div>

      <!-- Footer -->
      <div class="px-4 py-3 border-t border-gray-200 bg-gray-50">
        <div class="flex items-center justify-between">
          <span class="text-xs text-gray-500">
            {{ filteredAddresses.length }} address{{ filteredAddresses.length !== 1 ? 'es' : '' }} available
          </span>
          <button
            @click="refreshWallets"
            class="text-xs text-primary-600 hover:text-primary-700 font-medium"
          >
            Refresh
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useWalletStore, type WalletAddress } from '../../stores/wallet'

const walletStore = useWalletStore()
const showDropdown = ref(false)
const walletTypeFilter = ref<'all' | 'evm' | 'fvm'>('all')

// Wallet type filter options
const walletTypeOptions = [
  { value: 'all' as const, label: 'All Wallets' },
  { value: 'evm' as const, label: 'EVM' },
  { value: 'fvm' as const, label: 'FVM' }
]

// Computed
const filteredAddresses = computed(() => {
  let addresses = walletStore.addresses

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

const formatBalance = (balance: string): string => {
  try {
    const num = parseFloat(balance)
    if (num === 0) return '0'
    if (num < 0.001) return '<0.001'
    if (num < 1) return num.toFixed(3)
    if (num < 1000) return num.toFixed(2)
    if (num < 1000000) return `${(num / 1000).toFixed(1)}K`
    return `${(num / 1000000).toFixed(1)}M`
  } catch {
    return balance
  }
}

const toggleDropdown = () => {
  showDropdown.value = !showDropdown.value

  // Load addresses when opening dropdown if not already loaded
  if (showDropdown.value && walletStore.addresses.length === 0 && !walletStore.isLoading) {
    refreshWallets()
  }
}

const selectAddress = (address: WalletAddress) => {
  walletStore.saveDefaultAddress(address.address)
  showDropdown.value = false
}

const refreshWallets = async () => {
  await walletStore.fetchAddresses(undefined, true)
}

// Handle clicks outside to close dropdown
const handleClickOutside = (event: Event) => {
  const target = event.target as Element
  if (!target.closest('.wallet-selector')) {
    showDropdown.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  // Load wallet addresses on mount
  walletStore.fetchAddresses()
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.wallet-selector {
  @apply relative;
}
</style>