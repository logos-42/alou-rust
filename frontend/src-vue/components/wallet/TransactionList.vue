<template>
  <div class="transactions-section">
    <div class="section-header">
      <h3>{{ t('recentTransactions') }}</h3>
      <button @click="$emit('refresh')" class="refresh-btn">
        <span :class="{ spinning: isRefreshing }">🔄</span>
      </button>
    </div>
    
    <div v-if="transactions.length === 0" class="empty-state">
      <div class="empty-icon">📭</div>
      <p>{{ t('noTransactions') }}</p>
    </div>
    
    <div v-else class="transaction-list">
      <div
        v-for="tx in transactions"
        :key="tx.hash"
        class="transaction-item"
        @click="$emit('view-transaction', tx)"
      >
        <div class="tx-icon" :class="tx.type">
          <span v-if="tx.type === 'send'">📤</span>
          <span v-else-if="tx.type === 'receive'">📥</span>
          <span v-else>🔄</span>
        </div>
        <div class="tx-details">
          <div class="tx-title">{{ tx.type === 'send' ? t('sent') : t('received') }}</div>
          <div class="tx-address">{{ formatAddress(tx.to || tx.from) }}</div>
        </div>
        <div class="tx-amount">
          <div class="amount-value" :class="tx.type">
            {{ tx.type === 'send' ? '-' : '+' }}{{ tx.value }} {{ tx.token }}
          </div>
          <div class="tx-time">{{ formatTime(tx.timestamp) }}</div>
        </div>
        <div class="tx-status" :class="tx.status">
          <span v-if="tx.status === 'confirmed'">✓</span>
          <span v-else-if="tx.status === 'pending'">⏳</span>
          <span v-else>❌</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/composables/useI18n'

interface Transaction {
  hash: string
  type: 'send' | 'receive' | 'contract'
  from: string
  to: string
  value: string
  token: string
  timestamp: number
  status: 'confirmed' | 'pending' | 'failed'
}

interface Props {
  transactions: Transaction[]
  isRefreshing: boolean
}

defineProps<Props>()
defineEmits(['refresh', 'view-transaction'])

const { t } = useI18n()

function formatAddress(address: string): string {
  if (!address || address.length <= 10) return address
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

function formatTime(timestamp: number): string {
  const now = Date.now()
  const diff = now - timestamp
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)
  const days = Math.floor(diff / 86400000)
  
  if (minutes < 1) return t('justNow')
  if (minutes < 60) return `${minutes}${t('minutesAgo')}`
  if (hours < 24) return `${hours}${t('hoursAgo')}`
  return `${days}${t('daysAgo')}`
}
</script>

<style scoped>
.transactions-section {
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

.refresh-btn {
  background: transparent;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  padding: 0.5rem;
  border-radius: 0.5rem;
  transition: all 0.3s ease;
}

.refresh-btn:hover {
  background: var(--secondary-color);
}

.spinning {
  display: inline-block;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.empty-state {
  text-align: center;
  padding: 3rem 2rem;
  background: var(--surface);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.empty-state p {
  color: var(--text-secondary);
  font-size: 1rem;
}

.transaction-list {
  background: var(--background);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  overflow: hidden;
}

.transaction-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--border-color);
  cursor: pointer;
  transition: all 0.2s ease;
}

.transaction-item:last-child {
  border-bottom: none;
}

.transaction-item:hover {
  background: var(--surface);
}

.tx-icon {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
  flex-shrink: 0;
}

.tx-icon.send {
  background: rgba(239, 68, 68, 0.1);
}

.tx-icon.receive {
  background: rgba(16, 185, 129, 0.1);
}

.tx-details {
  flex: 1;
}

.tx-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 0.25rem;
}

.tx-address {
  font-size: 0.875rem;
  color: var(--text-secondary);
  font-family: monospace;
}

.tx-amount {
  text-align: right;
}

.amount-value {
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 0.25rem;
}

.amount-value.send {
  color: var(--error-color);
}

.amount-value.receive {
  color: var(--success-color);
}

.tx-time {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.tx-status {
  font-size: 1.25rem;
  flex-shrink: 0;
}

.tx-status.confirmed {
  color: var(--success-color);
}

.tx-status.pending {
  color: var(--warning-color);
}

.tx-status.failed {
  color: var(--error-color);
}
</style>
