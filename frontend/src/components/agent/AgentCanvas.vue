<script setup lang="ts">
import { ref } from 'vue'
import type { AgentProfile } from '@/types/agent'
import type { CSSProperties } from 'vue'

const props = defineProps<{
  agentProfile: AgentProfile
  agentStyle: CSSProperties
}>()

const emit = defineEmits<{
  (e: 'pointerdown', event: PointerEvent): void
  (e: 'pointermove', event: PointerEvent): void
  (e: 'pointerup', event: PointerEvent): void
  (e: 'pointerleave', event: PointerEvent): void
  (e: 'trigger-mcp'): void
  (e: 'refresh-wallet'): void
  (e: 'open-wallet'): void
}>()

const root = ref<HTMLElement | null>(null)

function handlePointerDown(event: PointerEvent) {
  emit('pointerdown', event)
}

function handleTriggerMcp() {
  emit('trigger-mcp')
}

function handleRefresh() {
  emit('refresh-wallet')
}

function handleOpenWallet() {
  emit('open-wallet')
}

defineExpose({
  root,
  getElement: () => root.value
})
</script>

<template>
  <main
    ref="root"
    class="canvas"
    @pointermove="$emit('pointermove', $event)"
    @pointerup="$emit('pointerup', $event)"
    @pointerleave="$emit('pointerleave', $event)"
  >
    <div class="network-background">
      <div class="orb" v-for="n in 8" :key="n"></div>
    </div>

    <div class="agent-node" :style="props.agentStyle" @pointerdown="handlePointerDown">
      <div class="agent-glow"></div>
      <div class="agent-avatar">
        <img :src="props.agentProfile.avatar" alt="agent" />
      </div>
      <div class="agent-label">
        <h2>{{ props.agentProfile.name }}</h2>
        <p>{{ props.agentProfile.role }}</p>
      </div>
      <div class="agent-actions">
        <button @click.stop="handleOpenWallet">钱包</button>
        <button @click.stop="handleTriggerMcp">MCP</button>
        <button @click.stop="handleRefresh">刷新</button>
      </div>
    </div>
  </main>
</template>

<style scoped>
.canvas {
  position: relative;
  background: radial-gradient(circle at top, rgba(99, 102, 241, 0.08), transparent 55%),
    radial-gradient(circle at bottom, rgba(14, 165, 233, 0.12), transparent 60%),
    var(--background);
  overflow: hidden;
}

.network-background {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  pointer-events: none;
}

.orb {
  width: 280px;
  height: 280px;
  border: 1px dashed rgba(99, 102, 241, 0.18);
  border-radius: 50%;
  animation: pulse 12s infinite ease-in-out;
}

.orb:nth-child(odd) {
  border-color: rgba(14, 165, 233, 0.18);
}

.orb:nth-child(2) {
  animation-delay: 1.2s;
}

.orb:nth-child(3) {
  animation-delay: 2.4s;
}

.agent-node {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 220px;
  height: 220px;
  border-radius: 50%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  cursor: grab;
  transition: transform 0.15s ease;
  user-select: none;
}

.agent-node:active {
  cursor: grabbing;
}

.agent-glow {
  position: absolute;
  inset: 0;
  background: radial-gradient(circle, rgba(99, 102, 241, 0.35), transparent 70%);
  filter: blur(6px);
  border-radius: 50%;
}

.agent-avatar {
  width: 120px;
  height: 120px;
  border-radius: 50%;
  overflow: hidden;
  border: 4px solid rgba(255, 255, 255, 0.6);
  box-shadow: 0 15px 40px rgba(99, 102, 241, 0.35);
  position: relative;
  z-index: 1;
}

.agent-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.agent-label {
  position: relative;
  z-index: 1;
  text-align: center;
  color: var(--text-primary);
}

.agent-label h2 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 700;
}

.agent-label p {
  margin: 0.25rem 0 0;
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.agent-actions {
  display: flex;
  gap: 0.5rem;
  position: relative;
  z-index: 1;
}

.agent-actions button {
  border: none;
  border-radius: 999px;
  padding: 0.45rem 0.9rem;
  background: rgba(255, 255, 255, 0.85);
  color: var(--text-primary);
  font-size: 0.75rem;
  cursor: pointer;
  box-shadow: 0 8px 16px rgba(15, 23, 42, 0.12);
}

.agent-actions button:hover {
  background: rgba(255, 255, 255, 1);
}

.app-shell.dark-mode .agent-actions button {
  background: rgba(30, 41, 59, 0.85);
  color: var(--text-primary);
}

.app-shell.dark-mode .agent-actions button:hover {
  background: rgba(30, 41, 59, 1);
}

@keyframes pulse {
  0%,
  100% {
    transform: scale(1);
    opacity: 0.4;
  }
  50% {
    transform: scale(1.12);
    opacity: 1;
  }
}
</style>

