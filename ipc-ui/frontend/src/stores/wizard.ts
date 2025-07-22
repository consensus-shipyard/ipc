import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface SubnetConfig {
  // Template selection
  selectedTemplate?: string
  questionnaire?: Record<string, string>

  // Basic configuration (mandatory)
  parent?: string
  from?: string
  minValidatorStake?: number
  minValidators?: number
  bottomupCheckPeriod?: number
  permissionMode?: 'collateral' | 'federated' | 'static'
  supplySourceKind?: 'native' | 'erc20'
  supplySourceAddress?: string
  minCrossMsgFee?: number
  genesisSubnetIpcContractsOwner?: string

  // Advanced configuration (optional)
  activeValidatorsLimit?: number
  validatorGater?: string
  validatorRewarder?: string
  collateralSourceKind?: 'native' | 'erc20'
  collateralSourceAddress?: string

  // Genesis configuration
  networkVersion?: number
  baseFee?: number
  powerScale?: number

  // Activation configuration
  activationMode?: 'federated' | 'static' | 'collateral'
  validatorPubkeys?: string[]
  validatorPower?: number[]
  validators?: Array<{
    from: string
    collateral: number
    initialBalance?: number
  }>

  // Deployment settings
  deployConfig?: {
    enabled?: boolean
    url?: string
    chainId?: number
    artifactsPath?: string
    subnetCreationPrivilege?: 'Unrestricted' | 'Whitelisted' | 'Restricted'
  }

  // Wallet import (for later phases)
  walletImports?: Array<{
    walletType: string
    path?: string
    privateKey?: string
  }>
}

export interface ValidationError {
  field: string
  message: string
}

export const useWizardStore = defineStore('wizard', () => {
  // State
  const config = ref<SubnetConfig>({})
  const currentStep = ref(1)
  const validationErrors = ref<ValidationError[]>([])
  const isValidating = ref(false)
  const isDirty = ref(false)

  // Computed
  const isConfigComplete = computed(() => {
    // Check if all mandatory fields are filled
    const required = [
      'parent',
      'minValidatorStake',
      'minValidators',
      'bottomupCheckPeriod',
      'permissionMode',
      'supplySourceKind',
      'genesisSubnetIpcContractsOwner'
    ]

    return required.every(field => {
      const value = config.value[field as keyof SubnetConfig]
      return value !== undefined && value !== null && value !== ''
    })
  })

  const hasErrors = computed(() => validationErrors.value.length > 0)

  const currentStepConfig = computed(() => {
    switch (currentStep.value) {
      case 1: // Template selection
        return {
          selectedTemplate: config.value.selectedTemplate,
          questionnaire: config.value.questionnaire
        }
      case 2: // Basic config
        return {
          parent: config.value.parent,
          from: config.value.from,
          minValidatorStake: config.value.minValidatorStake,
          minValidators: config.value.minValidators,
          bottomupCheckPeriod: config.value.bottomupCheckPeriod,
          permissionMode: config.value.permissionMode,
          supplySourceKind: config.value.supplySourceKind,
          supplySourceAddress: config.value.supplySourceAddress,
          minCrossMsgFee: config.value.minCrossMsgFee,
          genesisSubnetIpcContractsOwner: config.value.genesisSubnetIpcContractsOwner
        }
      case 3: // Advanced config
        return {
          activeValidatorsLimit: config.value.activeValidatorsLimit,
          validatorGater: config.value.validatorGater,
          validatorRewarder: config.value.validatorRewarder,
          collateralSourceKind: config.value.collateralSourceKind,
          collateralSourceAddress: config.value.collateralSourceAddress,
          networkVersion: config.value.networkVersion,
          baseFee: config.value.baseFee,
          powerScale: config.value.powerScale
        }
      case 4: // Activation config
        return {
          activationMode: config.value.activationMode,
          validatorPubkeys: config.value.validatorPubkeys,
          validatorPower: config.value.validatorPower,
          validators: config.value.validators
        }
      default:
        return {}
    }
  })

  // Actions
  const updateConfig = (updates: Partial<SubnetConfig>) => {
    config.value = { ...config.value, ...updates }
    isDirty.value = true
  }

  const setCurrentStep = (step: number) => {
    currentStep.value = step
  }

  const validateField = (field: string, value: any): string | null => {
    // Basic validation rules
    switch (field) {
      case 'parent':
        if (!value) return 'Parent network is required'
        if (typeof value === 'string' && !value.startsWith('/')) {
          return 'Parent network should start with /'
        }
        break

      case 'minValidatorStake':
        if (value === undefined || value === null) return 'Minimum validator stake is required'
        if (typeof value === 'number' && value <= 0) return 'Stake must be greater than 0'
        break

      case 'minValidators':
        if (value === undefined || value === null) return 'Minimum validators count is required'
        if (typeof value === 'number' && value < 1) return 'At least 1 validator is required'
        break

      case 'bottomupCheckPeriod':
        if (value === undefined || value === null) return 'Bottom-up checkpoint period is required'
        if (typeof value === 'number' && value < 1) return 'Period must be at least 1 epoch'
        break

      case 'genesisSubnetIpcContractsOwner':
        if (!value) return 'Genesis contracts owner is required'
        if (typeof value === 'string' && !/^0x[a-fA-F0-9]{40}$/.test(value)) {
          return 'Must be a valid Ethereum address'
        }
        break

      case 'supplySourceAddress':
        if (config.value.supplySourceKind === 'erc20' && !value) {
          return 'ERC20 address is required when using ERC20 supply source'
        }
        if (value && typeof value === 'string' && !/^0x[a-fA-F0-9]{40}$/.test(value)) {
          return 'Must be a valid Ethereum address'
        }
        break

      case 'collateralSourceAddress':
        if (config.value.collateralSourceKind === 'erc20' && !value) {
          return 'ERC20 address is required when using ERC20 collateral source'
        }
        if (value && typeof value === 'string' && !/^0x[a-fA-F0-9]{40}$/.test(value)) {
          return 'Must be a valid Ethereum address'
        }
        break
    }

    return null
  }

  const validateStep = (step: number): ValidationError[] => {
    const errors: ValidationError[] = []

    if (step === 2) { // Basic config validation
      const fields = ['parent', 'minValidatorStake', 'minValidators', 'bottomupCheckPeriod', 'permissionMode', 'supplySourceKind', 'genesisSubnetIpcContractsOwner']

      fields.forEach(field => {
        const value = config.value[field as keyof SubnetConfig]
        const error = validateField(field, value)
        if (error) {
          errors.push({ field, message: error })
        }
      })

      // Additional conditional validations
      if (config.value.supplySourceKind === 'erc20') {
        const error = validateField('supplySourceAddress', config.value.supplySourceAddress)
        if (error) {
          errors.push({ field: 'supplySourceAddress', message: error })
        }
      }
    }

    return errors
  }

  const setValidationErrors = (errors: ValidationError[]) => {
    validationErrors.value = errors
  }

  const clearValidationErrors = () => {
    validationErrors.value = []
  }

  const resetWizard = () => {
    config.value = {}
    currentStep.value = 1
    validationErrors.value = []
    isDirty.value = false
  }

  const exportConfig = () => {
    // Export as subnet-init.yaml format (for future phases)
    return config.value
  }

  return {
    // State
    config,
    currentStep,
    validationErrors,
    isValidating,
    isDirty,

    // Computed
    isConfigComplete,
    hasErrors,
    currentStepConfig,

    // Actions
    updateConfig,
    setCurrentStep,
    validateField,
    validateStep,
    setValidationErrors,
    clearValidationErrors,
    resetWizard,
    exportConfig
  }
})