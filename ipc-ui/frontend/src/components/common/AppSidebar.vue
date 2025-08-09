<script setup lang="ts">
import { useNetworkStore } from '@/stores/network'
import { useSubnetsStore } from '@/stores/subnets'
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
    name: 'Subnets',
    href: '/subnets',
    icon: 'network',
    current: false
  },
  {
    name: 'Contracts',
    href: '/contracts',
    icon: 'code',
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

// Stores
const subnetsStore = useSubnetsStore()
const networkStore = useNetworkStore()

// Real subnet data state
const loading = ref(true)
const error = ref<string | null>(null)
const systemStatus = ref({
  cliConnection: 'Unknown',
  walletsConfigured: 0,
  gateways: 0
})

// Gateway data state
const deployedGateways = ref<any[]>([])
const gatewaysLoading = ref(false)
const gatewaysError = ref<string | null>(null)

// Get recent subnets from store
const recentSubnets = computed(() => subnetsStore.recentSubnets)

// Fetch real subnet data
const fetchRecentSubnets = () => subnetsStore.fetchSubnets()

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
    // Discover gateways from IPC config first (finds previously deployed gateways)
    console.log('[AppSidebar] Starting gateway discovery...')
    const gatewaysResponse = await apiService.discoverGateways()
    console.log('[AppSidebar] Gateway discovery response:', gatewaysResponse.data)

    // Use the response from discovery which now returns the full list
    if (gatewaysResponse.data && gatewaysResponse.data.data && Array.isArray(gatewaysResponse.data.data)) {
      const gateways = gatewaysResponse.data.data
      console.log(`[AppSidebar] Found ${gateways.length} gateways from discovery`)

      // Log gateway details for debugging
      gateways.forEach((gateway: any, index: number) => {
        console.log(`[AppSidebar] Gateway ${index + 1}:`, {
          id: gateway.id,
          name: gateway.name,
          gateway_address: gateway.gateway_address,
          parent_network: gateway.parent_network,
          status: gateway.status
        })
      })

      systemStatus.value.gateways = gateways.length
      deployedGateways.value = gateways.slice(0, 3) // Show up to 3 gateways in sidebar
      console.log(`[AppSidebar] Set sidebar to show ${deployedGateways.value.length} gateways`)
    } else {
      console.warn('[AppSidebar] Invalid or empty gateway discovery response')
      systemStatus.value.gateways = 0
      deployedGateways.value = []
    }
  } catch (err) {
    console.error('[AppSidebar] Gateway discovery failed:', err)
    systemStatus.value.gateways = 0
    deployedGateways.value = []
  }
}

// Fetch deployed gateways specifically
const fetchDeployedGateways = async () => {
  try {
    console.log('[AppSidebar] Fetching deployed gateways specifically...')
    gatewaysLoading.value = true
    gatewaysError.value = null

    // Just fetch the current list without discovering again
    // (discovery is handled by fetchSystemStatus)
    const response = await apiService.getGateways()
    console.log('[AppSidebar] Deployed gateways response:', response.data)

    if (response.data && response.data.data && Array.isArray(response.data.data)) {
      console.log(`[AppSidebar] Fetched ${response.data.data.length} deployed gateways`)
      deployedGateways.value = response.data.data
      systemStatus.value.gateways = response.data.data.length
    } else {
      console.warn('[AppSidebar] Invalid deployed gateways response')
      deployedGateways.value = []
      systemStatus.value.gateways = 0
    }
  } catch (err: any) {
    console.error('[AppSidebar] Error fetching deployed gateways:', err)
    gatewaysError.value = err?.message || 'Failed to load gateways'
    deployedGateways.value = []
    systemStatus.value.gateways = 0
  } finally {
    gatewaysLoading.value = false
    console.log('[AppSidebar] Deployed gateways fetch completed')
  }
}

// Format subnet ID for display
const formatSubnetId = (id: string) => {
  if (!id) return 'Unknown'
  const parts = id.split('/')
  return parts[parts.length - 1]?.substring(0, 8) + '...' || 'Unknown'
}

// Format address for display
const formatAddress = (address: string) => {
  if (!address) return 'Unknown'
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

// Get status color for display
const getStatusColor = (status: string) => {
  switch (status.toLowerCase()) {
    case 'active': return 'text-green-500'
    case 'paused': return 'text-yellow-500'
    case 'deploying': return 'text-blue-500'
    case 'failed': return 'text-red-500'
    case 'pending approval': return 'text-orange-500'
    case 'approved - no validators': return 'text-blue-500'
    case 'inactive': return 'text-gray-500'
    default: return 'text-gray-500'
  }
}

// Get network connection status display
const getNetworkConnectionStatus = computed(() => {
  const status = networkStore.selectedNetworkStatus
  if (!status) return { text: 'Unknown', color: 'text-gray-500', bgColor: 'bg-gray-500' }

  if (status.connected) {
    return {
      text: 'Connected',
      color: 'text-green-600',
      bgColor: 'bg-green-500'
    }
  } else {
    return {
      text: 'Disconnected',
      color: 'text-red-600',
      bgColor: 'bg-red-500'
    }
  }
})

// SVG icons
const icons = {
  home: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z M9 22V12h6v10',
  plus: 'M12 4v16m8-8H4',
  network: 'M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z',
  code: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4',
  settings: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z'
}

// Lifecycle
onMounted(async () => {
  // Data fetching is now handled by the centralized app store
  // Just fetch system status which is sidebar-specific
  await fetchSystemStatus()
})
</script>

<template>
  <aside class="fixed top-16 left-0 z-40 w-64 h-screen bg-white border-r border-gray-200 shadow-sm">
    <div class="flex flex-col h-full px-4 py-4 overflow-y-auto">
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

      <!-- Deployed Gateways Section -->
      <div class="mt-8">
        <h3 class="px-4 text-xs font-semibold text-gray-400 uppercase tracking-wider">
          Deployed Gateways
        </h3>

        <!-- Loading State -->
        <div v-if="gatewaysLoading" class="mt-3 px-4">
          <div class="animate-pulse space-y-2">
            <div class="h-10 bg-gray-200 rounded"></div>
            <div class="h-10 bg-gray-200 rounded"></div>
          </div>
        </div>

        <!-- Error State -->
        <div v-else-if="gatewaysError" class="mt-3 px-4 py-2 text-xs text-red-600 bg-red-50 rounded-lg">
          Failed to load gateways
        </div>

        <!-- No Gateways -->
        <div v-else-if="deployedGateways.length === 0" class="mt-3 px-4 py-2 text-sm text-gray-500">
          No gateways found
        </div>

        <!-- Gateway Entries -->
        <div v-else class="mt-3 space-y-1">
          <div
            v-for="gateway in deployedGateways.slice(0, 3)"
            :key="gateway.id"
            class="px-4 py-2 text-sm text-gray-600 hover:bg-gray-50 rounded-lg cursor-pointer transition-colors"
          >
            <div class="font-medium truncate">{{ gateway.name }}</div>
            <div class="flex items-center justify-between text-xs">
              <span :class="gateway.status === 'active' ? 'text-green-500' : 'text-gray-400'">
                {{ gateway.status }}
              </span>
              <span class="text-gray-400 truncate ml-2" :title="gateway.gateway_address">
                {{ formatAddress(gateway.gateway_address) }}
              </span>
            </div>
            <div class="text-xs text-gray-400 mt-1">
              {{ gateway.parent_network }}
            </div>
          </div>

          <!-- View All Link -->
          <div v-if="deployedGateways.length > 3" class="mt-2 px-4">
            <button class="text-xs text-primary-600 hover:text-primary-700 font-medium">
              View all {{ deployedGateways.length }} gateways
            </button>
          </div>
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
              <span class="text-gray-600">Network</span>
              <span class="flex items-center" :class="getNetworkConnectionStatus.color">
                <div
                  :class="[
                    'w-2 h-2 rounded-full mr-2',
                    getNetworkConnectionStatus.bgColor,
                    { 'animate-pulse': networkStore.isTestingConnection }
                  ]"
                ></div>
                {{ getNetworkConnectionStatus.text }}
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