<template>
  <nav class="top-nav">
    <div class="nav-content">
      <div class="logo-section">
        <div class="logo">💖</div>
        <h1 class="app-title">{{ t('appTitle') }}</h1>
      </div>
      
      <div class="nav-controls">
        <div class="status-badge" :class="connectionStatus">
          <div class="status-dot"></div>
          <span>{{ statusText }}</span>
        </div>
        
        <button @click="$emit('toggle-theme')" class="theme-toggle" :title="t('theme')">
          <span v-if="isDarkMode">🌞</span>
          <span v-else>🌙</span>
        </button>
        
        <button 
          v-if="!isAuthenticated" 
          @click="$emit('go-to-login')" 
          class="login-btn"
        >
          <span>🔐</span>
          <span>{{ t('login') }}</span>
        </button>
        
        <div v-else class="user-menu">
          <button @click="toggleUserMenu" class="user-btn">
            <span>👤</span>
            <span>{{ userName }}</span>
          </button>
          <div v-if="showUserMenu" class="user-dropdown">
            <button @click="handleWalletClick" class="menu-item">
              <span>💰</span>
              <span>{{ t('walletManagement') }}</span>
            </button>
            <button @click="handleLogoutClick" class="menu-item">
              <span>🚪</span>
              <span>{{ t('logout') }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from '@/composables/useI18n'

interface Props {
  connectionStatus: 'connected' | 'disconnected' | 'error'
  isDarkMode: boolean
  isAuthenticated: boolean
  userName?: string
}

const props = defineProps<Props>()
const emit = defineEmits(['toggle-theme', 'go-to-login', 'go-to-wallet', 'logout'])

const { t } = useI18n()
const showUserMenu = ref(false)

const statusText = computed(() => {
  if (props.connectionStatus === 'connected') return t('connected')
  if (props.connectionStatus === 'disconnected') return t('disconnected')
  return t('connecting')
})

function toggleUserMenu() {
  showUserMenu.value = !showUserMenu.value
}

function handleWalletClick() {
  showUserMenu.value = false
  emit('go-to-wallet')
}

function handleLogoutClick() {
  showUserMenu.value = false
  emit('logout')
}
</script>

<style scoped>
.top-nav {
  width: 100%;
  background: var(--background);
  border-bottom: 1px solid var(--border-color);
  padding: 1rem 0;
  box-shadow: var(--shadow);
  z-index: 10;
  flex-shrink: 0;
}

.nav-content {
  width: 100%;
  padding: 0 clamp(1rem, 3vw, 3rem);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.logo-section {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.logo {
  font-size: 2rem;
}

.app-title {
  font-size: 1.5rem;
  font-weight: 700;
  margin: 0;
  background: linear-gradient(135deg, var(--primary-color), #8b5cf6);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.nav-controls {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 2rem;
  font-size: 0.875rem;
  font-weight: 500;
  background: var(--secondary-color);
  color: var(--text-secondary);
}

.status-badge.connected {
  background: rgba(16, 185, 129, 0.1);
  color: var(--success-color);
}

.status-badge.error {
  background: rgba(239, 68, 68, 0.1);
  color: var(--error-color);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-secondary);
}

.status-badge.connected .status-dot {
  background: var(--success-color);
  animation: pulse 2s infinite;
}

.theme-toggle, .login-btn, .user-btn {
  cursor: pointer;
  transition: all 0.3s ease;
}

.theme-toggle {
  background: var(--secondary-color);
  border: none;
  border-radius: 50%;
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.25rem;
}

.login-btn {
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 0.75rem;
  padding: 0.75rem 1.25rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
}

.user-menu {
  position: relative;
}

.user-btn {
  background: var(--secondary-color);
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  padding: 0.75rem 1.25rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-primary);
}

.user-dropdown {
  position: absolute;
  top: calc(100% + 0.5rem);
  right: 0;
  background: var(--background);
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  box-shadow: var(--shadow-lg);
  min-width: 200px;
  z-index: 100;
  overflow: hidden;
}

.menu-item {
  width: 100%;
  background: transparent;
  border: none;
  padding: 0.875rem 1.25rem;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  cursor: pointer;
  font-size: 0.875rem;
  color: var(--text-primary);
  transition: all 0.2s ease;
  text-align: left;
}

.menu-item:hover {
  background: var(--secondary-color);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
</style>
