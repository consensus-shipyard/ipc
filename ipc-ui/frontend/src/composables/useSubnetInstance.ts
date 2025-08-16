/**
 * Composable for managing subnet instance data
 */
import { ref, computed } from 'vue'
import type { SubnetInstance } from '@/types/subnet'
import { ConfigService } from '@/services/subnet/config.service'
import { formatAddress, formatAddressShort, extractSubnetActorAddress } from '@/utils/address'

export function useSubnetInstance(subnetId: string) {
  const instance = ref<SubnetInstance | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Computed properties
  const createdDate = computed(() => {
    if (!instance.value?.data?.created_at && !instance.value?.created_at) return 'Unknown'

    const dateStr = instance.value.data?.created_at || instance.value.created_at
    try {
      return new Date(dateStr).toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      })
    } catch (err) {
      console.warn('Error parsing created_at date:', dateStr)
      return 'Invalid Date'
    }
  })

  const totalStake = computed(() => {
    const validators = instance.value?.data?.validators || instance.value?.validators
    if (!validators) return '0'
    return validators
      .reduce((sum, v) => sum + parseFloat(v.stake || '0'), 0)
      .toFixed(2)
  })

  const totalPower = computed(() => {
    const validators = instance.value?.data?.validators || instance.value?.validators
    if (!validators) return 0
    return validators.reduce((sum, v) => sum + (v.power || 0), 0)
  })

  const gatewayAddress = computed(() => {
    const addr = instance.value?.data?.config?.gateway_addr || instance.value?.config?.gateway_addr
    if (!addr) return 'N/A'
    return formatAddress(addr)
  })

  const gatewayAddressShort = computed(() => {
    const addr = instance.value?.data?.config?.gateway_addr || instance.value?.config?.gateway_addr
    if (!addr) return 'N/A'
    return formatAddressShort(addr)
  })

  const subnetActorAddress = computed(() => {
    const id = instance.value?.data?.id || instance.value?.id
    if (!id) return 'N/A'
    return extractSubnetActorAddress(id)
  })

  const subnetActorAddressShort = computed(() => {
    // For subnet actor addresses, we typically want to show the full address
    return subnetActorAddress.value
  })

  const statusColor = computed(() => {
    const status = instance.value?.data?.status || instance.value?.status
    if (!status) return 'text-gray-600 bg-gray-50'

    switch (status.toLowerCase()) {
      case 'active': return 'text-green-600 bg-green-50'
      case 'paused': return 'text-yellow-600 bg-yellow-50'
      case 'deploying': return 'text-blue-600 bg-blue-50'
      case 'failed': return 'text-red-600 bg-red-50'
      default: return 'text-gray-600 bg-gray-50'
    }
  })

  const permissionMode = computed(() => {
    return instance.value?.data?.config?.permissionMode ||
           instance.value?.config?.permissionMode ||
           'unknown'
  })

  const validatorCount = computed(() => {
    return instance.value?.data?.validator_count ||
           instance.value?.data?.validators?.length ||
           instance.value?.validators?.length ||
           0
  })

  const validators = computed(() => {
    return instance.value?.data?.validators || instance.value?.validators || []
  })

  // Methods
  const fetchInstance = async () => {
    try {
      loading.value = true
      error.value = null

      const response = await ConfigService.getInstance(subnetId)

      // Check if we got HTML instead of JSON (indicates backend routing issue)
      if (typeof response.data === 'string' && (response.data as string).includes('<!DOCTYPE html>')) {
        error.value = 'Backend routing error: API endpoint returned HTML instead of JSON data.'
        return
      }

      if (response.data) {
        instance.value = response.data
      } else {
        error.value = 'Instance not found'
      }
    } catch (err) {
      console.error('Error fetching instance:', err)
      error.value = err instanceof Error ? err.message : 'Failed to load instance'
    } finally {
      loading.value = false
    }
  }

  const approveSubnet = async () => {
    if (!instance.value) return { success: false, error: 'No instance loaded' }

    try {
      const gatewayOwnerAddress = await ConfigService.getGatewayOwner(instance.value)
      const response = await ConfigService.approveSubnet(subnetId, gatewayOwnerAddress)

      if (response.data?.success) {
        // Refresh the instance data to show updated status
        await fetchInstance()
        return { success: true, message: response.data.message }
      } else {
        return { success: false, error: response.data?.error || 'Failed to approve subnet' }
      }
    } catch (err) {
      console.error('Error approving subnet:', err)
      return { success: false, error: err instanceof Error ? err.message : 'Failed to approve subnet' }
    }
  }

  const exportConfig = () => {
    if (instance.value) {
      ConfigService.exportConfig(instance.value)
    }
  }

  return {
    // State
    instance,
    loading,
    error,

    // Computed
    createdDate,
    totalStake,
    totalPower,
    gatewayAddress,
    gatewayAddressShort,
    subnetActorAddress,
    subnetActorAddressShort,
    statusColor,
    permissionMode,
    validatorCount,
    validators,

    // Methods
    fetchInstance,
    approveSubnet,
    exportConfig
  }
}
