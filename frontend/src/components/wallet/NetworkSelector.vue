<template>
  <div class="network-section">
    <h3>{{ t('networkSettings') }}</h3>
    <div class="network-grid">
      <button
        v-for="network in networks"
        :key="network.chainId"
        @click="$emit('switch-network', network)"
        class="network-card"
        :class="{ active: currentNetwork === network.chainId }"
      >
        <div class="network-icon">{{ network.icon }}</div>
        <div class="network-info">
          <div class="network-name">{{ network.name }}</div>
          <div class="network-type">{{ network.type }}</div>
        </div>
        <div v-if="currentNetwork === network.chainId" class="network-check">✓</div>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/composables/useI18n'

interface Network {
  chainId: string
  name: string
  type: string
  icon: string
  rpcUrl: string
}

interface Props {
  networks: Network[]
  currentNetwork: string
}

defineProps<Props>()
defineEmits(['switch-network'])

const { t } = useI18n()
</script>

<style scoped>
.network-section {
  margin-bottom: 2rem;
}

.network-section h3 {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 1.5rem;
}

.network-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 1rem;
}

.network-card {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1.25rem;
  background: var(--background);
  border: 2px solid var(--border-color);
  border-radius: 1rem;
  cursor: pointer;
  transition: all 0.3s ease;
  text-align: left;
}

.network-card:hover {
  border-color: var(--primary-color);
  transform: translateY(-2px);
  box-shadow: var(--shadow);
}

.network-card.active {
  border-color: var(--primary-color);
  background: rgba(99, 102, 241, 0.1);
}

.network-icon {
  font-size: 2rem;
  flex-shrink: 0;
}

.network-info {
  flex: 1;
}

.network-name {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 0.25rem;
}

.network-type {
  font-size: 0.75rem;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.network-check {
  font-size: 1.5rem;
  color: var(--primary-color);
}
</style>
