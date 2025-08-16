/**
 * Composable for managing chain statistics
 */
import { ref, type Ref, onUnmounted } from 'vue'
import type { ChainStats, SubnetStatus } from '@/types/subnet'
import { StatsService } from '@/services/subnet/stats.service'

export function useChainStats(subnetId: Ref<string | undefined>) {
  const chainStats = ref<ChainStats | null>(null)
  const subnetStatus = ref<SubnetStatus | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const refreshInterval = ref<number | null>(null)

  const fetch = async () => {
    if (!subnetId.value) {
      console.log('[useChainStats] No subnet ID, skipping fetch')
      return
    }

    try {
      loading.value = true
      error.value = null

      const stats = await StatsService.getAllStats(subnetId.value)

      chainStats.value = stats.chainStats
      subnetStatus.value = stats.subnetStatus
    } catch (err) {
      console.error('Error fetching chain stats:', err)
      error.value = err instanceof Error ? err.message : 'Failed to load chain statistics'
    } finally {
      loading.value = false
    }
  }

  const startAutoRefresh = (intervalMs: number = 10000) => {
    // Fetch immediately
    fetch()

    // Clear any existing interval
    stopAutoRefresh()

    // Set up periodic refresh
    refreshInterval.value = window.setInterval(() => {
      fetch()
    }, intervalMs)
  }

  const stopAutoRefresh = () => {
    if (refreshInterval.value !== null) {
      clearInterval(refreshInterval.value)
      refreshInterval.value = null
    }
  }

  // Cleanup on unmount
  onUnmounted(() => {
    stopAutoRefresh()
  })

  return {
    chainStats,
    subnetStatus,
    loading,
    error,
    fetch,
    startAutoRefresh,
    stopAutoRefresh
  }
}
