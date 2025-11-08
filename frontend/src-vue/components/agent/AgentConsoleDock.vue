<script setup lang="ts">
import { computed, ref } from 'vue'
import ChatInput from '@/components/ChatInput.vue'
import type { CSSProperties } from 'vue'

const props = defineProps<{
  modelValue: string
  isLoading: boolean
  style?: CSSProperties
  showOpenButton: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'send'): void
  (e: 'new-line'): void
  (e: 'open-conversation'): void
}>()

const inputValue = computed({
  get: () => props.modelValue,
  set: (value: string) => emit('update:modelValue', value)
})

const chatInputRef = ref<InstanceType<typeof ChatInput> | null>(null)

function handleSend() {
  emit('send')
}

function handleNewLine() {
  emit('new-line')
}

function handleOpen() {
  emit('open-conversation')
}

function adjustInputHeight() {
  chatInputRef.value?.adjustHeight()
}

defineExpose({ adjustInputHeight })
</script>

<template>
  <div class="console-dock" :style="style">
    <button v-if="showOpenButton" class="reopen-btn" @click="handleOpen">查看对话</button>
    <div class="console-input">
      <ChatInput
        ref="chatInputRef"
        v-model="inputValue"
        :is-loading="isLoading"
        @send="handleSend"
        @new-line="handleNewLine"
      />
    </div>
  </div>
</template>

<style scoped>
.console-dock {
  border-top: 1px solid rgba(15, 23, 42, 0.05);
  background: rgba(255, 255, 255, 0.92);
  padding: 0.35rem 0 0.55rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  backdrop-filter: blur(8px);
}

.app-shell.dark-mode .console-dock {
  background: rgba(15, 23, 42, 0.88);
  border-top: 1px solid rgba(148, 163, 184, 0.12);
}

.reopen-btn {
  align-self: flex-start;
  border: none;
  background: rgba(99, 102, 241, 0.12);
  color: var(--primary-color);
  border-radius: 999px;
  padding: 0.25rem 0.7rem;
  font-size: 0.72rem;
  cursor: pointer;
  box-shadow: var(--shadow);
  transition: background 0.2s ease;
}

.reopen-btn:hover {
  background: rgba(99, 102, 241, 0.2);
}

.console-input {
  width: 100%;
  padding: 0;
  border: none;
  background: transparent;
  box-shadow: none;
}

.app-shell.dark-mode .console-input {
  background: transparent;
}

.console-input :deep(.input-area) {
  padding: 0;
}

.console-input :deep(.input-container) {
  padding: 0;
}
</style>

