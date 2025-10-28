<template>
  <div class="messages-area" ref="container">
    <TransitionGroup name="message" tag="div">
      <div 
        v-for="message in messages" 
        :key="message.id"
        class="message-wrapper"
        :class="message.type"
      >
        <div class="message-bubble">
          <div class="message-content" v-html="formatMessage(message.content)"></div>
          <div class="message-footer">
            <span class="timestamp">{{ formatTime(message.timestamp) }}</span>
            <span v-if="message.source" class="source-tag">{{ formatSource(message.source) }}</span>
          </div>
        </div>
      </div>
    </TransitionGroup>
    
    <!-- 加载指示器 -->
    <div v-if="isLoading" class="message-wrapper assistant">
      <div class="message-bubble loading">
        <div class="typing-animation">
          <div class="typing-dots">
            <span></span>
            <span></span>
            <span></span>
          </div>
          <span class="typing-text">{{ t('thinking') }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '@/composables/useI18n'

interface Message {
  id: string
  type: 'user' | 'assistant'
  content: string
  timestamp: number
  source?: string
}

interface Props {
  messages: Message[]
  isLoading: boolean
}

defineProps<Props>()
const { t } = useI18n()
const container = ref<HTMLElement>()

function formatMessage(content: string): string {
  return content
    .replace(/\n/g, '<br>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/`(.*?)`/g, '<code>$1</code>')
    .replace(/•/g, '<span class="bullet">•</span>')
}

function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString('zh-CN', { 
    hour: '2-digit', 
    minute: '2-digit' 
  })
}

function formatSource(source: string): string {
  const sourceMap: Record<string, string> = {
    'wasm-core': 'WASM',
    'edge-worker-proxy': 'Edge',
    'http-backend-fallback': 'Backend',
    'system': 'System',
    'error': 'Error'
  }
  return sourceMap[source] || source
}

defineExpose({ container })
</script>

<style scoped>
.messages-area {
  width: 100%;
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  scroll-behavior: smooth;
  padding: 1rem 0;
}

.message-wrapper {
  display: flex;
  animation: fadeInUp 0.3s ease;
  padding: 0.75rem clamp(1rem, 5vw, 4rem);
}

.message-wrapper.user {
  background: var(--background);
}

.message-wrapper.assistant {
  background: var(--surface);
}

.message-bubble {
  width: 100%;
  max-width: 1400px;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

.message-content {
  line-height: 1.8;
  font-size: 1rem;
  width: 100%;
}

.message-content :deep(strong) {
  font-weight: 600;
}

.message-content :deep(code) {
  background: rgba(0, 0, 0, 0.1);
  padding: 0.125rem 0.375rem;
  border-radius: 0.375rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.875rem;
}

.message-content :deep(.bullet) {
  color: var(--primary-color);
  font-weight: bold;
  margin-right: 0.25rem;
}

.message-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 0.75rem;
  font-size: 0.75rem;
  opacity: 0.7;
}

.source-tag {
  background: rgba(0, 0, 0, 0.1);
  padding: 0.125rem 0.5rem;
  border-radius: 1rem;
  font-size: 0.625rem;
  font-weight: 500;
  text-transform: uppercase;
}

.message-bubble.loading {
  background: var(--secondary-color);
}

.typing-animation {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.typing-dots {
  display: flex;
  gap: 0.25rem;
}

.typing-dots span {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--primary-color);
  animation: typing 1.4s infinite ease-in-out;
}

.typing-dots span:nth-child(2) {
  animation-delay: 0.2s;
}

.typing-dots span:nth-child(3) {
  animation-delay: 0.4s;
}

.typing-text {
  color: var(--text-secondary);
  font-size: 0.875rem;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes typing {
  0%, 60%, 100% {
    transform: scale(1);
    opacity: 0.5;
  }
  30% {
    transform: scale(1.2);
    opacity: 1;
  }
}

.message-enter-active {
  transition: all 0.3s ease-out;
}

.message-enter-from {
  opacity: 0;
  transform: translateY(20px) scale(0.95);
}

.messages-area::-webkit-scrollbar {
  width: 6px;
}

.messages-area::-webkit-scrollbar-track {
  background: transparent;
}

.messages-area::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}
</style>
