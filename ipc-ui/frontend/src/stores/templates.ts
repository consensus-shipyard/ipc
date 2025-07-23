import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { apiService } from '../services/api'
import type { SubnetConfig } from './wizard'

export interface Template {
  id: string
  name: string
  description: string
  icon: string
  features: string[]
  recommended: string[]
  defaults: Partial<SubnetConfig>
  validation?: {
    requiredFields?: string[]
    recommendations?: Record<string, any>
  }
}

export interface TemplateQuestion {
  id: string
  question: string
  options: Array<{
    value: string
    label: string
    description: string
  }>
}

export const useTemplatesStore = defineStore('templates', () => {
  // Loading state
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const isInitialized = ref(false)

  // Template questions for the questionnaire
  const questions = ref<TemplateQuestion[]>([
    {
      id: 'useCase',
      question: "What's your primary use case?",
      options: [
        { value: 'development', label: 'Development/Testing', description: 'Local development and experimentation' },
        { value: 'staging', label: 'Staging/QA', description: 'Pre-production testing environment' },
        { value: 'production', label: 'Production Launch', description: 'Live production deployment' },
        { value: 'consortium', label: 'Private Consortium', description: 'Private or consortium network' }
      ]
    },
    {
      id: 'decentralization',
      question: "How important is decentralization?",
      options: [
        { value: 'critical', label: 'Very Important', description: 'Maximum decentralization with open participation' },
        { value: 'moderate', label: 'Moderate', description: 'Balanced approach with some barriers' },
        { value: 'minimal', label: 'Not Critical', description: 'Centralized or federated approach is fine' }
      ]
    },
    {
      id: 'throughput',
      question: "Expected transaction volume?",
      options: [
        { value: 'low', label: 'Low (< 1,000 tx/day)', description: 'Occasional usage, basic applications' },
        { value: 'medium', label: 'Medium (1,000-100k tx/day)', description: 'Regular business applications' },
        { value: 'high', label: 'High (> 100k tx/day)', description: 'High-throughput applications' }
      ]
    },
    {
      id: 'validators',
      question: "How many validators do you expect?",
      options: [
        { value: 'few', label: 'Few (1-10)', description: 'Small, controlled validator set' },
        { value: 'medium', label: 'Medium (10-100)', description: 'Moderate validator participation' },
        { value: 'many', label: 'Many (100+)', description: 'Large, diverse validator network' }
      ]
    }
  ])

  // Available templates with smart defaults
  const templates = ref<Template[]>([
    {
      id: 'development',
      name: 'Development Template',
      description: 'Perfect for local development and testing',
      icon: '🧪',
      features: [
        'Federated mode for quick setup',
        'Minimal validators (1-3)',
        'Low stakes and barriers',
        'Fast checkpoints',
        'Local network compatible'
      ],
      recommended: ['development'],
      defaults: {
        permissionMode: 'federated',
        minValidators: 1,
        minValidatorStake: 1.0,
        bottomupCheckPeriod: 10,
        supplySourceKind: 'native',
        minCrossMsgFee: 0.000001,
        networkVersion: 21,
        baseFee: 1000,
        powerScale: 3,
        activationMode: 'federated'
      }
    },
    {
      id: 'staging',
      name: 'Staging Template',
      description: 'Pre-production testing with realistic settings',
      icon: '🚀',
      features: [
        'Collateral mode',
        'Moderate stakes',
        'Realistic validator count',
        'Production-like settings',
        'Lower barriers for testing'
      ],
      recommended: ['staging'],
      defaults: {
        permissionMode: 'collateral',
        minValidators: 3,
        minValidatorStake: 10.0,
        bottomupCheckPeriod: 50,
        supplySourceKind: 'native',
        minCrossMsgFee: 0.01,
        networkVersion: 21,
        baseFee: 1000,
        powerScale: 3,
        activationMode: 'collateral'
      }
    },
    {
      id: 'production',
      name: 'Production Template',
      description: 'Battle-tested configuration for live deployments',
      icon: '🏭',
      features: [
        'Collateral mode',
        'High security settings',
        'Robust validator requirements',
        'Conservative parameters',
        'High stakes protection'
      ],
      recommended: ['production'],
      defaults: {
        permissionMode: 'collateral',
        minValidators: 5,
        minValidatorStake: 100.0,
        bottomupCheckPeriod: 100,
        supplySourceKind: 'native',
        minCrossMsgFee: 0.1,
        activeValidatorsLimit: 100,
        networkVersion: 21,
        baseFee: 1000,
        powerScale: 3,
        activationMode: 'collateral'
      }
    },
    {
      id: 'federated',
      name: 'Federated Network Template',
      description: 'For consortium and private networks',
      icon: '🤝',
      features: [
        'Federated mode',
        'Known validator set',
        'Flexible management',
        'Controlled access',
        'Custom governance'
      ],
      recommended: ['consortium'],
      defaults: {
        permissionMode: 'federated',
        minValidators: 3,
        minValidatorStake: 1.0,
        bottomupCheckPeriod: 50,
        supplySourceKind: 'native',
        minCrossMsgFee: 0.001,
        networkVersion: 21,
        baseFee: 1000,
        powerScale: 3,
        activationMode: 'federated'
      }
    },
    {
      id: 'l1-integration',
      name: 'L1 Integration Template',
      description: 'Connect directly to Ethereum/Filecoin mainnet',
      icon: '🌐',
      features: [
        'Mainnet parent networks',
        'Production-grade security',
        'Conservative settings',
        'High gas considerations',
        'Enterprise ready'
      ],
      recommended: ['production'],
      defaults: {
        permissionMode: 'collateral',
        minValidators: 7,
        minValidatorStake: 500.0,
        bottomupCheckPeriod: 200,
        supplySourceKind: 'native',
        minCrossMsgFee: 1.0,
        activeValidatorsLimit: 200,
        networkVersion: 21,
        baseFee: 10000,
        powerScale: 3,
        activationMode: 'collateral',
        deployConfig: {
          enabled: true,
          subnetCreationPrivilege: 'Restricted'
        }
      }
    },
    {
      id: 'testnet',
      name: 'Testnet Template',
      description: 'Optimized for public testnets',
      icon: '🧪',
      features: [
        'Pre-configured testnet parents',
        'Moderate security settings',
        'Testnet-optimized parameters',
        'Reasonable gas costs',
        'Easy experimentation'
      ],
      recommended: ['staging', 'development'],
      defaults: {
        permissionMode: 'collateral',
        minValidators: 2,
        minValidatorStake: 5.0,
        bottomupCheckPeriod: 30,
        supplySourceKind: 'native',
        minCrossMsgFee: 0.001,
        networkVersion: 21,
        baseFee: 1000,
        powerScale: 3,
        activationMode: 'collateral'
      }
    },
    {
      id: 'multi-token',
      name: 'Multi-token Template',
      description: 'ERC-20 based supply or collateral sources',
      icon: '🪙',
      features: [
        'ERC-20 integration',
        'Custom token contracts',
        'Flexible economics',
        'Token-specific validations',
        'Multi-asset support'
      ],
      recommended: ['production', 'staging'],
      defaults: {
        permissionMode: 'collateral',
        minValidators: 4,
        minValidatorStake: 50.0,
        bottomupCheckPeriod: 75,
        supplySourceKind: 'erc20',
        collateralSourceKind: 'erc20',
        minCrossMsgFee: 0.05,
        networkVersion: 21,
        baseFee: 1000,
        powerScale: 3,
        activationMode: 'collateral'
      }
    }
  ])

  // Common parent networks for dropdown
  const parentNetworks = ref([
    { value: '/r31337', label: 'Local Development (/r31337)', description: 'Local Hardhat/Anvil network' },
    { value: '/r11155111', label: 'Sepolia Testnet (/r11155111)', description: 'Ethereum Sepolia testnet' },
    { value: '/r314159', label: 'Calibration Testnet (/r314159)', description: 'Filecoin Calibration testnet' },
    { value: '/r1', label: 'Ethereum Mainnet (/r1)', description: 'Ethereum mainnet' },
    { value: '/r314', label: 'Filecoin Mainnet (/r314)', description: 'Filecoin mainnet' }
  ])

  // Actions
  const getTemplate = (id: string): Template | undefined => {
    return templates.value.find(template => template.id === id)
  }

  const getRecommendedTemplates = (answers: Record<string, string>): Template[] => {
    const useCase = answers.useCase
    const decentralization = answers.decentralization

    return templates.value.filter(template => {
      // Primary filtering by use case
      if (useCase && template.recommended.includes(useCase)) {
        return true
      }

      // Secondary filtering by decentralization preference
      if (decentralization === 'minimal' && template.id === 'federated') {
        return true
      }

      return false
    })
  }

  const getTemplateDefaults = (templateId: string): Partial<SubnetConfig> => {
    const template = getTemplate(templateId)
    return template?.defaults || {}
  }

  const getSmartDefaults = (templateId: string, answers?: Record<string, string>): Partial<SubnetConfig> => {
    const baseDefaults = getTemplateDefaults(templateId)

    if (!answers) return baseDefaults

    // Apply answer-based adjustments
    const adjustments: Partial<SubnetConfig> = {}

    // Adjust based on validator expectations
    if (answers.validators === 'few') {
      adjustments.minValidators = Math.min(baseDefaults.minValidators || 1, 3)
    } else if (answers.validators === 'many') {
      adjustments.minValidators = Math.max(baseDefaults.minValidators || 5, 7)
      adjustments.activeValidatorsLimit = 500
    }

    // Adjust based on throughput expectations
    if (answers.throughput === 'high') {
      adjustments.bottomupCheckPeriod = Math.max((baseDefaults.bottomupCheckPeriod || 50) * 0.5, 10)
      adjustments.baseFee = (baseDefaults.baseFee || 1000) * 0.1
    } else if (answers.throughput === 'low') {
      adjustments.bottomupCheckPeriod = (baseDefaults.bottomupCheckPeriod || 50) * 2
    }

    // Adjust based on decentralization preference
    if (answers.decentralization === 'critical') {
      adjustments.permissionMode = 'collateral'
      adjustments.minValidators = Math.max(baseDefaults.minValidators || 5, 5)
    } else if (answers.decentralization === 'minimal') {
      adjustments.permissionMode = 'federated'
    }

    return { ...baseDefaults, ...adjustments }
  }

  const validateTemplate = (templateId: string, config: SubnetConfig): string[] => {
    const template = getTemplate(templateId)
    if (!template) return []

    const warnings: string[] = []

    // Check template-specific validations
    if (templateId === 'production' && config.minValidatorStake && config.minValidatorStake < 50) {
      warnings.push('Production deployments should have higher validator stakes for security')
    }

    if (templateId === 'l1-integration' && config.bottomupCheckPeriod && config.bottomupCheckPeriod < 100) {
      warnings.push('L1 integration should use longer checkpoint periods to reduce gas costs')
    }

    if (templateId === 'multi-token' && config.supplySourceKind === 'native') {
      warnings.push('Multi-token template is designed for ERC-20 tokens')
    }

    return warnings
  }

  // API Integration functions
  const loadTemplates = async () => {
    if (isLoading.value || isInitialized.value) return

    isLoading.value = true
    error.value = null

    try {
      console.log('Loading templates from API...')
      const response = await apiService.getTemplates()

      if (response.data && Array.isArray(response.data)) {
        console.log('Loaded templates from API:', response.data.length)
        // Convert backend template format to frontend format if needed
        templates.value = response.data.map((template: any) => ({
          ...template,
          defaults: template.defaults || {}
        }))
      } else {
        console.log('Using fallback mock templates')
        // Keep existing mock templates as fallback
      }

      isInitialized.value = true
    } catch (err) {
      console.error('Failed to load templates from API:', err)
      error.value = 'Failed to load templates. Using offline templates.'
      // Keep existing mock templates as fallback
      isInitialized.value = true
    } finally {
      isLoading.value = false
    }
  }

  const refreshTemplates = async () => {
    isInitialized.value = false
    await loadTemplates()
  }

  // Initialize templates on first access
  const ensureInitialized = async () => {
    if (!isInitialized.value && !isLoading.value) {
      await loadTemplates()
    }
  }

  return {
    // State
    questions,
    templates: computed(() => templates.value),
    parentNetworks,
    isLoading: computed(() => isLoading.value),
    error: computed(() => error.value),
    isInitialized: computed(() => isInitialized.value),

    // Actions
    getTemplate,
    getRecommendedTemplates,
    getTemplateDefaults,
    getSmartDefaults,
    validateTemplate,
    loadTemplates,
    refreshTemplates,
    ensureInitialized
  }
})