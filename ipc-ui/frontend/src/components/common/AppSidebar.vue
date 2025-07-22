<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

// Navigation items
const navigationItems = [
  {
    name: 'Dashboard',
    href: '/',
    icon: 'home',
    current: false
  },
  {
    name: 'Deploy Subnet',
    href: '/wizard',
    icon: 'plus',
    current: false
  },
  {
    name: 'Settings',
    href: '/settings',
    icon: 'settings',
    current: false
  }
]

// Update current state based on route
const currentNavigationItems = computed(() => {
  return navigationItems.map(item => ({
    ...item,
    current: route.path === item.href || (item.href === '/wizard' && route.path.startsWith('/wizard'))
  }))
})

// SVG icons
const icons = {
  home: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z M9 22V12h6v10',
  plus: 'M12 4v16m8-8H4',
  settings: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z'
}
</script>

<template>
  <aside class="fixed top-16 left-0 z-40 w-64 h-screen bg-white border-r border-gray-200 shadow-sm">
    <div class="h-full px-3 py-4 overflow-y-auto">
      <!-- Navigation -->
      <nav class="space-y-2">
        <RouterLink
          v-for="item in currentNavigationItems"
          :key="item.name"
          :to="item.href"
          :class="[
            'flex items-center px-4 py-3 text-sm font-medium rounded-lg transition-colors',
            item.current
              ? 'bg-primary-50 text-primary-700 border-l-4 border-primary-500'
              : 'text-gray-700 hover:bg-gray-50 hover:text-gray-900'
          ]"
        >
          <svg
            :class="[
              'w-5 h-5 mr-3',
              item.current ? 'text-primary-500' : 'text-gray-400'
            ]"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              :d="icons[item.icon]"
            />
          </svg>
          {{ item.name }}
        </RouterLink>
      </nav>

      <!-- Recent Subnets Section -->
      <div class="mt-8">
        <h3 class="px-4 text-xs font-semibold text-gray-400 uppercase tracking-wider">
          Recent Subnets
        </h3>
        <div class="mt-3 space-y-1">
          <!-- Mock subnet entries - will be replaced with real data -->
          <div class="px-4 py-2 text-sm text-gray-600 hover:bg-gray-50 rounded-lg cursor-pointer">
            <div class="font-medium">Production Subnet A</div>
            <div class="text-xs text-gray-400">Active • /r31337/subnet-1</div>
          </div>
          <div class="px-4 py-2 text-sm text-gray-600 hover:bg-gray-50 rounded-lg cursor-pointer">
            <div class="font-medium">Test Environment</div>
            <div class="text-xs text-gray-400">Paused • /r31337/subnet-2</div>
          </div>
          <div class="px-4 py-2 text-sm text-gray-600 hover:bg-gray-50 rounded-lg cursor-pointer">
            <div class="font-medium">Dev Subnet</div>
            <div class="text-xs text-gray-400">Active • /r31337/subnet-3</div>
          </div>
        </div>
      </div>

      <!-- Status Section -->
      <div class="mt-8 px-4">
        <div class="bg-gray-50 rounded-lg p-4">
          <h4 class="text-sm font-medium text-gray-700 mb-2">System Status</h4>
          <div class="space-y-2 text-sm">
            <div class="flex items-center justify-between">
              <span class="text-gray-600">CLI Connection</span>
              <span class="flex items-center text-green-600">
                <div class="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
                Connected
              </span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-gray-600">Wallets</span>
              <span class="text-blue-600">3 configured</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>