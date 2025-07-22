<script setup lang="ts">
import { computed } from 'vue'
import { RouterView, useRoute } from 'vue-router'

const route = useRoute()

// Wizard steps configuration
const wizardSteps = [
  { name: 'Template', route: 'wizard-template', step: 1 },
  { name: 'Basic Config', route: 'wizard-basic', step: 2 },
  { name: 'Advanced', route: 'wizard-advanced', step: 3 },
  { name: 'Activation', route: 'wizard-activation', step: 4 },
  { name: 'Review', route: 'wizard-review', step: 5 },
  { name: 'Deploy', route: 'wizard-deploy', step: 6 }
]

// Get current step from route meta
const currentStep = computed(() => {
  return route.meta?.step || 1
})

// Calculate progress percentage
const progressPercentage = computed(() => {
  return Math.round((currentStep.value / wizardSteps.length) * 100)
})
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Wizard Header -->
    <div class="bg-white shadow-sm border-b">
      <div class="max-w-4xl mx-auto px-6 py-4">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-2xl font-bold text-gray-900">Deploy New Subnet</h1>
            <p class="text-gray-600 mt-1">Step {{ currentStep }} of {{ wizardSteps.length }}: {{ route.meta?.title }}</p>
          </div>

          <!-- Progress Bar -->
          <div class="flex items-center space-x-4">
            <div class="w-32 bg-gray-200 rounded-full h-2">
              <div
                class="bg-primary-600 h-2 rounded-full transition-all duration-300"
                :style="{ width: progressPercentage + '%' }"
              ></div>
            </div>
            <span class="text-sm font-medium text-gray-600">{{ progressPercentage }}%</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Step Navigation -->
    <div class="bg-white border-b">
      <div class="max-w-4xl mx-auto px-6 py-3">
        <nav class="flex space-x-8">
          <div
            v-for="step in wizardSteps"
            :key="step.step"
            :class="[
              'flex items-center space-x-2 py-2 text-sm font-medium',
              step.step === currentStep
                ? 'text-primary-600 border-b-2 border-primary-600'
                : step.step < currentStep
                  ? 'text-green-600'
                  : 'text-gray-400'
            ]"
          >
            <!-- Step Circle -->
            <div
              :class="[
                'flex items-center justify-center w-6 h-6 rounded-full text-xs font-bold',
                step.step === currentStep
                  ? 'bg-primary-600 text-white'
                  : step.step < currentStep
                    ? 'bg-green-500 text-white'
                    : 'bg-gray-200 text-gray-600'
              ]"
            >
              <svg
                v-if="step.step < currentStep"
                class="w-3 h-3"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
              <span v-else>{{ step.step }}</span>
            </div>

            <!-- Step Name -->
            <span>{{ step.name }}</span>
          </div>
        </nav>
      </div>
    </div>

    <!-- Main Content -->
    <div class="max-w-4xl mx-auto px-6 py-8">
      <RouterView />
    </div>
  </div>
</template>