<script setup lang="ts">
import { ref } from 'vue'
import MessageList from '@/components/MessageList.vue'
import type { Message } from '@/types/agent'
import type { CSSProperties } from 'vue'

defineProps<{
  style: CSSProperties
  connectionStatus: 'connected' | 'disconnected' | 'error'
  connectionStatusLabel: string
  messages: Message[]
  isLoading: boolean
}>()

const emit = defineEmits<{ (e: 'close'): void }>()

const messageListRef = ref<InstanceType<typeof MessageList>>()

function scrollToBottom() {
  const container = messageListRef.value?.container
  if (container) {
    container.scrollTop = container.scrollHeight
  }
}

defineExpose({ scrollToBottom })
</script>

<template>
  <div class="conversation-overlay" :style="style">
    <section class="conversation-panel">
      <header>
        <div class="title">
          <span class="emoji">💬</span>
          <span>会话上下文</span>
        </div>
        <div class="status" :class="connectionStatus">
          <span class="dot"></span>
          <span>{{ connectionStatusLabel }}</span>
        </div>
        <button class="close-btn" @click="emit('close')">✕</button>
      </header>
      <div class="conversation-body">
        <MessageList ref="messageListRef" :messages="messages" :is-loading="isLoading" />
      </div>
    </section>
  </div>
</template>

<style scoped>
.conversation-overlay {
  position: absolute;
  display: flex;
  justify-content: center;
  pointer-events: none;
}

.conversation-panel {
  width: 100%;
  max-width: 100%;
  background: rgba(255, 255, 255, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.4);
  border-radius: 1.25rem;
  padding: 0.85rem 1rem;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow);
  pointer-events: auto;
  max-height: clamp(240px, 42vh, 360px);
}

.app-shell.dark-mode .conversation-panel {
  background: rgba(15, 23, 42, 0.72);
  border-color: rgba(148, 163, 184, 0.16);
}

.conversation-panel header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}

.title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 600;
  color: var(--text-primary);
}

.status {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.status.connected .dot {
  background: #10b981;
}

.status.error .dot {
  background: #f59e0b;
}

.status.disconnected .dot {
  background: #ef4444;
}

.status .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #94a3b8;
}

.conversation-body {
  flex: 1;
  overflow-y: auto;
}

.conversation-panel :deep(.messages-area) {
  max-height: clamp(160px, 30vh, 240px);
  min-height: 140px;
  padding: 0.5rem 0.75rem 0.25rem;
  overflow-y: auto;
}

.conversation-panel :deep(.message-wrapper) {
  padding-inline: 0.5rem;
}

.conversation-panel :deep(.message-content) {
  font-size: 0.875rem;
}

.close-btn {
  border: none;
  background: rgba(148, 163, 184, 0.18);
  width: 28px;
  height: 28px;
  border-radius: 50%;
  cursor: pointer;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: rgba(148, 163, 184, 0.28);
}
</style>

