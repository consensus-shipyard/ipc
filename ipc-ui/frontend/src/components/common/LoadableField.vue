<template>
  <div class="loadable-field" :class="fieldClass">
    <span v-if="label" class="field-label">{{ label }}:</span>
    <span class="field-value">
      <ProgressiveLoader v-if="isLoading" :text="loadingText" :show-text="false" />
      <span v-else-if="error" class="error-text">{{ error }}</span>
      <slot v-else>
        <span :class="valueClass">{{ displayValue }}</span>
      </slot>
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import ProgressiveLoader from './ProgressiveLoader.vue'

interface Props {
  label?: string
  value?: any
  isLoading?: boolean
  error?: string | null
  loadingText?: string
  defaultValue?: string
  valueClass?: string
  fieldClass?: string
  formatter?: (value: any) => string
}

const props = withDefaults(defineProps<Props>(), {
  isLoading: false,
  error: null,
  loadingText: 'Loading',
  defaultValue: '—',
  valueClass: '',
  fieldClass: ''
})

const displayValue = computed(() => {
  if (props.value === null || props.value === undefined) {
    return props.defaultValue
  }

  if (props.formatter) {
    return props.formatter(props.value)
  }

  return String(props.value)
})
</script>

<style scoped>
.loadable-field {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
}

.field-label {
  font-weight: 500;
  color: #4b5563; /* text-gray-600 */
}

.field-value {
  color: #1f2937; /* text-gray-900 */
}

.error-text {
  color: #dc2626; /* text-red-600 */
  font-size: 0.875rem;
}
</style>
