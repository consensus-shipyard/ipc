<template>
  <span v-if="isLoading" class="inline-flex items-center">
    <div class="animate-spin w-3 h-3 border-2 border-gray-300 border-t-primary-600 rounded-full mr-1"></div>
    <span class="text-sm text-gray-500">{{ loadingText }}</span>
  </span>
  <span v-else-if="hasError" class="inline-flex items-center text-red-600">
    <svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
      <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
    </svg>
    <span class="text-xs">Error</span>
    <button
      v-if="canRetry"
      @click="$emit('retry')"
      class="ml-1 text-xs underline hover:no-underline"
    >
      Retry
    </button>
  </span>
  <span v-else>
    <slot>{{ fallbackValue }}</slot>
  </span>
</template>

<script setup lang="ts">
interface Props {
  isLoading?: boolean
  hasError?: boolean
  loadingText?: string
  fallbackValue?: string
  canRetry?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isLoading: false,
  hasError: false,
  loadingText: 'Loading...',
  fallbackValue: 'N/A',
  canRetry: true
})

defineEmits<{
  retry: []
}>()
</script>