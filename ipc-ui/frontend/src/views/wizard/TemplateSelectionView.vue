<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useWizardStore } from '../../stores/wizard'
import { useTemplatesStore } from '../../stores/templates'

const router = useRouter()
const wizardStore = useWizardStore()
const templatesStore = useTemplatesStore()

// Questionnaire state
const currentQuestion = ref(0)
const answers = ref<Record<string, string>>(wizardStore.config.questionnaire || {})

// Load from stores
const questions = computed(() => templatesStore.questions)
const templates = computed(() => templatesStore.templates)

// Computed recommended templates based on answers
const recommendedTemplates = computed(() => {
  return templatesStore.getRecommendedTemplates(answers.value)
})

// Show all templates if questionnaire not completed
const displayedTemplates = computed(() => {
  return Object.keys(answers.value).length === questions.value.length
    ? recommendedTemplates.value
    : templates.value
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
  currentQuestion.value = questions.value.length
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
  return Object.keys(answers.value).length === questions.value.length
})

// Initialize from existing store data
onMounted(() => {
  if (wizardStore.config.questionnaire) {
    answers.value = wizardStore.config.questionnaire
    if (Object.keys(answers.value).length === questions.value.length) {
      currentQuestion.value = questions.value.length
    }
  }
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