<script setup lang="ts">
import FieldLoadingIndicator from '@/components/common/FieldLoadingIndicator.vue'
import type { ChainStats, SubnetInstance, SubnetStatus } from '@/types/subnet'
import { computed } from 'vue'

interface Props {
  instance: SubnetInstance | null
  loading: boolean
  error: string | null
  createdDate: string
  totalStake: string
  gatewayAddress: string
  gatewayAddressShort: string
  statusColor: string
  copyingAddress: string | null
  chainStats: ChainStats | null
  subnetStatus: SubnetStatus | null
  loadingStats: boolean
  statsError: string | null
}

interface Emits {
  (e: 'copyToClipboard', text: string, type: string): void
  (e: 'fetchInstance'): void
  (e: 'fetchChainStats'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Computed properties for template
const loadingBasicInfo = computed(() => props.loading)
const basicInfoError = computed(() => props.error)
const loadingChainStats = computed(() => props.loadingStats)
</script>

<template>
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    <!-- Basic Information -->
    <div class="card">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">Basic Information</h3>
      <dl class="space-y-3">
        <div class="flex justify-between">
          <dt class="text-sm font-medium text-gray-500">Subnet ID</dt>
          <dd class="text-sm text-gray-900 font-mono">
            <FieldLoadingIndicator
              :is-loading="loadingBasicInfo"
              :has-error="!!basicInfoError"
              loading-text="Loading..."
              @retry="emit('fetchInstance')"
            >
              {{ instance?.data?.id || instance?.id }}
            </FieldLoadingIndicator>
          </dd>
        </div>
        <div class="flex justify-between">
          <dt class="text-sm font-medium text-gray-500">Name</dt>
          <dd class="text-sm text-gray-900">
            <FieldLoadingIndicator
              :is-loading="loadingBasicInfo"
              :has-error="!!basicInfoError"
              loading-text="Loading..."
              @retry="emit('fetchInstance')"
            >
              {{ instance?.data?.name || instance?.name }}
            </FieldLoadingIndicator>
          </dd>
        </div>
        <div class="flex justify-between">
          <dt class="text-sm font-medium text-gray-500">Status</dt>
          <dd>
            <FieldLoadingIndicator
              :is-loading="loadingBasicInfo"
              :has-error="!!basicInfoError"
              loading-text="Loading..."
              @retry="emit('fetchInstance')"
            >
              <span v-if="instance" :class="['inline-flex items-center px-2 py-1 rounded-full text-xs font-medium', statusColor]">
                {{ (instance.data?.status || instance.status || 'Unknown').charAt(0).toUpperCase() + (instance.data?.status || instance.status || 'unknown').slice(1) }}
              </span>
            </FieldLoadingIndicator>
          </dd>
        </div>
        <div class="flex justify-between">
          <dt class="text-sm font-medium text-gray-500">Template</dt>
          <dd class="text-sm text-gray-900">
            <FieldLoadingIndicator
              :is-loading="loadingBasicInfo"
              :has-error="!!basicInfoError"
              loading-text="Loading..."
              @retry="emit('fetchInstance')"
            >
              {{ instance?.data?.template || instance?.template }}
            </FieldLoadingIndicator>
          </dd>
        </div>
        <div class="flex justify-between">
          <dt class="text-sm font-medium text-gray-500">Parent Network</dt>
          <dd class="text-sm text-gray-900 font-mono">
            <FieldLoadingIndicator
              :is-loading="loadingBasicInfo"
              :has-error="!!basicInfoError"
              loading-text="Loading..."
              @retry="emit('fetchInstance')"
            >
              {{ instance?.data?.parent || instance?.parent }}
            </FieldLoadingIndicator>
          </dd>
        </div>
        <div class="flex justify-between">
          <dt class="text-sm font-medium text-gray-500">Gateway Contract</dt>
          <dd class="text-sm text-gray-900 font-mono relative">
            <FieldLoadingIndicator
              :is-loading="loadingBasicInfo"
              :has-error="!!basicInfoError"
              loading-text="Loading..."
              @retry="emit('fetchInstance')"
            >
              <button
                v-if="instance"
                @click="emit('copyToClipboard', gatewayAddress, 'gateway')"
                class="hover:bg-gray-100 px-2 py-1 rounded transition-colors cursor-pointer text-left"
                :title="copyingAddress === 'gateway' ? 'Copied!' : `Click to copy: ${gatewayAddress}`"
              >
                {{ gatewayAddressShort }}
                <svg v-if="copyingAddress === 'gateway'" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
              </button>
            </FieldLoadingIndicator>
          </dd>
        </div>
        <div class="flex justify-between">
          <dt class="text-sm font-medium text-gray-500">Created</dt>
          <dd class="text-sm text-gray-900">
            <FieldLoadingIndicator
              :is-loading="loadingBasicInfo"
              :has-error="!!basicInfoError"
              loading-text="Loading..."
              @retry="emit('fetchInstance')"
            >
              {{ createdDate }}
            </FieldLoadingIndicator>
          </dd>
        </div>
      </dl>
    </div>

    <!-- Quick Stats -->
    <div class="card">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900">Chain Statistics</h3>
        <div class="flex items-center space-x-2">
          <FieldLoadingIndicator
            :is-loading="loadingChainStats"
            :has-error="!!statsError"
            loading-text="Loading..."
            @retry="emit('fetchChainStats')"
          >
            <div v-if="subnetStatus?.is_active" class="flex items-center text-green-600">
              <div class="w-2 h-2 bg-green-500 rounded-full mr-2 animate-pulse"></div>
              <span class="text-sm font-medium">Active</span>
            </div>
            <div v-else class="flex items-center text-red-600">
              <div class="w-2 h-2 bg-red-500 rounded-full mr-2"></div>
              <span class="text-sm font-medium">Inactive</span>
            </div>
          </FieldLoadingIndicator>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div class="text-center p-4 bg-gray-50 rounded-lg">
          <div class="text-2xl font-bold text-gray-900">
            <FieldLoadingIndicator
              :is-loading="loadingChainStats"
              :has-error="!!statsError"
              loading-text="Loading..."
              @retry="emit('fetchChainStats')"
            >
              {{ chainStats?.block_height || subnetStatus?.block_height || 'N/A' }}
            </FieldLoadingIndicator>
          </div>
          <div class="text-sm text-gray-500">Block Height</div>
          <div v-if="chainStats?.latest_block_time" class="text-xs text-gray-400 mt-1">
            {{ new Date(chainStats.latest_block_time).toLocaleTimeString() }}
          </div>
        </div>

        <div class="text-center p-4 bg-gray-50 rounded-lg">
          <div class="text-2xl font-bold text-gray-900">
            <FieldLoadingIndicator
              :is-loading="loadingBasicInfo"
              :has-error="!!basicInfoError"
              loading-text="Loading..."
              @retry="emit('fetchInstance')"
            >
              {{ instance?.data?.validator_count || instance?.data?.validators?.length || instance?.validators?.length || 0 }}
            </FieldLoadingIndicator>
          </div>
          <div class="text-sm text-gray-500">Validators</div>
          <div class="text-xs text-gray-400 mt-1">
            <FieldLoadingIndicator
              :is-loading="loadingChainStats"
              :has-error="!!statsError"
              loading-text="Loading..."
              @retry="emit('fetchChainStats')"
            >
              {{ subnetStatus?.validators_online !== undefined ? `${subnetStatus.validators_online} online` : 'N/A online' }}
            </FieldLoadingIndicator>
          </div>
        </div>

        <div class="text-center p-4 bg-gray-50 rounded-lg">
          <div class="text-2xl font-bold text-gray-900">
            <FieldLoadingIndicator
              :is-loading="loadingChainStats"
              :has-error="!!statsError"
              loading-text="Loading..."
              @retry="emit('fetchChainStats')"
            >
              {{ chainStats?.transaction_count || 'N/A' }}
            </FieldLoadingIndicator>
          </div>
          <div class="text-sm text-gray-500">Total Transactions</div>
          <div class="text-xs text-gray-400 mt-1">
            <FieldLoadingIndicator
              :is-loading="loadingChainStats"
              :has-error="!!statsError"
              loading-text="Loading..."
              @retry="emit('fetchChainStats')"
            >
              {{ chainStats?.tps ? `${chainStats.tps.toFixed(1)} TPS` : 'N/A TPS' }}
            </FieldLoadingIndicator>
          </div>
        </div>

        <div class="text-center p-4 bg-gray-50 rounded-lg">
          <div class="text-2xl font-bold text-gray-900">
            <FieldLoadingIndicator
              :is-loading="loadingChainStats"
              :has-error="!!statsError"
              loading-text="Loading..."
              @retry="emit('fetchChainStats')"
            >
              <span v-if="subnetStatus?.consensus_status === 'healthy'" class="text-green-600">●</span>
              <span v-else-if="subnetStatus?.consensus_status === 'degraded'" class="text-yellow-600">●</span>
              <span v-else-if="subnetStatus?.consensus_status === 'offline'" class="text-red-600">●</span>
              <span v-else class="text-gray-400">●</span>
              {{ subnetStatus?.consensus_status || 'Unknown' }}
            </FieldLoadingIndicator>
          </div>
          <div class="text-sm text-gray-500">Consensus</div>
          <div class="text-xs text-gray-400 mt-1">
            <FieldLoadingIndicator
              :is-loading="loadingChainStats"
              :has-error="!!statsError"
              loading-text="Loading..."
              @retry="emit('fetchChainStats')"
            >
              {{ chainStats?.avg_block_time ? `${chainStats.avg_block_time.toFixed(1)}s avg block` : 'N/A avg block' }}
            </FieldLoadingIndicator>
          </div>
        </div>
      </div>

      <!-- Error state for stats -->
      <div v-if="statsError" class="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
        <p class="text-red-700 text-sm">{{ statsError }}</p>
        <button @click="emit('fetchChainStats')" class="text-red-600 hover:text-red-700 text-sm font-medium mt-1">
          Retry
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card {
  @apply bg-white rounded-lg shadow-sm border border-gray-200 p-6;
}
</style>
