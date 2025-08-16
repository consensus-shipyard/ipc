<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  modelValue?: string | number
  label?: string
  placeholder?: string
  type?: string
  required?: boolean
  error?: string
  helpText?: string
  disabled?: boolean
  suffix?: string
  prefix?: string
}

interface Emits {
  (e: 'update:modelValue', value: string | number): void
  (e: 'blur'): void
  (e: 'focus'): void
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  required: false,
  disabled: false
})

const emit = defineEmits<Emits>()

const inputClasses = computed(() => [
  'input-field',
  {
    'border-red-300 focus:border-red-500 focus:ring-red-500': props.error,
    'bg-gray-50': props.disabled,
    'pl-10': props.prefix,
    'pr-10': props.suffix
  }
])

const handleInput = (event: Event) => {
  const target = event.target as HTMLInputElement
  const value = props.type === 'number' ? parseFloat(target.value) || 0 : target.value
  emit('update:modelValue', value)
}
</script>

<template>
  <div class="space-y-1">
    <!-- Label -->
    <label v-if="label" class="block text-sm font-medium text-gray-700">
      {{ label }}
      <span v-if="required" class="text-red-500">*</span>
    </label>

    <!-- Input Container -->
    <div class="relative">
      <!-- Prefix -->
      <div v-if="prefix" class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
        <span class="text-gray-500 text-sm">{{ prefix }}</span>
      </div>

      <!-- Input Field -->
      <input
        :value="modelValue"
        :type="type"
        :placeholder="placeholder"
        :disabled="disabled"
        :class="inputClasses"
        @input="handleInput"
        @blur="$emit('blur')"
        @focus="$emit('focus')"
      />

      <!-- Suffix -->
      <div v-if="suffix" class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
        <span class="text-gray-500 text-sm">{{ suffix }}</span>
      </div>

      <!-- Error Icon -->
      <div v-if="error" class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
        <svg class="w-5 h-5 text-red-500" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
        </svg>
      </div>
    </div>

    <!-- Error Message -->
    <p v-if="error" class="text-sm text-red-600">
      {{ error }}
    </p>

    <!-- Help Text -->
    <p v-if="helpText && !error" class="text-sm text-gray-500">
      {{ helpText }}
    </p>
  </div>
</template>