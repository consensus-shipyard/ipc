<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { apiService } from '../../services/api'

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

// Real subnet data state
const recentSubnets = ref([])
const loading = ref(true)
const error = ref(null)
const systemStatus = ref({
  cliConnection: 'Unknown',
  walletsConfigured: 0,
  gateways: 0
})

// Fetch real subnet data
const fetchRecentSubnets = async () => {
  try {
    loading.value = true
    error.value = null

    const response = await apiService.getInstances()
    if (response.data) {
      // Get the 5 most recent subnets for the sidebar
      recentSubnets.value = response.data
        .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))
        .slice(0, 5)
    }
  } catch (err) {
    console.error('Error fetching recent subnets:', err)
    error.value = err.message || 'Failed to load subnets'
  } finally {
    loading.value = false
  }
}

// Fetch system status
const fetchSystemStatus = async () => {
  try {
    // Check CLI connection by trying to fetch templates
    const templatesResponse = await apiService.getTemplates()
    if (templatesResponse.data) {
      systemStatus.value.cliConnection = 'Connected'
    }
  } catch (err) {
    console.error('CLI connection check failed:', err)
    systemStatus.value.cliConnection = 'Disconnected'
  }

  try {
    // Check deployed gateways
    const gatewaysResponse = await apiService.getGateways()
    if (gatewaysResponse.data) {
      systemStatus.value.gateways = gatewaysResponse.data.length
    }
  } catch (err) {
    console.error('Gateway check failed:', err)
  }

  // For wallets, we can estimate based on subnets data
  // In a real implementation, this would be a separate API endpoint
  if (recentSubnets.value.length > 0) {
    systemStatus.value.walletsConfigured = 1 // At least one wallet must be configured to create subnets
  }
}

// Format subnet ID for display
const formatSubnetId = (id) => {
  const parts = id.split('/')
  return parts[parts.length - 1] || id
}

// Get status color for display
const getStatusColor = (status) => {
  switch (status.toLowerCase()) {
    case 'active': return 'text-green-500'
    case 'paused': return 'text-yellow-500'
    case 'deploying': return 'text-blue-500'
    case 'failed': return 'text-red-500'
    default: return 'text-gray-500'
  }
}

// SVG icons
const icons = {
  home: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z M9 22V12h6v10',
  plus: 'M12 4v16m8-8H4',
  settings: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z'
}

// Lifecycle
onMounted(async () => {
  await fetchRecentSubnets()
  await fetchSystemStatus()
})
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
              :d="icons[item.icon as keyof typeof icons]"
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

        <!-- Loading State -->
        <div v-if="loading" class="mt-3 px-4">
          <div class="animate-pulse space-y-2">
            <div class="h-10 bg-gray-200 rounded"></div>
            <div class="h-10 bg-gray-200 rounded"></div>
            <div class="h-10 bg-gray-200 rounded"></div>
          </div>
        </div>

        <!-- Error State -->
        <div v-else-if="error" class="mt-3 px-4 py-2 text-xs text-red-600 bg-red-50 rounded-lg">
          Failed to load subnets
        </div>

        <!-- No Subnets -->
        <div v-else-if="recentSubnets.length === 0" class="mt-3 px-4 py-2 text-sm text-gray-500">
          No subnets found
        </div>

        <!-- Real Subnet Entries -->
        <div v-else class="mt-3 space-y-1">
          <RouterLink
            v-for="subnet in recentSubnets"
            :key="subnet.id"
            :to="`/instance/${subnet.id}`"
            class="block px-4 py-2 text-sm text-gray-600 hover:bg-gray-50 rounded-lg cursor-pointer transition-colors"
          >
            <div class="font-medium truncate">{{ subnet.name }}</div>
            <div class="flex items-center justify-between text-xs">
              <span :class="getStatusColor(subnet.status)">{{ subnet.status }}</span>
              <span class="text-gray-400 truncate ml-2" :title="subnet.id">
                {{ formatSubnetId(subnet.id) }}
              </span>
            </div>
          </RouterLink>
        </div>
      </div>

      <!-- System Status Section -->
      <div class="mt-8 px-4">
        <div class="bg-gray-50 rounded-lg p-4">
          <h4 class="text-sm font-medium text-gray-700 mb-2">System Status</h4>
          <div class="space-y-2 text-sm">
            <div class="flex items-center justify-between">
              <span class="text-gray-600">CLI Connection</span>
              <span class="flex items-center" :class="systemStatus.cliConnection === 'Connected' ? 'text-green-600' : 'text-red-600'">
                <div
                  :class="[
                    'w-2 h-2 rounded-full mr-2',
                    systemStatus.cliConnection === 'Connected' ? 'bg-green-500' : 'bg-red-500'
                  ]"
                ></div>
                {{ systemStatus.cliConnection }}
              </span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-gray-600">Subnets</span>
              <span class="text-blue-600">{{ recentSubnets.length }} active</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-gray-600">Gateways</span>
              <span class="text-purple-600">{{ systemStatus.gateways }} configured</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>