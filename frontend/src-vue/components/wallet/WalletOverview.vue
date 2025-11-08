<template>
  <div class="wallet-overview-card">
    <div class="wallet-header">
      <div class="wallet-avatar">
        <div class="avatar-icon">👤</div>
      </div>
      <div class="wallet-details">
        <div class="wallet-address-full">{{ formatAddress(wallet.address) }}</div>
        <div class="wallet-network">
          <span class="network-dot" :class="currentNetwork"></span>
          {{ networkName }}
        </div>
      </div>
      <button @click="$emit('disconnect')" class="disconnect-btn">
        {{ t('disconnect') }}
      </button>
    </div>
    
    <div class="wallet-balances">
      <div class="balance-card">
        <div class="balance-label">ETH {{ t('balance') }}</div>
        <div class="balance-amount">{{ wallet.ethBalance || '0.0' }}</div>
        <div class="balance-usd">≈ ${{ calculateUSD(wallet.ethBalance, ethPrice) }}</div>
      </div>
      <div class="balance-card">
        <div class="balance-label">USDC {{ t('balance') }}</div>
        <div class="balance-amount">{{ wallet.usdcBalance || '0.0' }}</div>
        <div class="balance-usd">≈ ${{ wallet.usdcBalance || '0.0' }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/composables/useI18n'

interface Props {
  wallet: {
    address: string
    ethBalance: string
    usdcBalance: string
  }
  currentNetwork: string
  networkName: string
  ethPrice: number
}

defineProps<Props>()
defineEmits(['disconnect'])

const { t } = useI18n()

function formatAddress(address: string): string {
  if (!address || address.length <= 10) return address
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

function calculateUSD(ethAmount: string, price: number): string {
  return (parseFloat(ethAmount || '0') * price).toFixed(2)
}
</script>

<style scoped>
.wallet-overview-card {
  background: var(--background);
  border: 1px solid var(--border-color);
  border-radius: 1.5rem;
  padding: 2rem;
  margin-bottom: 2rem;
  box-shadow: var(--shadow);
}

.wallet-header {
  display: flex;
  align-items: center;
  gap: 1.5rem;
  margin-bottom: 2rem;
  padding-bottom: 2rem;
  border-bottom: 1px solid var(--border-color);
}

.wallet-avatar {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--primary-color), #8b5cf6);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.avatar-icon {
  font-size: 2rem;
}

.wallet-details {
  flex: 1;
}

.wallet-address-full {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 0.5rem;
  font-family: monospace;
}

.wallet-network {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.network-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--success-color);
  animation: pulse 2s infinite;
}

.disconnect-btn {
  padding: 0.75rem 1.5rem;
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  background: var(--surface);
  color: var(--text-primary);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.disconnect-btn:hover {
  background: var(--error-color);
  color: white;
  border-color: var(--error-color);
}

.wallet-balances {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1.5rem;
}

.balance-card {
  background: var(--surface);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  padding: 1.5rem;
  transition: all 0.3s ease;
}

.balance-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow);
}

.balance-label {
  font-size: 0.875rem;
  color: var(--text-secondary);
  margin-bottom: 0.75rem;
}

.balance-amount {
  font-size: 2rem;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 0.5rem;
}

.balance-usd {
  font-size: 1rem;
  color: var(--text-secondary);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
</style>
