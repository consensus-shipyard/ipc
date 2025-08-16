<script setup lang="ts">
import type { SubnetInstance } from '@/types/subnet'
import { formatAddress, formatAddressShort } from '@/utils/address'

interface Props {
  instance: SubnetInstance | null
  copyingAddress: string | null
}

interface Emits {
  (e: 'copyToClipboard', text: string, type: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Helper to format config key names
const formatConfigKey = (key: string | number | symbol): string => {
  const keyStr = String(key)
  return keyStr.replace(/([A-Z])/g, ' $1').replace(/^./, (str) => str.toUpperCase())
}

// Helper to determine if a value is an address
const isAddress = (key: string | number | symbol): boolean => {
  const keyStr = String(key)
  return keyStr === 'gateway_addr' || keyStr === 'registry_addr'
}
</script>

<template>
  <div class="space-y-6">
    <div class="card">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">Configuration Details</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div v-for="(value, key) in instance?.data?.config" :key="key" class="flex justify-between py-2 border-b border-gray-100">
          <dt class="text-sm font-medium text-gray-500 capitalize">
            {{ formatConfigKey(key) }}
          </dt>
          <dd class="text-sm text-gray-900">
            <span v-if="typeof value === 'boolean'" :class="value ? 'text-green-600' : 'text-red-600'">
              {{ value ? 'Yes' : 'No' }}
            </span>
            <button
              v-else-if="isAddress(key)"
              @click="emit('copyToClipboard', formatAddress(value), String(key))"
              class="font-mono hover:bg-gray-100 px-2 py-1 rounded transition-colors cursor-pointer text-left"
              :title="copyingAddress === String(key) ? 'Copied!' : `Click to copy: ${formatAddress(value)}`"
            >
              {{ formatAddressShort(value) }}
              <svg v-if="copyingAddress === String(key)" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            </button>
            <span v-else-if="typeof value === 'string' && value.startsWith('0x')" class="font-mono">
              {{ value.slice(0, 8) }}...{{ value.slice(-6) }}
            </span>
            <span v-else>{{ value }}</span>
          </dd>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card {
  @apply bg-white rounded-lg shadow-sm border border-gray-200 p-6;
}
</style>
