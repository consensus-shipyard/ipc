<template>
  <div class="gateway-selector relative">
    <!-- Gateway selector button -->
    <button
      @click="toggleDropdown"
      class="flex items-center space-x-2 px-3 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500 transition-colors"
      :class="{ 'bg-gray-50': showDropdown }"
      :disabled="gatewayStore.isLoading"
    >
      <!-- Gateway icon -->
      <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M19 11H5m14-7l2 2m0 0l2 2m-2-2v6m-2-2H5m14-7v2a2 2 0 01-2 2H5a2 2 0 01-2-2V4"/>
      </svg>

      <!-- Gateway name or placeholder -->
      <span class="max-w-32 truncate">
        <span v-if="gatewayStore.selectedGateway">
          {{ gatewayStore.selectedGateway.name }}
        </span>
        <span v-else-if="gatewayStore.isLoading" class="animate-pulse">
          Loading...
        </span>
        <span v-else class="text-gray-400">
          No Gateway
        </span>
      </span>

      <!-- Loading indicator or dropdown arrow -->
      <div v-if="gatewayStore.isLoading" class="animate-spin">
        <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
        </svg>
      </div>
      <svg v-else
        class="w-4 h-4 text-gray-400 transition-transform"
        :class="{ 'rotate-180': showDropdown }"
        fill="none" stroke="currentColor" viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
      </svg>
    </button>

    <!-- Dropdown menu -->
    <div
      v-if="showDropdown"
      class="absolute right-0 mt-2 w-80 bg-white rounded-md shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none z-50"
      @click.stop
    >
      <div class="py-1">
        <!-- Header -->
        <div class="px-4 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wide border-b border-gray-100 flex items-center justify-between">
          <span>L1 Gateways</span>
          <button
            @click="refreshGateways"
            :disabled="gatewayStore.isLoading"
            class="text-xs text-primary-600 hover:text-primary-700 font-medium disabled:opacity-50"
            :class="{ 'animate-pulse': gatewayStore.isLoading }"
          >
            Refresh
          </button>
        </div>

        <!-- No gateways message -->
        <div v-if="gatewayStore.availableGateways.length === 0 && !gatewayStore.isLoading"
             class="px-4 py-6 text-center">
          <svg class="w-8 h-8 mx-auto mb-2 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M19 11H5m14-7l2 2m0 0l2 2m-2-2v6m-2-2H5m14-7v2a2 2 0 01-2 2H5a2 2 0 01-2-2V4"/>
          </svg>
          <p class="text-sm text-gray-500 mb-2">No L1 gateways found</p>
          <p class="text-xs text-gray-400">Deploy a gateway to get started</p>
        </div>

        <!-- Gateway list -->
        <div v-else-if="!gatewayStore.isLoading" class="max-h-60 overflow-y-auto">
          <button
            v-for="gateway in gatewayStore.availableGateways"
            :key="gateway.id"
            @click="selectGateway(gateway.id)"
            class="w-full text-left px-4 py-3 hover:bg-gray-50 focus:outline-none focus:bg-gray-50 transition-colors"
            :class="{ 'bg-blue-50 border-l-4 border-l-blue-500': gateway.id === gatewayStore.selectedGatewayId }"
          >
            <div class="flex items-center justify-between">
              <div class="flex-1 min-w-0">
                <div class="flex items-center space-x-2">
                  <p class="text-sm font-medium text-gray-900 truncate">
                    {{ gateway.name }}
                  </p>
                  <span v-if="gateway.is_default"
                        class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-green-100 text-green-700">
                    Default
                  </span>
                </div>
                <p class="text-xs text-gray-500 font-mono mt-1">
                  {{ formatAddress(gateway.address) }}
                </p>
                <p class="text-xs text-gray-400 mt-1">
                  {{ gateway.network_name }} • {{ gateway.deployer_address.slice(0, 8) }}...
                </p>
              </div>
              <div v-if="gateway.id === gatewayStore.selectedGatewayId"
                   class="flex-shrink-0 ml-2">
                <svg class="w-4 h-4 text-blue-600" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                </svg>
              </div>
            </div>
          </button>
        </div>

        <!-- Loading state -->
        <div v-if="gatewayStore.isLoading" class="px-4 py-6">
          <div class="animate-pulse flex space-x-4">
            <div class="rounded-full bg-gray-300 h-8 w-8"></div>
            <div class="flex-1 space-y-2 py-1">
              <div class="h-4 bg-gray-300 rounded w-3/4"></div>
              <div class="h-3 bg-gray-300 rounded w-1/2"></div>
            </div>
          </div>
        </div>

        <!-- Actions -->
        <div class="border-t border-gray-100 py-1">
          <button
            @click="openGatewayManagement"
            class="w-full text-left px-4 py-3 text-sm text-gray-700 hover:bg-gray-50 focus:outline-none focus:bg-gray-50 transition-colors flex items-center space-x-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
            </svg>
            <span>Manage L1 Gateways</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Error message -->
    <div v-if="gatewayStore.error"
         class="absolute right-0 mt-2 w-80 bg-red-50 border border-red-200 rounded-md p-3 z-50">
      <div class="flex">
        <svg class="w-5 h-5 text-red-400 mr-2 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
        </svg>
        <div>
          <p class="text-sm text-red-800 font-medium">Gateway Error</p>
          <p class="text-sm text-red-700 mt-1">{{ gatewayStore.error }}</p>
        </div>
      </div>
    </div>

    <!-- Gateway Management Modal placeholder -->
    <!-- TODO: Implement GatewayManagement modal -->
  </div>
</template>

<script setup lang="ts">
import { useL1GatewaysStore } from '@/stores/l1-gateways'
import { onMounted, onUnmounted, ref } from 'vue'

const gatewayStore = useL1GatewaysStore()
const showDropdown = ref(false)

const toggleDropdown = () => {
  showDropdown.value = !showDropdown.value
}

const selectGateway = async (gatewayId: string) => {
  try {
    await gatewayStore.selectGateway(gatewayId)
    showDropdown.value = false
  } catch (error) {
    console.error('Failed to select gateway:', error)
    // Error will be shown via gatewayStore.error
  }
}

const refreshGateways = async () => {
  await gatewayStore.loadL1Gateways()
}

const openGatewayManagement = () => {
  showDropdown.value = false
  // TODO: Open gateway management modal
  console.log('Open gateway management modal')
}

const formatAddress = (address: string) => {
  if (!address) return ''
  return `${address.slice(0, 8)}...${address.slice(-6)}`
}

// Close dropdown when clicking outside
const handleClickOutside = (event: Event) => {
  const target = event.target as Element
  if (!target.closest('.gateway-selector')) {
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
.gateway-selector {
  min-width: 200px;
}
</style>