<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useTemplatesStore } from '../../stores/templates'
import { useWizardStore } from '../../stores/wizard'

const router = useRouter()
const wizardStore = useWizardStore()
const templatesStore = useTemplatesStore()

// Standard deployment steps
const deploymentSteps = [
  { id: 'validate', name: 'Validating Configuration', status: 'pending' },
  { id: 'prepare', name: 'Preparing Deployment Files', status: 'pending' },
  { id: 'contracts', name: 'Deploying Smart Contracts', status: 'pending' },
  { id: 'genesis', name: 'Creating Genesis Block', status: 'pending' },
  { id: 'validators', name: 'Initializing Validators', status: 'pending' },
  { id: 'activation', name: 'Activating Subnet', status: 'pending' },
  { id: 'verification', name: 'Running Verification', status: 'pending' }
]

// Real deployment state from wizard store
const deploymentId = computed(() => wizardStore.deploymentId)
const deploymentProgress = computed(() => wizardStore.deploymentProgress)
const deploymentLogs = computed(() => wizardStore.deploymentLogs)
const deploymentError = computed(() => wizardStore.deploymentError)
const isDeploying = computed(() => wizardStore.isDeploying)
const startTime = ref(new Date())

// Current step based on deployment progress
const currentStep = computed(() => {
  if (!deploymentProgress.value) return 0

  const stepIndex = deploymentSteps.findIndex(step =>
    step.id === deploymentProgress.value?.step
  )
  return stepIndex >= 0 ? stepIndex : 0
})

// Get configuration summary
const config = computed(() => wizardStore.config)
const selectedTemplate = computed(() => {
  return config.value.selectedTemplate
    ? templatesStore.getTemplate(config.value.selectedTemplate)
    : null
})

// Get step status based on deployment progress
const getStepStatus = (stepId: string, index: number) => {
  if (deploymentError.value) {
    // If there's an error and this step matches the current progress, it failed
    if (deploymentProgress.value?.step === stepId) {
      return 'error'
    }
    // Steps before the failed step should be completed
    return index < currentStep.value ? 'completed' : 'pending'
  }

  if (!deploymentProgress.value) {
    return index === 0 ? 'in_progress' : 'pending'
  }

  if (index < currentStep.value) {
    return 'completed'
  } else if (index === currentStep.value) {
    return deploymentProgress.value.status === 'completed' ? 'completed' : 'in_progress'
  } else {
    return 'pending'
  }
}

const getStepIcon = (stepId: string, index: number) => {
  const status = getStepStatus(stepId, index)
  switch (status) {
    case 'completed':
      return '✅'
    case 'in_progress':
      return '⏳'
    case 'error':
      return '❌'
    default:
      return '⏸️'
  }
}

const getStepColor = (stepId: string, index: number) => {
  const status = getStepStatus(stepId, index)
  switch (status) {
    case 'completed':
      return 'text-green-600 bg-green-50'
    case 'in_progress':
      return 'text-blue-600 bg-blue-50'
    case 'error':
      return 'text-red-600 bg-red-50'
    default:
      return 'text-gray-500 bg-gray-50'
  }
}

const isDeploymentComplete = computed(() => {
  return deploymentProgress.value?.status === 'completed' && !isDeploying.value
})

const hasDeploymentError = computed(() => {
  return !!deploymentError.value
})

const goToDashboard = () => {
  wizardStore.resetWizard()
  router.push({ name: 'dashboard' })
}

onMounted(async () => {
  // Initialize WebSocket connection for real-time progress updates
  if (!wizardStore.isConnected) {
    await wizardStore.initializeWebSocket()
  }

  // If no deployment is in progress, redirect back to review
  if (!isDeploying.value && !deploymentId.value) {
    console.warn('No deployment in progress, redirecting to review')
    router.push({ name: 'wizard-review' })
  }
})

onUnmounted(() => {
  // WebSocket cleanup is handled by the store
})
</script>

<template>
  <div class="space-y-8">
    <!-- Header -->
    <div class="text-center">
      <h2 class="text-3xl font-bold text-gray-900 mb-2">Subnet Deployment</h2>
      <p class="text-gray-600 text-lg">
        Your subnet is being deployed. Please wait...
      </p>
    </div>

    <!-- Deployment Info -->
    <div class="bg-gradient-to-r from-primary-50 to-blue-50 border border-primary-200 rounded-lg p-6">
      <div class="flex items-start space-x-4">
        <div class="text-4xl">{{ selectedTemplate?.icon || '🚀' }}</div>
        <div class="flex-1">
          <h3 class="font-semibold text-primary-800 text-xl mb-2">Deploying {{ selectedTemplate?.name || 'Subnet' }}</h3>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm text-primary-700">
            <div>
              <span class="font-semibold">Deployment ID:</span><br>
              <span class="font-mono">{{ deploymentId }}</span>
            </div>
            <div>
              <span class="font-semibold">Started:</span><br>
              {{ startTime.toLocaleTimeString() }}
            </div>
            <div>
              <span class="font-semibold">Parent Network:</span><br>
              {{ config.parent }}
            </div>
            <div>
              <span class="font-semibold">Mode:</span><br>
              {{ config.permissionMode }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Deployment Steps -->
    <div class="card">
      <h3 class="text-xl font-semibold text-gray-800 mb-6">Deployment Progress</h3>

      <div class="space-y-4">
        <div
          v-for="(step, index) in deploymentSteps"
          :key="step.id"
          :class="[
            'flex items-center p-4 rounded-lg transition-colors',
            getStepColor(step.id, index)
          ]"
        >
          <div class="flex items-center flex-1 space-x-4">
            <!-- Step Number/Icon -->
            <div class="flex-shrink-0 w-8 h-8 flex items-center justify-center rounded-full bg-white text-sm font-semibold">
              <span v-if="getStepStatus(step.id, index) === 'completed'">✓</span>
              <div v-else-if="getStepStatus(step.id, index) === 'in_progress'" class="animate-spin w-4 h-4 border-2 border-current border-t-transparent rounded-full"></div>
              <span v-else-if="getStepStatus(step.id, index) === 'error'">✗</span>
              <span v-else>{{ index + 1 }}</span>
            </div>

            <!-- Step Info -->
            <div class="flex-1">
              <div class="font-semibold">{{ step.name }}</div>
              <div v-if="step.status === 'in_progress'" class="text-sm opacity-75">
                This may take a few minutes...
              </div>
              <div v-else-if="step.status === 'completed'" class="text-sm opacity-75">
                Completed successfully
              </div>
            </div>

            <!-- Status -->
            <div class="text-2xl">
              {{ getStepIcon(step.id, index) }}
            </div>
          </div>
        </div>
      </div>

      <!-- Progress Bar -->
      <div class="mt-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-gray-600">Overall Progress</span>
          <span class="text-sm font-semibold text-gray-900">
            {{ Math.round(((currentStep + (deploymentSteps[currentStep]?.status === 'completed' ? 1 : 0)) / deploymentSteps.length) * 100) }}%
          </span>
        </div>
        <div class="w-full bg-gray-200 rounded-full h-2">
          <div
            class="bg-primary-600 h-2 rounded-full transition-all duration-1000 ease-out"
            :style="{ width: `${((currentStep + (deploymentSteps[currentStep]?.status === 'completed' ? 1 : 0)) / deploymentSteps.length) * 100}%` }"
          ></div>
        </div>
      </div>
    </div>

    <!-- Completion Status -->
    <div v-if="isDeploymentComplete" class="bg-green-50 border border-green-200 rounded-lg p-6">
      <div class="flex items-start space-x-4">
        <svg class="w-8 h-8 text-green-600 mt-1" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
        </svg>
        <div class="flex-1">
          <h3 class="text-xl font-semibold text-green-800 mb-2">🎉 Deployment Successful!</h3>
          <p class="text-green-700 mb-4">
            Your subnet has been successfully deployed and is now active. You can now manage it from your dashboard.
          </p>
          <div class="space-y-2 text-sm text-green-700">
            <div><strong>Subnet ID:</strong> <span class="font-mono">{{ deploymentId }}</span></div>
            <div><strong>Network Path:</strong> <span class="font-mono">{{ config.parent }}/{{ deploymentId }}</span></div>
            <div><strong>Status:</strong> Active</div>
            <div><strong>Validators:</strong> {{ config.validators?.length || config.validatorPubkeys?.length || 0 }} initialized</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex justify-center pt-6 border-t border-gray-200">
      <button
        v-if="!isDeploymentComplete"
        type="button"
        disabled
        class="btn-secondary opacity-50 cursor-not-allowed"
      >
        <svg class="w-4 h-4 mr-2 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        Deployment in Progress...
      </button>

      <button
        v-else
        type="button"
        @click="goToDashboard"
        class="btn-primary text-lg px-8 py-3"
      >
        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
        </svg>
        Go to Dashboard
      </button>
    </div>

    <!-- Important Notes -->
    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
      <div class="flex items-start space-x-3">
        <svg class="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
        </svg>
        <div>
          <h3 class="font-semibold text-blue-800 mb-1">Please Note</h3>
          <div class="text-blue-700 text-sm space-y-1">
            <p>• This is a demonstration of the deployment process</p>
            <p>• In the real implementation, this would connect to the backend service</p>
            <p>• WebSocket connections would provide real-time progress updates</p>
            <p>• Error handling and retry mechanisms would be available</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>