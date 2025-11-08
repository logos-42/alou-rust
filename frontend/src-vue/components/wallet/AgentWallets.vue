<template>
  <div v-if="wallets.length > 0" class="agent-wallets-section">
    <h3 class="section-title">🤖 智能体钱包</h3>
    <div class="agent-wallets-grid">
      <div v-for="wallet in wallets" :key="wallet.address" class="agent-wallet-card">
        <div class="wallet-header">
          <span class="wallet-chain">{{ wallet.chain }}</span>
          <span class="wallet-badge">AI</span>
        </div>
        <div class="wallet-address">{{ formatAddress(wallet.address) }}</div>
        <div class="wallet-balance">
          <span class="balance-label">余额:</span>
          <span class="balance-value">{{ wallet.balance || '0' }} {{ getChainSymbol(wallet.chain) }}</span>
        </div>
        <div class="wallet-info">
          <span class="info-item">交易: {{ wallet.transactions?.length || 0 }}</span>
          <span class="info-item">创建: {{ formatDate(wallet.created_at) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface AgentWallet {
  address: string
  chain: string
  balance: string
  created_at: number
  transactions?: any[]
}

interface Props {
  wallets: AgentWallet[]
}

defineProps<Props>()

function formatAddress(address: string): string {
  if (!address) return ''
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

function getChainSymbol(chain: string): string {
  const symbols: Record<string, string> = {
    'ethereum': 'ETH',
    'base': 'ETH',
    'polygon': 'MATIC'
  }
  return symbols[chain] || 'ETH'
}

function formatDate(timestamp: number): string {
  if (!timestamp) return ''
  const date = new Date(timestamp * 1000)
  return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}
</script>

<style scoped>
.agent-wallets-section {
  margin-top: 2rem;
  animation: fadeIn 0.5s ease 0.4s both;
}

.section-title {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 1rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.agent-wallets-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1rem;
}

.agent-wallet-card {
  background: var(--background);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  padding: 1.25rem;
  transition: all 0.3s ease;
}

.agent-wallet-card:hover {
  border-color: var(--primary-color);
  box-shadow: var(--shadow);
  transform: translateY(-2px);
}

.wallet-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.wallet-chain {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--primary-color);
  text-transform: capitalize;
}

.wallet-badge {
  background: linear-gradient(135deg, var(--primary-color), #8b5cf6);
  color: white;
  font-size: 0.75rem;
  font-weight: 600;
  padding: 0.25rem 0.5rem;
  border-radius: 0.375rem;
}

.wallet-address {
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  color: var(--text-secondary);
  margin-bottom: 0.75rem;
}

.wallet-balance {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  background: var(--surface);
  border-radius: 0.5rem;
  margin-bottom: 0.75rem;
}

.balance-label {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.balance-value {
  font-size: 1rem;
  font-weight: 600;
  color: var(--success-color);
}

.wallet-info {
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.info-item {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 768px) {
  .agent-wallets-grid {
    grid-template-columns: 1fr;
  }
}
</style>
