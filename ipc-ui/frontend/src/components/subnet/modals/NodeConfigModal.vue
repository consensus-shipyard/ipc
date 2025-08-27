<script setup lang="ts">
import type { NodeConfigData } from '@/types/subnet'
import { copyToClipboard } from '@/utils/clipboard'
import { computed } from 'vue'

interface Props {
  modelValue: boolean
  configData: NodeConfigData | null
  loading?: boolean
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const show = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const close = () => {
  show.value = false
}

const copyConfig = async () => {
  if (!props.configData) return

  const success = await copyToClipboard(props.configData.configYaml)
  if (success) {
    console.log('Node configuration copied to clipboard')
  }
}

const downloadConfig = () => {
  if (!props.configData) return

  const blob = new Blob([props.configData.configYaml], { type: 'text/yaml' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = props.configData.filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
    <div class="relative top-10 mx-auto p-5 border w-11/12 max-w-4xl shadow-lg rounded-md bg-white">
      <div class="mt-3">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-medium text-gray-900">
            Node Configuration for {{ configData?.validatorAddress || 'Validator' }}
          </h3>
          <button
            @click="close"
            class="text-gray-400 hover:text-gray-600"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div v-if="loading" class="flex items-center justify-center py-8">
          <div class="animate-spin inline-block w-8 h-8 border-4 border-current border-t-transparent rounded-full text-blue-600"></div>
          <span class="ml-3 text-gray-600">Generating node configuration...</span>
        </div>

        <div v-else-if="configData" class="space-y-6">
          <!-- Configuration File Section -->
          <div class="bg-gray-50 rounded-lg p-4">
            <div class="flex items-center justify-between mb-3">
              <h4 class="text-md font-semibold text-gray-900">Node Configuration File</h4>
              <div class="flex space-x-2">
                <button
                  @click="copyConfig"
                  class="btn-secondary text-sm"
                  title="Copy to clipboard"
                >
                  <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  Copy
                </button>
                <button
                  @click="downloadConfig"
                  class="btn-primary text-sm"
                  title="Download file"
                >
                  <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-4-4m4 4l4-4m3 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  Download
                </button>
              </div>
            </div>
            <p class="text-sm text-gray-600 mb-3">
              Save this configuration as <code class="bg-gray-200 px-1 rounded">{{ configData.filename }}</code>
            </p>
            <pre class="bg-white border rounded p-3 text-sm overflow-x-auto max-h-64"><code>{{ configData.configYaml }}</code></pre>
          </div>

          <!-- Commands Section -->
          <div class="bg-blue-50 rounded-lg p-4">
            <h4 class="text-md font-semibold text-gray-900 mb-3">Setup Commands</h4>
            <p class="text-sm text-gray-600 mb-4">
              Run these commands in order to set up and start your validator node:
            </p>

            <div class="space-y-4">
              <div v-for="command in configData.commands.commands" :key="command.step" class="bg-white rounded border p-3">
                <div class="flex items-start justify-between mb-2">
                  <div>
                    <h5 class="font-medium text-gray-900">Step {{ command.step }}: {{ command.title }}</h5>
                    <p class="text-sm text-gray-600">{{ command.description }}</p>
                  </div>
                  <span v-if="command.required" class="inline-flex px-2 py-1 text-xs font-medium rounded-full bg-red-100 text-red-800">
                    Required
                  </span>
                </div>
                <pre class="bg-gray-100 rounded p-2 text-sm overflow-x-auto"><code>{{ command.command }}</code></pre>
                <p v-if="command.condition" class="text-xs text-gray-500 mt-1">{{ command.condition }}</p>
              </div>
            </div>

            <!-- Prerequisites -->
            <div class="mt-4 p-3 bg-yellow-50 border border-yellow-200 rounded">
              <h5 class="font-medium text-yellow-800 mb-2">Prerequisites:</h5>
              <ul class="text-sm text-yellow-700 space-y-1">
                <li v-for="prerequisite in configData.commands.prerequisites" :key="prerequisite" class="flex items-start">
                  <span class="mr-2">•</span>
                  <span>{{ prerequisite }}</span>
                </li>
              </ul>
            </div>

            <!-- Important Notes -->
            <div class="mt-4 p-3 bg-orange-50 border border-orange-200 rounded">
              <h5 class="font-medium text-orange-800 mb-2">Important Notes:</h5>
              <ul class="text-sm text-orange-700 space-y-1">
                <li v-for="note in configData.commands.notes" :key="note" class="flex items-start">
                  <span class="mr-2">⚠️</span>
                  <span>{{ note }}</span>
                </li>
              </ul>
            </div>
          </div>
        </div>

        <div class="flex justify-end pt-4">
          <button
            @click="close"
            class="btn-secondary"
          >
            Close
          </button>
        </div>
      </div>
    </div>
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
