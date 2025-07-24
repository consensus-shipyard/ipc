<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

// Get the current page title from route meta
const pageTitle = computed(() => {
  return route.meta?.title || 'IPC Management Console'
})
</script>

<template>
  <header class="fixed top-0 left-0 right-0 z-50 bg-white border-b border-gray-200 shadow-sm">
    <div class="flex items-center justify-between h-16 px-6">
      <!-- Logo and Title -->
      <div class="flex items-center space-x-4">
        <RouterLink to="/" class="flex items-center space-x-3 hover:opacity-80 transition-opacity">
          <div class="w-8 h-8 bg-primary-600 rounded-lg flex items-center justify-center">
            <span class="text-white font-bold text-sm">IPC</span>
          </div>
          <span class="text-xl font-semibold text-gray-900">IPC Console</span>
        </RouterLink>

        <div class="hidden md:block text-gray-400">|</div>
        <div class="hidden md:block text-gray-600">{{ pageTitle }}</div>
      </div>

      <!-- Right side actions -->
      <div class="flex items-center space-x-4">
        <!-- Deploy New Subnet Button -->
        <RouterLink
          to="/wizard"
          class="btn-primary inline-flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Deploy New Subnet
        </RouterLink>

        <!-- Settings -->
        <RouterLink
          to="/settings"
          class="p-2 text-gray-400 hover:text-gray-600 transition-colors"
          title="Settings"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </RouterLink>
      </div>
    </div>
  </header>
</template>