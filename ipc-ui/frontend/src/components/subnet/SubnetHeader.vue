<script setup lang="ts">
import type { SubnetInstance } from '@/types/subnet'
import { useRouter } from 'vue-router'

interface Props {
  instance: SubnetInstance | null
  subnetId: string
  statusColor: string
}

const props = defineProps<Props>()
const router = useRouter()

const goBack = () => {
  router.push('/')
}

const displayStatus = () => {
  const status = props.instance?.data?.status || props.instance?.status || 'Unknown'
  return status.charAt(0).toUpperCase() + status.slice(1)
}
</script>

<template>
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
              {{ instance?.data?.name || instance?.name || 'Loading...' }}
            </h1>
            <p class="text-gray-600 mt-1">Subnet ID: {{ decodeURIComponent(subnetId) }}</p>
          </div>
        </div>

        <div v-if="instance" class="flex items-center space-x-3">
          <span
            :class="[
              'inline-flex items-center px-3 py-1 rounded-full text-sm font-medium',
              statusColor
            ]"
          >
            {{ displayStatus() }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
