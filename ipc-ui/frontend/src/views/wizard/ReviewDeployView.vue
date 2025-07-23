<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useTemplatesStore } from '../../stores/templates'
import { useWizardStore } from '../../stores/wizard'

const router = useRouter()
const wizardStore = useWizardStore()
const templatesStore = useTemplatesStore()

// State
const showAdvancedSection = ref(false)

// Get configuration summary
const config = computed(() => wizardStore.config)
const selectedTemplate = computed(() => {
  return config.value.selectedTemplate
    ? templatesStore.getTemplate(config.value.selectedTemplate)
    : null
})

// Configuration sections
const basicConfig = computed(() => ({
  parent: config.value.parent || 'Not configured',
  from: config.value.from || 'Default sender',
  minValidatorStake: config.value.minValidatorStake || 0,
  minValidators: config.value.minValidators || 1,
  bottomupCheckPeriod: config.value.bottomupCheckPeriod || 50,
  permissionMode: config.value.permissionMode || 'collateral',
  supplySourceKind: config.value.supplySourceKind || 'native',
  supplySourceAddress: config.value.supplySourceAddress || 'N/A',
  minCrossMsgFee: config.value.minCrossMsgFee || 0.000001,
  genesisSubnetIpcContractsOwner: config.value.genesisSubnetIpcContractsOwner || 'Not configured'
}))

const advancedConfig = computed(() => ({
  activeValidatorsLimit: config.value.activeValidatorsLimit || 'Unlimited',
  validatorGater: config.value.validatorGater || 'None',
  validatorRewarder: config.value.validatorRewarder || 'None',
  collateralSourceKind: config.value.collateralSourceKind || 'native',
  collateralSourceAddress: config.value.collateralSourceAddress || 'N/A',
  networkVersion: config.value.networkVersion || 21,
  baseFee: config.value.baseFee || 1000,
  powerScale: config.value.powerScale || 3
}))

const activationConfig = computed(() => {
  const mode = config.value.activationMode || config.value.permissionMode || 'collateral'

  if (mode === 'collateral') {
    return {
      mode: 'Collateral',
      validators: config.value.validators || [],
      totalStake: (config.value.validators || []).reduce((sum, v) => sum + (v.collateral || 0), 0),
      totalBalance: (config.value.validators || []).reduce((sum, v) => sum + (v.initialBalance || 0), 0)
    }
  } else {
    return {
      mode: mode === 'federated' ? 'Federated' : 'Static',
      pubkeys: config.value.validatorPubkeys || [],
      totalPower: (config.value.validatorPower || []).reduce((sum, p) => sum + (p || 0), 0)
    }
  }
})

// Check if configuration is complete
const isConfigComplete = computed(() => wizardStore.isConfigComplete)
const configErrors = computed(() => wizardStore.validationErrors)

// Export configuration
const exportConfig = () => {
  const exportData = wizardStore.exportConfig()
  const blob = new Blob([JSON.stringify(exportData, null, 2)], {
    type: 'application/json'
  })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = 'subnet-config.json'
  link.click()
  URL.revokeObjectURL(url)
}

// Navigation
const goToPreviousStep = () => {
  router.push({ name: 'wizard-activation' })
}

const startDeployment = () => {
  if (isConfigComplete.value) {
    router.push({ name: 'wizard-deploy' })
  }
}

// Edit functions
const editBasicConfig = () => {
  router.push({ name: 'wizard-basic' })
}

const editAdvancedConfig = () => {
  router.push({ name: 'wizard-advanced' })
}

const editActivationConfig = () => {
  router.push({ name: 'wizard-activation' })
}

// Helper functions
const formatAddress = (address: string) => {
  if (!address || address === 'N/A') return address
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

const formatNumber = (value: number | string, suffix = '') => {
  if (typeof value === 'string') return value
  return `${value.toLocaleString()}${suffix ? ' ' + suffix : ''}`
}
</script>

<template>
  <div class="space-y-8">
    <!-- Header -->
    <div class="text-center">
      <h2 class="text-3xl font-bold text-gray-900 mb-2">Review & Deploy</h2>
      <p class="text-gray-600 text-lg">
        Review your subnet configuration before deployment
      </p>
    </div>

    <!-- Template Summary -->
    <div v-if="selectedTemplate" class="bg-gradient-to-r from-primary-50 to-blue-50 border border-primary-200 rounded-lg p-6">
      <div class="flex items-start space-x-4">
        <div class="text-4xl">{{ selectedTemplate.icon }}</div>
        <div class="flex-1">
          <h3 class="font-semibold text-primary-800 text-xl mb-2">{{ selectedTemplate.name }}</h3>
          <p class="text-primary-700 mb-3">{{ selectedTemplate.description }}</p>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-xs text-primary-600">
            <div>
              <span class="font-semibold">Mode:</span> {{ basicConfig.permissionMode }}
            </div>
            <div>
              <span class="font-semibold">Validators:</span> {{ basicConfig.minValidators }}+ required
            </div>
            <div>
              <span class="font-semibold">Network:</span> {{ basicConfig.parent }}
            </div>
            <div>
              <span class="font-semibold">Stake:</span> {{ formatNumber(basicConfig.minValidatorStake, 'FIL') }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Configuration Status -->
    <div v-if="!isConfigComplete" class="bg-red-50 border border-red-200 rounded-lg p-4">
      <div class="flex items-start space-x-3">
        <svg class="w-5 h-5 text-red-600 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
        </svg>
        <div>
          <h3 class="font-semibold text-red-800 mb-1">Configuration Incomplete</h3>
          <div class="text-red-700 text-sm space-y-1">
            <p>Please complete the following required fields:</p>
            <ul class="list-disc list-inside ml-2">
              <li v-for="error in configErrors" :key="error.field">
                {{ error.message }}
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="bg-green-50 border border-green-200 rounded-lg p-4">
      <div class="flex items-start space-x-3">
        <svg class="w-5 h-5 text-green-600 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
        </svg>
        <div>
          <h3 class="font-semibold text-green-800 mb-1">Configuration Complete</h3>
          <p class="text-green-700 text-sm">
            All required parameters have been configured. Your subnet is ready for deployment.
          </p>
        </div>
      </div>
    </div>

    <!-- Configuration Sections -->
    <div class="space-y-6">
      <!-- Basic Configuration -->
      <div class="card">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-xl font-semibold text-gray-800">Basic Configuration</h3>
          <button
            type="button"
            @click="editBasicConfig"
            class="text-primary-600 hover:text-primary-700 text-sm font-medium"
          >
            Edit
          </button>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div class="space-y-3">
            <div>
              <dt class="text-sm font-medium text-gray-500">Parent Network</dt>
              <dd class="text-sm text-gray-900">{{ basicConfig.parent }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Permission Mode</dt>
              <dd class="text-sm text-gray-900 capitalize">{{ basicConfig.permissionMode }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Minimum Validators</dt>
              <dd class="text-sm text-gray-900">{{ formatNumber(basicConfig.minValidators) }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Supply Source</dt>
              <dd class="text-sm text-gray-900 capitalize">{{ basicConfig.supplySourceKind }}</dd>
            </div>
            <div v-if="basicConfig.supplySourceKind === 'erc20'">
              <dt class="text-sm font-medium text-gray-500">Supply Token Address</dt>
              <dd class="text-sm text-gray-900 font-mono">{{ formatAddress(basicConfig.supplySourceAddress) }}</dd>
            </div>
          </div>

          <div class="space-y-3">
            <div>
              <dt class="text-sm font-medium text-gray-500">Minimum Validator Stake</dt>
              <dd class="text-sm text-gray-900">{{ formatNumber(basicConfig.minValidatorStake, 'FIL') }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Checkpoint Period</dt>
              <dd class="text-sm text-gray-900">{{ formatNumber(basicConfig.bottomupCheckPeriod, 'epochs') }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Min Cross-Message Fee</dt>
              <dd class="text-sm text-gray-900">{{ basicConfig.minCrossMsgFee.toFixed(6) }} FIL</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Genesis Owner</dt>
              <dd class="text-sm text-gray-900 font-mono">{{ formatAddress(basicConfig.genesisSubnetIpcContractsOwner) }}</dd>
            </div>
            <div v-if="basicConfig.from !== 'Default sender'">
              <dt class="text-sm font-medium text-gray-500">From Address</dt>
              <dd class="text-sm text-gray-900 font-mono">{{ formatAddress(basicConfig.from) }}</dd>
            </div>
          </div>
        </div>
      </div>

      <!-- Advanced Configuration -->
      <div class="card">
        <div class="flex items-center justify-between mb-4">
          <button
            type="button"
            @click="showAdvancedSection = !showAdvancedSection"
            class="flex items-center space-x-2"
          >
            <h3 class="text-xl font-semibold text-gray-800">Advanced Configuration</h3>
            <svg
              :class="['w-5 h-5 text-gray-400 transition-transform', showAdvancedSection && 'rotate-180']"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <button
            type="button"
            @click="editAdvancedConfig"
            class="text-primary-600 hover:text-primary-700 text-sm font-medium"
          >
            Edit
          </button>
        </div>

        <div v-if="showAdvancedSection" class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div class="space-y-3">
            <div>
              <dt class="text-sm font-medium text-gray-500">Active Validators Limit</dt>
              <dd class="text-sm text-gray-900">{{ formatNumber(advancedConfig.activeValidatorsLimit) }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Validator Gater</dt>
              <dd class="text-sm text-gray-900 font-mono">{{ advancedConfig.validatorGater === 'None' ? 'None' : formatAddress(advancedConfig.validatorGater) }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Validator Rewarder</dt>
              <dd class="text-sm text-gray-900 font-mono">{{ advancedConfig.validatorRewarder === 'None' ? 'None' : formatAddress(advancedConfig.validatorRewarder) }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Collateral Source</dt>
              <dd class="text-sm text-gray-900 capitalize">{{ advancedConfig.collateralSourceKind }}</dd>
            </div>
          </div>

          <div class="space-y-3">
            <div v-if="advancedConfig.collateralSourceKind === 'erc20'">
              <dt class="text-sm font-medium text-gray-500">Collateral Token Address</dt>
              <dd class="text-sm text-gray-900 font-mono">{{ formatAddress(advancedConfig.collateralSourceAddress) }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Network Version</dt>
              <dd class="text-sm text-gray-900">{{ advancedConfig.networkVersion }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Base Fee</dt>
              <dd class="text-sm text-gray-900">{{ formatNumber(advancedConfig.baseFee, 'attoFIL') }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Power Scale</dt>
              <dd class="text-sm text-gray-900">{{ advancedConfig.powerScale }}</dd>
            </div>
          </div>
        </div>

        <div v-else class="text-sm text-gray-500">
          Click to view advanced configuration options
        </div>
      </div>

      <!-- Activation Configuration -->
      <div class="card">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-xl font-semibold text-gray-800">Activation Configuration</h3>
          <button
            type="button"
            @click="editActivationConfig"
            class="text-primary-600 hover:text-primary-700 text-sm font-medium"
          >
            Edit
          </button>
        </div>

        <div class="mb-4">
          <dt class="text-sm font-medium text-gray-500">Activation Mode</dt>
          <dd class="text-sm text-gray-900">{{ activationConfig.mode }}</dd>
        </div>

        <!-- Collateral Mode Summary -->
        <div v-if="activationConfig.mode === 'Collateral'" class="space-y-4">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <dt class="text-sm font-medium text-gray-500">Total Validators</dt>
              <dd class="text-sm text-gray-900">{{ activationConfig.validators.length }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Total Stake</dt>
              <dd class="text-sm text-gray-900">{{ formatNumber(activationConfig.totalStake, 'FIL') }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Total Initial Balance</dt>
              <dd class="text-sm text-gray-900">{{ formatNumber(activationConfig.totalBalance, 'FIL') }}</dd>
            </div>
          </div>

          <div v-if="activationConfig.validators.length > 0" class="overflow-x-auto">
            <table class="min-w-full text-sm">
              <thead class="bg-gray-50">
                <tr>
                  <th class="px-3 py-2 text-left font-medium text-gray-500">#</th>
                  <th class="px-3 py-2 text-left font-medium text-gray-500">Address</th>
                  <th class="px-3 py-2 text-left font-medium text-gray-500">Collateral</th>
                  <th class="px-3 py-2 text-left font-medium text-gray-500">Initial Balance</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-200">
                <tr v-for="(validator, index) in activationConfig.validators" :key="index">
                  <td class="px-3 py-2 text-gray-900">{{ index + 1 }}</td>
                  <td class="px-3 py-2 text-gray-900 font-mono">{{ formatAddress(validator.from) }}</td>
                  <td class="px-3 py-2 text-gray-900">{{ formatNumber(validator.collateral, 'FIL') }}</td>
                  <td class="px-3 py-2 text-gray-900">{{ formatNumber(validator.initialBalance, 'FIL') }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- Federated/Static Mode Summary -->
        <div v-else class="space-y-4">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <dt class="text-sm font-medium text-gray-500">Total Validators</dt>
              <dd class="text-sm text-gray-900">{{ activationConfig.pubkeys.length }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500">Total Voting Power</dt>
              <dd class="text-sm text-gray-900">{{ formatNumber(activationConfig.totalPower) }}</dd>
            </div>
          </div>

          <div v-if="activationConfig.pubkeys.length > 0" class="overflow-x-auto">
            <table class="min-w-full text-sm">
              <thead class="bg-gray-50">
                <tr>
                  <th class="px-3 py-2 text-left font-medium text-gray-500">#</th>
                  <th class="px-3 py-2 text-left font-medium text-gray-500">Public Key</th>
                  <th class="px-3 py-2 text-left font-medium text-gray-500">Power</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-200">
                <tr v-for="(pubkey, index) in activationConfig.pubkeys" :key="index">
                  <td class="px-3 py-2 text-gray-900">{{ index + 1 }}</td>
                  <td class="px-3 py-2 text-gray-900 font-mono">{{ pubkey.slice(0, 20) }}...</td>
                  <td class="px-3 py-2 text-gray-900">{{ config.validatorPower ? config.validatorPower[index] : 1 }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex flex-col sm:flex-row items-center justify-between gap-4 pt-6 border-t border-gray-200">
      <div class="flex items-center space-x-4">
        <button
          type="button"
          @click="goToPreviousStep"
          class="btn-secondary"
        >
          Previous
        </button>

        <button
          type="button"
          @click="exportConfig"
          class="text-gray-600 hover:text-gray-700 text-sm font-medium flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          Export Configuration
        </button>
      </div>

      <button
        type="button"
        @click="startDeployment"
        :disabled="!isConfigComplete"
        class="btn-primary text-lg px-8 py-3 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
        Deploy Subnet
      </button>
    </div>
  </div>
</template>