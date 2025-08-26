<template>
  <div v-if="show" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
    <div class="bg-white rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden">
      <div class="p-6 border-b border-gray-200">
        <div class="flex items-center justify-between">
          <h2 class="text-xl font-semibold text-gray-900">Manage Subnet Approvals</h2>
          <button
            @click="$emit('close')"
            class="text-gray-400 hover:text-gray-600"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <p class="text-sm text-gray-600 mt-2">
          Gateway: {{ gatewayAddress }}
        </p>
      </div>

      <div class="p-6 max-h-96 overflow-y-auto">
        <!-- Loading State -->
        <div v-if="loading" class="flex items-center justify-center py-8">
          <div class="flex items-center space-x-2">
            <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary-600"></div>
            <span class="text-gray-600">Loading pending approvals...</span>
          </div>
        </div>

        <!-- Error State -->
        <div v-else-if="error" class="text-center py-8">
          <div class="text-red-600 text-sm">{{ error }}</div>
          <button
            @click="loadPendingApprovals"
            class="mt-2 text-primary-600 hover:text-primary-700 text-sm"
          >
            Try Again
          </button>
        </div>

        <!-- No Pending Approvals -->
        <div v-else-if="pendingSubnets.length === 0" class="text-center py-8">
          <svg class="w-12 h-12 text-gray-400 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <h3 class="text-lg font-medium text-gray-900 mb-2">All Caught Up!</h3>
          <p class="text-gray-600">No subnets are pending approval for this gateway.</p>
        </div>

        <!-- Pending Subnets List -->
        <div v-else class="space-y-4">
          <div
            v-for="subnet in pendingSubnets"
            :key="subnet.subnet_id"
            class="border border-gray-200 rounded-lg p-4 hover:bg-gray-50 transition-colors"
          >
            <div class="flex items-start justify-between">
              <div class="flex-1">
                <div class="flex items-center space-x-2 mb-2">
                  <h4 class="font-medium text-gray-900">{{ subnet.subnet_id }}</h4>
                  <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                    Pending Approval
                  </span>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-gray-600">
                  <div>
                    <span class="font-medium">Parent:</span> {{ subnet.parent_id }}
                  </div>
                  <div>
                    <span class="font-medium">Created:</span>
                    {{ formatDate(subnet.created_at) }}
                  </div>
                  <div class="md:col-span-2">
                    <span class="font-medium">Registry:</span>
                    <code class="bg-gray-100 px-1 rounded text-xs">{{ subnet.registry_address }}</code>
                  </div>
                </div>
              </div>

              <div class="flex items-center space-x-2 ml-4">
                <button
                  @click="approveSubnet(subnet.subnet_id)"
                  :disabled="approvingSubnets.has(subnet.subnet_id)"
                  class="btn-primary text-sm"
                >
                  <svg v-if="approvingSubnets.has(subnet.subnet_id)" class="w-4 h-4 mr-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                  <svg v-else class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                  {{ approvingSubnets.has(subnet.subnet_id) ? 'Approving...' : 'Approve' }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="px-6 py-4 border-t border-gray-200 bg-gray-50">
        <div class="flex justify-between items-center">
          <button
            @click="loadPendingApprovals"
            :disabled="loading"
            class="btn-secondary text-sm"
          >
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            Refresh
          </button>

          <button
            @click="$emit('close')"
            class="btn-secondary"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { apiService } from '@/services/api'
import { useWalletStore } from '@/stores/wallet'
import { onMounted, ref, watch } from 'vue'

interface PendingSubnet {
  subnet_id: string
  parent_id: string
  gateway_address: string
  registry_address: string
  status: string
  created_at: string
}

interface Props {
  show: boolean
  gatewayAddress: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
  approved: [subnetId: string]
}>()

const loading = ref(false)
const error = ref<string | null>(null)
const pendingSubnets = ref<PendingSubnet[]>([])
const approvingSubnets = ref(new Set<string>())

const loadPendingApprovals = async () => {
  if (!props.gatewayAddress) return

  loading.value = true
  error.value = null

  try {
    const response = await apiService.listPendingApprovals(props.gatewayAddress)
    if (response.data.success) {
      pendingSubnets.value = response.data.data || []
    } else {
      error.value = response.data.error || 'Failed to load pending approvals'
    }
  } catch (err) {
    console.error('Failed to load pending approvals:', err)
    error.value = 'Failed to load pending approvals'
  } finally {
    loading.value = false
  }
}

const approveSubnet = async (subnetId: string) => {
  approvingSubnets.value.add(subnetId)

  try {
    console.log('Approving subnet:', subnetId)

    // Get the user's default wallet address to sign the approval transaction
    const walletStore = useWalletStore()
    await walletStore.fetchAddresses()

    const fromAddress = walletStore.defaultAddress
    if (!fromAddress) {
      error.value = 'Please select a wallet address to approve the subnet'
      approvingSubnets.value.delete(subnetId)
      return
    }

    console.log('Using wallet address for approval:', fromAddress)
    const response = await apiService.approveSubnet(subnetId, fromAddress)

    if (response.data.success) {
      // Remove from pending list
      pendingSubnets.value = pendingSubnets.value.filter(s => s.subnet_id !== subnetId)
      emit('approved', subnetId)
    } else {
      error.value = response.data.error || 'Failed to approve subnet'
    }
  } catch (err) {
    console.error('Failed to approve subnet:', err)
    error.value = 'Failed to approve subnet'
  } finally {
    approvingSubnets.value.delete(subnetId)
  }
}

const formatDate = (dateString: string) => {
  try {
    return new Date(dateString).toLocaleDateString() + ' ' + new Date(dateString).toLocaleTimeString()
  } catch {
    return dateString
  }
}

// Load pending approvals when modal opens
watch(() => props.show, (newShow) => {
  if (newShow) {
    loadPendingApprovals()
  }
})

onMounted(() => {
  if (props.show) {
    loadPendingApprovals()
  }
})
</script>