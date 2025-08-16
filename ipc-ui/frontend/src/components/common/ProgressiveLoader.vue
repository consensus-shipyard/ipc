<template>
  <span class="progressive-loader" :class="{ 'inline': inline }">
    <span v-if="showText" class="text">{{ text }}</span>
    <span class="dots">
      <span class="dot" :style="{ animationDelay: '0s' }">.</span>
      <span class="dot" :style="{ animationDelay: '0.2s' }">.</span>
      <span class="dot" :style="{ animationDelay: '0.4s' }">.</span>
    </span>
  </span>
</template>

<script setup lang="ts">
interface Props {
  text?: string
  showText?: boolean
  inline?: boolean
}

withDefaults(defineProps<Props>(), {
  text: 'Loading',
  showText: true,
  inline: true
})
</script>

<style scoped>
.progressive-loader {
  display: inline-flex;
  align-items: baseline;
  color: #6b7280; /* text-gray-500 */
}

.progressive-loader:not(.inline) {
  display: flex;
}

.text {
  margin-right: 2px;
}

.dots {
  display: inline-flex;
  letter-spacing: 0.1em;
}

.dot {
  animation: blink 1.4s infinite;
  animation-fill-mode: both;
}

@keyframes blink {
  0%, 80%, 100% {
    opacity: 0;
  }
  40% {
    opacity: 1;
  }
}
</style>
