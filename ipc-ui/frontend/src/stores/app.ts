import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import { useNetworkStore } from './network'
import { useSubnetsStore } from './subnets'
import { useGatewaysStore } from './gateways'
import { useL1GatewaysStore } from './l1-gateways'
import { updateConsoleStatus } from '../utils/banner'

interface AppLoadingState {
  isInitializing: boolean
  hasInitialized: boolean
  initializationError: string | null
}

export const useAppStore = defineStore('app', () => {
  // State
  const loadingState = ref<AppLoadingState>({
    isInitializing: false,
    hasInitialized: false,
    initializationError: null
  })

  // Get other stores
  const networkStore = useNetworkStore()
  const subnetsStore = useSubnetsStore()
  const gatewaysStore = useGatewaysStore()
  const l1GatewaysStore = useL1GatewaysStore()

  // Computed
  const isLoading = computed(() =>
    loadingState.value.isInitializing ||
    subnetsStore.isLoading ||
    gatewaysStore.loading
  )

  const hasError = computed(() =>
    loadingState.value.initializationError ||
    subnetsStore.error ||
    gatewaysStore.error
  )

  // Actions
  const initializeApp = async (force = false) => {
    // Don't initialize again if already done and not forced
    if (loadingState.value.hasInitialized && !force) {
      updateConsoleStatus('Already initialized', 'Skipping initialization')
      return
    }

    try {
      loadingState.value.isInitializing = true
      loadingState.value.initializationError = null

      console.log('[AppStore] Starting app initialization...')
      updateConsoleStatus('Initializing', 'Starting app initialization...')

      // Initialize network store first (this is fast - just localStorage)
      updateConsoleStatus('Networks', 'Loading network configurations...')
      await networkStore.initializeNetworks()

      // Then fetch data in parallel for better performance
      console.log('[AppStore] Fetching initial data...')
      updateConsoleStatus('Data loading', 'Fetching subnets and gateways...')
      await Promise.all([
        subnetsStore.loadSubnets(),
        gatewaysStore.fetchGateways(force),
        l1GatewaysStore.loadL1Gateways()
      ])

      loadingState.value.hasInitialized = true
      console.log('[AppStore] App initialization completed successfully')
      updateConsoleStatus('Initialization complete', 'All data loaded successfully! ✨')

    } catch (error: any) {
      console.error('[AppStore] App initialization failed:', error)
      updateConsoleStatus('Initialization failed', error.message || 'Unknown error')
      loadingState.value.initializationError = error.message || 'Failed to initialize app'
    } finally {
      loadingState.value.isInitializing = false
    }
  }

  const refreshAllData = async () => {
    console.log('[AppStore] Refreshing all data...')
    updateConsoleStatus('Refreshing', 'Updating all data...')

    try {
      // Refresh all data stores in parallel
      await Promise.all([
        subnetsStore.loadSubnets(),
        gatewaysStore.refreshGateways(),
        l1GatewaysStore.loadL1Gateways()
      ])

      console.log('[AppStore] Data refresh completed')
    } catch (error: any) {
      console.error('[AppStore] Data refresh failed:', error)
    }
  }

  const clearCache = () => {
    console.log('[AppStore] Clearing all cached data...')
    loadingState.value.hasInitialized = false
    // The individual stores will clear their own caches when fetched next time
  }

  return {
    // State
    loadingState: readonly(loadingState),

    // Computed
    isLoading,
    hasError,

    // Actions
    initializeApp,
    refreshAllData,
    clearCache
  }
})