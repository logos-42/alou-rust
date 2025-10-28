<template>
  <Transition name="fade">
    <div v-if="show" class="welcome-screen">
      <div class="welcome-content">
        <div class="welcome-icon">🚀</div>
        <h2>{{ t('welcome') }}</h2>
        <p>{{ t('welcomeDesc') }}</p>
        <div class="quick-actions">
          <button class="quick-action-btn" @click="$emit('quick-message', t('queryBalance'))">
            <span class="action-icon">💰</span>
            <span>{{ t('queryBalance') }}</span>
          </button>
          <button class="quick-action-btn" @click="$emit('quick-message', '如何发送代币？')">
            <span class="action-icon">📤</span>
            <span>{{ t('sendTokens') }}</span>
          </button>
          <button class="quick-action-btn" @click="$emit('quick-message', t('transactionHistory'))">
            <span class="action-icon">📊</span>
            <span>{{ t('transactionHistory') }}</span>
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { useI18n } from '@/composables/useI18n'

interface Props {
  show: boolean
}

defineProps<Props>()
defineEmits(['quick-message'])

const { t } = useI18n()
</script>

<style scoped>
.welcome-screen {
  min-height: calc(100vh - 300px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}

.welcome-content {
  max-width: 800px;
  width: 100%;
  text-align: center;
  padding: 0 2rem;
}

.welcome-icon {
  font-size: 4rem;
  margin-bottom: 1.5rem;
  animation: float 3s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-10px); }
}

.welcome-content h2 {
  font-size: 2.5rem;
  font-weight: 700;
  margin: 0 0 1rem 0;
  background: linear-gradient(135deg, var(--primary-color), #8b5cf6);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.welcome-content p {
  font-size: 1.25rem;
  color: var(--text-secondary);
  margin: 0 0 2.5rem 0;
  line-height: 1.6;
}

.quick-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-top: 2rem;
}

.quick-action-btn {
  background: var(--background);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  padding: 1.25rem 1.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
  cursor: pointer;
  transition: all 0.3s ease;
  color: var(--text-primary);
  font-size: 0.95rem;
  font-weight: 500;
}

.quick-action-btn:hover {
  background: var(--secondary-color);
  border-color: var(--primary-color);
  transform: translateY(-2px);
  box-shadow: var(--shadow);
}

.action-icon {
  font-size: 2rem;
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.4s ease, transform 0.4s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(-20px) scale(0.95);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(-20px) scale(0.95);
}

@media (max-width: 768px) {
  .quick-actions {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 480px) {
  .welcome-content h2 {
    font-size: 1.5rem;
  }
  
  .welcome-content p {
    font-size: 1rem;
  }
}
</style>
