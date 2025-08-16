<script setup lang="ts">
import type { SubnetInstance } from '@/types/subnet'

interface Props {
  instance: SubnetInstance | null
  approvingSubnet?: boolean
}

interface Emits {
  (e: 'approve'): void
  (e: 'pause'): void
  (e: 'resume'): void
  (e: 'testTx'): void
  (e: 'viewLogs'): void
  (e: 'exportConfig'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const status = () => props.instance?.data?.status || props.instance?.status || ''
</script>

<template>
  <div class="flex flex-wrap gap-3 mb-6">
    <button
      v-if="status() === 'active'"
      @click="emit('pause')"
      class="btn-secondary flex items-center"
    >
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      Pause Subnet
    </button>

    <button
      v-else-if="status() === 'paused'"
      @click="emit('resume')"
      class="btn-primary flex items-center"
    >
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1m4 0h1m-6 4h1m4 0h1M9 16h6" />
      </svg>
      Resume Subnet
    </button>

    <button
      v-if="status().toLowerCase() === 'pending approval'"
      :disabled="approvingSubnet"
      @click="emit('approve')"
      class="btn-primary flex items-center"
    >
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
      {{ approvingSubnet ? 'Approving...' : 'Approve Subnet' }}
    </button>

    <button
      v-if="status().toLowerCase() === 'active'"
      @click="emit('testTx')"
      class="btn-primary flex items-center"
    >
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
      </svg>
      Send Test Transaction
    </button>

    <button
      @click="emit('viewLogs')"
      class="btn-secondary flex items-center"
    >
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      View Logs
    </button>

    <button
      @click="emit('exportConfig')"
      class="btn-secondary flex items-center"
    >
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      Export Config
    </button>
  </div>
</template>

<style scoped>
.btn-primary {
  @apply inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}

.btn-secondary {
  @apply inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md shadow-sm text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}
</style>
