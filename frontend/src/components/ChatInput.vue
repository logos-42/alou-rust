<template>
  <div class="input-area">
    <div class="input-container">
      <div class="input-wrapper">
        <textarea
          :value="modelValue"
          @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
          @keydown.enter.exact.prevent="$emit('send')"
          @keydown.enter.shift.exact="$emit('new-line')"
          :placeholder="t('inputPlaceholder')"
          ref="textarea"
          class="message-input"
          rows="1"
        ></textarea>
        
        <div class="button-group">
          <button 
            @click="$emit('send')" 
            :disabled="!modelValue.trim() || isLoading"
            class="send-btn"
            :title="t('send')"
          >
            <svg v-if="!isLoading" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
            </svg>
            <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="currentColor" class="loading-icon">
              <path d="M12,4V2A10,10 0 0,0 2,12H4A8,8 0 0,1 12,4Z"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useI18n } from '@/composables/useI18n'

interface Props {
  modelValue: string
  isLoading: boolean
}

const props = defineProps<Props>()
defineEmits(['update:modelValue', 'send', 'new-line'])

const { t } = useI18n()
const textarea = ref<HTMLTextAreaElement>()

watch(() => props.modelValue, async () => {
  await nextTick()
  adjustHeight()
})

function adjustHeight() {
  if (textarea.value) {
    textarea.value.style.height = 'auto'
    textarea.value.style.height = Math.min(textarea.value.scrollHeight, 120) + 'px'
  }
}

defineExpose({ textarea, adjustHeight })
</script>

<style scoped>
.input-area {
  width: 100%;
  background: var(--background);
  border-top: 1px solid var(--border-color);
  padding: 1.25rem 0;
  display: flex;
  justify-content: center;
  flex-shrink: 0;
}

.input-container {
  width: 100%;
  max-width: 100%;
  padding: 0 clamp(1rem, 5vw, 4rem);
}

.input-wrapper {
  display: flex;
  gap: 1rem;
  align-items: flex-end;
  background: var(--background);
  border: 2px solid var(--border-color);
  border-radius: 1.75rem;
  padding: 1rem 1.5rem;
  transition: all 0.3s ease;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
  min-height: 64px;
}

.input-wrapper:focus-within {
  border-color: var(--primary-color);
  box-shadow: 0 4px 20px rgba(99, 102, 241, 0.15);
}

.message-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  font-family: inherit;
  font-size: 1rem;
  line-height: 1.6;
  color: var(--text-primary);
  min-height: 48px;
  max-height: 200px;
  padding: 8px 0;
}

.message-input::placeholder {
  color: var(--text-secondary);
}

.button-group {
  display: flex;
  align-items: center;
}

.send-btn {
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 0.875rem;
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
  margin-bottom: 2px;
}

.send-btn:hover:not(:disabled) {
  background: var(--primary-hover);
  transform: scale(1.05);
}

.send-btn:disabled {
  background: var(--text-secondary);
  opacity: 0.5;
  cursor: not-allowed;
}

.loading-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
