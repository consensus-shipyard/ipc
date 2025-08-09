<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import AppHeader from './components/common/AppHeader.vue'
import AppLoadingState from './components/common/AppLoadingState.vue'
import AppSidebar from './components/common/AppSidebar.vue'
import { useNetworkMonitoring } from './composables/useNetworkMonitoring'
import { useAppStore } from './stores/app'

const route = useRoute()

// Initialize app store for centralized data loading
const appStore = useAppStore()

// Initialize network monitoring for real-time connection status
const { networkStore } = useNetworkMonitoring()

// Check if we're in the wizard flow to show different layout
const isWizardRoute = computed(() => {
  return route.path.startsWith('/wizard')
})

// Initialize app data on mount
onMounted(() => {
  console.log('[App] Initializing application...')
  appStore.initializeApp()
})
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Global Loading State -->
    <AppLoadingState />

    <!-- App Header -->
    <AppHeader />

    <div class="flex h-screen pt-16"> <!-- pt-16 accounts for fixed header -->
      <!-- Sidebar (hidden on wizard pages) -->
      <AppSidebar v-if="!isWizardRoute" />

      <!-- Main Content -->
      <main
        :class="[
          'flex-1 overflow-auto',
          isWizardRoute ? 'ml-0' : 'ml-64' // ml-64 accounts for sidebar width
        ]"
      >
        <RouterView />
      </main>
    </div>
  </div>
</template>

<style scoped>
/* Additional custom styles if needed */
</style>
