import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface WalletAddress {
  address: string
  wallet_type: 'evm' | 'fvm'
  pubkey?: string
  balance?: string
  custom_label?: string
  is_compatible?: boolean // Set by network compatibility check
}

export const useWalletStore = defineStore('wallet', () => {
  // State
  const addresses = ref<WalletAddress[]>([])
  const defaultAddress = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const lastFetchTime = ref<number>(0)

  // Cache duration: 30 seconds
  const CACHE_DURATION = 30 * 1000

  // Computed
  const evmAddresses = computed(() => {
    if (!Array.isArray(addresses.value)) return []
    return addresses.value.filter(addr => addr.wallet_type === 'evm')
  })

  const fvmAddresses = computed(() => {
    if (!Array.isArray(addresses.value)) return []
    return addresses.value.filter(addr => addr.wallet_type === 'fvm')
  })

  const compatibleAddresses = computed(() => {
    if (!Array.isArray(addresses.value)) return []
    return addresses.value.filter(addr => addr.is_compatible !== false)
  })

  const defaultAddressDetails = computed(() => {
    if (!defaultAddress.value || !Array.isArray(addresses.value)) return null
    return addresses.value.find(addr => addr.address === defaultAddress.value) || null
  })

  // Actions
  const loadDefaultAddress = () => {
    const stored = localStorage.getItem('ipc-wallet-default-address')
    if (stored) {
      defaultAddress.value = stored
    }
  }

  const saveDefaultAddress = (address: string | null) => {
    defaultAddress.value = address
    if (address) {
      localStorage.setItem('ipc-wallet-default-address', address)
    } else {
      localStorage.removeItem('ipc-wallet-default-address')
    }
  }

  const loadPerFieldDefaults = (): Record<string, string> => {
    const stored = localStorage.getItem('ipc-wallet-field-defaults')
    return stored ? JSON.parse(stored) : {}
  }

  const saveFieldDefault = (fieldName: string, address: string) => {
    const fieldDefaults = loadPerFieldDefaults()
    fieldDefaults[fieldName] = address
    localStorage.setItem('ipc-wallet-field-defaults', JSON.stringify(fieldDefaults))
  }

  const getFieldDefault = (fieldName: string): string | null => {
    const fieldDefaults = loadPerFieldDefaults()
    return fieldDefaults[fieldName] || defaultAddress.value
  }

  const fetchAddresses = async (subnet?: string, forceRefresh = false) => {
    // Check cache first
    const now = Date.now()
    if (!forceRefresh && (now - lastFetchTime.value) < CACHE_DURATION && addresses.value.length > 0) {
      return
    }

    isLoading.value = true
    error.value = null

    try {
      const params = new URLSearchParams()
      if (subnet) {
        params.append('subnet', subnet)
      }

      const response = await fetch(`/api/wallets?${params.toString()}`)
      if (!response.ok) {
        throw new Error(`Failed to fetch wallet addresses: ${response.statusText}`)
      }

      const walletAddresses: WalletAddress[] = await response.json()
      addresses.value = walletAddresses
      lastFetchTime.value = now

      // Auto-select first address as default if none set
      if (!defaultAddress.value && walletAddresses.length > 0) {
        saveDefaultAddress(walletAddresses[0].address)
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch wallet addresses'
      console.error('Error fetching wallet addresses:', err)
    } finally {
      isLoading.value = false
    }
  }

  const updateNetworkCompatibility = (networkType: 'evm' | 'fvm' | 'both') => {
    if (!Array.isArray(addresses.value)) return
    addresses.value = addresses.value.map(addr => ({
      ...addr,
      is_compatible: networkType === 'both' || addr.wallet_type === networkType
    }))
  }

  const getAddressByAddress = (address: string): WalletAddress | undefined => {
    if (!Array.isArray(addresses.value)) return undefined
    return addresses.value.find(addr => addr.address === address)
  }

  const getAddressWithPubkey = (address: string): WalletAddress | undefined => {
    if (!Array.isArray(addresses.value)) return undefined
    return addresses.value.find(addr => addr.address === address && addr.pubkey)
  }

  // Initialize on store creation
  loadDefaultAddress()

  return {
    // State
    addresses: computed(() => addresses.value),
    defaultAddress: computed(() => defaultAddress.value),
    isLoading: computed(() => isLoading.value),
    error: computed(() => error.value),

    // Computed
    evmAddresses,
    fvmAddresses,
    compatibleAddresses,
    defaultAddressDetails,

    // Actions
    fetchAddresses,
    saveDefaultAddress,
    saveFieldDefault,
    getFieldDefault,
    updateNetworkCompatibility,
    getAddressByAddress,
    getAddressWithPubkey,
    loadPerFieldDefaults
  }
})