<script setup lang="ts">
import type { ChainStats, SubnetInstance, SubnetStatus } from '@/types/subnet'
import { computed } from 'vue'

interface Props {
  instance: SubnetInstance | null
  chainStats: ChainStats | null
  subnetStatus: SubnetStatus | null
  loadingStats: boolean
  totalStake: string
}

const props = defineProps<Props>()

// Computed properties
const validatorCount = computed(() => {
  return props.instance?.data?.validator_count ||
         props.instance?.data?.validators?.length ||
         props.instance?.validators?.length ||
         0
})
</script>

<template>
  <div class="space-y-6">
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <!-- Performance Metrics -->
      <div class="card">
        <h4 class="text-md font-semibold text-gray-900 mb-3">Performance</h4>
        <div class="space-y-3">
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">Block Height</span>
            <span class="text-sm font-medium text-gray-900">
              {{ chainStats?.block_height || subnetStatus?.block_height || 'N/A' }}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">Avg Block Time</span>
            <span class="text-sm font-medium text-gray-900">
              {{ chainStats?.avg_block_time ? `${chainStats.avg_block_time.toFixed(1)}s` : 'N/A' }}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">TPS</span>
            <span class="text-sm font-medium text-gray-900">
              {{ chainStats?.tps ? chainStats.tps.toFixed(1) : 'N/A' }}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">Pending Transactions</span>
            <span class="text-sm font-medium text-gray-900">
              {{ chainStats?.pending_transactions || 'N/A' }}
            </span>
          </div>
        </div>
      </div>

      <!-- Economic Metrics -->
      <div class="card">
        <h4 class="text-md font-semibold text-gray-900 mb-3">Economic</h4>
        <div class="space-y-3">
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">Total Supply</span>
            <span class="text-sm font-medium text-gray-900">{{ chainStats?.total_supply || 'N/A' }} FIL</span>
          </div>
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">Circulating</span>
            <span class="text-sm font-medium text-gray-900">{{ chainStats?.circulating_supply || 'N/A' }} FIL</span>
          </div>
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">Fees Collected</span>
            <span class="text-sm font-medium text-gray-900">{{ chainStats?.fees_collected || 'N/A' }} FIL</span>
          </div>
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">Total Stake</span>
            <span class="text-sm font-medium text-gray-900">{{ totalStake }} FIL</span>
          </div>
        </div>
      </div>

      <!-- Network Metrics -->
      <div class="card">
        <h4 class="text-md font-semibold text-gray-900 mb-3">Network</h4>
        <div class="space-y-3">
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">Active Validators</span>
            <span class="text-sm font-medium text-gray-900">{{ validatorCount }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">Validators Online</span>
            <span class="text-sm font-medium text-gray-900">
              {{ subnetStatus?.validators_online !== undefined ? subnetStatus.validators_online : 'N/A' }}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">Last Checkpoint</span>
            <span class="text-sm font-medium text-gray-900">
              {{ chainStats?.last_checkpoint || 'N/A' }}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-sm text-gray-500">Consensus Status</span>
            <span class="text-sm font-medium"
                  :class="{
                    'text-green-600': subnetStatus?.consensus_status === 'healthy',
                    'text-yellow-600': subnetStatus?.consensus_status === 'degraded',
                    'text-red-600': subnetStatus?.consensus_status === 'offline',
                    'text-gray-900': !subnetStatus?.consensus_status
                  }">
              {{ subnetStatus?.consensus_status || 'Unknown' }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Real-time Activity Chart Placeholder -->
    <div class="card">
      <div class="flex items-center justify-between mb-4">
        <h4 class="text-md font-semibold text-gray-900">Real-time Activity</h4>
        <div class="flex items-center space-x-2">
          <div v-if="loadingStats" class="animate-spin w-4 h-4 border-2 border-primary-600 border-t-transparent rounded-full"></div>
          <span class="text-xs text-gray-500">
            Last updated: {{ chainStats?.latest_block_time ? new Date(chainStats.latest_block_time).toLocaleTimeString() : 'Never' }}
          </span>
        </div>
      </div>

      <!-- Chain Health Indicators -->
      <div v-if="chainStats || subnetStatus" class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <div class="text-center p-4 border rounded-lg">
          <div class="text-lg font-semibold mb-1"
               :class="{
                 'text-green-600': subnetStatus?.is_active,
                 'text-red-600': subnetStatus?.is_active === false
               }">
            {{ subnetStatus?.is_active ? 'ACTIVE' : 'INACTIVE' }}
          </div>
          <div class="text-sm text-gray-500">Chain Status</div>
        </div>

        <div class="text-center p-4 border rounded-lg">
          <div class="text-lg font-semibold mb-1">
            {{ chainStats?.transaction_count || 'N/A' }}
          </div>
          <div class="text-sm text-gray-500">Total Transactions</div>
        </div>

        <div class="text-center p-4 border rounded-lg">
          <div class="text-lg font-semibold mb-1"
               :class="{
                 'text-green-600': subnetStatus?.sync_status === 'synced',
                 'text-yellow-600': subnetStatus?.sync_status === 'syncing',
                 'text-red-600': subnetStatus?.sync_status === 'behind'
               }">
            {{ subnetStatus?.sync_status?.toUpperCase() || 'UNKNOWN' }}
          </div>
          <div class="text-sm text-gray-500">Sync Status</div>
        </div>
      </div>

      <div v-else class="bg-gray-50 rounded-lg p-8 text-center">
        <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
        </svg>
        <p class="text-gray-600">Loading real-time metrics...</p>
        <p class="text-sm text-gray-500 mt-1">Chain statistics will appear here once available</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card {
  @apply bg-white rounded-lg shadow-sm border border-gray-200 p-6;
}
</style>
