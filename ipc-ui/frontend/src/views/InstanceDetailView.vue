<script setup lang="ts">
defineProps<{
  id: string
}>()
</script>

<template>
  <div class="p-6">
    <h1 class="text-3xl font-bold text-gray-900 mb-6">Subnet Details: {{ id }}</h1>

    <div class="card">
      <div class="text-center py-12 text-gray-500">
        <svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
        </svg>
        <p class="text-lg font-medium mb-2">Instance Details</p>
        <p>Detailed view of subnet {{ id }} will be available here.</p>
      </div>
    </div>
  </div>
</template>