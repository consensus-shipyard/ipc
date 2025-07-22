<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink } from 'vue-router'

// Mock data - will be replaced with real data from stores
const subnets = ref([
  {
    id: 'subnet-1',
    name: 'Production Subnet A',
    status: 'active',
    parentNetwork: 'Ethereum Mainnet',
    subnetId: '/r31337/subnet-1',
    validators: 5,
    totalStake: '150.5',
    blockHeight: 12543,
    lastCheckpoint: '2 hours ago',
    children: [
      { id: 'child-1', name: 'Child Subnet A1', status: 'active' },
      { id: 'child-2', name: 'Child Subnet A2', status: 'deploying' }
    ]
  },
  {
    id: 'subnet-2',
    name: 'Test Environment',
    status: 'paused',
    parentNetwork: 'Sepolia Testnet',
    subnetId: '/r31337/subnet-2',
    validators: 3,
    totalStake: '45.0',
    blockHeight: 8721,
    lastCheckpoint: '5 hours ago',
    children: [
      { id: 'child-3', name: 'Child Subnet B1', status: 'active' }
    ]
  },
  {
    id: 'subnet-3',
    name: 'Dev Subnet',
    status: 'active',
    parentNetwork: 'Local Network',
    subnetId: '/r31337/subnet-3',
    validators: 1,
    totalStake: '10.0',
    blockHeight: 1234,
    lastCheckpoint: '30 minutes ago',
    children: []
  }
])

const getStatusColor = (status: string) => {
  switch (status) {
    case 'active': return 'text-green-600 bg-green-50'
    case 'paused': return 'text-yellow-600 bg-yellow-50'
    case 'deploying': return 'text-blue-600 bg-blue-50'
    case 'failed': return 'text-red-600 bg-red-50'
    default: return 'text-gray-600 bg-gray-50'
  }
}

const getStatusIcon = (status: string) => {
  switch (status) {
    case 'active': return 'M5 13l4 4L19 7'
    case 'paused': return 'M6 4h4v16H6V4zM14 4h4v16h-4V4z'
    case 'deploying': return 'M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15'
    case 'failed': return 'M6 18L18 6M6 6l12 12'
    default: return 'M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z'
  }
}
</script>

<template>
  <div class="p-6">
    <!-- Dashboard Header -->
    <div class="mb-8">
      <h1 class="text-3xl font-bold text-gray-900 mb-2">Dashboard</h1>
      <p class="text-gray-600">Manage and monitor your IPC subnet deployments</p>
    </div>

    <!-- Quick Stats -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-600">Total Subnets</p>
            <p class="text-3xl font-bold text-gray-900">{{ subnets.length }}</p>
          </div>
          <div class="w-12 h-12 bg-primary-50 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14-7l2 2m0 0l2 2m-2-2v6m-2-2H5m14-7v2a2 2 0 01-2 2H5a2 2 0 01-2-2V4"/>
            </svg>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-600">Active Subnets</p>
            <p class="text-3xl font-bold text-green-600">
              {{ subnets.filter(s => s.status === 'active').length }}
            </p>
          </div>
          <div class="w-12 h-12 bg-green-50 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
            </svg>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-600">Total Validators</p>
            <p class="text-3xl font-bold text-gray-900">
              {{ subnets.reduce((sum, subnet) => sum + subnet.validators, 0) }}
            </p>
          </div>
          <div class="w-12 h-12 bg-blue-50 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>
            </svg>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-600">Total Stake</p>
            <p class="text-3xl font-bold text-purple-600">
              {{ subnets.reduce((sum, subnet) => sum + parseFloat(subnet.totalStake), 0).toFixed(1) }} FIL
            </p>
          </div>
          <div class="w-12 h-12 bg-purple-50 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1"/>
            </svg>
          </div>
        </div>
      </div>
    </div>

    <!-- Subnets List -->
    <div class="card">
      <div class="flex items-center justify-between mb-6">
        <h2 class="text-xl font-semibold text-gray-900">Your Subnets</h2>
        <RouterLink to="/wizard" class="btn-primary">
          Deploy New Subnet
        </RouterLink>
      </div>

      <div class="space-y-4">
        <div
          v-for="subnet in subnets"
          :key="subnet.id"
          class="border border-gray-200 rounded-lg p-6 hover:shadow-md transition-shadow"
        >
          <div class="flex items-start justify-between mb-4">
            <div class="flex-1">
              <div class="flex items-center space-x-3 mb-2">
                <h3 class="text-lg font-semibold text-gray-900">{{ subnet.name }}</h3>
                <span
                  :class="[
                    'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium',
                    getStatusColor(subnet.status)
                  ]"
                >
                  <svg
                    :class="['w-3 h-3 mr-1']"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      :d="getStatusIcon(subnet.status)"
                    />
                  </svg>
                  {{ subnet.status.charAt(0).toUpperCase() + subnet.status.slice(1) }}
                </span>
              </div>
              <p class="text-gray-600 text-sm mb-1">{{ subnet.subnetId }}</p>
              <p class="text-gray-500 text-sm">Parent: {{ subnet.parentNetwork }}</p>
            </div>

            <div class="flex space-x-2">
              <RouterLink
                :to="`/instance/${subnet.id}`"
                class="btn-secondary text-sm"
              >
                View Details
              </RouterLink>
            </div>
          </div>

          <!-- Subnet Metrics -->
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
            <div>
              <p class="text-sm text-gray-500">Validators</p>
              <p class="font-semibold text-gray-900">{{ subnet.validators }}</p>
            </div>
            <div>
              <p class="text-sm text-gray-500">Total Stake</p>
              <p class="font-semibold text-gray-900">{{ subnet.totalStake }} FIL</p>
            </div>
            <div>
              <p class="text-sm text-gray-500">Block Height</p>
              <p class="font-semibold text-gray-900">{{ subnet.blockHeight.toLocaleString() }}</p>
            </div>
            <div>
              <p class="text-sm text-gray-500">Last Checkpoint</p>
              <p class="font-semibold text-gray-900">{{ subnet.lastCheckpoint }}</p>
            </div>
          </div>

          <!-- Child Subnets (if any) -->
          <div v-if="subnet.children.length > 0" class="border-t pt-4">
            <p class="text-sm font-medium text-gray-700 mb-2">Child Subnets ({{ subnet.children.length }})</p>
            <div class="flex flex-wrap gap-2">
              <span
                v-for="child in subnet.children"
                :key="child.id"
                :class="[
                  'inline-flex items-center px-3 py-1 rounded-full text-xs font-medium',
                  getStatusColor(child.status)
                ]"
              >
                {{ child.name }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-if="subnets.length === 0" class="text-center py-12">
        <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14-7l2 2m0 0l2 2m-2-2v6m-2-2H5m14-7v2a2 2 0 01-2 2H5a2 2 0 01-2-2V4"/>
        </svg>
        <h3 class="mt-2 text-sm font-medium text-gray-900">No subnets deployed</h3>
        <p class="mt-1 text-sm text-gray-500">Get started by deploying your first subnet.</p>
        <div class="mt-6">
          <RouterLink to="/wizard" class="btn-primary">
            Deploy Your First Subnet
          </RouterLink>
        </div>
      </div>
    </div>
  </div>
</template>