<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import AppHeader from './components/common/AppHeader.vue'
import AppLoadingState from './components/common/AppLoadingState.vue'
import AppSidebar from './components/common/AppSidebar.vue'
import SplashScreen from './components/common/SplashScreen.vue'
import { useNetworkMonitoring } from './composables/useNetworkMonitoring'
import { useAppStore } from './stores/app'
import { updateConsoleStatus } from './utils/banner'

const route = useRoute()

// Splash screen state
const showSplash = ref(false)

// Initialize app store for centralized data loading
const appStore = useAppStore()

// Initialize network monitoring for real-time connection status
const { networkStore } = useNetworkMonitoring()

// Check if we're in the wizard flow to show different layout
const isWizardRoute = computed(() => {
  return route.path.startsWith('/wizard')
})

// Initialize app data on mount
onMounted(async () => {
  console.log('[App] Initializing application...')
  updateConsoleStatus('App mounted', 'Starting initialization sequence...')

  try {
    updateConsoleStatus('Loading stores', 'Initializing data stores...')
    await appStore.initializeApp()

    updateConsoleStatus('App ready', 'All systems operational! 🚀')

    // Show splash screen for a minimum time for better UX
    setTimeout(() => {
      showSplash.value = false
      updateConsoleStatus('UI ready', 'Welcome to IPC! 🌌')
    }, 2500)

  } catch (error) {
    console.error('[App] Initialization failed:', error)
    updateConsoleStatus('Error', `Initialization failed: ${error}`)
    // Still hide splash on error after a short delay
    setTimeout(() => {
      showSplash.value = false
    }, 1500)
  }
})
</script>

<template>
  <!-- Splash Screen -->
  <SplashScreen :show="showSplash" />

  <div class="min-h-screen bg-gray-50">

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
