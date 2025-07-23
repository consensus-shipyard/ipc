<script setup lang="ts">
import { computed, ref, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useWizardStore } from '../../stores/wizard'
import { useTemplatesStore } from '../../stores/templates'
import FormInput from '../../components/common/FormInput.vue'
import FormSelect from '../../components/common/FormSelect.vue'

const router = useRouter()
const wizardStore = useWizardStore()
const templatesStore = useTemplatesStore()

// Form state
const formData = ref({
  // Network Settings
  activeValidatorsLimit: wizardStore.config.activeValidatorsLimit || undefined,

  // Validator Management
  validatorGater: wizardStore.config.validatorGater || '',
  validatorRewarder: wizardStore.config.validatorRewarder || '',

  // Economic Parameters
  collateralSourceKind: wizardStore.config.collateralSourceKind || 'native',
  collateralSourceAddress: wizardStore.config.collateralSourceAddress || '',

  // Genesis Parameters
  networkVersion: wizardStore.config.networkVersion || 21,
  baseFee: wizardStore.config.baseFee || 1000,
  powerScale: wizardStore.config.powerScale || 3
})

// Collapsible sections state
const expandedSections = ref<Record<string, boolean>>({
  network: false,
  validators: false,
  economic: false,
  genesis: false
})

// Field errors
const fieldErrors = ref<Record<string, string>>({})

// Get selected template info
const selectedTemplate = computed(() => {
  return wizardStore.config.selectedTemplate
    ? templatesStore.getTemplate(wizardStore.config.selectedTemplate)
    : null
})

// Dropdown options
const collateralSourceOptions = [
  { value: 'native', label: 'Native', description: 'Use native FIL tokens for collateral' },
  { value: 'erc20', label: 'ERC-20', description: 'Use custom ERC-20 token contract for collateral' }
]

const networkVersionOptions = Array.from({ length: 10 }, (_, i) => ({
  value: 18 + i,
  label: `Network Version ${18 + i}`,
  description: i === 3 ? 'Current recommended version' : ''
}))

const powerScaleOptions = [
  { value: 0, label: '0 - Direct conversion (1 FIL = 1 Power)', description: 'No scaling applied' },
  { value: 3, label: '3 - Standard scaling (default)', description: 'Recommended for most networks' },
  { value: 6, label: '6 - High precision scaling', description: 'For very large networks' },
  { value: 9, label: '9 - Maximum precision scaling', description: 'For enterprise deployments' }
]

// Validation
const validateField = (field: string) => {
  const value = formData.value[field as keyof typeof formData.value]
  const error = wizardStore.validateField(field, value)

  if (error) {
    fieldErrors.value[field] = error
  } else {
    delete fieldErrors.value[field]
  }
}

const validateForm = () => {
  // Clear existing errors
  fieldErrors.value = {}

  // Conditional validations
  if (formData.value.collateralSourceKind === 'erc20' && formData.value.collateralSourceAddress) {
    validateField('collateralSourceAddress')
  }

  if (formData.value.validatorGater) {
    validateField('validatorGater')
  }

  if (formData.value.validatorRewarder) {
    validateField('validatorRewarder')
  }

  return Object.keys(fieldErrors.value).length === 0
}

// Save configuration to store
const saveConfig = () => {
  // Filter out empty/undefined values
  const cleanData = Object.fromEntries(
    Object.entries(formData.value).filter(([_, value]) =>
      value !== '' && value !== undefined && value !== null
    )
  )
  wizardStore.updateConfig(cleanData)
}

// Navigation
const goToNextStep = () => {
  if (validateForm()) {
    saveConfig()
    router.push({ name: 'wizard-activation' })
  }
}

const goToPreviousStep = () => {
  saveConfig() // Save current progress
  router.push({ name: 'wizard-basic' })
}

// Auto-save on changes
watch(formData, () => {
  saveConfig()
}, { deep: true })

// Real-time field validation
const handleFieldBlur = (field: string) => {
  validateField(field)
}

// Section toggle
const toggleSection = (section: string) => {
  expandedSections.value[section] = !expandedSections.value[section]
}

// Check if any fields in section have values
const hasValuesInSection = (section: string) => {
  switch (section) {
    case 'network':
      return formData.value.activeValidatorsLimit !== undefined
    case 'validators':
      return formData.value.validatorGater || formData.value.validatorRewarder
    case 'economic':
      return formData.value.collateralSourceKind !== 'native' || formData.value.collateralSourceAddress
    case 'genesis':
      return formData.value.networkVersion !== 21 || formData.value.baseFee !== 1000 || formData.value.powerScale !== 3
    default:
      return false
  }
}

// Initialize form on mount
onMounted(() => {
  setTimeout(() => {
    validateForm()
  }, 100)
})
</script>

<template>
  <div class="space-y-8">
    <!-- Template Info -->
    <div v-if="selectedTemplate" class="bg-gradient-to-r from-primary-50 to-blue-50 border border-primary-200 rounded-lg p-6">
      <div class="flex items-start space-x-4">
        <div class="text-3xl">{{ selectedTemplate.icon }}</div>
        <div>
          <h3 class="font-semibold text-primary-800 text-lg mb-1">{{ selectedTemplate.name }}</h3>
          <p class="text-primary-700 text-sm mb-2">{{ selectedTemplate.description }}</p>
          <div class="text-xs text-primary-600">
            Advanced settings are optional. Default values work well for most deployments.
          </div>
        </div>
      </div>
    </div>

    <!-- Form -->
    <div class="card">
      <div class="mb-6">
        <h2 class="text-2xl font-bold text-gray-900 mb-2">Advanced Configuration</h2>
        <p class="text-gray-600">Fine-tune your subnet with optional advanced settings.</p>
      </div>

      <form @submit.prevent="goToNextStep" class="space-y-6">
        <!-- Network Settings Section -->
        <div class="border border-gray-200 rounded-lg">
          <button
            type="button"
            @click="toggleSection('network')"
            class="w-full flex items-center justify-between p-4 text-left hover:bg-gray-50 transition-colors"
          >
            <div class="flex items-center space-x-3">
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9v-9m0-9v9" />
              </svg>
              <h3 class="text-lg font-semibold text-gray-800">Network Settings</h3>
              <span v-if="hasValuesInSection('network')" class="w-2 h-2 bg-primary-500 rounded-full"></span>
            </div>
            <svg
              :class="['w-5 h-5 text-gray-400 transition-transform', expandedSections.network && 'rotate-180']"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>

          <div v-if="expandedSections.network" class="p-4 pt-0 border-t border-gray-100">
            <div class="space-y-4">
              <p class="text-sm text-gray-500 mb-4">
                Configure network-level parameters and limits.
              </p>

              <FormInput
                v-model="formData.activeValidatorsLimit"
                type="number"
                label="Active Validators Limit"
                placeholder="100"
                help-text="Maximum number of active validators (leave empty for no limit)"
                :error="fieldErrors.activeValidatorsLimit"
                @blur="handleFieldBlur('activeValidatorsLimit')"
              />
            </div>
          </div>
        </div>

        <!-- Validator Management Section -->
        <div class="border border-gray-200 rounded-lg">
          <button
            type="button"
            @click="toggleSection('validators')"
            class="w-full flex items-center justify-between p-4 text-left hover:bg-gray-50 transition-colors"
          >
            <div class="flex items-center space-x-3">
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
              </svg>
              <h3 class="text-lg font-semibold text-gray-800">Validator Management</h3>
              <span v-if="hasValuesInSection('validators')" class="w-2 h-2 bg-primary-500 rounded-full"></span>
            </div>
            <svg
              :class="['w-5 h-5 text-gray-400 transition-transform', expandedSections.validators && 'rotate-180']"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>

          <div v-if="expandedSections.validators" class="p-4 pt-0 border-t border-gray-100">
            <div class="space-y-4">
              <p class="text-sm text-gray-500 mb-4">
                Optional contracts for custom validator gating and reward distribution.
              </p>

              <FormInput
                v-model="formData.validatorGater"
                label="Validator Gater Contract"
                placeholder="0x..."
                help-text="Contract address for custom validator admission logic"
                :error="fieldErrors.validatorGater"
                @blur="handleFieldBlur('validatorGater')"
              />

              <FormInput
                v-model="formData.validatorRewarder"
                label="Validator Rewarder Contract"
                placeholder="0x..."
                help-text="Contract address for custom validator reward distribution"
                :error="fieldErrors.validatorRewarder"
                @blur="handleFieldBlur('validatorRewarder')"
              />
            </div>
          </div>
        </div>

        <!-- Economic Parameters Section -->
        <div class="border border-gray-200 rounded-lg">
          <button
            type="button"
            @click="toggleSection('economic')"
            class="w-full flex items-center justify-between p-4 text-left hover:bg-gray-50 transition-colors"
          >
            <div class="flex items-center space-x-3">
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1" />
              </svg>
              <h3 class="text-lg font-semibold text-gray-800">Economic Parameters</h3>
              <span v-if="hasValuesInSection('economic')" class="w-2 h-2 bg-primary-500 rounded-full"></span>
            </div>
            <svg
              :class="['w-5 h-5 text-gray-400 transition-transform', expandedSections.economic && 'rotate-180']"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>

          <div v-if="expandedSections.economic" class="p-4 pt-0 border-t border-gray-100">
            <div class="space-y-4">
              <p class="text-sm text-gray-500 mb-4">
                Configure economic models and collateral sources.
              </p>

              <FormSelect
                v-model="formData.collateralSourceKind"
                label="Collateral Source"
                :options="collateralSourceOptions"
                help-text="Token type for validator collateral"
              />

              <!-- ERC-20 Collateral Address (conditional) -->
              <FormInput
                v-if="formData.collateralSourceKind === 'erc20'"
                v-model="formData.collateralSourceAddress"
                label="Collateral Source Address"
                placeholder="0x..."
                required
                :error="fieldErrors.collateralSourceAddress"
                help-text="ERC-20 contract address for collateral tokens"
                @blur="handleFieldBlur('collateralSourceAddress')"
              />
            </div>
          </div>
        </div>

        <!-- Genesis Parameters Section -->
        <div class="border border-gray-200 rounded-lg">
          <button
            type="button"
            @click="toggleSection('genesis')"
            class="w-full flex items-center justify-between p-4 text-left hover:bg-gray-50 transition-colors"
          >
            <div class="flex items-center space-x-3">
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              </svg>
              <h3 class="text-lg font-semibold text-gray-800">Genesis Parameters</h3>
              <span v-if="hasValuesInSection('genesis')" class="w-2 h-2 bg-primary-500 rounded-full"></span>
            </div>
            <svg
              :class="['w-5 h-5 text-gray-400 transition-transform', expandedSections.genesis && 'rotate-180']"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>

          <div v-if="expandedSections.genesis" class="p-4 pt-0 border-t border-gray-100">
            <div class="space-y-4">
              <p class="text-sm text-gray-500 mb-4">
                Fine-tune network genesis parameters.
              </p>

              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <FormSelect
                  v-model="formData.networkVersion"
                  label="Network Version"
                  :options="networkVersionOptions"
                  help-text="Filecoin network version for built-in actors"
                />

                <FormInput
                  v-model="formData.baseFee"
                  type="number"
                  label="Base Fee"
                  placeholder="1000"
                  suffix="attoFIL"
                  help-text="Base transaction fee in attoFIL"
                />
              </div>

              <FormSelect
                v-model="formData.powerScale"
                label="Power Scale"
                :options="powerScaleOptions"
                help-text="Decimals for FIL to Power conversion"
              />
            </div>
          </div>
        </div>

        <!-- Skip Option -->
        <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div class="flex items-start space-x-3">
            <svg class="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
            </svg>
            <div>
              <h3 class="font-semibold text-blue-800 mb-1">Advanced Settings Are Optional</h3>
              <p class="text-blue-700 text-sm">
                These settings have sensible defaults based on your selected template.
                You can proceed without configuring them or return to adjust them later.
              </p>
            </div>
          </div>
        </div>

        <!-- Navigation -->
        <div class="flex justify-between pt-6 border-t border-gray-200">
          <button
            type="button"
            @click="goToPreviousStep"
            class="btn-secondary"
          >
            Previous
          </button>

          <button
            type="submit"
            class="btn-primary"
          >
            Next: Activation Settings
          </button>
        </div>
      </form>
    </div>
  </div>
</template>