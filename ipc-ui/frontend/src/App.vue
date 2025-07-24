<script setup lang="ts">
import { computed } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import AppHeader from './components/common/AppHeader.vue'
import AppSidebar from './components/common/AppSidebar.vue'

const route = useRoute()

// Check if we're in the wizard flow to show different layout
const isWizardRoute = computed(() => {
  return route.path.startsWith('/wizard')
})
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- App Header -->
    <AppHeader />

    <div class="flex h-screen pt-16"> <!-- pt-16 accounts for fixed header -->
      <!-- Sidebar (hidden on wizard pages) -->
      <AppSidebar v-if="!isWizardRoute" />

      <!-- Main Content -->
      <main
        :class="[
          'flex-1 overflow-auto',
          isWizardRoute ? 'ml-0' : 'ml-64' // ml-64 accounts for sidebar width
        ]"
      >
        <RouterView />
      </main>
    </div>
  </div>
</template>

<style scoped>
/* Additional custom styles if needed */
</style>
