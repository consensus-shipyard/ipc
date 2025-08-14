<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import FieldLoadingIndicator from '../components/common/FieldLoadingIndicator.vue'
import { apiService } from '../services/api'

interface Validator {
  address: string
  stake: string
  power: number
  status: string
  // Additional properties for federated mode
  current_power?: number
  next_power?: number
  waiting?: boolean
  // Additional properties for collateral mode
  initial_balance?: number
}

interface SubnetInstance {
  id: string
  name: string
  status: string
  template?: string
  parent: string
  created_at: string
  validators: Validator[]
  config: Record<string, any>
  data?: {
    validator_count?: number
    validators?: Validator[]
    [key: string]: any
  }
}

interface ChainStats {
  total_supply: string
  circulating_supply: string
  fees_collected: string
  active_validators: number
  last_checkpoint: string
  uptime: string
  block_height: number
  transaction_count: number
  tps: number
  avg_block_time: number
  latest_block_time: string
  consensus_status: string
  validators_online: number
  pending_transactions?: number
}

interface SubnetStatus {
  status: string
  message: string
  is_active: boolean
  block_height: number
  validators_online: number
  consensus_status: string
  sync_status?: string
}

const router = useRouter()

// Props
const props = defineProps<{
  id: string
}>()

// State
const instance = ref<SubnetInstance | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const activeTab = ref('overview')
const approvingSubnet = ref(false)

// Individual loading states for granular control
const loadingBasicInfo = ref(true)
const loadingChainStats = ref(false)
const basicInfoError = ref<string | null>(null)

// Chain statistics state
const chainStats = ref<ChainStats | null>(null)
const subnetStatus = ref<SubnetStatus | null>(null)
const loadingStats = ref(false)
const statsError = ref<string | null>(null)
const statsRefreshInterval = ref<number | null>(null)

// Test transaction state
const showTestTxModal = ref(false)
const sendingTestTx = ref(false)
const testTxResult = ref<string | null>(null)
const testTxData = ref({
  type: 'simple' as 'transfer' | 'contract_call' | 'simple',
  network: 'subnet' as 'subnet' | 'l1',
  from: '',
  to: '',
  amount: '0.001',
  data: '',
  gas_limit: 21000
})

// Validator management state
const newValidator = ref({
  address: '',
  pubkey: '',
  power: 1,
  collateral: 0,
  initialBalance: 0
})

const addingValidator = ref(false)
const removingValidator = ref<Record<string, boolean>>({})
const updatingStake = ref<Record<string, boolean>>({})
const stakeAmounts = ref<Record<string, number>>({})

// Bulk federated validator management state
const showBulkManagement = ref(false)
const bulkValidators = ref<Array<{
  address: string
  pubkey: string
  power: number
  isNew?: boolean
}>>([])
const settingFederatedPower = ref(false)

// Add validator modal state
const showAddValidatorModal = ref(false)

// Node config modal state
const showNodeConfigModal = ref(false)
const nodeConfigData = ref<{
  validatorAddress: string
  configYaml: string
  commands: any
  filename: string
} | null>(null)
const loadingNodeConfig = ref(false)

// Computed
const createdDate = computed(() => {
  if (!instance.value || !instance.value.created_at) return 'Unknown'

  try {
    return new Date(instance.value.data?.created_at).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch (error) {
    console.warn('Error parsing created_at date:', instance.value.data?.created_at)
    return 'Invalid Date'
  }
})

const totalStake = computed(() => {
  if (!instance.value?.data?.validators) return '0'
  return instance.value.data?.validators
    .reduce((sum, v) => sum + parseFloat(v.stake || '0'), 0)
    .toFixed(2)
})

const totalPower = computed(() => {
  if (!instance.value?.data?.validators) return 0
  return instance.value.data?.validators
    .reduce((sum, v) => sum + (v.power || 0), 0)
})

const gatewayAddress = computed(() => {
  if (!instance.value?.data?.config?.gateway_addr) return 'N/A'
  return formatAddress(instance.value.data?.config?.gateway_addr)
})

const gatewayAddressShort = computed(() => {
  if (!instance.value?.data?.config?.gateway_addr) return 'N/A'
  return formatAddressShort(instance.value.data?.config?.gateway_addr)
})

// Add after the existing computed properties (around line 180)

const subnetActorAddress = computed(() => {
  if (!instance.value?.data?.id) return 'N/A'

  // Extract the subnet actor address from the subnet ID
  // For IPC subnets, the format is typically /r{chainId}/{actorAddress}
  // The address can be in Filecoin format (t410...) or Ethereum format (0x...)
  try {
    const subnetId = instance.value.data.id
    const parts = subnetId.split('/')

    if (parts.length >= 3) {
      // The last part should be the subnet actor address
      const actorAddress = parts[parts.length - 1]

      // Handle Filecoin t410 addresses (delegated/Ethereum-compatible addresses)
      if (actorAddress.startsWith('t410f')) {
        // t410f addresses are Filecoin representations of Ethereum addresses
        // The format is t410f{32-byte-address-in-base32}
        // For display purposes, we'll show the full Filecoin address
        return actorAddress
      }

      // Handle Ethereum addresses (with 0x prefix)
      if (actorAddress.startsWith('0x') && actorAddress.length === 42) {
        return actorAddress
      }

      // Handle raw hex addresses (40 hex chars without 0x)
      if (actorAddress.length === 40 && /^[a-fA-F0-9]+$/.test(actorAddress)) {
        return '0x' + actorAddress
      }

      // Handle other Filecoin address formats (f0, f1, f2, f3, f4)
      if (/^[tf][0-4]/.test(actorAddress)) {
        return actorAddress
      }
    }

    // Fallback: try to extract any address pattern from the raw ID
    // Look for Ethereum addresses
    const ethAddressMatch = subnetId.match(/0x[a-fA-F0-9]{40}/)
    if (ethAddressMatch) {
      return ethAddressMatch[0]
    }

    // Look for Filecoin addresses
    const filAddressMatch = subnetId.match(/[tf][0-4][a-zA-Z0-9]+/)
    if (filAddressMatch) {
      return filAddressMatch[0]
    }

    return 'N/A (unable to parse from subnet ID)'
  } catch (err) {
    console.warn('Error parsing subnet actor address from subnet ID:', err)
    return 'N/A (parse error)'
  }
})

const subnetActorAddressShort = computed(() => {
  // Always return the full address - no truncation
  return subnetActorAddress.value
})

// Copy to clipboard functionality
const copyingAddress = ref<string | null>(null)

const copyToClipboard = async (text: string, type: string = 'address') => {
  try {
    await navigator.clipboard.writeText(text)
    copyingAddress.value = type
    setTimeout(() => {
      copyingAddress.value = null
    }, 2000)
  } catch (err) {
    console.error('Failed to copy to clipboard:', err)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = text
    document.body.appendChild(textArea)
    textArea.focus()
    textArea.select()
    try {
      document.execCommand('copy')
      copyingAddress.value = type
      setTimeout(() => {
        copyingAddress.value = null
      }, 2000)
    } catch (fallbackErr) {
      console.error('Fallback copy failed:', fallbackErr)
    }
    document.body.removeChild(textArea)
  }
}

const statusColor = computed(() => {
  if (!instance.value || !instance.value.status) return 'text-gray-600 bg-gray-50'

  switch (instance.value.status.toLowerCase()) {
    case 'active': return 'text-green-600 bg-green-50'
    case 'paused': return 'text-yellow-600 bg-yellow-50'
    case 'deploying': return 'text-blue-600 bg-blue-50'
    case 'failed': return 'text-red-600 bg-red-50'
    default: return 'text-gray-600 bg-gray-50'
  }
})

// Methods
const fetchInstance = async () => {
  try {
    console.log('[InstanceDetailView] Starting fetchInstance, setting loading states...')
    loading.value = true
    loadingBasicInfo.value = true
    error.value = null
    basicInfoError.value = null

    // Add a small delay to ensure loading state is visible
    await new Promise(resolve => setTimeout(resolve, 100))

    // Decode the URL-encoded ID parameter
    const decodedId = decodeURIComponent(props.id)
    const response = await apiService.getInstance(decodedId)

    // Check if we got HTML instead of JSON (indicates backend routing issue)
    if (typeof response.data === 'string' && response.data.includes('<!DOCTYPE html>')) {
      const errorMsg = 'Backend routing error: API endpoint returned HTML instead of JSON data. This usually means the route is not properly configured.'
      error.value = errorMsg
      basicInfoError.value = errorMsg
      return
    }

    if (response.data) {
      instance.value = response.data
      // Now that we have instance data, fetch chain stats
      fetchChainStats()
    } else {
      const errorMsg = 'Instance not found'
      error.value = errorMsg
      basicInfoError.value = errorMsg
    }
  } catch (err) {
    console.error('Error fetching instance:', err)
    const errorMsg = err instanceof Error ? err.message : 'Failed to load instance'
    error.value = errorMsg
    basicInfoError.value = errorMsg
  } finally {
    console.log('[InstanceDetailView] Finishing fetchInstance, clearing loading states...')
    loading.value = false
    loadingBasicInfo.value = false
  }
}

// Helper function to format addresses
const formatAddress = (address: any) => {
  if (!address) return 'N/A'

  // Handle different address formats
  let addressStr = ''

  if (typeof address === 'string') {
    // Already a string, check if it needs 0x prefix
    addressStr = address
  } else if (Array.isArray(address)) {
    // Handle byte arrays - convert to hex string
    if (address.length >= 20 && address.every(b => typeof b === 'number' && b >= 0 && b <= 255)) {
      // This is a 20-byte (or longer) Ethereum address as array of numbers
      // Take only the first 20 bytes for the address
      const addressBytes = address.slice(0, 20)
      addressStr = '0x' + addressBytes.map(b => b.toString(16).padStart(2, '0')).join('')
    } else {
      return 'N/A (invalid array)'
    }
  } else if (typeof address === 'object') {
    // Handle object format
    if (address.route && Array.isArray(address.route)) {
      // Subnet ID format - extract the address from route
      const lastRoute = address.route[address.route.length - 1]
      if (lastRoute && Array.isArray(lastRoute) && lastRoute.length === 20) {
        addressStr = '0x' + lastRoute.map(b => b.toString(16).padStart(2, '0')).join('')
      } else {
        return 'N/A (invalid route)'
      }
    } else {
      return 'N/A (invalid object)'
    }
  } else if (typeof address === 'number') {
    return 'N/A (single number)'
  } else {
    return 'N/A (unknown format)'
  }

  // Ensure we have a valid hex address format
  if (addressStr && !addressStr.startsWith('0x') && addressStr.length === 40) {
    addressStr = '0x' + addressStr
  }

  // Validate the address length
  if (addressStr.startsWith('0x') && addressStr.length !== 42) {
    return 'N/A (invalid length)'
  }

  return addressStr
}

// Helper function to format address for short display
const formatAddressShort = (address: any) => {
  const fullAddress = formatAddress(address)
  if (fullAddress === 'N/A' || !fullAddress.startsWith('0x')) return fullAddress
  if (fullAddress.length < 14) return fullAddress // Don't truncate short addresses
  return `${fullAddress.slice(0, 8)}...${fullAddress.slice(-6)}`
}

const goBack = () => {
  router.push('/')
}

const exportConfig = () => {
  if (!instance.value) return

  const configData = {
    name: instance.value.name,
    config: instance.value.config,
    validators: instance.value.validators,
    exported_at: new Date().toISOString()
  }

  const blob = new Blob([JSON.stringify(configData, null, 2)], {
    type: 'application/json'
  })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `${instance.value.name}-config.json`
  link.click()
  URL.revokeObjectURL(url)
}

const pauseSubnet = async () => {
  // TODO: Implement pause functionality
  console.log('Pause subnet:', decodeURIComponent(props.id))
}

const resumeSubnet = async () => {
  // TODO: Implement resume functionality
  console.log('Resume subnet:', decodeURIComponent(props.id))
}

const viewLogs = () => {
  // TODO: Implement log viewing
  console.log('View logs for:', decodeURIComponent(props.id))
}

const getGatewayOwner = async (): Promise<string> => {
  if (!instance.value) return '0x0a36d7c34ba5523d5bf783bb47f62371e52e0298'

  try {
    // Try to get gateway information from the API
    const gatewaysResponse = await fetch('/api/gateways')
    const gatewaysResult = await gatewaysResponse.json()

    if (gatewaysResult && Array.isArray(gatewaysResult)) {
      // Find the gateway that matches this subnet's gateway address
      const gatewayAddr = instance.value.config?.gateway_addr?.toString()
      if (gatewayAddr) {
        const matchingGateway = gatewaysResult.find((gw: any) =>
          gw.gateway_address === gatewayAddr
        )
        if (matchingGateway) {
          return matchingGateway.deployer_address
        }
      }
    }
  } catch (err) {
    console.warn('Failed to fetch gateway information:', err)
  }

  // Fallback to config or default address
  return instance.value.config?.deployer_address || '0x0a36d7c34ba5523d5bf783bb47f62371e52e0298'
}

const approveSubnet = async () => {
  if (!instance.value) return

  try {
    approvingSubnet.value = true

    // Get the correct gateway owner address
    const gatewayOwnerAddress = await getGatewayOwner()

    // Use the API service with extended timeout for approval
    const response = await apiService.approveSubnet(props.id, gatewayOwnerAddress)

    if (response.data?.success) {
      console.log('Subnet approved successfully:', response.data.message)
      // Refresh the instance data to show updated status
      await fetchInstance()
    } else {
      console.error('Failed to approve subnet:', response.data?.error)
      error.value = response.data?.error || 'Failed to approve subnet'
    }
  } catch (err: any) {
    console.error('Error approving subnet:', err)
    error.value = err?.message || 'Failed to approve subnet'
  } finally {
    approvingSubnet.value = false
  }
}

// Validator management methods
const addValidator = async () => {
  if (!instance.value) return

  addingValidator.value = true
  try {
    const validatorData = {
      subnetId: decodeURIComponent(props.id),
      address: newValidator.value.address,
      permissionMode: instance.value.config.permissionMode || 'collateral',
      ...((instance.value.config.permissionMode === 'collateral') ? {
        collateral: newValidator.value.collateral,
        initialBalance: newValidator.value.initialBalance || undefined
      } : {
        pubkey: newValidator.value.pubkey,
        power: newValidator.value.power
      })
    }

    const response = await apiService.addValidator(validatorData)

    if (response.data.success) {
      // Reset form
      newValidator.value = {
        address: '',
        pubkey: '',
        power: 1,
        collateral: 0,
        initialBalance: 0
      }

      // Close modal
      showAddValidatorModal.value = false

      // Refresh instance data
      await fetchInstance()

      // Show success message (you could add a toast notification here)
      console.log('Validator added successfully')
    } else {
      error.value = response.data.error || 'Failed to add validator'
    }
  } catch (err) {
    console.error('Error adding validator:', err)
    error.value = err instanceof Error ? err.message : 'Failed to add validator'
  } finally {
    addingValidator.value = false
  }
}

const removeValidator = async (validatorAddress: string) => {
  if (!instance.value) return

  removingValidator.value = { ...removingValidator.value, [validatorAddress]: true }
  try {
    const validatorData = {
      subnetId: decodeURIComponent(props.id),
      address: validatorAddress
    }

    const response = await apiService.removeValidator(validatorData)

    if (response.data.success) {
      // Refresh instance data
      await fetchInstance()
      console.log('Validator removed successfully')
    } else {
      error.value = response.data.error || 'Failed to remove validator'
    }
  } catch (err) {
    console.error('Error removing validator:', err)
    error.value = err instanceof Error ? err.message : 'Failed to remove validator'
  } finally {
    removingValidator.value = { ...removingValidator.value, [validatorAddress]: false }
  }
}

const updateStake = async (validatorAddress: string, action: 'stake' | 'unstake') => {
  if (!instance.value) return

  const amount = stakeAmounts.value[validatorAddress]
  if (!amount || amount <= 0) {
    error.value = 'Please enter a valid stake amount'
    return
  }

  updatingStake.value = { ...updatingStake.value, [validatorAddress]: true }
  try {
    const stakeData = {
      subnetId: decodeURIComponent(props.id),
      address: validatorAddress,
      amount,
      action
    }

    const response = await apiService.updateValidatorStake(stakeData)

    if (response.data.success) {
      // Clear the input
      stakeAmounts.value = { ...stakeAmounts.value, [validatorAddress]: 0 }

      // Refresh instance data
      await fetchInstance()
      console.log(`Stake ${action} successful:`, response.data.message)
    } else {
      error.value = response.data.error || `Failed to ${action} validator`
    }
  } catch (err) {
    console.error(`Error ${action}ing validator:`, err)
    error.value = err instanceof Error ? err.message : `Failed to ${action} validator`
  } finally {
    updatingStake.value = { ...updatingStake.value, [validatorAddress]: false }
  }
}

// Node configuration methods
const showNodeConfig = async (validatorAddress: string) => {
  if (!instance.value) return

  loadingNodeConfig.value = true
  showNodeConfigModal.value = true

  try {
    const subnetId = encodeURIComponent(instance.value.id)

    // Fetch both node config and commands in parallel
    const [configResponse, commandsResponse] = await Promise.all([
      fetch(`/api/subnets/${subnetId}/node-config?validator_address=${encodeURIComponent(validatorAddress)}`).then(r => r.json()),
      fetch(`/api/subnets/${subnetId}/node-commands?validator_address=${encodeURIComponent(validatorAddress)}`).then(r => r.json())
    ])

    if (configResponse.success && commandsResponse.success) {
      nodeConfigData.value = {
        validatorAddress,
        configYaml: configResponse.data.config_yaml,
        commands: commandsResponse.data,
        filename: configResponse.data.filename
      }
    } else {
      error.value = 'Failed to generate node configuration'
      showNodeConfigModal.value = false
    }
  } catch (err) {
    console.error('Error fetching node config:', err)
    error.value = err instanceof Error ? err.message : 'Failed to generate node configuration'
    showNodeConfigModal.value = false
  } finally {
    loadingNodeConfig.value = false
  }
}

const closeNodeConfigModal = () => {
  showNodeConfigModal.value = false
  nodeConfigData.value = null
}

const copyNodeConfig = async () => {
  if (!nodeConfigData.value) return

  try {
    await navigator.clipboard.writeText(nodeConfigData.value.configYaml)
    // You could add a toast notification here
    console.log('Node configuration copied to clipboard')
  } catch (err) {
    console.error('Failed to copy to clipboard:', err)
  }
}

const downloadNodeConfig = () => {
  if (!nodeConfigData.value) return

  const blob = new Blob([nodeConfigData.value.configYaml], { type: 'text/yaml' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = nodeConfigData.value.filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

// Bulk federated validator management methods
const initializeBulkManagement = () => {
  if (!instance.value) return

  // Initialize with existing validators
  bulkValidators.value = instance.value.validators.map(validator => ({
    address: validator.address,
    pubkey: '', // Will need to be filled in manually
    power: validator.power || 1,
    isNew: false
  }))

  showBulkManagement.value = true
}

const addBulkValidator = () => {
  bulkValidators.value.push({
    address: '',
    pubkey: '',
    power: 1,
    isNew: true
  })
}

const removeBulkValidator = (index: number) => {
  bulkValidators.value.splice(index, 1)
}

const setBulkFederatedPower = async () => {
  if (!instance.value || bulkValidators.value.length === 0) return

  // Validate all validators have required fields
  const invalidValidators = bulkValidators.value.filter(v =>
    !v.address.trim() || !v.pubkey.trim() || v.power <= 0
  )

  if (invalidValidators.length > 0) {
    error.value = 'All validators must have a valid address, public key, and power > 0'
    return
  }

  settingFederatedPower.value = true
  try {
    // Find the first existing validator to use as fromAddress
    const fromAddress = instance.value.validators.length > 0 ?
      instance.value.validators[0].address :
      bulkValidators.value[0].address

    const powerData = {
      subnetId: decodeURIComponent(props.id),
      fromAddress,
      validators: bulkValidators.value.map(v => ({
        address: v.address,
        pubkey: v.pubkey,
        power: v.power
      }))
    }

    const response = await apiService.setFederatedPower(powerData)

    if (response.data.success) {
      showBulkManagement.value = false

      // Refresh instance data
      await fetchInstance()

      console.log('Bulk federated power set successfully:', response.data.message)
    } else {
      error.value = response.data.error || 'Failed to set federated power'
    }
  } catch (err) {
    console.error('Error setting bulk federated power:', err)
    error.value = err instanceof Error ? err.message : 'Failed to set federated power'
  } finally {
    settingFederatedPower.value = false
  }
}

// Chain statistics methods
const fetchChainStats = async () => {
  console.log('[InstanceDetailView] Fetching chain stats... instance:', instance.value)

  try {
    // Always set loading states to show loading indicators
    loadingStats.value = true
    loadingChainStats.value = true
    statsError.value = null

    console.log('[InstanceDetailView] Fetching chain stats... loadingStats:', loadingStats.value, 'loadingChainStats:', loadingChainStats.value)

    // Only make API calls if we have instance data
    if (!instance.value) {
      console.log('[InstanceDetailView] No instance data yet, skipping API calls but keeping loading state')
      return
    }

    const [statsResponse, statusResponse] = await Promise.all([
      apiService.getSubnetStats(decodeURIComponent(props.id)),
      apiService.getSubnetStatus(decodeURIComponent(props.id))
    ])

    if (statsResponse.data) {
      chainStats.value = statsResponse.data.data
    }

    if (statusResponse.data) {
      subnetStatus.value = statusResponse.data.data
    }
  } catch (err) {
    console.error('Error fetching chain stats:', err)
    statsError.value = err instanceof Error ? err.message : 'Failed to load chain statistics'
  } finally {
    loadingStats.value = false
    loadingChainStats.value = false
  }
}

const startStatsRefresh = () => {
  // Fetch stats immediately
  fetchChainStats()

  // Set up periodic refresh every 10 seconds
  if (statsRefreshInterval.value) {
    clearInterval(statsRefreshInterval.value)
  }

  statsRefreshInterval.value = setInterval(() => {
    fetchChainStats()
  }, 10000)
}

const stopStatsRefresh = () => {
  if (statsRefreshInterval.value) {
    clearInterval(statsRefreshInterval.value)
    statsRefreshInterval.value = null
  }
}

// Test transaction methods
const openTestTxModal = () => {
  console.log('Opening test transaction modal')
  showTestTxModal.value = true
  testTxResult.value = null

  // Force Vue to update the DOM immediately
  nextTick(() => {
    console.log('Modal should be visible now:', showTestTxModal.value)
  })

  // Set default from address if available
  if (instance.value?.validators && instance.value.validators.length > 0) {
    testTxData.value.from = instance.value.validators[0].address
  }

  // Set default to address as a different validator or gateway
  if (instance.value?.validators && instance.value.validators.length > 1) {
    testTxData.value.to = instance.value.validators[1].address
  } else if (instance.value?.config?.gateway_addr) {
    testTxData.value.to = instance.value.config.gateway_addr
  }
}

const sendTestTransaction = async () => {
  if (!instance.value) return

  sendingTestTx.value = true
  testTxResult.value = null

  const networkName = testTxData.value.network === 'subnet' ? 'Subnet' : 'Parent L1'

  try {
    const response = await apiService.sendTestTransaction(
      decodeURIComponent(props.id),
      testTxData.value
    )

    if (response.data.success) {
      testTxResult.value = `✅ Real transaction sent successfully!
        Network: ${networkName}
        Transaction Hash: ${response.data.txHash || 'N/A'}
        Block: ${response.data.blockNumber || 'Pending'}
        Gas Used: ${response.data.gasUsed || 'N/A'}

        ✅ Transaction successfully executed on the blockchain!`

      // Refresh stats after successful transaction
      setTimeout(() => {
        fetchChainStats()
      }, 2000)
    } else {
      testTxResult.value = `❌ Transaction failed on ${networkName}: ${response.data.error || 'Unknown error'}`
    }
  } catch (err) {
    console.error('Error sending test transaction:', err)
    testTxResult.value = `❌ Transaction failed on ${networkName}: ${err instanceof Error ? err.message : 'Network error'}`
  } finally {
    sendingTestTx.value = false
  }
}

const closeTestTxModal = () => {
  showTestTxModal.value = false
  testTxResult.value = null
  sendingTestTx.value = false
}

// Lifecycle
onMounted(() => {
  fetchInstance()
  startStatsRefresh()
})

// Watch for route changes
watch(() => props.id, (newId) => {
  if (newId) {
    fetchInstance()
    startStatsRefresh()
  }
})

// Cleanup on unmount
onUnmounted(() => {
  stopStatsRefresh()
})
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Header -->
    <div class="bg-white shadow-sm border-b">
      <div class="max-w-7xl mx-auto px-6 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <button
              @click="goBack"
              class="text-gray-600 hover:text-gray-700 flex items-center"
            >
              <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
              Back to Dashboard
            </button>
            <div>
              <h1 class="text-2xl font-bold text-gray-900">
                {{ instance?.data?.name || 'Loading...' }}
              </h1>
              <p class="text-gray-600 mt-1">Subnet ID: {{ decodeURIComponent(props.id) }}</p>
            </div>
          </div>

          <div v-if="instance" class="flex items-center space-x-3">
            <span
              :class="[
                'inline-flex items-center px-3 py-1 rounded-full text-sm font-medium',
                statusColor
              ]"
            >
              {{ (instance.data?.status || 'Unknown').charAt(0).toUpperCase() + (instance.data?.status || 'unknown').slice(1) }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="max-w-7xl mx-auto px-6 py-8">
      <div class="text-center py-12">
        <div class="animate-spin inline-block w-8 h-8 border-4 border-primary-600 border-t-transparent rounded-full"></div>
        <p class="mt-4 text-gray-600">Loading subnet details...</p>
      </div>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="max-w-7xl mx-auto px-6 py-8">
      <div class="bg-red-50 border border-red-200 rounded-lg p-6">
        <div class="flex items-start space-x-3">
          <svg class="w-6 h-6 text-red-600 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          <div>
            <h3 class="font-semibold text-red-800 mb-1">Error Loading Subnet</h3>
            <p class="text-red-700">{{ error }}</p>
            <button
              @click="fetchInstance"
              class="mt-3 text-red-600 hover:text-red-700 font-medium text-sm"
            >
              Try Again
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div v-else-if="instance || loadingBasicInfo" class="max-w-7xl mx-auto px-6 py-8">
      <!-- Quick Actions -->
      <div class="flex flex-wrap gap-3 mb-6">
        <button
          v-if="instance?.data?.status === 'active'"
          @click="pauseSubnet"
          class="btn-secondary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          Pause Subnet
        </button>

        <button
          v-else-if="instance?.data?.status === 'paused'"
          @click="resumeSubnet"
          class="btn-primary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1m4 0h1m-6 4h1m4 0h1M9 16h6" />
          </svg>
          Resume Subnet
        </button>

        <button
          v-if="instance?.data?.status?.toLowerCase() === 'pending approval'"
          :disabled="approvingSubnet"
          @click="approveSubnet"
          class="btn-primary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          {{ approvingSubnet ? 'Approving...' : 'Approve Subnet' }}
        </button>

        <button
          v-if="instance?.data?.status === 'active' || instance?.data?.status === 'Active'"
          @click="openTestTxModal"
          class="btn-primary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
          </svg>
          Send Test Transaction
        </button>

        <button
          @click="viewLogs"
          class="btn-secondary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          View Logs
        </button>

        <button
          @click="exportConfig"
          class="btn-secondary flex items-center"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          Export Config
        </button>
      </div>

      <!-- Tab Navigation -->
      <div class="border-b border-gray-200 mb-6">
        <nav class="flex space-x-8">
          <button
            @click="activeTab = 'overview'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'overview'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Overview
          </button>
          <button
            @click="activeTab = 'validators'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'validators'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
                            Validators ({{ instance.data?.validator_count || instance.validators?.length || 0 }} validator{{ (instance.data?.validator_count || instance.validators?.length || 0) !== 1 ? 's' : '' }})
          </button>
          <button
            @click="activeTab = 'configuration'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'configuration'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Configuration
          </button>
          <button
            @click="activeTab = 'contracts'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'contracts'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Contracts
          </button>
          <button
            @click="activeTab = 'metrics'"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm',
              activeTab === 'metrics'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
          >
            Metrics
          </button>
        </nav>
      </div>

      <!-- Tab Content -->
      <div class="space-y-6">
        <!-- Overview Tab -->
        <div v-if="activeTab === 'overview'" class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <!-- Basic Information -->
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Basic Information</h3>
            <dl class="space-y-3">
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Subnet ID</dt>
                <dd class="text-sm text-gray-900 font-mono">
                  <FieldLoadingIndicator
                    :is-loading="loadingBasicInfo"
                    :has-error="!!basicInfoError"
                    loading-text="Loading..."
                    @retry="fetchInstance"
                  >
                    {{ instance?.data?.id }}
                  </FieldLoadingIndicator>
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Name</dt>
                <dd class="text-sm text-gray-900">
                  <FieldLoadingIndicator
                    :is-loading="loadingBasicInfo"
                    :has-error="!!basicInfoError"
                    loading-text="Loading..."
                    @retry="fetchInstance"
                  >
                    {{ instance?.data?.name }}
                  </FieldLoadingIndicator>
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Status</dt>
                <dd>
                  <FieldLoadingIndicator
                    :is-loading="loadingBasicInfo"
                    :has-error="!!basicInfoError"
                    loading-text="Loading..."
                    @retry="fetchInstance"
                  >
                    <span v-if="instance" :class="['inline-flex items-center px-2 py-1 rounded-full text-xs font-medium', statusColor]">
                      {{ (instance.data?.status || 'Unknown').charAt(0).toUpperCase() + (instance.data?.status || 'unknown').slice(1) }}
                    </span>
                  </FieldLoadingIndicator>
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Template</dt>
                <dd class="text-sm text-gray-900">
                  <FieldLoadingIndicator
                    :is-loading="loadingBasicInfo"
                    :has-error="!!basicInfoError"
                    loading-text="Loading..."
                    @retry="fetchInstance"
                  >
                    {{ instance?.data?.template }}
                  </FieldLoadingIndicator>
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Parent Network</dt>
                <dd class="text-sm text-gray-900 font-mono">
                  <FieldLoadingIndicator
                    :is-loading="loadingBasicInfo"
                    :has-error="!!basicInfoError"
                    loading-text="Loading..."
                    @retry="fetchInstance"
                  >
                    {{ instance?.data?.parent }}
                  </FieldLoadingIndicator>
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Gateway Contract</dt>
                <dd class="text-sm text-gray-900 font-mono relative">
                  <FieldLoadingIndicator
                    :is-loading="loadingBasicInfo"
                    :has-error="!!basicInfoError"
                    loading-text="Loading..."
                    @retry="fetchInstance"
                  >
                    <button
                      v-if="instance"
                      @click="copyToClipboard(gatewayAddress, 'gateway')"
                      class="hover:bg-gray-100 px-2 py-1 rounded transition-colors cursor-pointer text-left"
                      :title="copyingAddress === 'gateway' ? 'Copied!' : `Click to copy: ${gatewayAddress}`"
                    >
                      {{ gatewayAddressShort }}
                      <svg v-if="copyingAddress === 'gateway'" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                      </svg>
                    </button>
                  </FieldLoadingIndicator>
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm font-medium text-gray-500">Created</dt>
                <dd class="text-sm text-gray-900">
                  <FieldLoadingIndicator
                    :is-loading="loadingBasicInfo"
                    :has-error="!!basicInfoError"
                    loading-text="Loading..."
                    @retry="fetchInstance"
                  >
                    {{ createdDate }}
                  </FieldLoadingIndicator>
                </dd>
              </div>
            </dl>
          </div>

          <!-- Quick Stats -->
          <div class="card">
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-lg font-semibold text-gray-900">Chain Statistics</h3>
              <div class="flex items-center space-x-2">
                <FieldLoadingIndicator
                  :is-loading="loadingChainStats"
                  :has-error="!!statsError"
                  loading-text="Loading..."
                  @retry="fetchChainStats"
                >
                  <div v-if="subnetStatus?.is_active" class="flex items-center text-green-600">
                    <div class="w-2 h-2 bg-green-500 rounded-full mr-2 animate-pulse"></div>
                    <span class="text-sm font-medium">Active</span>
                  </div>
                  <div v-else class="flex items-center text-red-600">
                    <div class="w-2 h-2 bg-red-500 rounded-full mr-2"></div>
                    <span class="text-sm font-medium">Inactive</span>
                  </div>
                </FieldLoadingIndicator>
              </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">
                  <FieldLoadingIndicator
                    :is-loading="loadingChainStats"
                    :has-error="!!statsError"
                    loading-text="Loading..."
                    @retry="fetchChainStats"
                  >
                    {{ chainStats?.block_height || subnetStatus?.block_height || 'N/A' }}
                  </FieldLoadingIndicator>
                </div>
                <div class="text-sm text-gray-500">Block Height</div>
                <div v-if="chainStats?.latest_block_time" class="text-xs text-gray-400 mt-1">
                  {{ new Date(chainStats.latest_block_time).toLocaleTimeString() }}
                </div>
              </div>

              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">
                  <FieldLoadingIndicator
                    :is-loading="loadingBasicInfo"
                    :has-error="!!basicInfoError"
                    loading-text="Loading..."
                    @retry="fetchInstance"
                  >
                    {{ instance?.data?.validator_count || instance?.data?.validators?.length || 0 }}
                  </FieldLoadingIndicator>
                </div>
                <div class="text-sm text-gray-500">Validators</div>
                <div class="text-xs text-gray-400 mt-1">
                  <FieldLoadingIndicator
                    :is-loading="loadingChainStats"
                    :has-error="!!statsError"
                    loading-text="Loading..."
                    @retry="fetchChainStats"
                  >
                    {{ subnetStatus?.validators_online !== undefined ? `${subnetStatus.validators_online} online` : 'N/A online' }}
                  </FieldLoadingIndicator>
                </div>
              </div>

              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">
                  <FieldLoadingIndicator
                    :is-loading="loadingChainStats"
                    :has-error="!!statsError"
                    loading-text="Loading..."
                    @retry="fetchChainStats"
                  >
                    {{ chainStats?.transaction_count || 'N/A' }}
                  </FieldLoadingIndicator>
                </div>
                <div class="text-sm text-gray-500">Total Transactions</div>
                <div class="text-xs text-gray-400 mt-1">
                  <FieldLoadingIndicator
                    :is-loading="loadingChainStats"
                    :has-error="!!statsError"
                    loading-text="Loading..."
                    @retry="fetchChainStats"
                  >
                    {{ chainStats?.tps ? `${chainStats.tps.toFixed(1)} TPS` : 'N/A TPS' }}
                  </FieldLoadingIndicator>
                </div>
              </div>

              <div class="text-center p-4 bg-gray-50 rounded-lg">
                <div class="text-2xl font-bold text-gray-900">
                  <FieldLoadingIndicator
                    :is-loading="loadingChainStats"
                    :has-error="!!statsError"
                    loading-text="Loading..."
                    @retry="fetchChainStats"
                  >
                    <span v-if="subnetStatus?.consensus_status === 'healthy'" class="text-green-600">●</span>
                    <span v-else-if="subnetStatus?.consensus_status === 'degraded'" class="text-yellow-600">●</span>
                    <span v-else-if="subnetStatus?.consensus_status === 'offline'" class="text-red-600">●</span>
                    <span v-else class="text-gray-400">●</span>
                    {{ subnetStatus?.consensus_status || 'Unknown' }}
                  </FieldLoadingIndicator>
                </div>
                <div class="text-sm text-gray-500">Consensus</div>
                <div class="text-xs text-gray-400 mt-1">
                  <FieldLoadingIndicator
                    :is-loading="loadingChainStats"
                    :has-error="!!statsError"
                    loading-text="Loading..."
                    @retry="fetchChainStats"
                  >
                    {{ chainStats?.avg_block_time ? `${chainStats.avg_block_time.toFixed(1)}s avg block` : 'N/A avg block' }}
                  </FieldLoadingIndicator>
                </div>
              </div>
            </div>

            <!-- Error state for stats -->
            <div v-if="statsError" class="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
              <p class="text-red-700 text-sm">{{ statsError }}</p>
              <button @click="fetchChainStats" class="text-red-600 hover:text-red-700 text-sm font-medium mt-1">
                Retry
              </button>
            </div>
          </div>
        </div>

        <!-- Validators Tab -->
        <div v-if="activeTab === 'validators'" class="space-y-6">
          <!-- Permission Mode Explanation -->
          <div class="p-4 bg-blue-50 border border-blue-200 rounded-lg">
            <h4 class="text-md font-semibold text-blue-800 mb-2 flex items-center">
              <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              {{
                instance?.data?.config?.permissionMode === 'federated' ? 'Federated Mode' :
                instance?.data?.config?.permissionMode === 'collateral' ? 'Collateral Mode' :
                instance?.data?.config?.permissionMode === 'static' ? 'Static Mode' :
                instance?.data?.config?.permissionMode === 'root' ? 'Root Network' :
                instance?.data?.config?.permissionMode === 'unknown' ? 'Unknown Mode (not set)' :
                `Unknown Mode (${instance?.data?.config?.permissionMode || 'not set'})`
              }}
            </h4>

            <div v-if="instance?.data?.config?.permissionMode === 'federated'" class="text-blue-700 text-sm">
              <p class="mb-2"><strong>Federated subnets</strong> use centralized validator management:</p>
              <ul class="list-disc list-inside space-y-1 ml-4">
                <li>Validators are added by setting their power directly</li>
                <li>No collateral staking required</li>
                <li>Network owner controls validator set</li>
                <li>Changes are applied to all validators simultaneously</li>
              </ul>
            </div>

            <div v-else-if="instance?.config?.permissionMode === 'collateral'" class="text-blue-700 text-sm">
              <p class="mb-2"><strong>Collateral subnets</strong> use stake-based validator management:</p>
              <ul class="list-disc list-inside space-y-1 ml-4">
                <li>Validators join by staking FIL collateral</li>
                <li>Minimum stake requirement: {{ instance?.config?.minValidatorStake || 'Not set' }} FIL</li>
                <li>Validators can increase/decrease their stake</li>
                <li>Higher stake generally means higher voting power</li>
              </ul>
            </div>

            <div v-else-if="instance?.config?.permissionMode === 'static'" class="text-blue-700 text-sm">
              <p class="mb-2"><strong>Static subnets</strong> use predefined validator sets:</p>
              <ul class="list-disc list-inside space-y-1 ml-4">
                <li>Validators are defined at subnet creation time</li>
                <li>No dynamic joining or leaving of validators</li>
                <li>Fixed validator set for the subnet's lifetime</li>
                <li>No staking or power changes after deployment</li>
              </ul>
            </div>

            <div v-else-if="instance?.config?.permissionMode === 'root'" class="text-blue-700 text-sm">
              <p class="mb-2"><strong>Root networks</strong> are the base layer networks:</p>
              <ul class="list-disc list-inside space-y-1 ml-4">
                <li>This is a root network, not a subnet</li>
                <li>Root networks don't have permission modes</li>
                <li>They serve as parent networks for subnets</li>
                <li>Validator management depends on the underlying consensus mechanism</li>
              </ul>
            </div>

            <div v-else class="text-yellow-700 text-sm">
              <p class="mb-2"><strong>{{ instance?.config?.permissionMode === 'unknown' ? 'Permission mode could not be determined' : 'Unrecognized permission mode' }}</strong>:</p>
              <ul class="list-disc list-inside space-y-1 ml-4">
                <li v-if="instance?.config?.permissionMode === 'unknown'">Unable to retrieve permission mode from the blockchain</li>
                <li v-else>Unrecognized permission mode: "{{ instance?.config?.permissionMode }}"</li>
                <li><strong>Possible causes:</strong></li>
                <li class="ml-4">• Parent network connectivity issues</li>
                <li class="ml-4">• Subnet not fully deployed or synchronized</li>
                <li class="ml-4">• IPC configuration problems</li>
                <li class="ml-4">• Network or blockchain synchronization delays</li>
                <li><strong>Troubleshooting:</strong></li>
                <li class="ml-4">• Check parent network configuration in IPC settings</li>
                <li class="ml-4">• Verify network connectivity</li>
                <li class="ml-4">• Wait for blockchain synchronization to complete</li>
                <li class="ml-4">• Check subnet deployment status</li>
              </ul>
            </div>
          </div>

          <!-- Bulk Federated Management (Federated Mode Only) -->
          <div v-if="instance?.config?.permissionMode === 'federated'" class="p-6 bg-blue-50 rounded-lg">
            <div class="flex items-center justify-between mb-4">
              <h4 class="text-md font-semibold text-gray-800 flex items-center">
                <svg class="w-5 h-5 mr-2 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                </svg>
                Advanced Federated Management
              </h4>
              <button
                v-if="!showBulkManagement"
                @click="initializeBulkManagement"
                class="btn-secondary text-sm"
              >
                Manage All Validators
              </button>
              <button
                v-else
                @click="showBulkManagement = false"
                class="btn-secondary text-sm"
              >
                Cancel
              </button>
            </div>

            <div v-if="!showBulkManagement" class="text-sm text-blue-700">
              <p class="mb-2">💡 <strong>Tip:</strong> Use bulk management to:</p>
              <ul class="list-disc list-inside space-y-1 ml-4">
                <li>Set power for all validators at once</li>
                <li>Add multiple validators simultaneously</li>
                <li>Manage the complete validator set in one operation</li>
              </ul>
            </div>

            <!-- Bulk Management Form -->
            <div v-if="showBulkManagement" class="space-y-4">
              <div class="bg-yellow-50 border border-yellow-200 rounded-md p-3 mb-4">
                <p class="text-yellow-800 text-sm">
                  <strong>⚠️ Important:</strong> This will set the complete validator set. All validators not listed here will be removed from the subnet.
                </p>
              </div>

              <div class="space-y-3">
                <div v-for="(validator, index) in bulkValidators" :key="index"
                     class="grid grid-cols-12 gap-2 items-center p-3 bg-white rounded border">
                  <div class="col-span-4">
                    <input
                      v-model="validator.address"
                      type="text"
                      placeholder="Validator Address (0x...)"
                      class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
                    />
                  </div>
                  <div class="col-span-4">
                    <input
                      v-model="validator.pubkey"
                      type="text"
                      placeholder="Public Key (0x04...)"
                      class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
                    />
                  </div>
                  <div class="col-span-2">
                    <input
                      v-model.number="validator.power"
                      type="number"
                      min="1"
                      placeholder="Power"
                      class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
                    />
                  </div>
                  <div class="col-span-1">
                    <span v-if="validator.isNew" class="text-xs text-green-600 font-medium">NEW</span>
                    <span v-else class="text-xs text-blue-600 font-medium">EXISTING</span>
                  </div>
                  <div class="col-span-1">
                    <button
                      @click="removeBulkValidator(index)"
                      type="button"
                      class="text-red-600 hover:text-red-800 p-1"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>

              <div class="flex items-center justify-between">
                <button
                  @click="addBulkValidator"
                  type="button"
                  class="btn-secondary text-sm"
                >
                  <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                  </svg>
                  Add Validator
                </button>

                <button
                  @click="setBulkFederatedPower"
                  :disabled="settingFederatedPower || bulkValidators.length === 0"
                  class="btn-primary"
                >
                  <div v-if="settingFederatedPower" class="animate-spin inline-block w-4 h-4 mr-2 border-2 border-current border-t-transparent rounded-full"></div>
                  {{ settingFederatedPower ? 'Setting Power...' : 'Set Federated Power' }}
                </button>
              </div>
            </div>
          </div>

          <!-- Validators Table -->
          <div class="card">
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-lg font-semibold text-gray-900">Validators</h3>
              <div class="flex items-center space-x-3">
                <div class="text-sm text-gray-500">
                  {{ instance.data?.validator_count || instance.validators?.length || 0 }} validator{{ (instance.data?.validator_count || instance.validators?.length || 0) !== 1 ? 's' : '' }}
                </div>
                <button
                  @click="showAddValidatorModal = true"
                  class="btn-primary"
                  title="Add new validator"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                  </svg>
                </button>
              </div>
            </div>

            <div v-if="(instance.data?.validators?.length || 0) === 0" class="text-center py-8 text-gray-500">
              <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
              </svg>
              <div class="space-y-2">
                <p class="font-medium text-gray-900">No Validators Found</p>
                <p class="text-sm text-gray-500 max-w-md mx-auto">
                  {{
                    instance?.config?.permissionMode === 'federated' ? 'No validators have been configured for this federated subnet yet.' :
                    instance?.config?.permissionMode === 'collateral' ? 'No validators have joined this subnet by staking collateral yet.' :
                    instance?.config?.permissionMode === 'static' ? 'No validators are configured for this static subnet.' :
                    instance?.config?.permissionMode === 'root' ? 'Root networks manage validators through their underlying consensus mechanism.' :
                    instance?.config?.permissionMode === 'unknown' ? 'Unable to retrieve validator information due to configuration issues.' :
                    'No validators have been configured for this subnet yet.'
                  }}
                </p>
                <div class="mt-4 text-xs text-gray-400 space-y-1">
                  <p><strong>Possible reasons:</strong></p>
                  <ul class="list-disc list-inside space-y-1 max-w-lg mx-auto text-left">
                    <li v-if="instance?.config?.permissionMode === 'root'">Root networks don't display validators in the subnet interface</li>
                    <li v-else-if="instance?.config?.permissionMode === 'unknown'">Permission mode could not be determined - check network connectivity</li>
                    <li v-else>The subnet was recently created and validators haven't joined yet</li>
                    <li v-if="instance?.config?.permissionMode !== 'root'">The parent network may not be properly configured in your IPC settings</li>
                    <li v-if="instance?.config?.permissionMode !== 'root'">Network connectivity issues preventing validator data retrieval</li>
                    <li v-if="instance?.config?.permissionMode !== 'root'">Validators may be configured but not yet visible due to synchronization delays</li>
                    <li v-if="instance?.config?.permissionMode === 'unknown'">Blockchain synchronization is still in progress</li>
                  </ul>
                </div>
                <div v-if="instance?.data?.config?.permissionMode === 'unknown'" class="mt-4 p-3 bg-yellow-50 border border-yellow-200 rounded-md">
                  <p class="text-yellow-800 text-sm font-medium mb-2">⚠️ Configuration Issue Detected</p>
                  <p class="text-yellow-700 text-xs">
                    The subnet's permission mode could not be determined. This usually indicates a connectivity or configuration problem.
                    Check the browser console and server logs for more details.
                  </p>
                </div>
              </div>
              <button
                v-if="instance?.data?.config?.permissionMode !== 'root' && instance?.data?.config?.permissionMode !== 'unknown'"
                @click="showAddValidatorModal = true"
                class="mt-6 btn-primary"
              >
                Add Validator
              </button>
              <div v-else-if="instance?.data?.config?.permissionMode === 'unknown'" class="mt-6 space-x-2">
                <button
                  @click="fetchInstance"
                  class="btn-secondary"
                >
                  Retry Loading
                </button>
              </div>
            </div>

            <div v-else class="overflow-x-auto">
              <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                  <tr>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Address
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Stake
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Power
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Status
                    </th>
                    <th v-if="instance?.data?.config?.permissionMode === 'collateral'" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Stake Actions
                    </th>
                    <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                  <tr v-for="validator in (instance.data?.validators || instance.validators)" :key="validator.address">
                    <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900">
                      <button
                        @click="copyToClipboard(validator.address, validator.address)"
                        class="hover:bg-gray-100 px-2 py-1 rounded transition-colors cursor-pointer text-left"
                        :title="copyingAddress === validator.address ? 'Copied!' : `Click to copy: ${validator.address}`"
                      >
                        {{ validator.address }}
                        <svg v-if="copyingAddress === validator.address" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                          <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                        </svg>
                      </button>
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      <div v-if="instance.data?.config?.permissionMode === 'collateral'">
                        {{ validator.stake }} FIL
                        <span v-if="validator.initial_balance" class="block text-xs text-gray-500">
                          Initial: {{ validator.initial_balance }} FIL
                        </span>
                      </div>
                      <div v-else>
                        {{ validator.stake }} FIL
                      </div>
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      <div v-if="instance.data?.config?.permissionMode === 'federated'">
                        <div class="flex items-center space-x-2">
                          <span>{{ validator.current_power || validator.power || '0' }}</span>
                          <span v-if="validator.next_power !== undefined && validator.current_power !== validator.next_power"
                                class="text-blue-600 text-xs">
                            → {{ validator.next_power }}
                          </span>
                        </div>
                        <div v-if="validator.waiting" class="text-yellow-600 text-xs">
                          ⏳ Pending
                        </div>
                      </div>
                      <div v-else>
                        {{ validator.power }}
                      </div>
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap">
                      <span :class="[
                        'inline-flex px-2 py-1 text-xs font-semibold rounded-full',
                        validator.status === 'Active' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                      ]">
                        {{ validator.status }}
                      </span>
                      <!-- Power transition indicator for federated mode -->
                      <span v-if="instance.data?.config?.permissionMode === 'federated' && validator.current_power !== validator.next_power"
                            class="ml-2 inline-flex px-2 py-1 text-xs font-medium rounded-full bg-yellow-100 text-yellow-800">
                        Power Changing
                      </span>
                    </td>
                    <td v-if="instance.data?.config?.permissionMode === 'collateral'" class="px-6 py-4 whitespace-nowrap">
                      <div class="flex items-center space-x-2">
                        <input
                          v-model.number="stakeAmounts[validator.address]"
                          type="number"
                          step="0.01"
                          min="0"
                          placeholder="Amount"
                          class="w-20 px-2 py-1 text-xs border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
                        />
                        <button
                          @click="updateStake(validator.address, 'stake')"
                          :disabled="updatingStake[validator.address]"
                          class="btn-secondary text-xs px-2 py-1"
                        >
                          Stake
                        </button>
                        <button
                          @click="updateStake(validator.address, 'unstake')"
                          :disabled="updatingStake[validator.address]"
                          class="btn-secondary text-xs px-2 py-1"
                        >
                          Unstake
                        </button>
                      </div>
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-right">
                      <div class="flex items-center justify-end space-x-2">
                        <!-- Node Config Button -->
                        <button
                          @click="showNodeConfig(validator.address)"
                          class="text-blue-600 hover:text-blue-700 p-2 rounded hover:bg-blue-50 transition-colors"
                          title="View node configuration"
                        >
                          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                          </svg>
                        </button>
                        <!-- Remove Button -->
                        <button
                          @click="removeValidator(validator.address)"
                          :disabled="removingValidator[validator.address]"
                          class="text-red-600 hover:text-red-700 p-2 rounded hover:bg-red-50 transition-colors"
                          :title="removingValidator[validator.address] ? 'Removing...' : 'Remove validator'"
                        >
                          <svg v-if="removingValidator[validator.address]" class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                          </svg>
                          <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                          </svg>
                        </button>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- Add Validator Modal -->
          <div v-if="showAddValidatorModal" class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
            <div class="relative top-20 mx-auto p-5 border w-11/12 max-w-md shadow-lg rounded-md bg-white">
              <div class="mt-3">
                <div class="flex items-center justify-between mb-4">
                  <h3 class="text-lg font-medium text-gray-900">Add New Validator</h3>
                  <button
                    @click="showAddValidatorModal = false"
                    class="text-gray-400 hover:text-gray-600"
                  >
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>

                <!-- Mode-specific instructions -->
                <div class="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded-md">
                  <div v-if="instance?.data?.config?.permissionMode === 'federated'" class="text-yellow-800 text-sm">
                    <p class="font-medium mb-1">📋 Federated Mode Instructions:</p>
                    <p>Enter the validator's Ethereum address, public key, and desired power level. The validator will be added to the network with the specified power.</p>
                  </div>

                  <div v-else-if="instance?.data?.config?.permissionMode === 'collateral'" class="text-yellow-800 text-sm">
                    <p class="font-medium mb-1">💰 Collateral Mode Instructions:</p>
                    <p>Enter the validator's address and collateral amount. The validator must have sufficient FIL to stake the specified collateral.</p>
                  </div>
                </div>

                <form @submit.prevent="addValidator" class="space-y-4">
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Validator Address *
                    </label>
                    <input
                      v-model="newValidator.address"
                      type="text"
                      placeholder="0x..."
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                      required
                    />
                  </div>

                  <div v-if="instance?.data?.config?.permissionMode === 'federated' || instance?.data?.config?.permissionMode === 'static'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Public Key *
                    </label>
                    <input
                      v-model="newValidator.pubkey"
                      type="text"
                      placeholder="0x04..."
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                      required
                    />
                  </div>

                  <div v-if="instance?.data?.config?.permissionMode === 'federated' || instance?.data?.config?.permissionMode === 'static'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Power
                    </label>
                    <input
                      v-model.number="newValidator.power"
                      type="number"
                      min="1"
                      placeholder="1"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                    />
                  </div>

                  <div v-if="instance?.data?.config?.permissionMode === 'collateral'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Collateral (FIL) *
                    </label>
                    <input
                      v-model.number="newValidator.collateral"
                      type="number"
                      step="0.01"
                      min="0"
                      placeholder="10.0"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                      required
                    />
                  </div>

                  <div v-if="instance?.data?.config?.permissionMode === 'collateral'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Initial Balance (FIL)
                    </label>
                    <input
                      v-model.number="newValidator.initialBalance"
                      type="number"
                      step="0.01"
                      min="0"
                      placeholder="0.0"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                    />
                  </div>

                  <div class="flex justify-end space-x-3 pt-4">
                    <button
                      type="button"
                      @click="showAddValidatorModal = false"
                      class="btn-secondary"
                    >
                      Cancel
                    </button>
                    <button
                      type="submit"
                      :disabled="addingValidator"
                      class="btn-primary"
                    >
                      <div v-if="addingValidator" class="animate-spin inline-block w-4 h-4 mr-2 border-2 border-current border-t-transparent rounded-full"></div>
                      {{ addingValidator ? 'Adding...' : 'Add Validator' }}
                    </button>
                  </div>
                </form>
              </div>
            </div>
          </div>

          <!-- Node Config Modal -->
          <div v-if="showNodeConfigModal" class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
            <div class="relative top-10 mx-auto p-5 border w-11/12 max-w-4xl shadow-lg rounded-md bg-white">
              <div class="mt-3">
                <div class="flex items-center justify-between mb-4">
                  <h3 class="text-lg font-medium text-gray-900">
                    Node Configuration for {{ nodeConfigData?.validatorAddress || 'Validator' }}
                  </h3>
                  <button
                    @click="closeNodeConfigModal"
                    class="text-gray-400 hover:text-gray-600"
                  >
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>

                <div v-if="loadingNodeConfig" class="flex items-center justify-center py-8">
                  <div class="animate-spin inline-block w-8 h-8 border-4 border-current border-t-transparent rounded-full text-blue-600"></div>
                  <span class="ml-3 text-gray-600">Generating node configuration...</span>
                </div>

                <div v-else-if="nodeConfigData" class="space-y-6">
                  <!-- Configuration File Section -->
                  <div class="bg-gray-50 rounded-lg p-4">
                    <div class="flex items-center justify-between mb-3">
                      <h4 class="text-md font-semibold text-gray-900">Node Configuration File</h4>
                      <div class="flex space-x-2">
                        <button
                          @click="copyNodeConfig"
                          class="btn-secondary text-sm"
                          title="Copy to clipboard"
                        >
                          <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                          </svg>
                          Copy
                        </button>
                        <button
                          @click="downloadNodeConfig"
                          class="btn-primary text-sm"
                          title="Download file"
                        >
                          <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-4-4m4 4l4-4m3 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                          </svg>
                          Download
                        </button>
                      </div>
                    </div>
                    <p class="text-sm text-gray-600 mb-3">
                      Save this configuration as <code class="bg-gray-200 px-1 rounded">{{ nodeConfigData.filename }}</code>
                    </p>
                    <pre class="bg-white border rounded p-3 text-sm overflow-x-auto max-h-64"><code>{{ nodeConfigData.configYaml }}</code></pre>
                  </div>

                  <!-- Commands Section -->
                  <div class="bg-blue-50 rounded-lg p-4">
                    <h4 class="text-md font-semibold text-gray-900 mb-3">Setup Commands</h4>
                    <p class="text-sm text-gray-600 mb-4">
                      Run these commands in order to set up and start your validator node:
                    </p>

                    <div class="space-y-4">
                      <div v-for="command in nodeConfigData.commands.commands" :key="command.step" class="bg-white rounded border p-3">
                        <div class="flex items-start justify-between mb-2">
                          <div>
                            <h5 class="font-medium text-gray-900">Step {{ command.step }}: {{ command.title }}</h5>
                            <p class="text-sm text-gray-600">{{ command.description }}</p>
                          </div>
                          <span v-if="command.required" class="inline-flex px-2 py-1 text-xs font-medium rounded-full bg-red-100 text-red-800">
                            Required
                          </span>
                        </div>
                        <pre class="bg-gray-100 rounded p-2 text-sm overflow-x-auto"><code>{{ command.command }}</code></pre>
                        <p v-if="command.condition" class="text-xs text-gray-500 mt-1">{{ command.condition }}</p>
                      </div>
                    </div>

                    <!-- Prerequisites -->
                    <div class="mt-4 p-3 bg-yellow-50 border border-yellow-200 rounded">
                      <h5 class="font-medium text-yellow-800 mb-2">Prerequisites:</h5>
                      <ul class="text-sm text-yellow-700 space-y-1">
                        <li v-for="prerequisite in nodeConfigData.commands.prerequisites" :key="prerequisite" class="flex items-start">
                          <span class="mr-2">•</span>
                          <span>{{ prerequisite }}</span>
                        </li>
                      </ul>
                    </div>

                    <!-- Important Notes -->
                    <div class="mt-4 p-3 bg-orange-50 border border-orange-200 rounded">
                      <h5 class="font-medium text-orange-800 mb-2">Important Notes:</h5>
                      <ul class="text-sm text-orange-700 space-y-1">
                        <li v-for="note in nodeConfigData.commands.notes" :key="note" class="flex items-start">
                          <span class="mr-2">⚠️</span>
                          <span>{{ note }}</span>
                        </li>
                      </ul>
                    </div>
                  </div>
                </div>

                <div class="flex justify-end pt-4">
                  <button
                    @click="closeNodeConfigModal"
                    class="btn-secondary"
                  >
                    Close
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Test Transaction Modal -->
          <div v-if="showTestTxModal" class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
            <div class="relative top-20 mx-auto p-5 border w-11/12 max-w-lg shadow-lg rounded-md bg-white">
              <div class="mt-3">
                <div class="flex items-center justify-between mb-4">
                  <h3 class="text-lg font-medium text-gray-900">Send Test Transaction</h3>
                  <button
                    @click="closeTestTxModal"
                    class="text-gray-400 hover:text-gray-600"
                  >
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>

                <!-- Transaction type explanation -->
                <div class="mb-4 p-3 bg-blue-50 border border-blue-200 rounded-md">
                  <div class="text-blue-800 text-sm">
                    <p class="font-medium mb-1">🔍 Test Transaction</p>
                    <p>Send a transaction to verify network functionality. Choose between testing the subnet or the parent L1 network.</p>
                  </div>
                </div>

                <form @submit.prevent="sendTestTransaction" class="space-y-4">
                  <!-- Network Selection -->
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Target Network
                    </label>
                    <select
                      v-model="testTxData.network"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                    >
                      <option value="subnet">Subnet (Test subnet validators and consensus)</option>
                      <option value="l1">Parent L1 (Test parent network connectivity)</option>
                    </select>
                    <p class="text-xs text-gray-500 mt-1" v-if="testTxData.network === 'subnet'">
                      ✅ Tests if subnet validators are online and processing transactions
                    </p>
                    <p class="text-xs text-gray-500 mt-1" v-else>
                      ✅ Tests connectivity to parent network ({{ instance?.config?.parent_endpoint || 'parent chain' }})
                    </p>
                  </div>

                  <!-- Transaction Type -->
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Transaction Type
                    </label>
                    <select
                      v-model="testTxData.type"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                    >
                      <option value="simple">Simple Test Transaction</option>
                      <option value="transfer">FIL Transfer</option>
                      <option value="contract_call">Contract Call</option>
                    </select>
                  </div>

                  <div v-if="testTxData.type === 'transfer' || testTxData.type === 'contract_call'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      From Address
                    </label>
                    <input
                      v-model="testTxData.from"
                      type="text"
                      placeholder="0x..."
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                    />
                  </div>

                  <div v-if="testTxData.type === 'transfer' || testTxData.type === 'contract_call'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      To Address
                    </label>
                    <input
                      v-model="testTxData.to"
                      type="text"
                      placeholder="0x..."
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                    />
                  </div>

                  <div v-if="testTxData.type === 'transfer'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Amount (FIL)
                    </label>
                    <input
                      v-model="testTxData.amount"
                      type="number"
                      step="0.001"
                      min="0"
                      placeholder="0.001"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                    />
                  </div>

                  <div v-if="testTxData.type === 'contract_call'">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Contract Data (Hex)
                    </label>
                    <textarea
                      v-model="testTxData.data"
                      placeholder="0x..."
                      rows="3"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                    ></textarea>
                  </div>

                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Gas Limit
                    </label>
                    <input
                      v-model.number="testTxData.gas_limit"
                      type="number"
                      min="21000"
                      placeholder="21000"
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
                    />
                  </div>

                  <!-- Transaction Result -->
                  <div v-if="testTxResult" class="p-3 rounded-md"
                       :class="testTxResult.includes('successfully') ? 'bg-green-50 border border-green-200' : 'bg-red-50 border border-red-200'">
                    <div class="text-sm"
                         :class="testTxResult.includes('successfully') ? 'text-green-800' : 'text-red-800'">
                      <pre class="whitespace-pre-wrap">{{ testTxResult }}</pre>
                    </div>
                  </div>

                  <div class="flex justify-end space-x-3 pt-4">
                    <button
                      type="button"
                      @click="closeTestTxModal"
                      class="btn-secondary"
                    >
                      Close
                    </button>
                    <button
                      type="submit"
                      :disabled="sendingTestTx"
                      class="btn-primary"
                    >
                      <div v-if="sendingTestTx" class="animate-spin inline-block w-4 h-4 mr-2 border-2 border-current border-t-transparent rounded-full"></div>
                      {{ sendingTestTx ? 'Sending...' : 'Send Test Transaction' }}
                    </button>
                  </div>
                </form>
              </div>
            </div>
          </div>
        </div>

        <!-- Configuration Tab -->
        <div v-if="activeTab === 'configuration'" class="space-y-6">
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Configuration Details</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div v-for="(value, key) in instance?.data?.config" :key="key" class="flex justify-between py-2 border-b border-gray-100">
                <dt class="text-sm font-medium text-gray-500 capitalize">
                  {{ typeof key === 'string' ? key.replace(/([A-Z])/g, ' $1').replace(/^./, (str: string) => str.toUpperCase()) : key }}
                </dt>
                <dd class="text-sm text-gray-900">
                  <span v-if="typeof value === 'boolean'" :class="value ? 'text-green-600' : 'text-red-600'">
                    {{ value ? 'Yes' : 'No' }}
                  </span>
                  <button
                    v-else-if="(typeof key === 'string' && key === 'gateway_addr') || (typeof key === 'string' && key === 'registry_addr')"
                    @click="copyToClipboard(formatAddress(value), key)"
                    class="font-mono hover:bg-gray-100 px-2 py-1 rounded transition-colors cursor-pointer text-left"
                    :title="copyingAddress === key ? 'Copied!' : `Click to copy: ${formatAddress(value)}`"
                  >
                    {{ formatAddressShort(value) }}
                    <svg v-if="copyingAddress === key" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                    </svg>
                  </button>
                  <span v-else-if="typeof value === 'string' && value.startsWith('0x')" class="font-mono">
                    {{ value.slice(0, 8) }}...{{ value.slice(-6) }}
                  </span>
                  <span v-else>{{ value }}</span>
                </dd>
              </div>
            </div>
          </div>
        </div>

        <!-- Contracts Tab -->
        <div v-if="activeTab === 'contracts'" class="space-y-6">
          <!-- Related Contracts Overview -->
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Related Contracts</h3>
            <p class="text-gray-600 mb-6">Smart contracts associated with this subnet and its operations.</p>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <!-- Gateway Contract -->
              <div class="border border-gray-200 rounded-lg p-4">
                <div class="flex items-start justify-between mb-4">
                  <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 bg-primary-100 rounded-lg flex items-center justify-center">
                      <svg class="w-5 h-5 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                      </svg>
                    </div>
                    <div>
                      <h4 class="font-semibold text-gray-900">Gateway Contract</h4>
                      <p class="text-sm text-gray-600">Manages subnet registration and cross-chain messaging</p>
                    </div>
                  </div>
                  <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-primary-100 text-primary-800">
                    Gateway
                  </span>
                </div>

                <div class="space-y-3 mb-4">
                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Address</span>
                    <button
                      @click="copyToClipboard(gatewayAddress, 'gateway')"
                      class="text-sm font-mono text-gray-900 hover:bg-gray-100 px-2 py-1 rounded transition-colors"
                      :title="copyingAddress === 'gateway' ? 'Copied!' : `Click to copy: ${gatewayAddress}`"
                    >
                      {{ gatewayAddressShort }}
                      <svg v-if="copyingAddress === 'gateway'" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                      </svg>
                    </button>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Network</span>
                    <span class="text-sm text-gray-900 font-mono">{{ instance?.data?.parent || instance?.parent }}</span>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Status</span>
                    <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                      Active
                    </span>
                  </div>
                </div>

                <div class="flex space-x-2 pt-3 border-t border-gray-200">
                  <button class="btn-secondary text-xs flex-1">
                    <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    Inspect
                  </button>
                  <RouterLink :to="`/contracts`" class="btn-secondary text-xs">
                    Manage
                  </RouterLink>
                </div>
              </div>

              <!-- Registry Contract -->
              <div class="border border-gray-200 rounded-lg p-4">
                <div class="flex items-start justify-between mb-4">
                  <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                      <svg class="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M9 12h6m-6 4h6M7 20l4-16m6 16l-4-16" />
                      </svg>
                    </div>
                    <div>
                      <h4 class="font-semibold text-gray-900">Registry Contract</h4>
                      <p class="text-sm text-gray-600">Stores subnet metadata and configurations</p>
                    </div>
                  </div>
                  <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                    Registry
                  </span>
                </div>

                <div class="space-y-3 mb-4">
                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Address</span>
                    <button
                      @click="copyToClipboard(instance?.data?.config?.registry_addr ? formatAddress(instance.data.config.registry_addr) : 'N/A', 'registry')"
                      class="text-sm font-mono text-gray-900 hover:bg-gray-100 px-2 py-1 rounded transition-colors"
                      :title="copyingAddress === 'registry' ? 'Copied!' : `Click to copy registry address`"
                    >
                      {{ instance?.data?.config?.registry_addr ? formatAddressShort(instance.data.config.registry_addr) : 'N/A' }}
                      <svg v-if="copyingAddress === 'registry'" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                      </svg>
                    </button>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Network</span>
                    <span class="text-sm text-gray-900 font-mono">{{ instance?.data?.parent || instance?.parent }}</span>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Status</span>
                    <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                      Active
                    </span>
                  </div>
                </div>

                <div class="flex space-x-2 pt-3 border-t border-gray-200">
                  <button class="btn-secondary text-xs flex-1">
                    <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    Inspect
                  </button>
                  <RouterLink :to="`/contracts`" class="btn-secondary text-xs">
                    Manage
                  </RouterLink>
                </div>
              </div>

              <!-- Subnet Actor Contract -->
              <div class="border border-gray-200 rounded-lg p-4">
                <div class="flex items-start justify-between mb-4">
                  <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 bg-green-100 rounded-lg flex items-center justify-center">
                      <svg class="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M19 11H5m14-7H3m14 14H9m6-7l-6 6-4-4" />
                      </svg>
                    </div>
                    <div>
                      <h4 class="font-semibold text-gray-900">Subnet Actor</h4>
                      <p class="text-sm text-gray-600">Core subnet logic and validator management</p>
                    </div>
                  </div>
                  <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                    Subnet
                  </span>
                </div>

                <div class="space-y-3 mb-4">
                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Contract Address</span>
                    <button
                      @click="copyToClipboard(subnetActorAddress, 'subnet-actor')"
                      class="text-sm font-mono text-gray-900 hover:bg-gray-100 px-2 py-1 rounded transition-colors"
                      :title="copyingAddress === 'subnet-actor' ? 'Copied!' : `Click to copy: ${subnetActorAddress}`"
                    >
                      {{ subnetActorAddressShort }}
                      <svg v-if="copyingAddress === 'subnet-actor'" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                      </svg>
                    </button>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Subnet ID</span>
                    <button
                      @click="copyToClipboard(instance?.data?.id || instance?.id || '', 'subnet-id')"
                      class="text-sm font-mono text-gray-900 hover:bg-gray-100 px-2 py-1 rounded transition-colors"
                      :title="copyingAddress === 'subnet-id' ? 'Copied!' : `Click to copy: ${instance?.data?.id || instance?.id}`"
                    >
                      {{ (instance?.data?.id || instance?.id || '').slice(0, 20) }}...
                      <svg v-if="copyingAddress === 'subnet-id'" class="inline-block w-4 h-4 ml-1 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                      </svg>
                    </button>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Network</span>
                    <span class="text-sm text-gray-900 font-mono">{{ instance?.data?.parent || instance?.parent }}</span>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Permission Mode</span>
                    <span class="text-sm text-gray-900 capitalize">{{ instance?.data?.config?.permissionMode || 'N/A' }}</span>
                  </div>

                  <div class="flex justify-between items-center">
                    <span class="text-sm font-medium text-gray-500">Status</span>
                    <span :class="['inline-flex items-center px-2 py-1 rounded-full text-xs font-medium', statusColor]">
                      {{ (instance?.data?.status || 'Unknown').charAt(0).toUpperCase() + (instance?.data?.status || 'unknown').slice(1) }}
                    </span>
                  </div>
                </div>

                <div class="flex space-x-2 pt-3 border-t border-gray-200">
                  <button class="btn-secondary text-xs flex-1">
                    <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    Inspect
                  </button>
                  <button
                    v-if="instance?.data?.status?.toLowerCase() === 'pending approval'"
                    :disabled="approvingSubnet"
                    @click="approveSubnet"
                    class="btn-primary text-xs"
                  >
                    {{ approvingSubnet ? 'Approving...' : 'Approve' }}
                  </button>
                </div>
              </div>

              <!-- Additional IPC Contracts (if any) -->
              <div class="border border-gray-200 rounded-lg p-4">
                <div class="flex items-start justify-between mb-4">
                  <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 bg-purple-100 rounded-lg flex items-center justify-center">
                      <svg class="w-5 h-5 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                      </svg>
                    </div>
                    <div>
                      <h4 class="font-semibold text-gray-900">IPC Contracts</h4>
                      <p class="text-sm text-gray-600">Additional subnet-specific contracts</p>
                    </div>
                  </div>
                  <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                    IPC
                  </span>
                </div>

                <div class="text-center py-6 text-gray-500">
                  <svg class="mx-auto h-8 w-8 text-gray-400 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M9 12h6m-6 4h6M7 20l4-16m6 16l-4-16" />
                  </svg>
                  <p class="text-sm">No additional contracts deployed</p>
                  <button class="text-primary-600 hover:text-primary-700 text-sm font-medium mt-1">
                    Deploy IPC Contract
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Contract Configuration -->
          <div class="card">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">Contract Configuration</h3>
            <div class="bg-gray-50 rounded-lg p-4">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <h4 class="font-medium text-gray-900 mb-3">Gateway Settings</h4>
                  <div class="space-y-2 text-sm">
                                          <div class="flex justify-between">
                        <span class="text-gray-600">Min Validator Stake</span>
                        <span class="font-mono">{{ instance.config?.minValidatorStake || 'N/A' }} FIL</span>
                      </div>
                      <div class="flex justify-between">
                        <span class="text-gray-600">Min Validators</span>
                        <span class="font-mono">{{ instance.config?.minValidators || 'N/A' }}</span>
                      </div>
                      <div class="flex justify-between">
                        <span class="text-gray-600">Bottom-up Period</span>
                        <span class="font-mono">{{ instance.config?.bottomupCheckPeriod || 'N/A' }} blocks</span>
                      </div>
                  </div>
                </div>

                <div>
                  <h4 class="font-medium text-gray-900 mb-3">Subnet Settings</h4>
                  <div class="space-y-2 text-sm">
                    <div class="flex justify-between">
                      <span class="text-gray-600">Supply Source</span>
                      <span class="capitalize">{{ instance?.data?.config?.supplySourceKind || 'N/A' }}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-gray-600">Collateral Source</span>
                      <span class="capitalize">{{ instance?.data?.config?.collateralSourceKind || 'N/A' }}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-gray-600">Cross-msg Fee</span>
                      <span class="font-mono">{{ instance?.data?.config?.minCrossMsgFee || 'N/A' }} FIL</span>
                    </div>
                  </div>
                </div>
              </div>

              <div class="mt-6 pt-4 border-t border-gray-200">
                <div class="flex justify-between items-center">
                  <div>
                    <h4 class="font-medium text-gray-900">Contract Upgrades</h4>
                    <p class="text-sm text-gray-600">Manage contract versions and upgrades</p>
                  </div>
                  <button class="btn-secondary text-sm">
                    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                    </svg>
                    Check for Updates
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Metrics Tab -->
        <div v-if="activeTab === 'metrics'" class="space-y-6">
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <!-- Performance Metrics -->
            <div class="card">
              <h4 class="text-md font-semibold text-gray-900 mb-3">Performance</h4>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Block Height</span>
                  <span class="text-sm font-medium text-gray-900">
                    {{ chainStats?.block_height || subnetStatus?.block_height || 'N/A' }}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Avg Block Time</span>
                  <span class="text-sm font-medium text-gray-900">
                    {{ chainStats?.avg_block_time ? `${chainStats.avg_block_time.toFixed(1)}s` : 'N/A' }}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">TPS</span>
                  <span class="text-sm font-medium text-gray-900">
                    {{ chainStats?.tps ? chainStats.tps.toFixed(1) : 'N/A' }}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Pending Transactions</span>
                  <span class="text-sm font-medium text-gray-900">
                    {{ chainStats?.pending_transactions || 'N/A' }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Economic Metrics -->
            <div class="card">
              <h4 class="text-md font-semibold text-gray-900 mb-3">Economic</h4>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Total Supply</span>
                  <span class="text-sm font-medium text-gray-900">{{ chainStats?.total_supply || 'N/A' }} FIL</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Circulating</span>
                  <span class="text-sm font-medium text-gray-900">{{ chainStats?.circulating_supply || 'N/A' }} FIL</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Fees Collected</span>
                  <span class="text-sm font-medium text-gray-900">{{ chainStats?.fees_collected || 'N/A' }} FIL</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Total Stake</span>
                  <span class="text-sm font-medium text-gray-900">{{ totalStake }} FIL</span>
                </div>
              </div>
            </div>

            <!-- Network Metrics -->
            <div class="card">
              <h4 class="text-md font-semibold text-gray-900 mb-3">Network</h4>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Active Validators</span>
                  <span class="text-sm font-medium text-gray-900">{{ instance.data?.validator_count || instance.validators?.length || 0 }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Validators Online</span>
                  <span class="text-sm font-medium text-gray-900">
                    {{ subnetStatus?.validators_online !== undefined ? subnetStatus.validators_online : 'N/A' }}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Last Checkpoint</span>
                  <span class="text-sm font-medium text-gray-900">
                    {{ chainStats?.last_checkpoint || 'N/A' }}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500">Consensus Status</span>
                  <span class="text-sm font-medium"
                        :class="{
                          'text-green-600': subnetStatus?.consensus_status === 'healthy',
                          'text-yellow-600': subnetStatus?.consensus_status === 'degraded',
                          'text-red-600': subnetStatus?.consensus_status === 'offline',
                          'text-gray-900': !subnetStatus?.consensus_status
                        }">
                    {{ subnetStatus?.consensus_status || 'Unknown' }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- Real-time Activity Chart Placeholder -->
          <div class="card">
            <div class="flex items-center justify-between mb-4">
              <h4 class="text-md font-semibold text-gray-900">Real-time Activity</h4>
              <div class="flex items-center space-x-2">
                <div v-if="loadingStats" class="animate-spin w-4 h-4 border-2 border-primary-600 border-t-transparent rounded-full"></div>
                <span class="text-xs text-gray-500">
                  Last updated: {{ chainStats?.latest_block_time ? new Date(chainStats.latest_block_time).toLocaleTimeString() : 'Never' }}
                </span>
              </div>
            </div>

            <!-- Chain Health Indicators -->
            <div v-if="chainStats || subnetStatus" class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
              <div class="text-center p-4 border rounded-lg">
                <div class="text-lg font-semibold mb-1"
                     :class="{
                       'text-green-600': subnetStatus?.is_active,
                       'text-red-600': subnetStatus?.is_active === false
                     }">
                  {{ subnetStatus?.is_active ? 'ACTIVE' : 'INACTIVE' }}
                </div>
                <div class="text-sm text-gray-500">Chain Status</div>
              </div>

              <div class="text-center p-4 border rounded-lg">
                <div class="text-lg font-semibold mb-1">
                  {{ chainStats?.transaction_count || 'N/A' }}
                </div>
                <div class="text-sm text-gray-500">Total Transactions</div>
              </div>

              <div class="text-center p-4 border rounded-lg">
                <div class="text-lg font-semibold mb-1"
                     :class="{
                       'text-green-600': subnetStatus?.sync_status === 'synced',
                       'text-yellow-600': subnetStatus?.sync_status === 'syncing',
                       'text-red-600': subnetStatus?.sync_status === 'behind'
                     }">
                  {{ subnetStatus?.sync_status?.toUpperCase() || 'UNKNOWN' }}
                </div>
                <div class="text-sm text-gray-500">Sync Status</div>
              </div>
            </div>

            <div v-else class="bg-gray-50 rounded-lg p-8 text-center">
              <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
              </svg>
              <p class="text-gray-600">Loading real-time metrics...</p>
              <p class="text-sm text-gray-500 mt-1">Chain statistics will appear here once available</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card {
  @apply bg-white rounded-lg shadow-sm border border-gray-200 p-6;
}

.btn-primary {
  @apply inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}

.btn-secondary {
  @apply inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md shadow-sm text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}

.address-button {
  @apply transition-all duration-200;
}

.address-button:hover {
  @apply bg-gray-100 shadow-sm;
}
</style>