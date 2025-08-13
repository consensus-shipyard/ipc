<template>
  <div class="subnet-status-indicator">
    <!-- Main status badge -->
    <span :class="getStatusClasses()">
      <component :is="getStatusIcon()" class="w-3 h-3 mr-1" />
      {{ getStatusLabel() }}
    </span>

    <!-- Progress indicator for transitional states -->
    <div v-if="isTransitionalState()" class="mt-2">
      <div class="flex items-center justify-between text-xs text-gray-500 mb-1">
        <span>{{ getProgressLabel() }}</span>
        <span>{{ getProgressPercentage() }}%</span>
      </div>
      <div class="w-full bg-gray-200 rounded-full h-1.5">
        <div
          class="bg-blue-600 h-1.5 rounded-full transition-all duration-300"
          :style="{ width: `${getProgressPercentage()}%` }"
        ></div>
      </div>
    </div>

    <!-- Additional context for states that need action -->
    <div v-if="needsAction()" class="mt-2 p-2 bg-blue-50 border border-blue-200 rounded-lg">
      <div class="flex items-start space-x-2">
        <svg class="w-4 h-4 text-blue-600 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <div class="text-sm text-blue-800">
          <p class="font-medium">{{ getActionTitle() }}</p>
          <p class="mt-1">{{ subnet.status_info.next_action_required }}</p>

          <!-- Action buttons for different states -->
                     <div v-if="props.showActionButtons" class="mt-2 space-x-2">
            <button
              v-if="subnet.status === 'waiting_for_validators'"
              @click="$emit('start-validators')"
              class="inline-flex items-center px-2 py-1 text-xs font-medium text-blue-700 bg-blue-100 border border-blue-300 rounded hover:bg-blue-200 transition-colors"
            >
              <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1.586a1 1 0 01.707.293l2.414 2.414a1 1 0 00.707.293H15M9 10v4a2 2 0 002 2h2a2 2 0 002-2v-4M9 10V9a2 2 0 012-2h2a2 2 0 012 2v1"/>
              </svg>
              Setup Validators
            </button>
            <button
              @click="$emit('troubleshoot')"
              class="inline-flex items-center px-2 py-1 text-xs font-medium text-gray-700 bg-gray-100 border border-gray-300 rounded hover:bg-gray-200 transition-colors"
            >
              <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
              Troubleshoot
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Error message display -->
    <div v-if="subnet.status_info.error_message" class="mt-2 p-2 bg-red-50 border border-red-200 rounded-lg">
      <div class="flex items-start space-x-2">
        <svg class="w-4 h-4 text-red-600 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <div class="text-sm">
          <p class="font-medium text-red-800">Error</p>
          <p class="text-red-700 mt-1">{{ subnet.status_info.error_message }}</p>
        </div>
      </div>
    </div>

    <!-- Detailed status information (when expanded) -->
    <div v-if="showDetails" class="mt-3 space-y-2 text-sm border-t border-gray-200 pt-3">
      <div class="flex justify-between">
        <span class="text-gray-500">Genesis Available:</span>
        <span :class="subnet.status_info.genesis_available ? 'text-green-600' : 'text-gray-400'">
          {{ subnet.status_info.genesis_available ? 'Yes' : 'No' }}
        </span>
      </div>
      <div class="flex justify-between">
        <span class="text-gray-500">Validators:</span>
        <span>{{ subnet.status_info.active_validators }} / {{ subnet.status_info.validator_count }}</span>
      </div>
      <div v-if="subnet.status_info.permission_mode" class="flex justify-between">
        <span class="text-gray-500">Permission Mode:</span>
        <span class="capitalize">{{ subnet.status_info.permission_mode }}</span>
      </div>
      <div v-if="subnet.status_info.last_block_time" class="flex justify-between">
        <span class="text-gray-500">Last Block:</span>
        <span class="text-xs">{{ formatTime(subnet.status_info.last_block_time) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SubnetInstance, SubnetLifecycleState } from '@/stores/subnets'
import { computed } from 'vue'

interface Props {
  subnet: SubnetInstance
  showDetails?: boolean
  showActionButtons?: boolean
}

interface Emits {
  (e: 'start-validators'): void
  (e: 'troubleshoot'): void
  (e: 'refresh'): void
}

const props = withDefaults(defineProps<Props>(), {
  showDetails: false,
  showActionButtons: true
})

const emit = defineEmits<Emits>()

const getStatusClasses = () => {
  const baseClasses = 'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium'

  switch (props.subnet.status) {
    case 'deploying':
    case 'initializing':
      return `${baseClasses} bg-yellow-100 text-yellow-800 animate-pulse`
    case 'waiting_for_validators':
      return `${baseClasses} bg-blue-100 text-blue-800`
    case 'healthy':
      return `${baseClasses} bg-green-100 text-green-800`
    case 'active':
    case 'syncing':
      return `${baseClasses} bg-green-100 text-green-700`
    case 'degraded':
      return `${baseClasses} bg-orange-100 text-orange-800`
    case 'offline':
    case 'failed':
      return `${baseClasses} bg-red-100 text-red-800`
    default:
      return `${baseClasses} bg-gray-100 text-gray-800`
  }
}

const getStatusLabel = () => {
  const labels: Record<SubnetLifecycleState, string> = {
    deploying: 'Deploying',
    deployed: 'Deployed',
    initializing: 'Initializing',
    waiting_for_validators: 'Awaiting Validators',
    active: 'Active',
    syncing: 'Syncing',
    healthy: 'Healthy',
    degraded: 'Degraded',
    offline: 'Offline',
    failed: 'Failed',
    unknown: 'Unknown'
  }
  return labels[props.subnet.status] || props.subnet.status
}

const getStatusIcon = () => {
  switch (props.subnet.status) {
    case 'deploying':
    case 'initializing':
      return 'svg' // Will be replaced with spinning icon
    case 'waiting_for_validators':
      return 'svg' // Clock icon
    case 'healthy':
      return 'svg' // Check circle
    case 'active':
    case 'syncing':
      return 'svg' // Play circle
    case 'degraded':
      return 'svg' // Warning
    case 'offline':
    case 'failed':
      return 'svg' // X circle
    default:
      return 'svg' // Question mark
  }
}

const isTransitionalState = () => {
  return ['deploying', 'initializing', 'syncing'].includes(props.subnet.status)
}

const getProgressLabel = () => {
  switch (props.subnet.status) {
    case 'deploying': return 'Deploying contracts...'
    case 'initializing': return 'Initializing subnet...'
    case 'syncing': return 'Syncing with network...'
    default: return 'Processing...'
  }
}

const getProgressPercentage = () => {
  // This could be enhanced with real progress data from the backend
  switch (props.subnet.status) {
    case 'deploying': return 25
    case 'initializing': return 60
    case 'syncing': return 85
    default: return 0
  }
}

const needsAction = () => {
  return ['waiting_for_validators', 'offline', 'failed', 'degraded'].includes(props.subnet.status)
}

const getActionTitle = () => {
  const titles: Record<string, string> = {
    waiting_for_validators: 'Ready for Validators',
    offline: 'Validators Offline',
    failed: 'Deployment Issue',
    degraded: 'Partial Validator Outage'
  }
  return titles[props.subnet.status] || 'Action Required'
}

const formatTime = (timestamp: string) => {
  try {
    return new Date(timestamp).toLocaleString()
  } catch {
    return timestamp
  }
}

// Define icons as computed properties for better type safety
const SpinningIcon = computed(() => 'svg')
const ClockIcon = computed(() => 'svg')
const CheckCircleIcon = computed(() => 'svg')
const PlayCircleIcon = computed(() => 'svg')
const WarningIcon = computed(() => 'svg')
const XCircleIcon = computed(() => 'svg')
const QuestionMarkIcon = computed(() => 'svg')
</script>

<style scoped>
.subnet-status-indicator {
  @apply space-y-1;
}

.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: .5;
  }
}
</style>