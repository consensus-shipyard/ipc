<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import FormInput from '../../components/common/FormInput.vue'
import FormSelect from '../../components/common/FormSelect.vue'
import { useTemplatesStore } from '../../stores/templates'
import { useWizardStore } from '../../stores/wizard'

const router = useRouter()
const wizardStore = useWizardStore()
const templatesStore = useTemplatesStore()

// Form state
const formData = ref({
  parent: wizardStore.config.parent || '',
  from: wizardStore.config.from || '',
  minValidatorStake: wizardStore.config.minValidatorStake || 0,
  minValidators: wizardStore.config.minValidators || 1,
  bottomupCheckPeriod: wizardStore.config.bottomupCheckPeriod || 50,
  permissionMode: wizardStore.config.permissionMode || 'collateral',
  supplySourceKind: wizardStore.config.supplySourceKind || 'native',
  supplySourceAddress: wizardStore.config.supplySourceAddress || '',
  minCrossMsgFee: wizardStore.config.minCrossMsgFee || 0.000001,
  genesisSubnetIpcContractsOwner: wizardStore.config.genesisSubnetIpcContractsOwner || ''
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
const parentNetworkOptions = computed(() => templatesStore.parentNetworks)

const permissionModeOptions = [
  { value: 'collateral', label: 'Collateral', description: 'Validators must stake collateral to participate' },
  { value: 'federated', label: 'Federated', description: 'Known set of validators with governance control' },
  { value: 'static', label: 'Static', description: 'Fixed validator set that cannot change' }
]

const supplySourceOptions = [
  { value: 'native', label: 'Native', description: 'Use native FIL tokens' },
  { value: 'erc20', label: 'ERC-20', description: 'Use custom ERC-20 token contract' }
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

  // Validate all required fields
  const requiredFields = ['parent', 'minValidatorStake', 'minValidators', 'bottomupCheckPeriod', 'permissionMode', 'supplySourceKind', 'genesisSubnetIpcContractsOwner']

  requiredFields.forEach(field => {
    validateField(field)
  })

  // Conditional validations
  if (formData.value.supplySourceKind === 'erc20') {
    validateField('supplySourceAddress')
  }

  return Object.keys(fieldErrors.value).length === 0
}

// Save configuration to store
const saveConfig = () => {
  wizardStore.updateConfig(formData.value)
}

// Navigation
const goToNextStep = () => {
  if (validateForm()) {
    saveConfig()
    router.push({ name: 'wizard-advanced' })
  }
}

const goToPreviousStep = () => {
  saveConfig() // Save current progress
  router.push({ name: 'wizard-template' })
}

// Auto-save on changes
watch(formData, () => {
  saveConfig()
}, { deep: true })

// Real-time field validation
const handleFieldBlur = (field: string) => {
  validateField(field)
}

// Handle parent network selection
const handleParentChange = (value: string) => {
  formData.value.parent = value
  validateField('parent')
}

// Handle custom parent network entry
const customParentNetwork = ref('')
const showCustomParent = ref(false)

const addCustomParent = () => {
  if (customParentNetwork.value.trim()) {
    formData.value.parent = customParentNetwork.value.trim()
    showCustomParent.value = false
    customParentNetwork.value = ''
    validateField('parent')
  }
}

// Initialize form on mount
onMounted(() => {
  // If coming from template selection, data should already be populated
  // Just validate the form to show any issues
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
            Smart defaults have been applied based on your template selection.
          </div>
        </div>
      </div>
    </div>

    <!-- Form -->
    <div class="card">
      <div class="mb-6">
        <h2 class="text-2xl font-bold text-gray-900 mb-2">Basic Configuration</h2>
        <p class="text-gray-600">Configure the essential parameters for your subnet deployment.</p>
      </div>

      <form @submit.prevent="goToNextStep" class="space-y-6">
        <!-- Network Configuration Section -->
        <div class="space-y-6">
          <h3 class="text-lg font-semibold text-gray-800 border-b border-gray-200 pb-2">
            Network Configuration
          </h3>

          <!-- Parent Network -->
          <div class="space-y-3">
            <FormSelect
              v-model="formData.parent"
              label="Parent Network"
              placeholder="Select parent network"
              :options="parentNetworkOptions"
              required
              :error="fieldErrors.parent"
              help-text="The parent network this subnet will connect to"
              @change="handleParentChange"
            />

            <!-- Custom Parent Network Option -->
            <div class="flex items-center space-x-2">
              <button
                type="button"
                @click="showCustomParent = !showCustomParent"
                class="text-sm text-primary-600 hover:text-primary-700"
              >
                + Add custom parent network
              </button>
            </div>

            <div v-if="showCustomParent" class="flex space-x-2">
              <FormInput
                v-model="customParentNetwork"
                placeholder="/r<chain-id>"
                class="flex-1"
              />
              <button
                type="button"
                @click="addCustomParent"
                class="btn-primary"
              >
                Add
              </button>
              <button
                type="button"
                @click="showCustomParent = false"
                class="btn-secondary"
              >
                Cancel
              </button>
            </div>
          </div>

          <!-- From Address -->
          <FormInput
            v-model="formData.from"
            label="From Address (Optional)"
            placeholder="0x..."
            help-text="Address creating the subnet (defaults to global sender if not specified)"
            :error="fieldErrors.from"
            @blur="handleFieldBlur('from')"
          />
        </div>

        <!-- Validator Configuration Section -->
        <div class="space-y-6">
          <h3 class="text-lg font-semibold text-gray-800 border-b border-gray-200 pb-2">
            Validator Configuration
          </h3>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <FormInput
              v-model="formData.minValidatorStake"
              type="number"
              label="Minimum Validator Stake"
              placeholder="1.0"
              suffix="FIL"
              required
              :error="fieldErrors.minValidatorStake"
              help-text="Minimum collateral required per validator"
              @blur="handleFieldBlur('minValidatorStake')"
            />

            <FormInput
              v-model="formData.minValidators"
              type="number"
              label="Minimum Validators"
              placeholder="1"
              required
              :error="fieldErrors.minValidators"
              help-text="Minimum number of validators to bootstrap the network"
              @blur="handleFieldBlur('minValidators')"
            />
          </div>

          <FormSelect
            v-model="formData.permissionMode"
            label="Permission Mode"
            :options="permissionModeOptions"
            required
            :error="fieldErrors.permissionMode"
            help-text="How validators can join the network"
          />
        </div>

        <!-- Economic Configuration Section -->
        <div class="space-y-6">
          <h3 class="text-lg font-semibold text-gray-800 border-b border-gray-200 pb-2">
            Economic Configuration
          </h3>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <FormInput
              v-model="formData.bottomupCheckPeriod"
              type="number"
              label="Bottom-up Checkpoint Period"
              placeholder="50"
              suffix="epochs"
              required
              :error="fieldErrors.bottomupCheckPeriod"
              help-text="How often checkpoints are submitted to parent"
              @blur="handleFieldBlur('bottomupCheckPeriod')"
            />

            <FormInput
              v-model="formData.minCrossMsgFee"
              type="number"
              label="Minimum Cross-Message Fee"
              placeholder="0.000001"
              suffix="FIL"
              :error="fieldErrors.minCrossMsgFee"
              help-text="Minimum fee for cross-network messages"
              @blur="handleFieldBlur('minCrossMsgFee')"
            />
          </div>

          <FormSelect
            v-model="formData.supplySourceKind"
            label="Supply Source"
            :options="supplySourceOptions"
            required
            :error="fieldErrors.supplySourceKind"
            help-text="Token type for subnet supply"
          />

          <!-- ERC-20 Address (conditional) -->
          <FormInput
            v-if="formData.supplySourceKind === 'erc20'"
            v-model="formData.supplySourceAddress"
            label="Supply Source Address"
            placeholder="0x..."
            required
            :error="fieldErrors.supplySourceAddress"
            help-text="ERC-20 contract address for token supply"
            @blur="handleFieldBlur('supplySourceAddress')"
          />
        </div>

        <!-- Governance Configuration Section -->
        <div class="space-y-6">
          <h3 class="text-lg font-semibold text-gray-800 border-b border-gray-200 pb-2">
            Governance Configuration
          </h3>

          <FormInput
            v-model="formData.genesisSubnetIpcContractsOwner"
            label="Genesis Contracts Owner"
            placeholder="0x..."
            required
            :error="fieldErrors.genesisSubnetIpcContractsOwner"
            help-text="Address that will own the IPC diamond contracts at genesis"
            @blur="handleFieldBlur('genesisSubnetIpcContractsOwner')"
          />
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
            Next: Advanced Settings
          </button>
        </div>
      </form>
    </div>
  </div>
</template>