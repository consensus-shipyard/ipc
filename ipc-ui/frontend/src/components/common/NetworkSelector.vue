<template>
  <div class="network-selector relative">
    <!-- Network selector and refresh buttons container -->
    <div class="flex items-center space-x-1">
      <!-- Network selector button -->
      <button
        @click="toggleDropdown"
        class="flex items-center space-x-2 px-3 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-l-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500 transition-colors"
        :class="{ 'bg-gray-50': showDropdown }"
      >
        <!-- Network connection status indicator -->
        <div
          class="w-2 h-2 rounded-full"
          :class="{
            'bg-green-500': networkStore.selectedNetworkStatus?.connected === true,
            'bg-red-500': networkStore.selectedNetworkStatus?.connected === false,
            'bg-gray-400': !networkStore.selectedNetworkStatus,
            'animate-pulse': networkStore.isTestingConnection
          }"
          :title="getConnectionStatusTitle()"
        ></div>

        <!-- Network name -->
        <span class="max-w-32 truncate">
          {{ networkStore.selectedNetwork?.name || 'No Network' }}
        </span>

        <!-- Network type badge -->
        <span
          v-if="networkStore.selectedNetwork"
          class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium"
          :class="{
            'bg-green-100 text-green-700': networkStore.selectedNetwork.type === 'mainnet',
            'bg-yellow-100 text-yellow-700': networkStore.selectedNetwork.type === 'testnet',
            'bg-blue-100 text-blue-700': networkStore.selectedNetwork.type === 'local',
            'bg-gray-100 text-gray-700': networkStore.selectedNetwork.type === 'custom'
          }"
        >
          {{ networkStore.selectedNetwork.type.toUpperCase() }}
        </span>

        <!-- Dropdown arrow -->
        <svg
          class="w-4 h-4 text-gray-400 transition-transform"
          :class="{ 'rotate-180': showDropdown }"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      <!-- Manual refresh button - now separate from main button -->
      <button
        @click.stop="refreshConnection"
        :disabled="networkStore.isTestingConnection"
        class="p-2 text-gray-400 hover:text-gray-600 focus:outline-none focus:ring-2 focus:ring-primary-500 bg-white border border-l-0 border-gray-300 rounded-r-md hover:bg-gray-50 transition-colors"
        :class="{ 'animate-spin': networkStore.isTestingConnection }"
        title="Test network connection"
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </button>
    </div>

    <!-- Dropdown menu -->
    <div
      v-show="showDropdown"
      class="absolute right-0 mt-2 w-80 bg-white border border-gray-200 rounded-md shadow-lg z-50"
      @click.stop
    >
      <div class="py-1">
        <!-- Current network header -->
        <div class="px-4 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wide border-b border-gray-100 flex items-center justify-between">
          <span>Available Networks</span>
          <button
            @click="testAllNetworks"
            :disabled="networkStore.isTestingConnection"
            class="text-xs text-primary-600 hover:text-primary-700 font-medium disabled:opacity-50"
            :class="{ 'animate-pulse': networkStore.isTestingConnection }"
          >
            Test All
          </button>
        </div>

        <!-- Network list -->
        <div class="max-h-60 overflow-y-auto">
          <button
            v-for="network in networkStore.networks"
            :key="network.id"
            @click="selectNetwork(network.id)"
            class="w-full text-left px-4 py-3 hover:bg-gray-50 focus:outline-none focus:bg-gray-50 transition-colors"
            :class="{ 'bg-blue-50 border-l-4 border-l-blue-500': network.id === networkStore.selectedNetworkId }"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center space-x-3">
                <!-- Connection status indicator -->
                <div
                  class="w-2 h-2 rounded-full flex-shrink-0"
                  :class="{
                    'bg-green-500': getNetworkStatus(network.id)?.connected === true,
                    'bg-red-500': getNetworkStatus(network.id)?.connected === false,
                    'bg-gray-400': !getNetworkStatus(network.id),
                    'animate-pulse': networkStore.isTestingConnection
                  }"
                  :title="getNetworkConnectionTitle(network.id)"
                ></div>

                <div>
                  <div class="font-medium text-gray-900">{{ network.name }}</div>
                  <div class="text-xs text-gray-500 truncate max-w-48">{{ network.rpcUrl }}</div>
                  <!-- Connection details -->
                  <div v-if="getNetworkStatus(network.id)" class="text-xs text-gray-400 mt-1">
                    <span v-if="getNetworkStatus(network.id)?.connected && getNetworkStatus(network.id)?.response_time_ms">
                      {{ getNetworkStatus(network.id)?.response_time_ms }}ms
                    </span>
                    <span v-else-if="!getNetworkStatus(network.id)?.connected && getNetworkStatus(network.id)?.error" class="text-red-500">
                      {{ getNetworkStatus(network.id)?.error?.substring(0, 30) }}{{ (getNetworkStatus(network.id)?.error?.length || 0) > 30 ? '...' : '' }}
                    </span>
                    <span v-if="getNetworkStatus(network.id)?.last_checked">
                      • {{ formatLastChecked(getNetworkStatus(network.id)?.last_checked || '') }}
                    </span>
                  </div>
                </div>
              </div>

              <div class="flex items-center space-x-2">
                <!-- Type badge -->
                <span
                  class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium"
                  :class="{
                    'bg-green-100 text-green-700': network.type === 'mainnet',
                    'bg-yellow-100 text-yellow-700': network.type === 'testnet',
                    'bg-blue-100 text-blue-700': network.type === 'local',
                    'bg-gray-100 text-gray-700': network.type === 'custom'
                  }"
                >
                  {{ network.type.toUpperCase() }}
                </span>

                <!-- Selected indicator -->
                <svg
                  v-if="network.id === networkStore.selectedNetworkId"
                  class="w-4 h-4 text-blue-600"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path
                    fill-rule="evenodd"
                    d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                    clip-rule="evenodd"
                  />
                </svg>
              </div>
            </div>
          </button>
        </div>

        <!-- Divider -->
        <div class="border-t border-gray-100"></div>

        <!-- Manage networks button -->
        <button
          @click="openManageNetworks"
          class="w-full text-left px-4 py-3 text-sm text-gray-700 hover:bg-gray-50 focus:outline-none focus:bg-gray-50 transition-colors flex items-center space-x-2"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          <span>Manage Networks</span>
        </button>
      </div>
    </div>

    <!-- Manage Networks Modal -->
    <NetworkManagement
      v-if="showManageModal"
      @close="closeManageNetworks"
    />
  </div>
</template>

<script setup lang="ts">
import { useNetworkStore } from '@/stores/network'
import { onMounted, onUnmounted, ref } from 'vue'
import NetworkManagement from './NetworkManagement.vue'

const networkStore = useNetworkStore()
const showDropdown = ref(false)
const showManageModal = ref(false)

const toggleDropdown = () => {
  showDropdown.value = !showDropdown.value
}

const selectNetwork = (networkId: string) => {
  networkStore.selectNetwork(networkId)
  showDropdown.value = false
}

const openManageNetworks = () => {
  showDropdown.value = false
  showManageModal.value = true
}

const closeManageNetworks = () => {
  showManageModal.value = false
}

const refreshConnection = async () => {
  if (!networkStore.isTestingConnection) {
    await networkStore.testSelectedNetworkConnection()
  }
}

const testAllNetworks = async () => {
  if (!networkStore.isTestingConnection) {
    await networkStore.testAllNetworkConnections()
  }
}

const getNetworkStatus = (networkId: string) => {
  return networkStore.networkStatuses.get(networkId) || null
}

const getConnectionStatusTitle = () => {
  const status = networkStore.selectedNetworkStatus
  if (!status) return 'Connection status unknown'

  if (status.connected) {
    const timeInfo = status.response_time_ms ? ` (${status.response_time_ms}ms)` : ''
    const lastChecked = status.last_checked ? ` - Last checked: ${formatLastChecked(status.last_checked)}` : ''
    return `Connected${timeInfo}${lastChecked}`
  } else {
    const lastChecked = status.last_checked ? ` - Last checked: ${formatLastChecked(status.last_checked)}` : ''
    return status.error ? `Disconnected: ${status.error}${lastChecked}` : `Disconnected${lastChecked}`
  }
}

const getNetworkConnectionTitle = (networkId: string) => {
  const status = getNetworkStatus(networkId)
  if (!status) return 'Connection status unknown'

  if (status.connected) {
    const timeInfo = status.response_time_ms ? ` (${status.response_time_ms}ms)` : ''
    const lastChecked = status.last_checked ? ` - Last checked: ${formatLastChecked(status.last_checked)}` : ''
    return `Connected${timeInfo}${lastChecked}`
  } else {
    const lastChecked = status.last_checked ? ` - Last checked: ${formatLastChecked(status.last_checked)}` : ''
    return status.error ? `Disconnected: ${status.error}${lastChecked}` : `Disconnected${lastChecked}`
  }
}

const formatLastChecked = (timestamp: string) => {
  try {
    const date = new Date(timestamp)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffSeconds = Math.floor(diffMs / 1000)
    const diffMinutes = Math.floor(diffSeconds / 60)

    if (diffSeconds < 60) {
      return `${diffSeconds}s ago`
    } else if (diffMinutes < 60) {
      return `${diffMinutes}m ago`
    } else {
      return date.toLocaleTimeString()
    }
  } catch {
    return 'Unknown'
  }
}

// Close dropdown when clicking outside
const handleClickOutside = (event: Event) => {
  const target = event.target as HTMLElement
  const dropdown = document.querySelector('.network-selector')

  if (dropdown && !dropdown.contains(target)) {
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