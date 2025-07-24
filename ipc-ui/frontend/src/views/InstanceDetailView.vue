<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { apiService } from '../services/api'

interface Validator {
  address: string
  stake: string
  power: number
  status: string
}

interface SubnetInstance {
  id: string
  name: string
  status: string
  template: string
  parent: string
  created_at: string
  validators: Validator[]
  config: Record<string, any>
}

const route = useRoute()
const router = useRouter()

// Props
const props = defineProps<{
  id: string
}>()

// State
const instance = ref<SubnetInstance | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const activeTab = ref('overview')

// Computed
const createdDate = computed(() => {
  if (!instance.value || !instance.value.created_at) return 'Unknown'

  try {
    return new Date(instance.value.created_at).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch (error) {
    console.warn('Error parsing created_at date:', instance.value.created_at)
    return 'Invalid Date'
  }
})

const totalStake = computed(() => {
  if (!instance.value?.validators) return '0'
  return instance.value.validators
    .reduce((sum, v) => sum + parseFloat(v.stake || '0'), 0)
    .toFixed(2)
})

const totalPower = computed(() => {
  if (!instance.value?.validators) return 0
  return instance.value.validators
    .reduce((sum, v) => sum + (v.power || 0), 0)
})

const statusColor = computed(() => {
  if (!instance.value || !instance.value.status) return 'text-gray-600 bg-gray-50'

  switch (instance.value.status.toLowerCase()) {
    case 'active': return 'text-green-600 bg-green-50'
    case 'paused': return 'text-yellow-600 bg-yellow-50'
    case 'deploying': return 'text-blue-600 bg-blue-50'
    case 'failed': return 'text-red-600 bg-red-50'
    default: return 'text-gray-600 bg-gray-50'
  }
})

// Methods
const fetchInstance = async () => {
  try {
    loading.value = true
    error.value = null

    // Decode the URL-encoded ID parameter
    const decodedId = decodeURIComponent(props.id)
    const response = await apiService.getInstance(decodedId)

    // Check if we got HTML instead of JSON (indicates backend routing issue)
    if (typeof response.data === 'string' && response.data.includes('<!DOCTYPE html>')) {
      error.value = 'Backend routing error: API endpoint returned HTML instead of JSON data. This usually means the route is not properly configured.'
      return
    }

    if (response.data) {
      instance.value = response.data
    } else {
      error.value = 'Instance not found'
    }
  } catch (err) {
    console.error('Error fetching instance:', err)
    error.value = err instanceof Error ? err.message : 'Failed to load instance'
  } finally {
    loading.value = false
  }
}

const goBack = () => {
  router.push('/')
}

const exportConfig = () => {
  if (!instance.value) return

  const configData = {
    name: instance.value.name,
    config: instance.value.config,
    validators: instance.value.validators,
    exported_at: new Date().toISOString()
  }

  const blob = new Blob([JSON.stringify(configData, null, 2)], {
    type: 'application/json'
  })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `${instance.value.name}-config.json`
  link.click()
  URL.revokeObjectURL(url)
}

const pauseSubnet = async () => {
  // TODO: Implement pause functionality
  console.log('Pause subnet:', decodeURIComponent(props.id))
}

const resumeSubnet = async () => {
  // TODO: Implement resume functionality
  console.log('Resume subnet:', decodeURIComponent(props.id))
}

const viewLogs = () => {
  // TODO: Implement log viewing
  console.log('View logs for:', decodeURIComponent(props.id))
}

// Lifecycle
onMounted(() => {
  fetchInstance()
})

// Watch for route changes
watch(() => props.id, (newId) => {
  if (newId) {
    fetchInstance()
  }
})
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Header -->
    <div class="bg-white shadow-sm border-b">
      <div class="max-w-7xl mx-auto px-6 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <button
              @click="goBack"
              class="text-gray-600 hover:text-gray-700 flex items-center"
            >
              <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
              Back to Dashboard
            </button>
            <div>
              <h1 class="text-2xl font-bold text-gray-900">
                {{ instance?.name || 'Loading...' }}
              </h1>
              <p class="text-gray-600 mt-1">Subnet ID: {{ decodeURIComponent(props.id) }}</p>
            </div>
          </div>

          <div v-if="instance" class="flex items-center space-x-3">
            <span
              :class="[
                'inline-flex items-center px-3 py-1 rounded-full text-sm font-medium',
                statusColor
              ]"
            >
              {{ (instance.status || 'Unknown').charAt(0).toUpperCase() + (instance.status || 'unknown').slice(1) }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="max-w-7xl mx-auto px-6 py-8">
      <div class="text-center py-12">
        <div class="animate-spin inline-block w-8 h-8 border-4 border-primary-600 border-t-transparent rounded-full"></div>
        <p class="mt-4 text-gray-600">Loading subnet details...</p>
      </div>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="max-w-7xl mx-auto px-6 py-8">
      <div class="bg-red-50 border border-red-200 rounded-lg p-6">
        <div class="flex items-start space-x-3">
          <svg class="w-6 h-6 text-red-600 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          <div>
            <h3 class="font-semibold text-red-800 mb-1">Error Loading Subnet</h3>
            <p class="text-red-700">{{ error }}</p>
            <button
              @click="fetchInstance"
              class="mt-3 text-red-600 hover:text-red-700 font-medium text-sm"
            >
              Try Again
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div v-else-if="instance" class="max-w-7xl mx-auto px-6 py-8">
      <!-- Quick Actions -->
      <div class="flex flex-wrap gap-3 mb-6">
        <button
          v-if="instance.status === 'active'"
          @click="pauseSubnet"
          class="btn-secondary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          Pause Subnet
        </button>

        <button
          v-else-if="instance.status === 'paused'"
          @click="resumeSubnet"
          class="btn-primary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1m4 0h1m-6 4h1m4 0h1M9 16h6" />
          </svg>
          Resume Subnet
        </button>

        <button
          @click="viewLogs"
          class="btn-secondary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          View Logs
        </button>

        <button
          @click="exportConfig"
          class="btn-secondary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          Export Config
        </button>
      </div>

      <!-- Tab Navigation -->
      <div class="border-b border-gray-200 mb-6">
        <nav class="flex space-x-8">
          <button
            @click="activeTab = 'overview'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'overview'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Overview
          </button>
          <button
            @click="activeTab = 'validators'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'validators'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Validators ({{ instance.validators.length }})
          </button>
          <button
            @click="activeTab = 'configuration'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'configuration'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Configuration
          </button>
          <button
            @click="activeTab = 'metrics'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'metrics'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Metrics
          </button>
        </nav>
      </div>

      <!-- Tab Content -->
      <div class="space-y-6">
        <!-- Overview Tab -->
        <div v-if="activeTab === 'overview'" class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <!-- Basic Information -->
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Basic Information</h3>
            <dl class="space-y-3">
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Subnet ID</dt>
                <dd class="text-sm text-gray-900 font-mono">{{ instance.id }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Name</dt>
                <dd class="text-sm text-gray-900">{{ instance.name }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Status</dt>
                <dd>
                  <span :class="['inline-flex items-center px-2 py-1 rounded-full text-xs font-medium', statusColor]">
                    {{ (instance.status || 'Unknown').charAt(0).toUpperCase() + (instance.status || 'unknown').slice(1) }}
                  </span>
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Template</dt>
                <dd class="text-sm text-gray-900">{{ instance.template }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Parent Network</dt>
                <dd class="text-sm text-gray-900 font-mono">{{ instance.parent }}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Created</dt>
                <dd class="text-sm text-gray-900">{{ createdDate }}</dd>
              </div>
            </dl>
          </div>

          <!-- Quick Stats -->
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Quick Stats</h3>
            <div class="grid grid-cols-2 gap-4">
              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">{{ instance.validators.length }}</div>
                <div class="text-sm text-gray-500">Validators</div>
              </div>
              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">{{ totalStake }}</div>
                <div class="text-sm text-gray-500">Total Stake (FIL)</div>
              </div>
              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">{{ totalPower }}</div>
                <div class="text-sm text-gray-500">Total Power</div>
              </div>
              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">{{ instance.config.permissionMode || 'N/A' }}</div>
                <div class="text-sm text-gray-500">Mode</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Validators Tab -->
        <div v-if="activeTab === 'validators'" class="space-y-6">
          <div class="card">
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-lg font-semibold text-gray-900">Validators</h3>
              <div class="text-sm text-gray-500">
                {{ instance.validators.length }} validator{{ instance.validators.length !== 1 ? 's' : '' }}
              </div>
            </div>

            <div v-if="instance.validators.length === 0" class="text-center py-8 text-gray-500">
              <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
              </svg>
              <p class="font-medium">No validators configured</p>
              <p class="text-sm">This subnet has no validators set up yet.</p>
            </div>

            <div v-else class="overflow-x-auto">
              <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                  <tr>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Address
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Stake
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Power
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Status
                    </th>
                  </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                  <tr v-for="validator in instance.validators" :key="validator.address">
                    <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900">
                      {{ validator.address.slice(0, 8) }}...{{ validator.address.slice(-6) }}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {{ validator.stake }} FIL
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {{ validator.power }}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap">
                      <span :class="[
                        'inline-flex px-2 py-1 text-xs font-semibold rounded-full',
                        validator.status === 'Active' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                      ]">
                        {{ validator.status }}
                      </span>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- Configuration Tab -->
        <div v-if="activeTab === 'configuration'" class="space-y-6">
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Configuration Details</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div v-for="(value, key) in instance.config" :key="key" class="flex justify-between py-2 border-b border-gray-100">
                <dt class="text-sm font-medium text-gray-500 capitalize">
                  {{ key.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase()) }}
                </dt>
                <dd class="text-sm text-gray-900">
                  <span v-if="typeof value === 'boolean'" :class="value ? 'text-green-600' : 'text-red-600'">
                    {{ value ? 'Yes' : 'No' }}
                  </span>
                  <span v-else-if="typeof value === 'string' && value.startsWith('0x')" class="font-mono">
                    {{ value.slice(0, 8) }}...{{ value.slice(-6) }}
                  </span>
                  <span v-else>{{ value }}</span>
                </dd>
              </div>
            </div>
          </div>
        </div>

        <!-- Metrics Tab -->
        <div v-if="activeTab === 'metrics'" class="space-y-6">
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <!-- Performance Metrics -->
            <div class="card">
              <h4 class="text-md font-semibold text-gray-900 mb-3">Performance</h4>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Block Height</span>
                  <span class="text-sm font-medium text-gray-900">{{ Math.floor(Math.random() * 10000) + 1000 }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Avg Block Time</span>
                  <span class="text-sm font-medium text-gray-900">2.1s</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">TPS</span>
                  <span class="text-sm font-medium text-gray-900">{{ Math.floor(Math.random() * 100) + 50 }}</span>
                </div>
              </div>
            </div>

            <!-- Economic Metrics -->
            <div class="card">
              <h4 class="text-md font-semibold text-gray-900 mb-3">Economic</h4>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Total Supply</span>
                  <span class="text-sm font-medium text-gray-900">{{ (Math.random() * 1000000).toFixed(0) }} FIL</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Circulating</span>
                  <span class="text-sm font-medium text-gray-900">{{ (Math.random() * 500000).toFixed(0) }} FIL</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Fees Collected</span>
                  <span class="text-sm font-medium text-gray-900">{{ (Math.random() * 1000).toFixed(2) }} FIL</span>
                </div>
              </div>
            </div>

            <!-- Network Metrics -->
            <div class="card">
              <h4 class="text-md font-semibold text-gray-900 mb-3">Network</h4>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Active Validators</span>
                  <span class="text-sm font-medium text-gray-900">{{ instance.validators.length }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Last Checkpoint</span>
                  <span class="text-sm font-medium text-gray-900">{{ Math.floor(Math.random() * 60) + 1 }}m ago</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Uptime</span>
                  <span class="text-sm font-medium text-gray-900">{{ (Math.random() * 10 + 90).toFixed(1) }}%</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Activity Chart Placeholder -->
          <div class="card">
            <h4 class="text-md font-semibold text-gray-900 mb-4">Activity Overview</h4>
            <div class="bg-gray-50 rounded-lg p-8 text-center">
              <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
              </svg>
              <p class="text-gray-600">Activity charts and detailed metrics will be available here</p>
              <p class="text-sm text-gray-500 mt-1">Real-time performance monitoring coming soon</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card {
  @apply bg-white rounded-lg shadow-sm border border-gray-200 p-6;
}

.btn-primary {
  @apply inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}

.btn-secondary {
  @apply inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md shadow-sm text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}
</style>