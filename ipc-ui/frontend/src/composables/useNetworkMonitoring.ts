import { useNetworkStore } from '@/stores/network'
import { onMounted, onUnmounted, watch } from 'vue'

/**
 * Composable for managing network connection monitoring
 * Handles page visibility, cleanup, and lifecycle management
 */
export function useNetworkMonitoring() {
  const networkStore = useNetworkStore()

  // Handle page visibility changes to pause/resume monitoring
  const handleVisibilityChange = () => {
    if (document.hidden) {
      console.log('[NetworkMonitoring] Page hidden - pausing connection testing')
      networkStore.disablePeriodicTesting()
    } else {
      console.log('[NetworkMonitoring] Page visible - resuming connection testing')
      networkStore.enablePeriodicTesting()
      // Test immediately when page becomes visible
      networkStore.testSelectedNetworkConnection()
    }
  }

  // Handle window focus/blur events as backup for visibility
  const handleFocus = () => {
    if (!networkStore.isPeriodicTestingEnabled) {
      console.log('[NetworkMonitoring] Window focused - resuming connection testing')
      networkStore.enablePeriodicTesting()
      networkStore.testSelectedNetworkConnection()
    }
  }

  const handleBlur = () => {
    console.log('[NetworkMonitoring] Window blurred - pausing connection testing')
    networkStore.disablePeriodicTesting()
  }

  // Setup monitoring
  onMounted(() => {
    console.log('[NetworkMonitoring] Setting up network monitoring')

    // Listen for page visibility changes
    document.addEventListener('visibilitychange', handleVisibilityChange)

    // Listen for window focus/blur as backup
    window.addEventListener('focus', handleFocus)
    window.addEventListener('blur', handleBlur)

    // Ensure monitoring is enabled if page is visible
    if (!document.hidden) {
      networkStore.enablePeriodicTesting()
    }
  })

  // Cleanup monitoring
  onUnmounted(() => {
    console.log('[NetworkMonitoring] Cleaning up network monitoring')

    // Remove event listeners
    document.removeEventListener('visibilitychange', handleVisibilityChange)
    window.removeEventListener('focus', handleFocus)
    window.removeEventListener('blur', handleBlur)

    // Stop periodic testing
    networkStore.cleanup()
  })

  return {
    networkStore,
    testConnection: networkStore.testSelectedNetworkConnection,
    enableMonitoring: networkStore.enablePeriodicTesting,
    disableMonitoring: networkStore.disablePeriodicTesting
  }
}