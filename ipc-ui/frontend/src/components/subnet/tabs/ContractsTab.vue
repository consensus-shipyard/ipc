<script setup lang="ts">
import type { SubnetInstance } from '@/types/subnet'
import { formatAddress, formatAddressShort } from '@/utils/address'
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

interface Props {
  instance: SubnetInstance | null
  gatewayAddress: string
  gatewayAddressShort: string
  subnetActorAddress: string
  subnetActorAddressShort: string
  statusColor: string
  copyingAddress: string | null
  approvingSubnet: boolean
}

interface Emits {
  (e: 'copyToClipboard', text: string, type: string): void
  (e: 'approveSubnet'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Helper to get registry address
const registryAddress = computed(() => {
  const addr = props.instance?.data?.config?.registry_addr || props.instance?.config?.registry_addr
  return addr ? formatAddress(addr) : 'N/A'
})

const registryAddressShort = computed(() => {
  const addr = props.instance?.data?.config?.registry_addr || props.instance?.config?.registry_addr
  return addr ? formatAddressShort(addr) : 'N/A'
})
</script>

<template>
  <div class="space-y-6">
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
                @click="emit('copyToClipboard', gatewayAddress, 'gateway')"
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
            <RouterLink to="/contracts" class="btn-secondary text-xs">
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
                @click="emit('copyToClipboard', registryAddress, 'registry')"
                class="text-sm font-mono text-gray-900 hover:bg-gray-100 px-2 py-1 rounded transition-colors"
                :title="copyingAddress === 'registry' ? 'Copied!' : `Click to copy registry address`"
              >
                {{ registryAddressShort }}
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
            <RouterLink to="/contracts" class="btn-secondary text-xs">
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
                @click="emit('copyToClipboard', subnetActorAddress, 'subnet-actor')"
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
                @click="emit('copyToClipboard', instance?.data?.id || instance?.id || '', 'subnet-id')"
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
                {{ (instance?.data?.status || instance?.status || 'Unknown').charAt(0).toUpperCase() + (instance?.data?.status || instance?.status || 'unknown').slice(1) }}
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
              @click="emit('approveSubnet')"
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
                <span class="font-mono">{{ instance?.config?.minValidatorStake || 'N/A' }} FIL</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Min Validators</span>
                <span class="font-mono">{{ instance?.config?.minValidators || 'N/A' }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Bottom-up Period</span>
                <span class="font-mono">{{ instance?.config?.bottomupCheckPeriod || 'N/A' }} blocks</span>
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
</style>
