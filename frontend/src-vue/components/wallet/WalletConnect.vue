<template>
  <div class="connect-section">
    <div class="connect-card">
      <div class="connect-icon">🔐</div>
      <h2>{{ t('connectWallet') }}</h2>
      <p>{{ t('connectWalletDesc') }}</p>
      <div class="wallet-options">
        <button @click="$emit('connect-metamask')" class="wallet-btn metamask">
          <img src="https://upload.wikimedia.org/wikipedia/commons/3/36/MetaMask_Fox.svg" alt="MetaMask" />
          <span>MetaMask</span>
        </button>
        <button @click="$emit('connect-walletconnect')" class="wallet-btn walletconnect">
          <span>🔗</span>
          <span>WalletConnect</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/composables/useI18n'

defineEmits(['connect-metamask', 'connect-walletconnect'])
const { t } = useI18n()
</script>

<style scoped>
.connect-section {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 60vh;
  padding: 2rem;
}

.connect-card {
  max-width: 500px;
  width: 100%;
  text-align: center;
  background: var(--background);
  border: 1px solid var(--border-color);
  border-radius: 1.5rem;
  padding: 3rem 2rem;
  box-shadow: var(--shadow-lg);
}

.connect-icon {
  font-size: 4rem;
  margin-bottom: 1.5rem;
}

.connect-card h2 {
  font-size: 2rem;
  font-weight: 700;
  margin: 0 0 1rem 0;
  color: var(--text-primary);
}

.connect-card p {
  font-size: 1.125rem;
  color: var(--text-secondary);
  margin: 0 0 2rem 0;
}

.wallet-options {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.wallet-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 1.25rem 2rem;
  border: 2px solid var(--border-color);
  border-radius: 1rem;
  background: var(--surface);
  color: var(--text-primary);
  font-size: 1.125rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.wallet-btn:hover {
  border-color: var(--primary-color);
  background: var(--background);
  transform: translateY(-2px);
  box-shadow: var(--shadow);
}

.wallet-btn img {
  width: 32px;
  height: 32px;
}

.wallet-btn span:first-child {
  font-size: 2rem;
}
</style>
