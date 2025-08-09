<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useTemplatesStore } from '../../stores/templates'
import { useWizardStore } from '../../stores/wizard'

const router = useRouter()
const wizardStore = useWizardStore()
const templatesStore = useTemplatesStore()

// Questionnaire state
const currentQuestion = ref(0)
const answers = ref<Record<string, string>>(wizardStore.config.questionnaire || {})
const questionnaireSkipped = ref(false)

// Load from stores
const questions = computed(() => templatesStore.questions)
const templates = computed(() => templatesStore.templates)

// Computed recommended templates based on answers
const recommendedTemplates = computed(() => {
  return templatesStore.getRecommendedTemplates(answers.value)
})

// Show all templates if questionnaire not completed or skipped
const displayedTemplates = computed(() => {
  // If questionnaire was completed with answers, show recommendations
  if (!questionnaireSkipped.value && Object.keys(answers.value).length === questions.value.length) {
    return recommendedTemplates.value
  }
  // Otherwise show all templates
  return templates.value
})

// Navigation functions
const selectAnswer = (questionId: string, value: string) => {
  answers.value[questionId] = value

  // Save to store
  wizardStore.updateConfig({
    questionnaire: answers.value
  })

  if (currentQuestion.value < questions.value.length - 1) {
    currentQuestion.value++
  }
}

const goToPreviousQuestion = () => {
  if (currentQuestion.value > 0) {
    currentQuestion.value--
  }
}

const skipQuestionnaire = () => {
  questionnaireSkipped.value = true
  currentQuestion.value = questions.value.length

  // Save to store that questionnaire was skipped
  wizardStore.updateConfig({
    questionnaireSkipped: true
  })
}

const selectTemplate = (templateId: string) => {
  // Apply template defaults to wizard config
  const smartDefaults = templatesStore.getSmartDefaults(templateId, answers.value)

  wizardStore.updateConfig({
    selectedTemplate: templateId,
    questionnaire: answers.value,
    ...smartDefaults
  })

  // Navigate to basic config step
  router.push({ name: 'wizard-basic' })
}

const questionnaireCompleted = computed(() => {
  return questionnaireSkipped.value || Object.keys(answers.value).length === questions.value.length
})

// Show questionnaire section
const showQuestionnaire = computed(() => {
  return !questionnaireSkipped.value && currentQuestion.value < questions.value.length
})

// Initialize from existing store data
onMounted(async () => {
  // Initialize templates from API
  await templatesStore.ensureInitialized()

  if (wizardStore.config.questionnaire) {
    answers.value = wizardStore.config.questionnaire
    if (Object.keys(answers.value).length === questions.value.length) {
      currentQuestion.value = questions.value.length
    }
  }

  // Check if questionnaire was previously skipped
  if (wizardStore.config.questionnaireSkipped) {
    questionnaireSkipped.value = true
    currentQuestion.value = questions.value.length
  }
})
</script>

<template>
  <div>
    <!-- Questionnaire Section -->
    <div v-if="showQuestionnaire" class="card mb-8">
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
          {{ questionnaireCompleted && !questionnaireSkipped ? 'Recommended Templates' : 'Available Templates' }}
        </h2>
        <p class="text-gray-600">
          {{ questionnaireCompleted && !questionnaireSkipped
            ? 'Based on your answers, these templates are recommended for your use case:'
            : questionnaireSkipped
            ? 'Choose a template to get started. Each template provides smart defaults for your subnet configuration. Select "Manual Configuration" for complete control over all settings:'
            : 'Choose a template to get started, or answer the questions above for personalized recommendations:'
          }}
        </p>
      </div>

      <!-- Help text when questionnaire skipped -->
      <div v-if="questionnaireSkipped" class="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
        <div class="flex items-start space-x-3">
          <svg class="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
          </svg>
          <div>
            <h3 class="font-semibold text-blue-800 mb-1">Template Selection</h3>
            <p class="text-blue-700 text-sm">
              Templates provide pre-configured settings for common use cases. After selecting a template, you'll be able to review and modify all settings in the following steps. Choose "Manual Configuration" if you prefer to set everything yourself.
            </p>
          </div>
        </div>
      </div>

      <!-- Loading State -->
      <div v-if="templatesStore.isLoading" class="text-center py-12">
        <div class="animate-spin inline-block w-8 h-8 border-4 border-current border-t-transparent text-primary-600 rounded-full" role="status" aria-label="loading">
        </div>
        <p class="mt-2 text-gray-600">Loading templates...</p>
      </div>

      <!-- Error State -->
      <div v-else-if="templatesStore.error" class="text-center py-12">
        <div class="text-yellow-600 mb-2">
          <svg class="w-12 h-12 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
        </div>
        <p class="text-gray-600 mb-4">{{ templatesStore.error }}</p>
        <button @click="templatesStore.refreshTemplates()" class="btn-primary">
          Retry Loading Templates
        </button>
      </div>

      <!-- Templates Grid -->
      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
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

      <!-- Show all templates toggle if questionnaire completed with answers -->
      <div v-if="questionnaireCompleted && !questionnaireSkipped && recommendedTemplates.length < templates.length" class="mt-8 text-center">
        <button
          @click="answers = {}"
          class="text-primary-600 hover:text-primary-700 text-sm font-medium"
        >
          Show all templates
        </button>
      </div>

      <!-- Empty state if no recommendations -->
      <div v-if="questionnaireCompleted && !questionnaireSkipped && recommendedTemplates.length === 0" class="text-center py-8">
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