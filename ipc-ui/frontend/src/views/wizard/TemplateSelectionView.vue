<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

// Questionnaire state
const currentQuestion = ref(0)
const answers = ref<Record<string, string>>({})

// Questions for template selection
const questions = [
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
]

// Template definitions based on our design doc
const templates = [
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
    recommended: ['development']
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
    recommended: ['staging']
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
    recommended: ['production']
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
    recommended: ['consortium']
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
    recommended: ['production']
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
    recommended: ['staging', 'development']
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
    recommended: ['production', 'staging']
  }
]

// Computed recommended templates based on answers
const recommendedTemplates = computed(() => {
  const useCase = answers.value.useCase
  const decentralization = answers.value.decentralization

  return templates.filter(template => {
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
})

// Show all templates if questionnaire not completed
const displayedTemplates = computed(() => {
  return Object.keys(answers.value).length === questions.length
    ? recommendedTemplates.value
    : templates
})

// Navigation functions
const selectAnswer = (questionId: string, value: string) => {
  answers.value[questionId] = value

  if (currentQuestion.value < questions.length - 1) {
    currentQuestion.value++
  }
}

const goToPreviousQuestion = () => {
  if (currentQuestion.value > 0) {
    currentQuestion.value--
  }
}

const skipQuestionnaire = () => {
  currentQuestion.value = questions.length
}

const selectTemplate = (templateId: string) => {
  // In Phase 2, we'll store this in Pinia store
  // For now, just navigate to next step
  router.push({ name: 'wizard-basic', query: { template: templateId } })
}

const questionnaireCompleted = computed(() => {
  return Object.keys(answers.value).length === questions.length
})
</script>

<template>
  <div>
    <!-- Questionnaire Section -->
    <div v-if="!questionnaireCompleted" class="card mb-8">
      <div class="flex items-center justify-between mb-6">
        <h2 class="text-2xl font-bold text-gray-900">Let's find the right template for you</h2>
        <button
          @click="skipQuestionnaire"
          class="text-gray-500 hover:text-gray-700 text-sm underline"
        >
          Skip questionnaire
        </button>
      </div>

      <div class="mb-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm font-medium text-gray-600">
            Question {{ currentQuestion + 1 }} of {{ questions.length }}
          </span>
          <div class="w-48 bg-gray-200 rounded-full h-2">
            <div
              class="bg-primary-600 h-2 rounded-full transition-all duration-300"
              :style="{ width: Math.round(((currentQuestion + 1) / questions.length) * 100) + '%' }"
            ></div>
          </div>
        </div>
      </div>

      <div class="space-y-6">
        <h3 class="text-xl font-semibold text-gray-800">
          {{ questions[currentQuestion].question }}
        </h3>

        <div class="grid gap-3">
          <button
            v-for="option in questions[currentQuestion].options"
            :key="option.value"
            @click="selectAnswer(questions[currentQuestion].id, option.value)"
            :class="[
              'p-4 text-left border-2 rounded-lg transition-all hover:border-primary-300 hover:bg-primary-50',
              answers[questions[currentQuestion].id] === option.value
                ? 'border-primary-500 bg-primary-50'
                : 'border-gray-200 bg-white'
            ]"
          >
            <div class="font-semibold text-gray-900 mb-1">{{ option.label }}</div>
            <div class="text-sm text-gray-600">{{ option.description }}</div>
          </button>
        </div>

        <!-- Navigation -->
        <div class="flex justify-between pt-6">
          <button
            v-if="currentQuestion > 0"
            @click="goToPreviousQuestion"
            class="btn-secondary"
          >
            Previous
          </button>
          <div v-else></div>

          <div class="text-sm text-gray-500">
            {{ Object.keys(answers).length }} of {{ questions.length }} answered
          </div>
        </div>
      </div>
    </div>

    <!-- Template Selection Section -->
    <div class="card">
      <div class="mb-6">
        <h2 class="text-2xl font-bold text-gray-900 mb-2">
          {{ questionnaireCompleted ? 'Recommended Templates' : 'Available Templates' }}
        </h2>
        <p class="text-gray-600">
          {{ questionnaireCompleted
            ? 'Based on your answers, these templates are recommended for your use case:'
            : 'Choose a template to get started, or answer the questions above for personalized recommendations:'
          }}
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div
          v-for="template in displayedTemplates"
          :key="template.id"
          class="border border-gray-200 rounded-lg p-6 hover:border-primary-300 hover:shadow-md transition-all cursor-pointer"
          @click="selectTemplate(template.id)"
        >
          <div class="text-center mb-4">
            <div class="text-4xl mb-2">{{ template.icon }}</div>
            <h3 class="text-lg font-semibold text-gray-900">{{ template.name }}</h3>
            <p class="text-sm text-gray-600 mt-1">{{ template.description }}</p>
          </div>

          <ul class="space-y-2 mb-4">
            <li
              v-for="feature in template.features"
              :key="feature"
              class="flex items-start space-x-2 text-sm"
            >
              <svg class="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
              <span class="text-gray-700">{{ feature }}</span>
            </li>
          </ul>

          <button class="w-full btn-primary">
            Select Template
          </button>
        </div>
      </div>

      <!-- Show all templates toggle if questionnaire completed -->
      <div v-if="questionnaireCompleted && recommendedTemplates.length < templates.length" class="mt-8 text-center">
        <button
          @click="answers = {}"
          class="text-primary-600 hover:text-primary-700 text-sm font-medium"
        >
          Show all templates
        </button>
      </div>

      <!-- Empty state if no recommendations -->
      <div v-if="questionnaireCompleted && recommendedTemplates.length === 0" class="text-center py-8">
        <p class="text-gray-600 mb-4">No specific recommendations found based on your answers.</p>
        <button
          @click="answers = {}"
          class="btn-secondary"
        >
          View All Templates
        </button>
      </div>
    </div>
  </div>
</template>