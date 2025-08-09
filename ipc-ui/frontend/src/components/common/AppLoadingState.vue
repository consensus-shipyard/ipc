<template>
  <div v-if="appStore.isLoading" class="fixed inset-0 z-50 flex items-center justify-center bg-white bg-opacity-90">
    <div class="text-center">
      <!-- Loading spinner -->
      <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mb-4"></div>

      <!-- Loading text -->
      <h3 class="text-lg font-medium text-gray-900 mb-2">
        {{ loadingText }}
      </h3>

      <!-- Detailed status -->
      <p class="text-sm text-gray-600 max-w-md">
        {{ loadingDetails }}
      </p>

      <!-- Error state -->
      <div v-if="appStore.hasError" class="mt-4 p-4 bg-red-50 rounded-lg border border-red-200">
        <div class="flex items-center">
          <svg class="w-5 h-5 text-red-600 mr-2" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          <span class="text-sm font-medium text-red-700">Failed to load application data</span>
        </div>
        <p class="text-sm text-red-600 mt-1">{{ appStore.hasError }}</p>
        <button
          @click="appStore.initializeApp(true)"
          class="mt-2 px-3 py-1 text-xs bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
        >
          Retry
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useAppStore } from '@/stores/app'
import { computed } from 'vue'

const appStore = useAppStore()

const loadingText = computed(() => {
  if (appStore.hasError) {
    return 'Connection Error'
  }

  if (appStore.loadingState.isInitializing) {
    return 'Loading IPC Console...'
  }

  return 'Connecting...'
})

const loadingDetails = computed(() => {
  if (appStore.hasError) {
    return 'Unable to connect to the IPC backend. Please make sure the IPC CLI is running.'
  }

  if (appStore.loadingState.isInitializing) {
    return 'Fetching subnet instances, gateways, and network configuration...'
  }

  return 'Establishing connection to IPC services...'
})
</script>

<style scoped>
/* Custom styles for the loading state */
.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>