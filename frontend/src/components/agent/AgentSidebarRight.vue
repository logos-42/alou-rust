<script setup lang="ts">
import type { InteractionLog, TransactionItem, WalletSnapshot } from '@/types/agent'

const props = defineProps<{
  walletSnapshot: WalletSnapshot | null
  transactions: TransactionItem[]
  interactionLogs: InteractionLog[]
  isInteractionCollapsed: boolean
  isCollapsed: boolean
}>()

const emit = defineEmits<{
  (e: 'refresh-wallet'): void
  (e: 'toggle-interaction'): void
  (e: 'toggle-collapse'): void
}>()

function copyAddress(address: string) {
  navigator.clipboard.writeText(address).catch((error) => {
    console.error('Failed to copy address:', error)
  })
}

function formatDate(timestamp: number) {
  return new Date(timestamp).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' })
}

function formatTime(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

function formatLogDetail(log: InteractionLog) {
  if (!log.detail) return ''

  if (log.action === 'channel_selected') {
    const name = (log.detail.name as string) || '频道'
    const status = (log.detail.statusLabel as string) || ''
    return status ? `${name} · ${status}` : name
  }

  if (log.action === 'user_message') {
    return (log.detail.content as string) || ''
  }

  if (log.action === 'wallet_refresh' && log.detail.balance) {
    const balance = log.detail.balance as string
    const token = (log.detail.token as string) || 'ETH'
    return `${balance} ${token}`
  }

  try {
    return JSON.stringify(log.detail)
  } catch (error) {
    console.error('Failed to stringify log detail', error)
    return ''
  }
}
</script>

<template>
  <aside class="sidebar-right" :class="{ collapsed: props.isCollapsed }">
    <button class="collapse-toggle" @click="emit('toggle-collapse')" aria-label="切换右侧栏">
      <span v-if="props.isCollapsed">⋯</span>
      <span v-else>▶</span>
    </button>

    <div class="sidebar-content" v-if="!props.isCollapsed">
      <section class="wallet-card">
        <header>
          <div class="title">
            <span class="emoji">💰</span>
            <span>智能体资产</span>
          </div>
          <button class="refresh-btn" @click="emit('refresh-wallet')">刷新</button>
        </header>
        <div class="wallet-body" v-if="props.walletSnapshot">
          <div class="balance">
            <div class="amount">{{ props.walletSnapshot.balance }} ETH</div>
            <div class="fiat">≈ {{ props.walletSnapshot.balanceFiat }} USD</div>
          </div>
          <div class="address" @click="copyAddress(props.walletSnapshot.address)">
            {{ props.walletSnapshot.address.slice(0, 6) }}...{{ props.walletSnapshot.address.slice(-4) }}
          </div>
          <div class="network">当前网络：{{ props.walletSnapshot.networkLabel }}</div>
        </div>
        <div v-else class="wallet-empty">
          <p>尚未连接钱包</p>
          <slot name="connect-action"></slot>
        </div>
      </section>

      <section class="history-card">
        <header>
          <div class="title">
            <span class="emoji">📜</span>
            <span>转账历史</span>
          </div>
        </header>
        <ul>
          <li v-for="tx in props.transactions" :key="tx.id" :class="tx.status">
            <div class="tx-main">
              <div class="tx-amount">{{ tx.direction === 'out' ? '-' : '+' }}{{ tx.amount }} {{ tx.token }}</div>
              <div class="tx-status">{{ tx.statusLabel }}</div>
            </div>
            <div class="tx-sub">
              <span class="tx-address">{{ tx.counterparty.slice(0, 6) }}...{{ tx.counterparty.slice(-4) }}</span>
              <span class="tx-time">{{ formatDate(tx.timestamp) }}</span>
            </div>
          </li>
        </ul>
      </section>

      <section class="context-card" :class="{ collapsed: props.isInteractionCollapsed }">
        <header>
          <div class="title">
            <span class="emoji">🧠</span>
            <span>互动记录</span>
          </div>
          <button class="collapse-btn" @click="emit('toggle-interaction')">
            <span v-if="props.isInteractionCollapsed">展开</span>
            <span v-else>收起</span>
          </button>
        </header>
        <div class="context-body" v-show="!props.isInteractionCollapsed">
          <ul>
            <li v-for="log in props.interactionLogs" :key="log.id">
              <div class="log-main">
                <span class="log-label">{{ log.label }}</span>
                <span class="log-time">{{ formatTime(log.timestamp) }}</span>
              </div>
            <div class="log-detail" v-if="log.detail">
              {{ formatLogDetail(log) }}
              </div>
            </li>
          </ul>
        </div>
      </section>
    </div>
  </aside>
</template>

<style scoped>

.sidebar-right {
  position: relative;
  width: 340px;
  background: var(--surface);
  border-left: 1px solid var(--border-color);
  transition: width 0.3s ease;
  overflow: hidden;
}

.sidebar-right.collapsed {
  width: 64px;
}

.collapse-toggle {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  background: var(--surface);
  box-shadow: var(--shadow);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1rem;
  line-height: 1;
}

.sidebar-content {
  height: 100%;
  overflow-y: auto;
  padding: 1.15rem;
  display: flex;
  flex-direction: column;
  gap: 1.05rem;
}

@media (max-width: 1280px) {
  .sidebar-right {
    width: 300px;
  }

  .sidebar-right.collapsed {
    width: 64px;
  }
}

@media (max-width: 1024px) {
  .sidebar-right {
    position: absolute;
    right: 0;
    top: 72px;
    bottom: 0;
    z-index: 10;
    box-shadow: var(--shadow);
  }

  .sidebar-right.collapsed {
    width: 64px;
  }
}

@media (max-width: 768px) {
  .sidebar-right {
    width: 280px;
  }

  .sidebar-right.collapsed {
    width: 64px;
  }
}

.wallet-card,
.history-card,
.context-card {
  background: rgba(255, 255, 255, 0.65);
  border-radius: 1.1rem;
  padding: 1rem;
  box-shadow: var(--shadow);
  border: 1px solid rgba(255, 255, 255, 0.3);
}

.app-shell.dark-mode .wallet-card,
.app-shell.dark-mode .history-card,
.app-shell.dark-mode .context-card {
  background: rgba(15, 23, 42, 0.78);
  border-color: rgba(148, 163, 184, 0.12);
}

section header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.8rem;
}

.title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 600;
  color: var(--text-primary);
}

.refresh-btn {
  border: none;
  background: rgba(99, 102, 241, 0.12);
  color: var(--primary-color);
  border-radius: 999px;
  padding: 0.3rem 0.7rem;
  font-size: 0.72rem;
  cursor: pointer;
}

.wallet-body {
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
}

.balance {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
}

.amount {
  font-size: 1.6rem;
  font-weight: 700;
}

.fiat {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.address {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.85rem;
  color: var(--text-secondary);
  cursor: pointer;
}

.wallet-empty {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  align-items: flex-start;
}

.history-card ul {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.history-card li {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  padding: 0.6rem;
  border-radius: 0.9rem;
  background: rgba(148, 163, 184, 0.08);
}

.history-card li.confirmed {
  border-left: 3px solid #10b981;
}

.history-card li.pending {
  border-left: 3px solid #f59e0b;
}

.history-card li.failed {
  border-left: 3px solid #ef4444;
}

.tx-main {
  display: flex;
  justify-content: space-between;
  font-weight: 600;
}

.tx-sub {
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.context-card.collapsed .context-body {
  display: none;
}

.collapse-btn {
  border: none;
  padding: 0.35rem 0.8rem;
  border-radius: 999px;
  background: rgba(99, 102, 241, 0.12);
  color: var(--primary-color);
  cursor: pointer;
  font-size: 0.75rem;
}

.context-body ul {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
  max-height: 200px;
  overflow-y: auto;
}

.context-body li {
  background: rgba(148, 163, 184, 0.12);
  border-radius: 0.9rem;
  padding: 0.6rem 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.8rem;
  color: var(--text-secondary);
}

.log-main {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: 600;
  color: var(--text-primary);
}

.log-detail {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.72rem;
  opacity: 0.8;
  word-break: break-word;
}
</style>

