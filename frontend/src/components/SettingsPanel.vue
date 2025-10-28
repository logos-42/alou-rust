<template>
  <div>
    <!-- 设置按钮 -->
    <button @click="$emit('toggle')" class="settings-btn" :title="t('settings')">
      <span>⚙️</span>
    </button>

    <!-- 设置面板 -->
    <Transition name="slide">
      <div v-if="show" class="settings-panel">
        <div class="settings-header">
          <h3>{{ t('settings') }}</h3>
          <button @click="$emit('toggle')" class="close-btn">✕</button>
        </div>
        <div class="settings-content">
          <div class="setting-item">
            <label>{{ t('language') }}</label>
            <select :value="language" @change="handleLanguageChange" class="language-select">
              <option value="zh">中文</option>
              <option value="en">English</option>
            </select>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/composables/useI18n'

interface Props {
  show: boolean
  language: string
}

defineProps<Props>()
const emit = defineEmits(['toggle', 'change-language'])

const { t } = useI18n()

function handleLanguageChange(event: Event) {
  const target = event.target as HTMLSelectElement
  emit('change-language', target.value)
}
</script>

<style scoped>
.settings-btn {
  position: fixed;
  bottom: 6rem;
  left: 1.5rem;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: var(--primary-color);
  color: white;
  border: none;
  font-size: 1.25rem;
  cursor: pointer;
  box-shadow: var(--shadow-lg);
  transition: all 0.3s ease;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
}

.settings-btn:hover {
  transform: scale(1.1) rotate(90deg);
  box-shadow: 0 8px 24px rgba(99, 102, 241, 0.4);
}

.settings-panel {
  position: fixed;
  bottom: 7.5rem;
  left: 1.5rem;
  width: 300px;
  background: var(--background);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  box-shadow: var(--shadow-lg);
  z-index: 100;
  overflow: hidden;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--border-color);
  background: var(--surface);
}

.settings-header h3 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 1.5rem;
  color: var(--text-secondary);
  cursor: pointer;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.5rem;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: var(--secondary-color);
  color: var(--text-primary);
}

.settings-content {
  padding: 1.5rem;
}

.setting-item {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.setting-item label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-primary);
}

.language-select {
  padding: 0.75rem 1rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--background);
  color: var(--text-primary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.language-select:hover {
  border-color: var(--primary-color);
}

.language-select:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.slide-enter-active,
.slide-leave-active {
  transition: all 0.3s ease;
}

.slide-enter-from {
  opacity: 0;
  transform: translateY(20px) scale(0.95);
}

.slide-leave-to {
  opacity: 0;
  transform: translateY(20px) scale(0.95);
}

@media (max-width: 768px) {
  .settings-btn {
    bottom: 5.5rem;
    left: 1rem;
    width: 44px;
    height: 44px;
    font-size: 1.125rem;
  }

  .settings-panel {
    left: 1rem;
    right: 1rem;
    width: auto;
    bottom: 5.5rem;
  }
}
</style>
