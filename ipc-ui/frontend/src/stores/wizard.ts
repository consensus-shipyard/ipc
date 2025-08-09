import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { apiService } from '../services/api'
import { wsService, type WebSocketCallbacks } from '../services/websocket'
import type { DeploymentProgress } from '../config/api'

export interface SubnetConfig {
  // Template selection
  selectedTemplate?: string
  questionnaire?: Record<string, string>
  questionnaireSkipped?: boolean

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

  // Gateway configuration
  gatewayMode?: 'existing' | 'deploy' | 'deployed' | 'custom'
  customGatewayAddress?: string
  customRegistryAddress?: string
  selectedDeployedGateway?: string

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

  // Deployment state
  const isDeploying = ref(false)
  const deploymentId = ref<string | null>(null)
  const subnetId = ref<string | null>(null) // Actual subnet ID from deployment result
  const deploymentProgress = ref<DeploymentProgress | null>(null)
  const deploymentError = ref<string | null>(null)
  const deploymentLogs = ref<string[]>([])

  // WebSocket connection state
  const isConnected = ref(false)

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
      'genesisSubnetIpcContractsOwner',
      'gatewayMode'
    ]

    const isBasicComplete = required.every(field => {
      const value = config.value[field as keyof SubnetConfig]
      return value !== undefined && value !== null && value !== ''
    })

    // Check gateway mode specific requirements
    if (config.value.gatewayMode === 'custom') {
      const customGatewayComplete = config.value.customGatewayAddress && config.value.customRegistryAddress
      return isBasicComplete && customGatewayComplete
    } else if (config.value.gatewayMode === 'deployed') {
      const deployedGatewaySelected = config.value.selectedDeployedGateway
      return isBasicComplete && deployedGatewaySelected
    }

    return isBasicComplete
  })

  const hasErrors = computed(() => validationErrors.value.length > 0)

  const currentStepConfig = computed(() => {
    switch (currentStep.value) {
      case 1: // Template selection
        return {
          selectedTemplate: config.value.selectedTemplate,
          questionnaire: config.value.questionnaire,
          questionnaireSkipped: config.value.questionnaireSkipped
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

  // WebSocket integration
  const initializeWebSocket = async () => {
    const callbacks: WebSocketCallbacks = {
      onOpen: () => {
        console.log('WebSocket connected')
        isConnected.value = true
      },
      onClose: () => {
        console.log('WebSocket disconnected')
        isConnected.value = false
      },
      onError: (error) => {
        console.error('WebSocket error:', error)
        isConnected.value = false
      },
      onDeploymentProgress: (progress: DeploymentProgress) => {
        console.log('Deployment progress:', progress)
        deploymentProgress.value = progress

        // Add to logs
        if (progress.message) {
          deploymentLogs.value.push(`[${progress.step}] ${progress.message}`)
        }

        // Handle completion or failure
        if (progress.status === 'completed') {
          isDeploying.value = false
          // Extract actual subnet ID from deployment progress
          if (progress.subnet_id) {
            subnetId.value = progress.subnet_id
            console.log('Deployment completed successfully, subnet ID:', progress.subnet_id)
          } else {
            console.log('Deployment completed successfully')
          }
        } else if (progress.status === 'failed') {
          isDeploying.value = false
          // Try both error and message fields for compatibility
          const errorMessage = progress.error || progress.message || 'Deployment failed'
          deploymentError.value = errorMessage
          console.error('Deployment failed:', errorMessage)
          console.log('Full progress object:', progress)
        }
      }
    }

    // Initialize WebSocket service with callbacks
    Object.assign(wsService.callbacks, callbacks)

    try {
      await wsService.connect()
    } catch (error) {
      console.error('Failed to connect to WebSocket:', error)
    }
  }

  // Deployment functions
  const startDeployment = async () => {
    if (!isConfigComplete.value) {
      throw new Error('Configuration is incomplete')
    }

    isDeploying.value = true
    deploymentError.value = null
    deploymentLogs.value = []
    deploymentProgress.value = null

    try {
      console.log('Starting deployment with config:', config.value)

      // Ensure WebSocket is connected for progress updates
      if (!isConnected.value) {
        await initializeWebSocket()
      }

      // Send deployment request to backend
      const response = await apiService.deploy({
        template: config.value.selectedTemplate || 'default',
        config: config.value
      })

      if (response.data && response.data.deployment_id) {
        deploymentId.value = response.data.deployment_id
        console.log('Deployment started:', deploymentId.value)

        // Subscribe to deployment progress updates
        if (deploymentId.value) {
          wsService.subscribeToDeployment(deploymentId.value)
        }

        return deploymentId.value
      } else {
        throw new Error('Invalid response from deployment API')
      }
    } catch (error) {
      isDeploying.value = false
      deploymentError.value = error instanceof Error ? error.message : 'Deployment failed'
      console.error('Deployment error:', error)
      throw error
    }
  }

  const cancelDeployment = () => {
    if (deploymentId.value) {
      // TODO: Implement cancel deployment API call
      console.log('Canceling deployment:', deploymentId.value)
    }

    isDeploying.value = false
    deploymentId.value = null
    subnetId.value = null
    deploymentProgress.value = null
  }

  const resetDeployment = () => {
    isDeploying.value = false
    deploymentId.value = null
    subnetId.value = null
    deploymentProgress.value = null
    deploymentError.value = null
    deploymentLogs.value = []
  }

  const retryDeployment = async () => {
    if (!isConfigComplete.value) {
      throw new Error('Configuration is incomplete')
    }

    console.log('Retrying deployment with current config')

    // Reset deployment state but keep the config
    deploymentError.value = null
    deploymentLogs.value = []
    deploymentProgress.value = null

    // Start a new deployment
    return await startDeployment()
  }

  // Configuration persistence
  const saveConfiguration = async (name: string) => {
    try {
      const configData = {
        name,
        config: config.value,
        timestamp: new Date().toISOString()
      }

      await apiService.saveConfig(configData)
      console.log('Configuration saved:', name)
    } catch (error) {
      console.error('Failed to save configuration:', error)
      throw error
    }
  }

  const loadConfiguration = async (name: string) => {
    try {
      const response = await apiService.loadConfig(name)

      if (response.data && response.data.config) {
        config.value = response.data.config
        isDirty.value = false
        console.log('Configuration loaded:', name)
      }
    } catch (error) {
      console.error('Failed to load configuration:', error)
      throw error
    }
  }

  return {
    // State
    config,
    currentStep,
    validationErrors,
    isValidating,
    isDirty,

    // Deployment state
    isDeploying: computed(() => isDeploying.value),
    deploymentId: computed(() => deploymentId.value),
    subnetId: computed(() => subnetId.value),
    deploymentProgress: computed(() => deploymentProgress.value),
    deploymentError: computed(() => deploymentError.value),
    deploymentLogs: computed(() => deploymentLogs.value),
    isConnected: computed(() => isConnected.value),

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
    exportConfig,

    // Deployment actions
    initializeWebSocket,
    startDeployment,
    cancelDeployment,
    resetDeployment,
    retryDeployment,

    // Configuration management
    saveConfiguration,
    loadConfiguration
  }
})