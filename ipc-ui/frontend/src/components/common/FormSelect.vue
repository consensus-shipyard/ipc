<script setup lang="ts">
import { computed } from 'vue'

interface Option {
  value: string | number
  label: string
  description?: string
  disabled?: boolean
}

interface Props {
  modelValue?: string | number
  label?: string
  placeholder?: string
  options: Option[]
  required?: boolean
  error?: string
  helpText?: string
  disabled?: boolean
}

interface Emits {
  (e: 'update:modelValue', value: string | number): void
  (e: 'change', value: string | number): void
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: 'Select an option...',
  required: false,
  disabled: false
})

const emit = defineEmits<Emits>()

const selectClasses = computed(() => [
  'input-field appearance-none bg-white',
  {
    'border-red-300 focus:border-red-500 focus:ring-red-500': props.error,
    'bg-gray-50': props.disabled
  }
])

const selectedOption = computed(() => {
  return props.options.find(option => option.value === props.modelValue)
})

const handleChange = (event: Event) => {
  const target = event.target as HTMLSelectElement
  const value = target.value
  emit('update:modelValue', value)
  emit('change', value)
}
</script>

<template>
  <div class="space-y-1">
    <!-- Label -->
    <label v-if="label" class="block text-sm font-medium text-gray-700">
      {{ label }}
      <span v-if="required" class="text-red-500">*</span>
    </label>

    <!-- Select Container -->
    <div class="relative">
      <!-- Select Field -->
      <select
        :value="modelValue"
        :disabled="disabled"
        :class="selectClasses"
        @change="handleChange"
      >
        <option value="" disabled>{{ placeholder }}</option>
        <option
          v-for="option in options"
          :key="option.value"
          :value="option.value"
          :disabled="option.disabled"
        >
          {{ option.label }}
        </option>
      </select>

      <!-- Dropdown Arrow -->
      <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
        <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </div>

      <!-- Error Icon -->
      <div v-if="error" class="absolute inset-y-0 right-8 flex items-center pr-3 pointer-events-none">
        <svg class="w-5 h-5 text-red-500" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
        </svg>
      </div>
    </div>

    <!-- Selected Option Description -->
    <p v-if="selectedOption?.description && !error" class="text-sm text-blue-600">
      {{ selectedOption.description }}
    </p>

    <!-- Error Message -->
    <p v-if="error" class="text-sm text-red-600">
      {{ error }}
    </p>

    <!-- Help Text -->
    <p v-if="helpText && !error && !selectedOption?.description" class="text-sm text-gray-500">
      {{ helpText }}
    </p>
  </div>
</template>