<script setup lang="ts">
import type { SubnetInstance } from '@/types/subnet'
import { computed } from 'vue'

interface Props {
  modelValue: string
  instance: SubnetInstance | null
}

interface Emits {
  (e: 'update:modelValue', value: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const activeTab = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const validatorCount = computed(() => {
  return props.instance?.data?.validator_count ||
         props.instance?.data?.validators?.length ||
         props.instance?.validators?.length ||
         0
})

const tabs = [
  { id: 'overview', label: 'Overview' },
  { id: 'validators', label: computed(() => `Validators (${validatorCount.value})`) },
  { id: 'configuration', label: 'Configuration' },
  { id: 'contracts', label: 'Contracts' },
  { id: 'metrics', label: 'Metrics' }
]
</script>

<template>
  <div class="border-b border-gray-200 mb-6">
    <nav class="flex space-x-8">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        @click="activeTab = tab.id"
        :class="[
          'py-2 px-1 border-b-2 font-medium text-sm',
          activeTab === tab.id
            ? 'border-primary-500 text-primary-600'
            : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
        ]"
      >
        {{ typeof tab.label === 'string' ? tab.label : tab.label.value }}
      </button>
    </nav>
  </div>
</template>
