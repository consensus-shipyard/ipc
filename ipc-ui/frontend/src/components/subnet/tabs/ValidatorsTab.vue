<script setup lang="ts">
import type { SubnetInstance, Validator, BulkValidator } from '@/types/subnet'
import { computed } from 'vue'

interface Props {
  instance: SubnetInstance | null
  showAddValidatorModal: boolean
  showBulkManagement: boolean
  removingValidator: Record<string, boolean>
  updatingStake: Record<string, boolean>
  stakeAmounts: Record<string, number>
  bulkValidators: BulkValidator[]
  settingFederatedPower: boolean
  copyingAddress: string | null
  approvingSubnet: boolean
}

interface Emits {
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
  (e: 'copyToClipboard', text: string, type: string): void
  (e: 'approveSubnet'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Computed properties
const permissionMode = computed(() => {
  return props.instance?.data?.config?.permissionMode || 
         props.instance?.config?.permissionMode || 
         'unknown'
})

const validators = computed(() => {
  return props.instance?.data?.validators || props.instance?.validators || []
})

const isFederatedMode = computed(() => permissionMode.value === 'federated')

// Helper methods
const updateStakeAmount = (address: string, value: number) => {
  emit('update:stakeAmounts', { ...props.stakeAmounts, [address]: value })
}

const updateBulkValidator = (index: number, field: keyof BulkValidator, value: any) => {
  const updated = [...props.bulkValidators]
  updated[index] = { ...updated[index], [field]: value }
  emit('update:bulkValidators', updated)
}
</script>

<template>
  <div class="space-y-6">
    <!-- Permission Mode Explanation -->
    <div class="p-4 bg-blue-50 border border-blue-200 rounded-lg">
      <h4 class="text-md font-semibold text-blue-800 mb-2 flex items-center">
        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        {{
          permissionMode === 'federated' ? 'Federated Mode' :
          permissionMode === 'collateral' ? 'Collateral Mode' :
          permissionMode === 'static' ? 'Static Mode' :
          permissionMode === 'root' ? 'Root Network' :
          permissionMode === 'unknown' ? 'Unknown Mode (not set)' :
          `Unknown Mode (${permissionMode || 'not set'})`
        }}
      </h4>

      <div v-if="permissionMode === 'federated'" class="text-blue-700 text-sm">
        <p class="mb-2"><strong>Federated subnets</strong> use centralized validator management:</p>
        <ul class="list-disc list-inside space-y-1 ml-4">
          <li>Validators are added by setting their power directly</li>
          <li>No collateral staking required</li>
          <li>Network owner controls validator set</li>
          <li>Changes are applied to all validators simultaneously</li>
        </ul>
      </div>

      <div v-else-if="permissionMode === 'collateral'" class="text-blue-700 text-sm">
        <p class="mb-2"><strong>Collateral subnets</strong> use stake-based validator management:</p>
        <ul class="list-disc list-inside space-y-1 ml-4">
          <li>Validators join by staking FIL collateral</li>
          <li>Minimum stake requirement: {{ instance?.config?.minValidatorStake || 'Not set' }} FIL</li>
          <li>Validators can increase/decrease their stake</li>
          <li>Higher stake generally means higher voting power</li>
        </ul>
      </div>

      <div v-else-if="permissionMode === 'static'" class="text-blue-700 text-sm">
        <p class="mb-2"><strong>Static subnets</strong> use predefined validator sets:</p>
        <ul class="list-disc list-inside space-y-1 ml-4">
          <li>Validators are defined at subnet creation time</li>
          <li>No dynamic joining or leaving of validators</li>
          <li>Fixed validator set for the subnet's lifetime</li>
          <li>No staking or power changes after deployment</li>
        </ul>
      </div>

      <div v-else-if="permissionMode === 'root'" class="text-blue-700 text-sm">
        <p class="mb-2"><strong>Root networks</strong> are the base layer networks:</p>
        <ul class="list-disc list-inside space-y-1 ml-4">
          <li>This is a root network, not a subnet</li>
          <li>Root networks don't have permission modes</li>
          <li>They serve as parent networks for subnets</li>
          <li>Validator management depends on the underlying consensus mechanism</li>
        </ul>
      </div>

      <div v-else class="text-yellow-700 text-sm">
        <p class="mb-2"><strong>{{ permissionMode === 'unknown' ? 'Permission mode could not be determined' : 'Unrecognized permission mode' }}</strong>:</p>
        <ul class="list-disc list-inside space-y-1 ml-4">
          <li v-if="permissionMode === 'unknown'">Unable to retrieve permission mode from the blockchain</li>
          <li v-else>Unrecognized permission mode: "{{ permissionMode }}"</li>
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
    <div v-if="isFederatedMode" class="p-6 bg-blue-50 rounded-lg">
      <div class="flex items-center justify-between mb-4">
        <h4 class="text-md font-semibold text-gray-800 flex items-center">
          <svg class="w-5 h-5 mr-2 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          Advanced Federated Management
        </h4>
        <button
          v-if="!showBulkManagement"
          @click="emit('initializeBulkManagement')"
          class="btn-secondary text-sm"
        >
          Manage All Validators
        </button>
        <button
          v-else
          @click="emit('update:showBulkManagement', false)"
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
                :value="validator.address"
                @input="updateBulkValidator(index, 'address', ($event.target as HTMLInputElement).value)"
                type="text"
                placeholder="Validator Address (0x...)"
                class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
              />
            </div>
            <div class="col-span-4">
              <input
                :value="validator.pubkey"
                @input="updateBulkValidator(index, 'pubkey', ($event.target as HTMLInputElement).value)"
                type="text"
                placeholder="Public Key (0x04...)"
                class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
              />
            </div>
            <div class="col-span-2">
              <input
                :value="validator.power"
                @input="updateBulkValidator(index, 'power', Number(($event.target as HTMLInputElement).value))"
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
                @click="emit('removeBulkValidator', index)"
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

        <div class="flex justify-between items-center mt-4">
          <button
            @click="emit('addBulkValidator')"
            type="button"
            class="btn-secondary text-sm"
          >
            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Validator
          </button>

          <button
            @click="emit('setBulkFederatedPower')"
            :disabled="settingFederatedPower || bulkValidators.length === 0"
            class="btn-primary text-sm"
          >
            <svg v-if="!settingFederatedPower" class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
            <svg v-else class="animate-spin w-4 h-4 mr-2" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {{ settingFederatedPower ? 'Setting Power...' : 'Set Federated Power' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Validators List -->
    <div class="card">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900">
          Validators ({{ validators.length }})
        </h3>
        <div class="space-x-2">
          <button
            v-if="instance?.data?.status?.toLowerCase() === 'pending approval'"
            :disabled="approvingSubnet"
            @click="emit('approveSubnet')"
            class="btn-primary"
          >
            {{ approvingSubnet ? 'Approving...' : 'Approve Subnet' }}
          </button>
          <button
            v-if="permissionMode !== 'static'"
            @click="emit('update:showAddValidatorModal', true)"
            class="btn-primary"
          >
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Validator
          </button>
        </div>
      </div>

      <div v-if="validators.length === 0" class="text-center py-8 text-gray-500">
        <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
        </svg>
        <p class="mb-2">No validators yet</p>
        <p class="text-sm">Add validators to secure and operate your subnet</p>
      </div>

      <div v-else class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Address
              </th>
              <th v-if="permissionMode === 'collateral'" scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Stake
              </th>
              <th v-if="permissionMode !== 'collateral'" scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Power
              </th>
              <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Status
              </th>
              <th scope="col" class="relative px-4 py-3">
                <span class="sr-only">Actions</span>
              </th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            <tr v-for="validator in validators" :key="validator.address">
              <td class="px-4 py-4 whitespace-nowrap">
                <button
                  @click="emit('copyToClipboard', validator.address, 'validator-' + validator.address)"
                  class="font-mono text-sm text-gray-900 hover:bg-gray-100 px-2 py-1 rounded transition-colors"
                  :title="copyingAddress === 'validator-' + validator.address ? 'Copied!' : `Click to copy: ${validator.address}`"
                >
                  {{ validator.address.slice(0, 6) }}...{{ validator.address.slice(-4) }}
                  <svg v-if="copyingAddress === 'validator-' + validator.address" 
                       class="inline-block w-4 h-4 ml-1 text-green-600" 
                       fill="currentColor" 
                       viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                  </svg>
                </button>
              </td>
              <td v-if="permissionMode === 'collateral'" class="px-4 py-4 whitespace-nowrap">
                <div class="text-sm text-gray-900">{{ validator.stake }} FIL</div>
                <div v-if="validator.initial_balance" class="text-xs text-gray-500">
                  Initial: {{ validator.initial_balance }} FIL
                </div>
              </td>
              <td v-if="permissionMode !== 'collateral'" class="px-4 py-4 whitespace-nowrap">
                <div class="text-sm text-gray-900">
                  {{ validator.power || validator.current_power || 'N/A' }}
                </div>
                <div v-if="validator.next_power !== undefined && validator.next_power !== validator.current_power" 
                     class="text-xs text-gray-500">
                  Next: {{ validator.next_power }}
                </div>
              </td>
              <td class="px-4 py-4 whitespace-nowrap">
                <span v-if="validator.status === 'active'" 
                      class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                  Active
                </span>
                <span v-else-if="validator.status === 'pending'" 
                      class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                  Pending
                </span>
                <span v-else-if="validator.waiting" 
                      class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                  Waiting
                </span>
                <span v-else 
                      class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                  {{ validator.status || 'Unknown' }}
                </span>
              </td>
              <td class="px-4 py-4 whitespace-nowrap text-right text-sm font-medium">
                <div class="flex items-center justify-end space-x-2">
                  <!-- Collateral Mode Actions -->
                  <div v-if="permissionMode === 'collateral'" class="flex items-center space-x-2">
                    <input
                      :value="stakeAmounts[validator.address] || ''"
                      @input="updateStakeAmount(validator.address, Number(($event.target as HTMLInputElement).value))"
                      type="number"
                      min="0"
                      step="0.1"
                      placeholder="Amount"
                      class="w-20 px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
                    />
                    <button
                      @click="emit('updateStake', validator.address, 'stake')"
                      :disabled="updatingStake[validator.address] || !stakeAmounts[validator.address]"
                      class="text-primary-600 hover:text-primary-900 text-sm"
                    >
                      {{ updatingStake[validator.address] ? 'Staking...' : 'Stake' }}
                    </button>
                    <button
                      @click="emit('updateStake', validator.address, 'unstake')"
                      :disabled="updatingStake[validator.address] || !stakeAmounts[validator.address]"
                      class="text-yellow-600 hover:text-yellow-900 text-sm"
                    >
                      {{ updatingStake[validator.address] ? 'Unstaking...' : 'Unstake' }}
                    </button>
                  </div>

                  <!-- Common Actions -->
                  <button
                    @click="emit('showNodeConfig', validator.address)"
                    class="text-blue-600 hover:text-blue-900"
                    title="View node configuration"
                  >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                  </button>

                  <button
                    v-if="permissionMode !== 'static'"
                    @click="emit('removeValidator', validator.address)"
                    :disabled="removingValidator[validator.address]"
                    class="text-red-600 hover:text-red-900"
                    title="Remove validator"
                  >
                    <svg v-if="!removingValidator[validator.address]" 
                         class="w-5 h-5" 
                         fill="none" 
                         stroke="currentColor" 
                         viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                    <svg v-else 
                         class="animate-spin w-5 h-5" 
                         fill="none" 
                         viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" 
                            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
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
