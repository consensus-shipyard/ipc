<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AddressSelector from '../../components/common/AddressSelector.vue'
import FormInput from '../../components/common/FormInput.vue'
import FormSelect from '../../components/common/FormSelect.vue'
import { apiService } from '../../services/api'
import { useL1GatewaysStore } from '../../stores/l1-gateways'
import { useTemplatesStore } from '../../stores/templates'
import { useWizardStore } from '../../stores/wizard'

const router = useRouter()
const wizardStore = useWizardStore()
const templatesStore = useTemplatesStore()
const l1GatewaysStore = useL1GatewaysStore()

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
  genesisSubnetIpcContractsOwner: wizardStore.config.genesisSubnetIpcContractsOwner || '',
  gatewayMode: wizardStore.config.gatewayMode || 'deploy',
  customGatewayAddress: wizardStore.config.customGatewayAddress || '',
  customRegistryAddress: wizardStore.config.customRegistryAddress || '',
  selectedDeployedGateway: wizardStore.config.selectedDeployedGateway || '',
  selectedL1Gateway: wizardStore.config.selectedL1Gateway || ''
})

// Gateway type definition
interface GatewayInfo {
  id: string
  name?: string
  address: string
  registry_address: string
  deployer_address: string
  parent_network: string
  deployed_at: string
  is_active: boolean
  subnet_count: number
  description?: string
}

// Deployed gateways data
const deployedGateways = ref<GatewayInfo[]>([])
const loadingGateways = ref(false)

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

const gatewayModeOptions = [
  {
    value: 'deploy',
    label: 'Deploy New Gateway',
    description: 'Deploy your own gateway contracts (full control, no approval needed)'
  },
  {
    value: 'l1-gateway',
    label: 'L1 Gateway',
    description: 'Deploy to selected L1 gateway from the top menu'
  },
  {
    value: 'subnet-gateway',
    label: 'Subnet Gateway',
    description: 'Deploy to one of your existing subnet gateways'
  },
  {
    value: 'custom',
    label: 'Custom Gateway',
    description: 'Use previously deployed gateway contracts you own'
  }
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
  const requiredFields = ['parent', 'minValidatorStake', 'minValidators', 'bottomupCheckPeriod', 'permissionMode', 'supplySourceKind', 'genesisSubnetIpcContractsOwner', 'gatewayMode']

  requiredFields.forEach(field => {
    validateField(field)
  })

  // Conditional validations
  if (formData.value.supplySourceKind === 'erc20') {
    validateField('supplySourceAddress')
  }

  // Gateway mode specific validations
  if (formData.value.gatewayMode === 'custom') {
    validateField('customGatewayAddress')
    validateField('customRegistryAddress')
  } else if (formData.value.gatewayMode === 'subnet-gateway') {
    if (!formData.value.selectedDeployedGateway) {
      fieldErrors.value.selectedDeployedGateway = 'Please select a deployed gateway'
      return false
    }
  } else if (formData.value.gatewayMode === 'l1-gateway') {
    if (!formData.value.selectedL1Gateway && !l1GatewaysStore.selectedGateway) {
      fieldErrors.value.selectedL1Gateway = 'Please select an L1 gateway from the top menu'
      return false
    }
  }

  return Object.keys(fieldErrors.value).length === 0
}

// Save configuration to store
const saveConfig = () => {
  wizardStore.updateConfig(formData.value)
}

// Navigation
// Load deployed gateways from API with discovery and deduplication
const loadDeployedGateways = async () => {
  try {
    console.log('[Gateway Discovery] Starting gateway discovery process...')
    loadingGateways.value = true

    // First, discover gateways from IPC config (this will also handle deduplication)
    console.log('[Gateway Discovery] Calling discoverGateways API...')
    const discoverResponse = await apiService.discoverGateways()

    console.log('[Gateway Discovery] Raw discovery response:', discoverResponse.data)

    if (discoverResponse.data && discoverResponse.data.data && Array.isArray(discoverResponse.data.data)) {
      const discoveredGateways: GatewayInfo[] = discoverResponse.data.data
      console.log(`[Gateway Discovery] Found ${discoveredGateways.length} gateways from discovery`)

      // Log each gateway for debugging
      discoveredGateways.forEach((gateway, index) => {
        console.log(`[Gateway Discovery] Gateway ${index + 1}:`, {
          id: gateway.id,
          name: gateway.name,
          address: gateway.address,
          parent_network: gateway.parent_network,
          deployed_at: gateway.deployed_at,
          status: gateway.is_active
        })
      })

      // Apply frontend deduplication as a backup (in case backend missed something)
      const uniqueGateways = deduplicateGateways(discoveredGateways)
      console.log(`[Gateway Deduplication] After frontend deduplication: ${uniqueGateways.length} unique gateways`)

      // The backend already handles filtering and deduplication properly
      // No need for additional filtering here since all returned gateways are valid deployable gateways
      deployedGateways.value = uniqueGateways.sort((a, b) =>
        new Date(b.deployed_at).getTime() - new Date(a.deployed_at).getTime()
      )

      console.log('[Gateway Discovery] Final gateway list:', deployedGateways.value.map(g => ({
        id: g.id,
        name: g.name,
        address: g.address,
        parent_network: g.parent_network,
        subnet_count: g.subnet_count
      })))

      // Debug: Check for duplicate IDs
      const ids = deployedGateways.value.map(g => g.id)
      const uniqueIds = new Set(ids)
      if (ids.length !== uniqueIds.size) {
        console.warn('[Gateway Discovery] Duplicate IDs detected:', ids)
        const duplicateIds = ids.filter((id, index) => ids.indexOf(id) !== index)
        console.warn('[Gateway Discovery] Duplicate IDs found:', duplicateIds)
      }
    } else {
      console.warn('[Gateway Discovery] Invalid or empty discovery response')
      deployedGateways.value = []
    }
  } catch (error) {
    console.error('[Gateway Discovery] Error during gateway discovery:', error)
    // Fallback to regular gateways API if discovery fails
    try {
      console.log('[Gateway Discovery] Falling back to regular gateways API...')
      const fallbackResponse = await apiService.getGateways()
      if (fallbackResponse.data && Array.isArray(fallbackResponse.data)) {
        const fallbackGateways = deduplicateGateways(fallbackResponse.data)
        deployedGateways.value = fallbackGateways.sort((a, b) =>
          new Date(b.deployed_at).getTime() - new Date(a.deployed_at).getTime()
        )
        console.log(`[Gateway Discovery] Fallback loaded ${fallbackGateways.length} gateways`)
      } else {
        deployedGateways.value = []
      }
    } catch (fallbackError) {
      console.error('[Gateway Discovery] Fallback also failed:', fallbackError)
      deployedGateways.value = []
    }
  } finally {
    loadingGateways.value = false
    console.log('[Gateway Discovery] Gateway loading completed')
  }
}

// Deduplicate gateways based on gateway address and parent network
const deduplicateGateways = (gateways: GatewayInfo[]): GatewayInfo[] => {
  console.log('[Gateway Deduplication] Starting deduplication process...')
  console.log('[Gateway Deduplication] Input gateways:', gateways.map(g => ({ id: g.id, address: g.address, network: g.parent_network })))

  const seen = new Map<string, GatewayInfo>()
  const duplicates: GatewayInfo[] = []

  gateways.forEach((gateway, index) => {
    // Create a unique key based on gateway address and parent network
    const uniqueKey = `${gateway.address.toLowerCase()}-${gateway.parent_network}`
    console.log(`[Gateway Deduplication] Processing gateway ${index + 1}:`, {
      id: gateway.id,
      address: gateway.address,
      network: gateway.parent_network,
      uniqueKey
    })

    if (seen.has(uniqueKey)) {
      const existing = seen.get(uniqueKey)!
      console.log(`[Gateway Deduplication] Found duplicate gateway:`, {
        duplicate: {
          id: gateway.id,
          name: gateway.name,
          address: gateway.address,
          network: gateway.parent_network,
          deployed_at: gateway.deployed_at
        },
        existing: {
          id: existing.id,
          name: existing.name,
          address: existing.address,
          network: existing.parent_network,
          deployed_at: existing.deployed_at
        }
      })

      // Keep the one with the more recent deployment date
      if (new Date(gateway.deployed_at) > new Date(existing.deployed_at)) {
        console.log(`[Gateway Deduplication] Replacing existing with newer duplicate`)
        duplicates.push(existing)
        seen.set(uniqueKey, gateway)
      } else {
        console.log(`[Gateway Deduplication] Keeping existing, discarding duplicate`)
        duplicates.push(gateway)
      }
    } else {
      seen.set(uniqueKey, gateway)
    }
  })

  const uniqueGateways = Array.from(seen.values())
  console.log(`[Gateway Deduplication] Removed ${duplicates.length} duplicates, kept ${uniqueGateways.length} unique gateways`)

  if (duplicates.length > 0) {
    console.log('[Gateway Deduplication] Removed duplicates:', duplicates.map(d => ({
      id: d.id,
      name: d.name,
      address: d.address,
      network: d.parent_network
    })))
  }

  return uniqueGateways
}

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

// Watch L1 gateway selection changes from the top menu
watch(() => l1GatewaysStore.selectedGatewayId, (newGatewayId, oldGatewayId) => {
  console.log('[BasicConfig] L1 Gateway selection changed:', { newGatewayId, oldGatewayId, gatewayMode: formData.value.gatewayMode })

  if (formData.value.gatewayMode === 'l1-gateway') {
    formData.value.selectedL1Gateway = newGatewayId || ''
    console.log('[BasicConfig] Updated form selectedL1Gateway:', formData.value.selectedL1Gateway)
  }
}, { immediate: true })

// Watch gateway mode changes to sync L1 gateway selection when switching to l1-gateway mode
watch(() => formData.value.gatewayMode, (newMode, oldMode) => {
  console.log('[BasicConfig] Gateway mode changed:', { newMode, oldMode })

  if (newMode === 'l1-gateway') {
    // When switching to L1 gateway mode, sync the current selection
    const currentL1Selection = l1GatewaysStore.selectedGatewayId
    if (currentL1Selection) {
      formData.value.selectedL1Gateway = currentL1Selection
      console.log('[BasicConfig] Synced L1 gateway on mode change:', currentL1Selection)
    }
  } else if (newMode === 'subnet-gateway') {
    loadDeployedGateways()
  }
})



// Load deployed gateways on component mount
onMounted(() => {
  if (formData.value.gatewayMode === 'subnet-gateway') {
    loadDeployedGateways()
  }
})

// Real-time field validation
const handleFieldBlur = (field: string) => {
  validateField(field)
}

// Handle parent network selection
const handleParentChange = (value: string | number) => {
  formData.value.parent = String(value)
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

const selectGateway = (gatewayId: string) => {
  console.log('[Gateway Selection] Selecting gateway:', gatewayId)
  console.log('[Gateway Selection] Current selected:', formData.value.selectedDeployedGateway)
  console.log('[Gateway Selection] Available gateway IDs:', deployedGateways.value.map(g => g.id))
  console.log('[Gateway Selection] Gateway clicked details:', deployedGateways.value.find(g => g.id === gatewayId))
  formData.value.selectedDeployedGateway = gatewayId
  console.log('[Gateway Selection] New selected:', formData.value.selectedDeployedGateway)
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
          <AddressSelector
            v-model="formData.from"
            label="From Address (Optional)"
            placeholder="0x... or select from wallet"
            help-text="Address creating the subnet (defaults to global sender if not specified)"
            :error="fieldErrors.from"
            field-name="fromAddress"
            network-type="both"
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

          <AddressSelector
            v-model="formData.genesisSubnetIpcContractsOwner"
            label="Genesis Contracts Owner"
            placeholder="0x... or select from wallet"
            required
            :error="fieldErrors.genesisSubnetIpcContractsOwner"
            help-text="Address that will own the IPC diamond contracts at genesis"
            field-name="genesisContractsOwner"
            network-type="both"
            @blur="handleFieldBlur('genesisSubnetIpcContractsOwner')"
          />
        </div>

        <!-- Gateway Configuration Section -->
        <div class="space-y-6">
          <h3 class="text-lg font-semibold text-gray-800 border-b border-gray-200 pb-2">
            Gateway Configuration
          </h3>

          <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <div class="flex items-start space-x-3">
              <svg class="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
              </svg>
              <div class="text-sm text-blue-800">
                <p class="font-medium mb-1">Choose Your Gateway Strategy</p>
                <p>
                  <strong>Deploy New:</strong> Creates your own gateway contracts where you have full control and can approve subnets instantly.<br>
                  <strong>L1 Gateway:</strong> Uses the L1 gateway selected in the top menu bar for deployment to the root network.<br>
                  <strong>Subnet Gateway:</strong> Deploy under one of your existing subnet gateways.<br>
                  <strong>Custom:</strong> Uses previously deployed gateway contracts that you own.
                </p>
              </div>
            </div>
          </div>

          <FormSelect
            v-model="formData.gatewayMode"
            label="Gateway Mode"
            :options="gatewayModeOptions"
            required
            :error="fieldErrors.gatewayMode"
            help-text="How to handle gateway contracts for subnet management"
          />

          <!-- L1 Gateway Selection (conditional) -->
          <div v-if="formData.gatewayMode === 'l1-gateway'" class="space-y-4 p-4 bg-blue-50 border border-blue-200 rounded-lg">
            <h4 class="text-sm font-semibold text-gray-700">Using Selected L1 Gateway</h4>
            <div class="flex items-center space-x-3 p-3 bg-white border border-blue-200 rounded-lg">
              <svg class="w-5 h-5 text-blue-600 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14-7l2 2m0 0l2 2m-2-2v6m-2-2H5m14-7v2a2 2 0 01-2 2H5a2 2 0 01-2-2V4"/>
              </svg>
              <div class="flex-1">
                <p class="text-sm font-medium text-gray-900">Gateway selection managed in top menu</p>
                <p class="text-xs text-gray-600 mt-1">Use the gateway selector in the top menu bar to choose your L1 gateway</p>
              </div>
            </div>
          </div>

          <!-- Subnet Gateway Selection (conditional) -->
          <div v-if="formData.gatewayMode === 'subnet-gateway'" class="space-y-4 p-4 bg-blue-50 border border-blue-200 rounded-lg">
            <h4 class="text-sm font-semibold text-gray-700">Select Your Deployed Gateway</h4>

            <div v-if="deployedGateways.length === 0" class="text-center py-8 text-gray-500">
              <svg class="w-12 h-12 mx-auto mb-4 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14-7l2 2m0 0l2 2m-2-2v6m-2-2H5m14-7v2a2 2 0 01-2 2H5a2 2 0 01-2-2V4"/>
              </svg>
              <p class="text-lg font-medium mb-1">No Deployed Gateways Found</p>
              <p class="text-sm">You haven't deployed any gateways yet. Choose "Deploy New Gateway" to create one.</p>
            </div>

            <div v-else class="space-y-3">
              <div class="space-y-2">
                <div
                  v-for="gateway in deployedGateways"
                  :key="gateway.id"
                  class="flex items-center p-3 border rounded-lg cursor-pointer transition-colors"
                  :class="formData.selectedDeployedGateway === gateway.id ? 'border-blue-500 bg-blue-50' : 'border-gray-200 hover:bg-gray-50'"
                  @click="selectGateway(gateway.id)"
                >
                  <input
                    type="radio"
                    name="deployed-gateway"
                    :value="gateway.id"
                    v-model="formData.selectedDeployedGateway"
                    class="mr-3 text-blue-600"
                    @click.stop
                  />
                  <div class="flex-1">
                    <div class="flex items-center justify-between">
                      <h5 class="font-medium text-gray-900">{{ gateway.name }}</h5>
                      <div class="flex items-center space-x-2">
                        <span class="text-xs px-2 py-1 rounded-full bg-blue-100 text-blue-800">
                          {{ gateway.subnet_count }} subnet{{ gateway.subnet_count !== 1 ? 's' : '' }}
                        </span>
                        <span class="text-xs px-2 py-1 rounded-full"
                              :class="gateway.is_active ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'">
                          {{ gateway.is_active ? 'Active' : 'Inactive' }}
                        </span>
                      </div>
                    </div>
                    <p class="text-sm text-gray-600 mt-1">Gateway contract serving {{ gateway.subnet_count }} subnet{{ gateway.subnet_count !== 1 ? 's' : '' }}</p>
                    <div class="flex text-xs text-gray-500 mt-2 space-x-4">
                      <span>Gateway: {{ gateway.address.slice(0, 8) }}...{{ gateway.address.slice(-6) }}</span>
                      <span>Network: {{ gateway.parent_network }}</span>
                      <span>Deployed: {{ new Date(gateway.deployed_at).toLocaleDateString() }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Custom Gateway Addresses (conditional) -->
          <div v-if="formData.gatewayMode === 'custom'" class="space-y-4 p-4 bg-gray-50 border border-gray-200 rounded-lg">
            <h4 class="text-sm font-semibold text-gray-700">Custom Gateway Contracts</h4>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <FormInput
                v-model="formData.customGatewayAddress"
                label="Gateway Contract Address"
                placeholder="0x..."
                required
                :error="fieldErrors.customGatewayAddress"
                help-text="Address of your deployed gateway contract"
                @blur="handleFieldBlur('customGatewayAddress')"
              />

              <FormInput
                v-model="formData.customRegistryAddress"
                label="Registry Contract Address"
                placeholder="0x..."
                required
                :error="fieldErrors.customRegistryAddress"
                help-text="Address of your deployed registry contract"
                @blur="handleFieldBlur('customRegistryAddress')"
              />
            </div>
          </div>

          <!-- Deploy Mode Info -->
          <div v-if="formData.gatewayMode === 'deploy'" class="p-4 bg-green-50 border border-green-200 rounded-lg">
            <div class="flex items-start space-x-3">
              <svg class="w-5 h-5 text-green-600 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
              <div class="text-sm text-green-800">
                <p class="font-medium mb-1">✨ Recommended for Development</p>
                <p>New gateway contracts will be deployed to the parent chain using your address. You'll become the gateway owner with full control over subnet approvals.</p>
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
            Next: Advanced Settings
          </button>
        </div>
      </form>
    </div>
  </div>
</template>