<script setup lang="ts">
import type { Channel } from '@/types/agent'

const props = defineProps<{
  channels: Channel[]
  activeChannelId: string
  keyword: string
}>()

const emit = defineEmits<{
  (e: 'update:keyword', value: string): void
  (e: 'select-channel', channel: Channel): void
  (e: 'create-channel'): void
}>()

function handleKeywordInput(event: Event) {
  emit('update:keyword', (event.target as HTMLInputElement).value)
}

function handleSelect(channel: Channel) {
  emit('select-channel', channel)
}

function handleCreate() {
  emit('create-channel')
}
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="brand">alou</div>
      <button class="new-channel-btn" @click="handleCreate">
        <span>＋</span>
      </button>
    </div>

    <div class="channel-search">
      <input type="text" :value="props.keyword" placeholder="搜索智能体" @input="handleKeywordInput" />
    </div>

    <div class="channel-list">
      <div
        v-for="channel in props.channels"
        :key="channel.id"
        class="channel-item"
        :class="{ active: channel.id === props.activeChannelId }"
        @click="handleSelect(channel)"
      >
        <div class="channel-icon" :style="{ background: channel.color }">{{ channel.icon }}</div>
        <div class="channel-info">
          <div class="channel-name">{{ channel.name }}</div>
          <div class="channel-meta">
            <span class="status-dot" :class="channel.status"></span>
            <span class="channel-status">{{ channel.statusLabel }}</span>
          </div>
        </div>
        <div class="channel-date">{{ new Date(channel.updatedAt).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' }) }}</div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  background: var(--surface);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  backdrop-filter: blur(16px);
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem 0.6rem;
}

.brand {
  font-size: 1.2rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.new-channel-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  background: var(--secondary-color);
  color: var(--primary-color);
  font-size: 1.1rem;
  cursor: pointer;
  transition: transform 0.2s ease;
}

.new-channel-btn:hover {
  transform: rotate(90deg);
}

.channel-search {
  padding: 0 1.25rem 0.75rem;
}

.channel-search input {
  width: 100%;
  border-radius: 0.75rem;
  border: 1px solid var(--border-color);
  padding: 0.6rem 0.9rem;
  background: transparent;
  color: var(--text-primary);
}

.channel-search input::placeholder {
  color: var(--text-secondary);
}

.channel-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 0.65rem 0.8rem;
}

.channel-item {
  display: grid;
  grid-template-columns: 40px 1fr auto;
  gap: 0.55rem;
  padding: 0.45rem 0.55rem;
  border-radius: 0.85rem;
  cursor: pointer;
  color: var(--text-secondary);
  align-items: center;
  transition: background 0.2s ease, transform 0.2s ease;
}

.channel-item:hover {
  background: rgba(99, 102, 241, 0.08);
  transform: translateY(-2px);
}

.channel-item.active {
  background: var(--secondary-color);
  color: var(--text-primary);
}

.channel-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 1.05rem;
  font-weight: 600;
}

.channel-info {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.channel-name {
  font-weight: 600;
  color: var(--text-primary);
}

.channel-meta {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-secondary);
}

.status-dot.online {
  background: #10b981;
}

.status-dot.busy {
  background: #f59e0b;
}

.status-dot.offline {
  background: #94a3b8;
}

.channel-date {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

@media (max-width: 768px) {
  .sidebar {
    display: none;
  }
}
</style>

