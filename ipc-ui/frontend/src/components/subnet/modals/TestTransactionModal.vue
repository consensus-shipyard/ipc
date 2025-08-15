<script setup lang="ts">
import { ConfigService } from '@/services/subnet/config.service'
import type { SubnetInstance, TestTransactionData } from '@/types/subnet'
import { computed, ref, watch } from 'vue'

interface Props {
  modelValue: boolean
  instance: SubnetInstance | null
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void
  (e: 'transactionSent'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const show = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const sending = ref(false)
const result = ref<string | null>(null)

const testTxData = ref<TestTransactionData>({
  type: 'simple',
  network: 'subnet',
  from: '',
  to: '',
  amount: '0.001',
  data: '',
  gas_limit: 21000
})

// Set default addresses when modal opens
watch(show, (newValue) => {
  if (newValue && props.instance) {
    // Set default from address if available
    if (props.instance.data?.validators && props.instance.data.validators.length > 0) {
      testTxData.value.from = props.instance.data.validators[0].address
    } else if (props.instance.validators && props.instance.validators.length > 0) {
      testTxData.value.from = props.instance.validators[0].address
    }

    // Set default to address as a different validator or gateway
    if (props.instance.data?.validators && props.instance.data.validators.length > 1) {
      testTxData.value.to = props.instance.data.validators[1].address
    } else if (props.instance.validators && props.instance.validators.length > 1) {
      testTxData.value.to = props.instance.validators[1].address
    } else if (props.instance.config?.gateway_addr) {
      testTxData.value.to = String(props.instance.config.gateway_addr)
    }
  }
})

const close = () => {
  show.value = false
  result.value = null
  sending.value = false
}

const sendTransaction = async () => {
  if (!props.instance) return

  sending.value = true
  result.value = null

  const networkName = testTxData.value.network === 'subnet' ? 'Subnet' : 'Parent L1'

  try {
    const subnetId = props.instance.data?.id || props.instance.id
    const response = await ConfigService.sendTestTransaction(subnetId, testTxData.value)

    if (response.data.success) {
      result.value = `✅ Real transaction sent successfully!
        Network: ${networkName}
        Transaction Hash: ${response.data.txHash || 'N/A'}
        Block: ${response.data.blockNumber || 'Pending'}
        Gas Used: ${response.data.gasUsed || 'N/A'}

        ✅ Transaction successfully executed on the blockchain!`

      emit('transactionSent')
    } else {
      result.value = `❌ Transaction failed on ${networkName}: ${response.data.error || 'Unknown error'}`
    }
  } catch (err) {
    console.error('Error sending test transaction:', err)
    result.value = `❌ Transaction failed on ${networkName}: ${err instanceof Error ? err.message : 'Network error'}`
  } finally {
    sending.value = false
  }
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
    <div class="relative top-20 mx-auto p-5 border w-11/12 max-w-lg shadow-lg rounded-md bg-white">
      <div class="mt-3">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-medium text-gray-900">Send Test Transaction</h3>
          <button
            @click="close"
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

        <form @submit.prevent="sendTransaction" class="space-y-4">
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
          <div v-if="result" class="p-3 rounded-md"
               :class="result.includes('successfully') ? 'bg-green-50 border border-green-200' : 'bg-red-50 border border-red-200'">
            <div class="text-sm"
                 :class="result.includes('successfully') ? 'text-green-800' : 'text-red-800'">
              <pre class="whitespace-pre-wrap">{{ result }}</pre>
            </div>
          </div>

          <div class="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              @click="close"
              class="btn-secondary"
            >
              Close
            </button>
            <button
              type="submit"
              :disabled="sending"
              class="btn-primary"
            >
              <div v-if="sending" class="animate-spin inline-block w-4 h-4 mr-2 border-2 border-current border-t-transparent rounded-full"></div>
              {{ sending ? 'Sending...' : 'Send Test Transaction' }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<style scoped>
.btn-primary {
  @apply inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}

.btn-secondary {
  @apply inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md shadow-sm text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500;
}
</style>
