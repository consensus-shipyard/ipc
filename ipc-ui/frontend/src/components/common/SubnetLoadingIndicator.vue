<template>
  <div
    v-if="isLoading || hasError"
    class="absolute inset-0 bg-white bg-opacity-90 flex items-center justify-center z-10 rounded-lg"
  >
    <!-- Loading State -->
    <div v-if="isLoading" class="text-center">
      <div class="animate-spin inline-block w-6 h-6 border-3 border-primary-600 border-t-transparent rounded-full mb-2"></div>
      <p class="text-sm text-gray-600">{{ loadingText }}</p>
    </div>

    <!-- Error State -->
    <div v-else-if="hasError" class="text-center p-4">
      <svg class="w-8 h-8 text-red-600 mx-auto mb-2" fill="currentColor" viewBox="0 0 20 20">
        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
      </svg>
      <p class="text-sm text-red-700 mb-3">{{ errorMessage }}</p>
      <button
        @click="$emit('retry')"
        class="text-xs bg-red-100 hover:bg-red-200 text-red-700 px-3 py-1 rounded-md transition-colors"
      >
        Retry
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  isLoading?: boolean
  hasError?: boolean
  errorMessage?: string
  loadingText?: string
}

const props = withDefaults(defineProps<Props>(), {
  isLoading: false,
  hasError: false,
  errorMessage: 'Failed to load subnet data',
  loadingText: 'Loading...'
})

defineEmits<{
  retry: []
}>()
</script>