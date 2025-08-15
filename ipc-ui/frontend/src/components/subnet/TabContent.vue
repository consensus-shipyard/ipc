<script setup lang="ts">
import type { BulkValidator, ChainStats, SubnetInstance, SubnetStatus } from '@/types/subnet'
import { computed } from 'vue'
import ConfigurationTab from './tabs/ConfigurationTab.vue'
import ContractsTab from './tabs/ContractsTab.vue'
import MetricsTab from './tabs/MetricsTab.vue'
import OverviewTab from './tabs/OverviewTab.vue'
import ValidatorsTab from './tabs/ValidatorsTab.vue'

interface Props {
  activeTab: string
  instance: SubnetInstance | null
  loading: boolean
  error: string | null
  createdDate: string
  totalStake: string
  gatewayAddress: string
  gatewayAddressShort: string
  subnetActorAddress: string
  subnetActorAddressShort: string
  statusColor: string
  copyingAddress: string | null
  chainStats: ChainStats | null
  subnetStatus: SubnetStatus | null
  loadingStats: boolean
  statsError: string | null
  // For modals
  showAddValidatorModal: boolean
  showBulkManagement: boolean
  // For validators tab
  removingValidator: Record<string, boolean>
  updatingStake: Record<string, boolean>
  stakeAmounts: Record<string, number>
  bulkValidators: BulkValidator[]
  settingFederatedPower: boolean
  // Node config
  loadingNodeConfig: boolean
  approvingSubnet: boolean
}

interface Emits {
  (e: 'copyToClipboard', text: string, type: string): void
  (e: 'fetchInstance'): void
  (e: 'fetchChainStats'): void
  (e: 'update:showAddValidatorModal', value: boolean): void
  (e: 'update:showBulkManagement', value: boolean): void
  (e: 'removeValidator', address: string): void
  (e: 'updateStake', address: string, action: 'stake' | 'unstake'): void
  (e: 'showNodeConfig', address: string): void
  (e: 'initializeBulkManagement'): void
  (e: 'addBulkValidator'): void
  (e: 'removeBulkValidator', index: number): void
  (e: 'setBulkFederatedPower'): void
  (e: 'update:bulkValidators', validators: BulkValidator[]): void
  (e: 'update:stakeAmounts', amounts: Record<string, number>): void
  (e: 'approveSubnet'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Pass through computed properties for child components
const permissionMode = computed(() => {
  return props.instance?.data?.config?.permissionMode ||
         props.instance?.config?.permissionMode ||
         'unknown'
})

const validators = computed(() => {
  return props.instance?.data?.validators || props.instance?.validators || []
})

const validatorCount = computed(() => {
  return props.instance?.data?.validator_count ||
         props.instance?.data?.validators?.length ||
         props.instance?.validators?.length ||
         0
})
</script>

<template>
  <div class="space-y-6">
    <!-- Overview Tab -->
    <OverviewTab
      v-if="activeTab === 'overview'"
      :instance="instance"
      :loading="loading"
      :error="error"
      :created-date="createdDate"
      :total-stake="totalStake"
      :gateway-address="gatewayAddress"
      :gateway-address-short="gatewayAddressShort"
      :status-color="statusColor"
      :copying-address="copyingAddress"
      :chain-stats="chainStats"
      :subnet-status="subnetStatus"
      :loading-stats="loadingStats"
      :stats-error="statsError"
      @copy-to-clipboard="(text, type) => emit('copyToClipboard', text, type)"
      @fetch-instance="emit('fetchInstance')"
      @fetch-chain-stats="emit('fetchChainStats')"
    />

    <!-- Validators Tab -->
    <ValidatorsTab
      v-else-if="activeTab === 'validators'"
      :instance="instance"
      :show-add-validator-modal="showAddValidatorModal"
      :show-bulk-management="showBulkManagement"
      :removing-validator="removingValidator"
      :updating-stake="updatingStake"
      :stake-amounts="stakeAmounts"
      :bulk-validators="bulkValidators"
      :setting-federated-power="settingFederatedPower"
      :copying-address="copyingAddress"
      :approving-subnet="approvingSubnet"
      @update:show-add-validator-modal="emit('update:showAddValidatorModal', $event)"
      @update:show-bulk-management="emit('update:showBulkManagement', $event)"
      @remove-validator="emit('removeValidator', $event)"
      @update-stake="(address, action) => emit('updateStake', address, action)"
      @show-node-config="emit('showNodeConfig', $event)"
      @initialize-bulk-management="emit('initializeBulkManagement')"
      @add-bulk-validator="emit('addBulkValidator')"
      @remove-bulk-validator="emit('removeBulkValidator', $event)"
      @set-bulk-federated-power="emit('setBulkFederatedPower')"
      @update:bulk-validators="emit('update:bulkValidators', $event)"
      @update:stake-amounts="emit('update:stakeAmounts', $event)"
      @copy-to-clipboard="(text, type) => emit('copyToClipboard', text, type)"
      @approve-subnet="emit('approveSubnet')"
    />

    <!-- Configuration Tab -->
    <ConfigurationTab
      v-else-if="activeTab === 'configuration'"
      :instance="instance"
      :copying-address="copyingAddress"
      @copy-to-clipboard="(text, type) => emit('copyToClipboard', text, type)"
    />

    <!-- Contracts Tab -->
    <ContractsTab
      v-else-if="activeTab === 'contracts'"
      :instance="instance"
      :gateway-address="gatewayAddress"
      :gateway-address-short="gatewayAddressShort"
      :subnet-actor-address="subnetActorAddress"
      :subnet-actor-address-short="subnetActorAddressShort"
      :status-color="statusColor"
      :copying-address="copyingAddress"
      :approving-subnet="approvingSubnet"
      @copy-to-clipboard="(text, type) => emit('copyToClipboard', text, type)"
      @approve-subnet="emit('approveSubnet')"
    />

    <!-- Metrics Tab -->
    <MetricsTab
      v-else-if="activeTab === 'metrics'"
      :instance="instance"
      :chain-stats="chainStats"
      :subnet-status="subnetStatus"
      :loading-stats="loadingStats"
      :total-stake="totalStake"
    />
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
