<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { getAsciiArt, getBannerLines } from '../../utils/banner'

// Animation state
const showBanner = ref(false)
const showLines = ref(false)
const currentLineIndex = ref(0)

const asciiArt = getAsciiArt()
const bannerLines = getBannerLines()

// Props
interface Props {
  show: boolean
}

defineProps<Props>()

// Animate in the banner
onMounted(() => {
  setTimeout(() => {
    showBanner.value = true
  }, 100)

  setTimeout(() => {
    showLines.value = true
    animateLines()
  }, 800)
})

// Animate text lines one by one
const animateLines = () => {
  const interval = setInterval(() => {
    currentLineIndex.value++
    if (currentLineIndex.value >= bannerLines.length) {
      clearInterval(interval)
    }
  }, 300)
}
</script>

<template>
  <Transition
    name="splash"
    enter-active-class="transition-all duration-500 ease-out"
    leave-active-class="transition-all duration-300 ease-in"
    enter-from-class="opacity-0 scale-95"
    enter-to-class="opacity-100 scale-100"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-105"
  >
    <div
      v-if="show"
      class="fixed inset-0 z-50 flex items-center justify-center bg-gradient-to-br from-purple-900 via-blue-900 to-indigo-900 backdrop-blur-sm"
    >
      <!-- Animated background particles -->
      <div class="absolute inset-0 overflow-hidden">
        <div
          v-for="i in 50"
          :key="i"
          class="absolute w-1 h-1 bg-white rounded-full opacity-20 animate-pulse"
          :style="{
            left: Math.random() * 100 + '%',
            top: Math.random() * 100 + '%',
            animationDelay: Math.random() * 3 + 's',
            animationDuration: (Math.random() * 2 + 1) + 's'
          }"
        />
      </div>

      <!-- Main splash content -->
      <div class="relative text-center max-w-4xl mx-auto px-8">
        <!-- ASCII Art Banner -->
        <Transition
          name="banner"
          enter-active-class="transition-all duration-700 ease-out"
          enter-from-class="opacity-0 transform translate-y-8"
          enter-to-class="opacity-100 transform translate-y-0"
        >
          <div v-if="showBanner" class="mb-8">
            <pre class="ascii-art text-lg md:text-xl lg:text-2xl font-mono leading-tight tracking-wider">{{ asciiArt }}</pre>
          </div>
        </Transition>

        <!-- Animated Text Lines -->
        <Transition
          name="lines"
          enter-active-class="transition-all duration-500 ease-out"
          enter-from-class="opacity-0 transform translate-y-4"
          enter-to-class="opacity-100 transform translate-y-0"
        >
          <div v-if="showLines" class="space-y-4">
            <TransitionGroup
              name="line"
              tag="div"
              enter-active-class="transition-all duration-500 ease-out"
              enter-from-class="opacity-0 transform translate-x-4"
              enter-to-class="opacity-100 transform translate-x-0"
            >
              <div
                v-for="(line, index) in bannerLines"
                v-show="index <= currentLineIndex"
                :key="index"
                class="text-lg md:text-xl text-emerald-300 font-medium"
                :style="{ transitionDelay: `${index * 100}ms` }"
              >
                {{ line }}
              </div>
            </TransitionGroup>
          </div>
        </Transition>

        <!-- Loading indicator -->
        <div class="mt-12 flex flex-col items-center space-y-4">
          <div class="flex space-x-2">
            <div
              v-for="i in 3"
              :key="i"
              class="w-3 h-3 bg-cyan-400 rounded-full animate-bounce"
              :style="{ animationDelay: `${(i - 1) * 0.2}s` }"
            />
          </div>
          <p class="text-gray-300 text-sm animate-pulse">
            Initializing InterPlanetary Consensus...
          </p>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.ascii-art {
  background: linear-gradient(
    45deg,
    #8b5cf6,  /* purple-500 */
    #7c3aed,  /* violet-600 */
    #6366f1,  /* indigo-500 */
    #3b82f6,  /* blue-500 */
    #06b6d4,  /* cyan-500 */
    #0891b2   /* cyan-600 */
  );
  background-size: 300% 300%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: gradient-shift 3s ease-in-out infinite;
  text-shadow: 0 0 30px rgba(139, 92, 246, 0.3);
}

@keyframes gradient-shift {
  0%, 100% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
}

/* Ensure proper text rendering for ASCII art */
.ascii-art {
  font-family: 'Courier New', 'Monaco', 'Menlo', monospace;
  white-space: pre;
  line-height: 1.1;
}

/* Custom animation for particles */
@keyframes float {
  0%, 100% {
    transform: translateY(0px);
  }
  50% {
    transform: translateY(-10px);
  }
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .ascii-art {
    font-size: 0.7rem;
  }
}

@media (max-width: 640px) {
  .ascii-art {
    font-size: 0.6rem;
  }
}
</style>