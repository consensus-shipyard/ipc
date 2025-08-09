<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AddressSelector from '../../components/common/AddressSelector.vue'
import FormInput from '../../components/common/FormInput.vue'
import { useTemplatesStore } from '../../stores/templates'
import { useWalletStore } from '../../stores/wallet'
import { useWizardStore } from '../../stores/wizard'

const router = useRouter()
const wizardStore = useWizardStore()
const templatesStore = useTemplatesStore()
const walletStore = useWalletStore()

// Get permission mode from basic config to determine activation type
const permissionMode = computed(() => wizardStore.config.permissionMode || 'collateral')
const activationMode = computed(() => {
  // Activation mode follows permission mode for most cases
  return wizardStore.config.activationMode || permissionMode.value
})

// Form state for federated/static mode
const federatedData = ref({
  validatorPubkeys: wizardStore.config.validatorPubkeys || [''],
  validatorPower: wizardStore.config.validatorPower || [1]
})

// Form state for collateral mode
const collateralValidators = ref(
  wizardStore.config.validators || [
    { from: '', collateral: 0, initialBalance: 0 }
  ]
)

// Field errors
const fieldErrors = ref<Record<string, string>>({})

// Get selected template info
const selectedTemplate = computed(() => {
  return wizardStore.config.selectedTemplate
    ? templatesStore.getTemplate(wizardStore.config.selectedTemplate)
    : null
})

// Get minimum validators from basic config
const minValidators = computed(() => wizardStore.config.minValidators || 1)

// Validation functions
const validateEthereumAddress = (address: string): string | null => {
  if (!address) return 'Address is required'
  if (!/^0x[a-fA-F0-9]{40}$/.test(address)) {
    return 'Must be a valid Ethereum address'
  }
  return null
}

const validatePubkey = (pubkey: string): string | null => {
  if (!pubkey) return 'Public key is required'
  // Accept both compressed (0x02/0x03 + 64 hex chars) and uncompressed (0x04 + 128 hex chars) public keys
  if (!/^0x(02|03)[a-fA-F0-9]{64}$/.test(pubkey) && !/^0x04[a-fA-F0-9]{128}$/.test(pubkey)) {
    return 'Must be a valid compressed (0x02/0x03...) or uncompressed (0x04...) public key'
  }
  return null
}

const validatePositiveNumber = (value: number, fieldName: string): string | null => {
  if (value === undefined || value === null || isNaN(value)) {
    return `${fieldName} is required`
  }
  if (value <= 0) {
    return `${fieldName} must be greater than 0`
  }
  return null
}

// Federated/Static mode functions
const addValidator = () => {
  federatedData.value.validatorPubkeys.push('')
  federatedData.value.validatorPower.push(1)
}

const removeValidator = (index: number) => {
  if (federatedData.value.validatorPubkeys.length > 1) {
    federatedData.value.validatorPubkeys.splice(index, 1)
    federatedData.value.validatorPower.splice(index, 1)
  }
}

const validateFederatedForm = (): boolean => {
  fieldErrors.value = {}
  let hasErrors = false

  // Validate minimum number of validators
  if (federatedData.value.validatorPubkeys.length < minValidators.value) {
    fieldErrors.value.minValidators = `At least ${minValidators.value} validator(s) required`
    hasErrors = true
  }

  // Validate each validator
  federatedData.value.validatorPubkeys.forEach((pubkey, index) => {
    const pubkeyError = validatePubkey(pubkey)
    if (pubkeyError) {
      fieldErrors.value[`pubkey_${index}`] = pubkeyError
      hasErrors = true
    }

    const powerError = validatePositiveNumber(federatedData.value.validatorPower[index], 'Power')
    if (powerError) {
      fieldErrors.value[`power_${index}`] = powerError
      hasErrors = true
    }
  })

  return !hasErrors
}

// Collateral mode functions
const addCollateralValidator = () => {
  collateralValidators.value.push({
    from: '',
    collateral: 0,
    initialBalance: 0
  })
}

const removeCollateralValidator = (index: number) => {
  if (collateralValidators.value.length > 1) {
    collateralValidators.value.splice(index, 1)
  }
}

const validateCollateralForm = (): boolean => {
  fieldErrors.value = {}
  let hasErrors = false

  // Validate minimum number of validators
  if (collateralValidators.value.length < minValidators.value) {
    fieldErrors.value.minValidators = `At least ${minValidators.value} validator(s) required`
    hasErrors = true
  }

  // Validate each validator
  collateralValidators.value.forEach((validator, index) => {
    const addressError = validateEthereumAddress(validator.from)
    if (addressError) {
      fieldErrors.value[`address_${index}`] = addressError
      hasErrors = true
    }

    const collateralError = validatePositiveNumber(validator.collateral, 'Collateral')
    if (collateralError) {
      fieldErrors.value[`collateral_${index}`] = collateralError
      hasErrors = true
    }

    // Check minimum stake requirement
    const minStake = wizardStore.config.minValidatorStake || 1
    if (validator.collateral < minStake) {
      fieldErrors.value[`collateral_${index}`] = `Collateral must be at least ${minStake} FIL`
      hasErrors = true
    }
  })

  return !hasErrors
}

// Save and navigation
const saveConfig = () => {
  if (activationMode.value === 'collateral') {
    wizardStore.updateConfig({
      activationMode: activationMode.value,
      validators: collateralValidators.value.filter(v => v.from && v.collateral > 0)
    })
  } else {
    wizardStore.updateConfig({
      activationMode: activationMode.value,
      validatorPubkeys: federatedData.value.validatorPubkeys.filter(pk => pk),
      validatorPower: federatedData.value.validatorPower
    })
  }
}

const validateForm = (): boolean => {
  return activationMode.value === 'collateral'
    ? validateCollateralForm()
    : validateFederatedForm()
}

const goToNextStep = () => {
  if (validateForm()) {
    saveConfig()
    router.push({ name: 'wizard-review' })
  }
}

const goToPreviousStep = () => {
  saveConfig()
  router.push({ name: 'wizard-advanced' })
}

// Auto-save on changes
watch([federatedData, collateralValidators], () => {
  saveConfig()
}, { deep: true })

// Initialize validator arrays based on template or minimum requirements
onMounted(() => {
  // Load wallet addresses for pubkey lookup
  walletStore.fetchAddresses()

  // Initialize minimum validators if needed
  if (activationMode.value === 'collateral' && collateralValidators.value.length < minValidators.value) {
    while (collateralValidators.value.length < minValidators.value) {
      addCollateralValidator()
    }
  } else if (activationMode.value !== 'collateral' && federatedData.value.validatorPubkeys.length < minValidators.value) {
    while (federatedData.value.validatorPubkeys.length < minValidators.value) {
      addValidator()
    }
  }
})

// Helper to get address from pubkey by looking up in wallet store
const getAddressFromPubkey = (pubkey: string): string => {
  if (!pubkey) return ''

  // Find the address that corresponds to this pubkey
  const walletAddress = walletStore.addresses.find(addr => addr.pubkey === pubkey)
  return walletAddress?.address || ''
}

// Handle address selection for federated validators
const handleValidatorAddressSelected = (index: number, walletAddress: any) => {
  if (walletAddress.pubkey) {
    // Auto-populate the pubkey from the selected address
    federatedData.value.validatorPubkeys[index] = walletAddress.pubkey
  }
}

// Handle manual address input (for when someone types an address manually)
const handleAddressSelection = (index: number, address: string) => {
  // Look up the pubkey for this address
  const walletAddress = walletStore.getAddressByAddress(address)
  if (walletAddress?.pubkey) {
    federatedData.value.validatorPubkeys[index] = walletAddress.pubkey
  }
}

// Validate a specific field
const validateField = (fieldName: string) => {
  // Clear previous error
  delete fieldErrors.value[fieldName]
}
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
            Configure validators based on your {{ permissionMode }} permission mode.
          </div>
        </div>
      </div>
    </div>

    <!-- Form -->
    <div class="card">
      <div class="mb-6">
        <h2 class="text-2xl font-bold text-gray-900 mb-2">Activation Configuration</h2>
        <p class="text-gray-600">
          Configure the initial validator set for your {{ permissionMode }} subnet.
        </p>
      </div>

      <!-- Permission Mode Info -->
      <div class="bg-gray-50 border border-gray-200 rounded-lg p-4 mb-6">
        <div class="flex items-start space-x-3">
          <svg class="w-5 h-5 text-gray-500 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
          </svg>
          <div>
            <h3 class="font-semibold text-gray-800 mb-1">{{ permissionMode.charAt(0).toUpperCase() + permissionMode.slice(1) }} Mode Selected</h3>
            <p class="text-gray-700 text-sm">
              <span v-if="permissionMode === 'collateral'">
                Validators must stake collateral to participate. You need to specify validator addresses and their stake amounts.
              </span>
              <span v-else-if="permissionMode === 'federated'">
                Known set of validators with governance control. You need to specify validator public keys and their power distribution.
              </span>
              <span v-else>
                Fixed validator set that cannot change. You need to specify validator public keys and their power distribution.
              </span>
            </p>
          </div>
        </div>
      </div>

      <form @submit.prevent="goToNextStep" class="space-y-6">
        <!-- Collateral Mode Form -->
        <div v-if="activationMode === 'collateral'">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-semibold text-gray-800">Initial Validators</h3>
            <span class="text-sm text-gray-500">Minimum: {{ minValidators }}</span>
          </div>

          <!-- Validation Error for Min Validators -->
          <div v-if="fieldErrors.minValidators" class="mb-4 text-sm text-red-600">
            {{ fieldErrors.minValidators }}
          </div>

          <div class="space-y-4">
            <div
              v-for="(validator, index) in collateralValidators"
              :key="index"
              class="border border-gray-200 rounded-lg p-4"
            >
              <div class="flex items-center justify-between mb-4">
                <h4 class="font-medium text-gray-800">Validator {{ index + 1 }}</h4>
                <button
                  v-if="collateralValidators.length > 1"
                  type="button"
                  @click="removeCollateralValidator(index)"
                  class="text-red-600 hover:text-red-700 text-sm"
                >
                  Remove
                </button>
              </div>

              <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <AddressSelector
                  v-model="validator.from"
                  label="Validator Address"
                  placeholder="0x... or select from wallet"
                  required
                  :error="fieldErrors[`address_${index}`]"
                  help-text="Ethereum address of the validator"
                  :field-name="`validator_${index}`"
                  network-type="evm"
                  :show-pubkey="true"
                />

                <FormInput
                  v-model="validator.collateral"
                  type="number"
                  label="Collateral Amount"
                  placeholder="1.0"
                  suffix="FIL"
                  required
                  :error="fieldErrors[`collateral_${index}`]"
                  help-text="FIL to lock as collateral"
                />

                <FormInput
                  v-model="validator.initialBalance"
                  type="number"
                  label="Initial Balance"
                  placeholder="0.0"
                  suffix="FIL"
                  help-text="Starting FIL balance (optional)"
                />
              </div>
            </div>
          </div>

          <button
            type="button"
            @click="addCollateralValidator"
            class="mt-4 flex items-center text-primary-600 hover:text-primary-700 text-sm font-medium"
          >
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Another Validator
          </button>
        </div>

        <!-- Federated/Static Mode Form -->
        <div v-else>
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-semibold text-gray-800">Validator Public Keys & Power</h3>
            <span class="text-sm text-gray-500">Minimum: {{ minValidators }}</span>
          </div>

          <!-- Validation Error for Min Validators -->
          <div v-if="fieldErrors.minValidators" class="mb-4 text-sm text-red-600">
            {{ fieldErrors.minValidators }}
          </div>

          <div class="space-y-4">
            <div
              v-for="(pubkey, index) in federatedData.validatorPubkeys"
              :key="index"
              class="border border-gray-200 rounded-lg p-4"
            >
              <div class="flex items-center justify-between mb-4">
                <h4 class="font-medium text-gray-800">Validator {{ index + 1 }}</h4>
                <button
                  v-if="federatedData.validatorPubkeys.length > 1"
                  type="button"
                  @click="removeValidator(index)"
                  class="text-red-600 hover:text-red-700 text-sm"
                >
                  Remove
                </button>
              </div>

              <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div class="md:col-span-3">
                  <AddressSelector
                    :model-value="getAddressFromPubkey(federatedData.validatorPubkeys[index])"
                    label="Validator Address (Public Key will be auto-filled)"
                    placeholder="Select address or enter pubkey manually"
                    :error="fieldErrors[`pubkey_${index}`]"
                    help-text="Select an EVM address to auto-fill pubkey, or enter pubkey manually"
                    :field-name="`federatedValidator_${index}`"
                    network-type="evm"
                    :show-pubkey="true"
                    @update:modelValue="handleAddressSelection(index, $event)"
                    @address-selected="handleValidatorAddressSelected(index, $event)"
                  />

                  <!-- Manual pubkey input fallback -->
                  <FormInput
                    v-if="!getAddressFromPubkey(federatedData.validatorPubkeys[index])"
                    v-model="federatedData.validatorPubkeys[index]"
                    label="Or Enter Public Key Manually"
                    placeholder="0x04..."
                    class="mt-2"
                    help-text="65-byte uncompressed public key (starts with 0x04)"
                    @blur="validateField(`pubkey_${index}`)"
                  />

                  <!-- Show resolved pubkey -->
                  <div
                    v-else-if="federatedData.validatorPubkeys[index]"
                    class="mt-2 p-2 bg-gray-50 rounded text-xs"
                  >
                    <span class="font-medium text-gray-700">Public Key:</span>
                    <code class="ml-2 text-gray-600">{{ federatedData.validatorPubkeys[index] }}</code>
                  </div>
                </div>

                <FormInput
                  v-model="federatedData.validatorPower[index]"
                  type="number"
                  label="Voting Power"
                  placeholder="1"
                  required
                  :error="fieldErrors[`power_${index}`]"
                  help-text="Validator's voting weight"
                />
              </div>
            </div>
          </div>

          <button
            type="button"
            @click="addValidator"
            class="mt-4 flex items-center text-primary-600 hover:text-primary-700 text-sm font-medium"
          >
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Another Validator
          </button>
        </div>

        <!-- Summary Info -->
        <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div class="flex items-start space-x-3">
            <svg class="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
            </svg>
            <div>
              <h3 class="font-semibold text-blue-800 mb-1">Activation Summary</h3>
              <div class="text-blue-700 text-sm space-y-1">
                <p v-if="activationMode === 'collateral'">
                  • {{ collateralValidators.filter(v => v.from && v.collateral > 0).length }} validators configured
                </p>
                <p v-else>
                  • {{ federatedData.validatorPubkeys.filter(pk => pk).length }} validators configured
                </p>
                <p>• Minimum required: {{ minValidators }}</p>
                <p v-if="activationMode === 'collateral'">
                  • Total stake: {{ collateralValidators.reduce((sum, v) => sum + (v.collateral || 0), 0).toFixed(2) }} FIL
                </p>
                <p v-else>
                  • Total voting power: {{ federatedData.validatorPower.reduce((sum, p) => sum + (p || 0), 0) }}
                </p>
              </div>
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
            Next: Review & Deploy
          </button>
        </div>
      </form>
    </div>
  </div>
</template>