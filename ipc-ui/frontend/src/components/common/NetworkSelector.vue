<template>
  <div class="network-selector relative">
    <!-- Network selector button -->
    <button
      @click="toggleDropdown"
      class="flex items-center space-x-2 px-3 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500 transition-colors"
      :class="{ 'bg-gray-50': showDropdown }"
    >
      <!-- Network status indicator -->
      <div
        class="w-2 h-2 rounded-full"
        :class="{
          'bg-green-500': networkStore.selectedNetwork?.type === 'mainnet',
          'bg-yellow-500': networkStore.selectedNetwork?.type === 'testnet',
          'bg-blue-500': networkStore.selectedNetwork?.type === 'local',
          'bg-gray-500': networkStore.selectedNetwork?.type === 'custom' || !networkStore.selectedNetwork
        }"
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

    <!-- Dropdown menu -->
    <div
      v-show="showDropdown"
      class="absolute right-0 mt-2 w-80 bg-white border border-gray-200 rounded-md shadow-lg z-50"
      @click.stop
    >
      <div class="py-1">
        <!-- Current network header -->
        <div class="px-4 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wide border-b border-gray-100">
          Available Networks
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
                  <div class="text-xs text-gray-500 truncate max-w-48">{{ network.rpcUrl }}</div>
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