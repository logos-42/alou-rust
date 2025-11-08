<template>
  <div class="contract-wallet-section">
    <div class="section-header">
      <h3>{{ t('agentContractWallet') }}</h3>
      <button v-if="!contractWallet" @click="$emit('create')" class="create-btn">
        {{ t('createContractWallet') }}
      </button>
    </div>
    
    <div v-if="!contractWallet" class="empty-state">
      <div class="empty-icon">🤖</div>
      <p>{{ t('noContractWallet') }}</p>
      <p class="hint">{{ t('contractWalletHint') }}</p>
    </div>
    
    <div v-else class="contract-wallet-card">
      <div class="contract-header">
        <div class="contract-icon">🤖</div>
        <div class="contract-info">
          <div class="contract-label">{{ t('agentWallet') }}</div>
          <div class="contract-address">{{ formatAddress(contractWallet.address) }}</div>
        </div>
      </div>
      
      <div class="contract-balance">
        <div class="balance-label">{{ t('totalBalance') }}</div>
        <div class="balance-amount">{{ contractWallet.balance }} ETH</div>
      </div>
      
      <div class="contract-actions">
        <button @click="$emit('deposit')" class="action-btn primary">
          {{ t('deposit') }}
        </button>
        <button @click="$emit('withdraw')" class="action-btn">
          {{ t('withdraw') }}
        </button>
        <button @click="$emit('manage')" class="action-btn">
          {{ t('manage') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/composables/useI18n'

interface ContractWallet {
  address: string
  balance: string
}

interface Props {
  contractWallet: ContractWallet | null
}

defineProps<Props>()
defineEmits(['create', 'deposit', 'withdraw', 'manage'])

const { t } = useI18n()

function formatAddress(address: string): string {
  if (!address || address.length <= 10) return address
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}
</script>

<style scoped>
.contract-wallet-section {
  margin-bottom: 2rem;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.section-header h3 {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
}

.create-btn {
  padding: 0.75rem 1.5rem;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 0.75rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.create-btn:hover {
  background: var(--primary-hover);
  transform: translateY(-2px);
}

.empty-state {
  text-align: center;
  padding: 3rem 2rem;
  background: var(--surface);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.empty-state p {
  color: var(--text-secondary);
  font-size: 1rem;
  margin: 0.5rem 0;
}

.hint {
  font-size: 0.875rem !important;
  font-style: italic;
}

.contract-wallet-card {
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.1), rgba(139, 92, 246, 0.1));
  border: 2px solid var(--primary-color);
  border-radius: 1.5rem;
  padding: 2rem;
}

.contract-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 2rem;
  padding-bottom: 2rem;
  border-bottom: 1px solid var(--border-color);
}

.contract-icon {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: var(--primary-color);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 2rem;
  flex-shrink: 0;
}

.contract-info {
  flex: 1;
}

.contract-label {
  font-size: 0.875rem;
  color: var(--text-secondary);
  margin-bottom: 0.5rem;
}

.contract-address {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
  font-family: monospace;
}

.contract-balance {
  text-align: center;
  margin-bottom: 2rem;
}

.balance-label {
  font-size: 0.875rem;
  color: var(--text-secondary);
  margin-bottom: 0.5rem;
}

.balance-amount {
  font-size: 2.5rem;
  font-weight: 700;
  color: var(--text-primary);
}

.contract-actions {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
}

.action-btn {
  padding: 1rem;
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  background: var(--background);
  color: var(--text-primary);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.action-btn:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow);
}

.action-btn.primary {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
}

.action-btn.primary:hover {
  background: var(--primary-hover);
}

@media (max-width: 768px) {
  .contract-actions {
    grid-template-columns: 1fr;
  }
}
</style>
