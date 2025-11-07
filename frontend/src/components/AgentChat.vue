<template>
  <div class="app-shell" :class="{ 'dark-mode': isDarkMode }">
    <ChatHeader
      :connection-status="connectionStatus"
      :is-dark-mode="isDarkMode"
      :is-authenticated="authStore.isAuthenticated"
      :user-name="authStore.userName"
      @toggle-theme="toggleDarkMode"
      @go-to-login="goToLogin"
      @go-to-wallet="goToWallet"
      @logout="handleLogout"
    />

    <div class="workspace">
      <AgentSidebarLeft
        :channels="filteredChannels"
        :active-channel-id="activeChannelId"
        :keyword="channelKeyword"
        @update:keyword="updateChannelKeyword"
        @select-channel="selectChannel"
        @create-channel="createChannel"
      />

      <AgentCanvas
        ref="canvasRef"
        :agent-profile="agentProfile"
        :agent-style="agentStyle"
        @pointerdown="startDrag"
        @pointermove="onDrag"
        @pointerup="stopDrag"
        @pointerleave="stopDrag"
        @trigger-mcp="triggerMcp"
        @refresh-wallet="refreshWallet"
        @open-wallet="goToWallet"
      />

      <AgentSidebarRight
        :wallet-snapshot="walletSnapshot"
        :transactions="transactions"
        :interaction-logs="interactionLogs"
        :is-interaction-collapsed="isInteractionCollapsed"
        :is-collapsed="isSidebarCollapsed"
        @refresh-wallet="refreshWallet"
        @toggle-interaction="toggleInteractionPanel"
        @toggle-collapse="toggleSidebar"
      >
        <template #connect-action>
          <button @click="goToWallet">立即连接</button>
        </template>
      </AgentSidebarRight>

      <AgentConversationOverlay
        v-if="showConversationPanel"
        ref="conversationOverlayRef"
        :style="conversationOverlayStyle"
        :connection-status="connectionStatus"
        :connection-status-label="connectionStatusLabel"
        :messages="messages"
        :is-loading="isLoading"
        @close="closeConversationPanel"
      />
    </div>

    <AgentConsoleDock
      ref="consoleDockRef"
      v-model="currentMessage"
      :is-loading="isLoading"
      :style="consoleDockStyle"
      :show-open-button="!showConversationPanel && messages.length > 0"
      @send="sendMessage"
      @new-line="newLine"
      @open-conversation="openConversationPanel"
    />

    <button class="language-switch" @click="toggleLanguage">
      {{ languageLabel }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useI18n } from '@/composables/useI18n'
import { walletService, type WalletInstruction } from '@/services/wallet.service'
import ChatHeader from './ChatHeader.vue'
import AgentSidebarLeft from './agent/AgentSidebarLeft.vue'
import AgentSidebarRight from './agent/AgentSidebarRight.vue'
import AgentCanvas from './agent/AgentCanvas.vue'
import AgentConsoleDock from './agent/AgentConsoleDock.vue'
import AgentConversationOverlay from './agent/AgentConversationOverlay.vue'
import type {
  AgentProfile,
  Channel,
  InteractionLog,
  Message,
  TransactionItem,
  WalletSnapshot,
  ContextEvent
} from '@/types/agent'

interface AgentResponse {
  content?: string
  response?: string
  status?: string
  timestamp?: number
  session_id?: string
  source?: string
  error?: string
  tool_calls?: Array<{
    id: string
    name: string
    result: any
  }>
  wallet_instructions?: WalletInstruction[]
}

const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL || (import.meta.env.DEV ? 'http://localhost:8787' : 'https://alou-edge.yuanjieliu65.workers.dev')

const router = useRouter()
const authStore = useAuthStore()
const { initLanguage, setLanguage, currentLanguage } = useI18n()

const isDarkMode = ref(false)
const connectionStatus = ref<'connected' | 'disconnected' | 'error'>('disconnected')
const connectionStatusLabel = computed(() => {
  if (connectionStatus.value === 'connected') return '已连接'
  if (connectionStatus.value === 'error') return '服务异常'
  return '未连接'
})
const isSidebarCollapsed = ref(true)
const isInteractionCollapsed = ref(true)
const isConversationVisible = ref(false)

const messages = ref<Message[]>([])
const currentMessage = ref('')
const isLoading = ref(false)
const sessionId = ref(`frontend_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`)
const isSessionReady = ref(false)
const conversationOverlayRef = ref<InstanceType<typeof AgentConversationOverlay> | null>(null)
const consoleDockRef = ref<InstanceType<typeof AgentConsoleDock> | null>(null)
const contextEvents = ref<ContextEvent[]>([])
const interactionLogs = ref<InteractionLog[]>([])
const viewportWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 1440)
const languageLabel = computed(() => (currentLanguage.value === 'zh' ? '中 / EN' : 'EN / 中'))
const showConversationPanel = computed(() => messages.value.length > 0 && isConversationVisible.value)

const canvasRef = ref<InstanceType<typeof AgentCanvas> | null>(null)
const isDragging = ref(false)
const dragOffset = reactive({ x: 0, y: 0 })
const agentPosition = reactive({ x: 0, y: 0 })

const channelKeyword = ref('')
const activeChannelId = ref('dev-relay')

const channels = ref<Channel[]>([
  { id: 'dev-relay', name: 'TRX Smart Contract Staking', status: 'online', statusLabel: '在线', icon: '⚡', color: 'linear-gradient(135deg,#6366f1,#8b5cf6)', updatedAt: Date.now() - 2 * 60 * 60 * 1000 },
  { id: 'eth-announce', name: 'ETH Contract Announcement', status: 'busy', statusLabel: '执行任务', icon: '⬡', color: 'linear-gradient(135deg,#0ea5e9,#2563eb)', updatedAt: Date.now() - 6 * 60 * 60 * 1000 },
  { id: 'firefly', name: 'Firefly Research', status: 'offline', statusLabel: '离线', icon: '🛰️', color: 'linear-gradient(135deg,#ec4899,#f97316)', updatedAt: Date.now() - 24 * 60 * 60 * 1000 },
  { id: 'wallet-ops', name: 'Wallet Operations', status: 'online', statusLabel: '在线', icon: '💼', color: 'linear-gradient(135deg,#14b8a6,#0ea5e9)', updatedAt: Date.now() - 30 * 60 * 1000 }
])

const agentProfile = reactive<AgentProfile>({
  name: 'alou',
  role: 'Web3 Multi-Agent Coordinator',
  avatar: 'https://avatars.githubusercontent.com/u/16309930?v=4'
})

const walletSnapshot = ref<WalletSnapshot | null>(null)

const transactions = ref<TransactionItem[]>([
  { id: 'tx-1', direction: 'out', amount: '0.42', token: 'ETH', counterparty: '0x1F345...ab91', status: 'confirmed', statusLabel: '已完成', timestamp: Date.now() - 4 * 60 * 60 * 1000 },
  { id: 'tx-2', direction: 'in', amount: '250', token: 'USDC', counterparty: '0x72ab...cc87', status: 'pending', statusLabel: '确认中', timestamp: Date.now() - 40 * 60 * 1000 },
  { id: 'tx-3', direction: 'out', amount: '1.2', token: 'ETH', counterparty: '0xbf12...9980', status: 'failed', statusLabel: '失败', timestamp: Date.now() - 3 * 24 * 60 * 60 * 1000 }
])

const filteredChannels = computed(() => {
  if (!channelKeyword.value.trim()) {
    return channels.value
  }
  const keyword = channelKeyword.value.toLowerCase()
  return channels.value.filter((channel) => channel.name.toLowerCase().includes(keyword))
})

const NODE_BOUNDARY = 140

const agentStyle = computed(() => ({
  transform: `translate(calc(-50% + ${agentPosition.x}px), calc(-50% + ${agentPosition.y}px))`
}))

const consoleDockStyle = computed(() => {
  if (viewportWidth.value <= 1024) {
    return { margin: '0 1rem 0 1rem' }
  }

  const leftWidth = viewportWidth.value <= 1280 ? 240 : 300
  const rightWidth = isSidebarCollapsed.value ? 80 : 340

  return {
    marginLeft: `${leftWidth + 24}px`,
    marginRight: `${rightWidth + 24}px`
  }
})

const conversationOverlayStyle = computed(() => {
  if (viewportWidth.value <= 1024) {
    return { left: '1rem', right: '1rem', bottom: '6rem' }
  }

  const leftWidth = viewportWidth.value <= 1280 ? 240 : 300
  const rightWidth = isSidebarCollapsed.value ? 80 : 340

  return {
    left: `${leftWidth + 24}px`,
    right: `${rightWidth + 24}px`,
    bottom: '6.5rem'
  }
})

onMounted(async () => {
  const savedTheme = localStorage.getItem('alou-theme')
  if (savedTheme) {
    isDarkMode.value = savedTheme === 'dark'
  } else {
    isDarkMode.value = window.matchMedia('(prefers-color-scheme: dark)').matches
  }

  initLanguage()
  await checkConnection()
  await createSession()
  isSessionReady.value = true
  await refreshWallet()

  window.addEventListener('wallet-changed', handleWalletChanged as EventListener)
  window.addEventListener('resize', handleResize)
  window.addEventListener('pointerup', handleGlobalPointerUp)
})

onUnmounted(() => {
  window.removeEventListener('wallet-changed', handleWalletChanged as EventListener)
  window.removeEventListener('resize', handleResize)
  window.removeEventListener('pointerup', handleGlobalPointerUp)
})

watch(isDarkMode, (value) => {
  localStorage.setItem('alou-theme', value ? 'dark' : 'light')
})

const ACTION_LABELS: Record<string, string> = {
  channel_selected: '切换频道',
  create_channel: '创建频道',
  toggle_theme: '主题切换',
  toggle_language: '语言切换',
  toggle_sidebar: '侧边栏',
  wallet_refresh: '刷新资产',
  wallet_event: '钱包变更',
  navigate_wallet: '打开钱包',
  navigate_login: '跳转登录',
  logout: '退出登录',
  agent_drag_start: '移动智能体',
  agent_drag_end: '智能体位置',
  trigger_mcp: '调用 MCP',
  user_message: '用户消息',
  wallet_instruction: '钱包指令'
}

function recordInteraction(action: string, detail?: Record<string, unknown>, label?: string) {
  const timestamp = Date.now()
  const entry: InteractionLog = {
    id: `log_${timestamp}_${Math.random().toString(36).slice(2, 6)}`,
    action,
    label: label || ACTION_LABELS[action] || action,
    timestamp,
    detail
  }

  interactionLogs.value = [entry, ...interactionLogs.value].slice(0, 20)
  contextEvents.value.push({ action, detail, timestamp })
  if (contextEvents.value.length > 50) {
    contextEvents.value.splice(0, contextEvents.value.length - 50)
  }

  window.dispatchEvent(new CustomEvent('agent-context-event', { detail: { action, detail, timestamp } }))
}

function appendMessage(message: Message) {
  messages.value.push(message)
  if (!isConversationVisible.value) {
    isConversationVisible.value = true
  }
}

async function createSession() {
  try {
    const walletAddress = localStorage.getItem('wallet_address')
    const response = await fetch(`${API_BASE_URL}/api/session`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ wallet_address: walletAddress || undefined })
    })

    if (response.ok) {
      const data = await response.json()
      sessionId.value = data.session_id
    }
  } catch (error) {
    console.error('Failed to create session:', error)
  }
}

async function handleToolCalls(toolCalls: Array<{ id: string; name: string; result: any }>) {
  for (const toolCall of toolCalls) {
    if (toolCall.name === 'wallet_manager' && toolCall.result) {
      const result = toolCall.result

      if (result.instruction) {
        try {
          if (result.action === 'switch_network' && result.network) {
            const success = await walletService.switchNetwork(result.network)
            if (success) {
              recordInteraction('wallet_instruction', {
                instruction: 'switch_network',
                chainId: result.network.chainId,
                name: result.network.name
              })
              appendMessage({
                id: `system_${Date.now()}`,
                type: 'assistant',
                content: `✅ 已成功切换到 ${result.network.name} (${result.network.type})`,
                timestamp: Date.now(),
                source: 'system'
              })
              await nextTick()
              scrollToBottom()
            }
          } else {
            await walletService.executeInstruction(result.instruction)
            recordInteraction('wallet_instruction', {
              instruction: result.instruction?.method || 'unknown'
            })
          }
        } catch (error) {
          console.error('Failed to execute wallet instruction:', error)
          appendMessage({
            id: `error_${Date.now()}`,
            type: 'assistant',
            content: `❌ 钱包操作失败：${error instanceof Error ? error.message : '未知错误'}`,
            timestamp: Date.now(),
            source: 'error'
          })
          await nextTick()
          scrollToBottom()
        }
      }
    }
  }
}

async function sendMessage() {
  if (!currentMessage.value.trim() || isLoading.value) return

  if (!isSessionReady.value) {
    await createSession()
    isSessionReady.value = true
  }

  const messageToSend = currentMessage.value.trim()
  const userMessage: Message = {
    id: `user_${Date.now()}`,
    type: 'user',
    content: messageToSend,
    timestamp: Date.now()
  }

  appendMessage(userMessage)
  currentMessage.value = ''
  isLoading.value = true
  recordInteraction('user_message', { content: messageToSend })

  await nextTick()
  scrollToBottom()

  const contextSnapshot = contextEvents.value.splice(0, contextEvents.value.length)

  try {
    const walletAddress = localStorage.getItem('wallet_address')
    const response = await fetch(`${API_BASE_URL}/api/agent/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        session_id: sessionId.value,
        message: messageToSend,
        wallet_address: walletAddress || undefined,
        context_events: contextSnapshot
      })
    })

    if (response.ok) {
      const data: AgentResponse = await response.json()

      if (data.tool_calls) {
        await handleToolCalls(data.tool_calls)
      }

      const assistantMessage: Message = {
        id: `assistant_${Date.now()}`,
        type: 'assistant',
        content: data.content || data.response || '收到响应',
        timestamp: data.timestamp || Date.now(),
        source: data.source || 'alou-edge'
      }
      appendMessage(assistantMessage)

      if (data.session_id) {
        sessionId.value = data.session_id
      }
    } else {
      const errorData = await response.json().catch(() => ({ error: '未知错误' }))
      throw new Error(`HTTP ${response.status}: ${errorData.error || response.statusText}`)
    }
  } catch (error) {
    appendMessage({
      id: `error_${Date.now()}`,
      type: 'assistant',
      content: `❌ 抱歉，发生了错误：${error instanceof Error ? error.message : '未知错误'}`,
      timestamp: Date.now(),
      source: 'error'
    })
  } finally {
    isLoading.value = false
    await nextTick()
    scrollToBottom()
    consoleDockRef.value?.adjustInputHeight()
  }
}

function newLine() {
  currentMessage.value += '\n'
}

function scrollToBottom() {
  requestAnimationFrame(() => {
    conversationOverlayRef.value?.scrollToBottom()
  })
}

async function checkConnection() {
  try {
    const response = await fetch(`${API_BASE_URL}/api/health`)
    connectionStatus.value = response.ok ? 'connected' : 'error'
  } catch (error) {
    console.error('Connection check failed:', error)
    connectionStatus.value = 'disconnected'
  }
}

async function refreshWallet() {
  try {
    const info = await walletService.getCurrentWalletInfo()
    if (!info) {
      walletSnapshot.value = null
      recordInteraction('wallet_refresh', { connected: false }, '刷新资产')
      return
    }

    const balance = await walletService.getBalance(info.address)
    walletSnapshot.value = {
      address: info.address,
      chainId: info.chainId,
      balance,
      balanceFiat: (parseFloat(balance || '0') * 3400).toFixed(2),
      networkLabel: resolveNetworkName(info.chainId)
    }
    recordInteraction('wallet_refresh', {
      connected: true,
      balance,
      token: 'ETH'
    }, '刷新资产')
  } catch (error) {
    console.error('Failed to refresh wallet info:', error)
  }
}

async function handleWalletChanged(event: Event) {
  const detail = (event as CustomEvent<{ address?: string }>).detail
  recordInteraction('wallet_event', { address: detail?.address })
  await refreshWallet()
  await createSession()
  isSessionReady.value = true
}

function resolveNetworkName(chainId: string) {
  const mapping: Record<string, string> = {
    '0x1': 'Ethereum Mainnet',
    '0x14a34': 'Base Sepolia',
    '0x2105': 'Base Mainnet'
  }
  return mapping[chainId] || `Chain ${chainId}`
}

function updateChannelKeyword(value: string) {
  channelKeyword.value = value
}

function selectChannel(channel: Channel) {
  activeChannelId.value = channel.id
  recordInteraction('channel_selected', {
    channelId: channel.id,
    name: channel.name,
    status: channel.status,
    statusLabel: channel.statusLabel
  })
}

function createChannel() {
  recordInteraction('create_channel')
  // TODO: 打开频道创建流程
}

function toggleSidebar() {
  isSidebarCollapsed.value = !isSidebarCollapsed.value
  recordInteraction('toggle_sidebar', { collapsed: isSidebarCollapsed.value })
}

function toggleInteractionPanel() {
  isInteractionCollapsed.value = !isInteractionCollapsed.value
}

function toggleDarkMode() {
  isDarkMode.value = !isDarkMode.value
  recordInteraction('toggle_theme', { theme: isDarkMode.value ? 'dark' : 'light' })
}

function openConversationPanel() {
  if (!isConversationVisible.value) {
    isConversationVisible.value = true
    nextTick(() => scrollToBottom())
  }
}

function toggleLanguage() {
  const next = currentLanguage.value === 'zh' ? 'en' : 'zh'
  setLanguage(next)
  recordInteraction('toggle_language', { language: next })
}

function closeConversationPanel() {
  isConversationVisible.value = false
}

function goToLogin() {
  recordInteraction('navigate_login')
  router.push('/login')
}

function goToWallet() {
  recordInteraction('navigate_wallet')
  router.push('/wallet')
}

async function handleLogout() {
  recordInteraction('logout')
  await authStore.logout()
}

function startDrag(event: PointerEvent) {
  const canvasElement = canvasRef.value?.getElement?.()
  if (!canvasElement) return

  isDragging.value = true
  const rect = canvasElement.getBoundingClientRect()
  const centerX = rect.left + rect.width / 2
  const centerY = rect.top + rect.height / 2
  dragOffset.x = event.clientX - (centerX + agentPosition.x)
  dragOffset.y = event.clientY - (centerY + agentPosition.y)
  ;(event.target as HTMLElement).setPointerCapture(event.pointerId)
  recordInteraction('agent_drag_start', { x: agentPosition.x, y: agentPosition.y })
}

function onDrag(event: PointerEvent) {
  if (!isDragging.value) return
  const canvasElement = canvasRef.value?.getElement?.()
  if (!canvasElement) return

  const rect = canvasElement.getBoundingClientRect()
  const centerX = rect.left + rect.width / 2
  const centerY = rect.top + rect.height / 2

  const nextX = event.clientX - centerX - dragOffset.x
  const nextY = event.clientY - centerY - dragOffset.y

  const limitX = Math.max(rect.width / 2 - NODE_BOUNDARY, 0)
  const limitY = Math.max(rect.height / 2 - NODE_BOUNDARY, 0)

  agentPosition.x = Math.min(Math.max(nextX, -limitX), limitX)
  agentPosition.y = Math.min(Math.max(nextY, -limitY), limitY)
}

function stopDrag(event: PointerEvent) {
  if (isDragging.value) {
    isDragging.value = false
    ;(event.target as HTMLElement).releasePointerCapture?.(event.pointerId)
    recordInteraction('agent_drag_end', { x: agentPosition.x, y: agentPosition.y })
  }
}

function handleGlobalPointerUp() {
  isDragging.value = false
}

function clampPosition() {
  const canvasElement = canvasRef.value?.getElement?.()
  if (!canvasElement) return

  const rect = canvasElement.getBoundingClientRect()
  const limitX = Math.max(rect.width / 2 - NODE_BOUNDARY, 0)
  const limitY = Math.max(rect.height / 2 - NODE_BOUNDARY, 0)
  agentPosition.x = Math.min(Math.max(agentPosition.x, -limitX), limitX)
  agentPosition.y = Math.min(Math.max(agentPosition.y, -limitY), limitY)
}

function handleResize() {
  viewportWidth.value = window.innerWidth
  clampPosition()
}

function triggerMcp() {
  console.log('Trigger MCP UI rendering pipeline')
  recordInteraction('trigger_mcp')
}
</script>

<style scoped>
.app-shell {
  --primary-color: #6366f1;
  --primary-hover: #5855eb;
  --secondary-color: rgba(99, 102, 241, 0.12);
  --text-primary: #0f172a;
  --text-secondary: #475569;
  --border-color: rgba(15, 23, 42, 0.08);
  --background: #f8fafc;
  --surface: rgba(255, 255, 255, 0.85);
  --shadow: 0 12px 32px rgba(15, 23, 42, 0.12);

  position: fixed;
  inset: 0;
  display: flex;
  flex-direction: column;
  background: var(--background);
  color: var(--text-primary);
  transition: background 0.3s ease, color 0.3s ease;
  backdrop-filter: blur(24px);
}

.app-shell.dark-mode {
  --primary-color: #818cf8;
  --primary-hover: #6366f1;
  --secondary-color: rgba(129, 140, 248, 0.16);
  --text-primary: #e2e8f0;
  --text-secondary: #94a3b8;
  --border-color: rgba(148, 163, 184, 0.2);
  --background: #0f172a;
  --surface: rgba(15, 23, 42, 0.78);
  --shadow: 0 12px 32px rgba(2, 6, 23, 0.55);
}

.workspace {
  position: relative;
  flex: 1;
  display: grid;
  grid-template-columns: 300px 1fr auto;
  overflow: hidden;
}

.language-switch {
  position: fixed;
  right: 1.5rem;
  bottom: 1.5rem;
  z-index: 40;
  border: none;
  border-radius: 999px;
  padding: 0.45rem 1rem;
  background: rgba(15, 23, 42, 0.88);
  color: #fff;
  font-size: 0.78rem;
  font-weight: 600;
  letter-spacing: 0.06em;
  cursor: pointer;
  box-shadow: 0 16px 30px rgba(15, 23, 42, 0.35);
  transition: background 0.2s ease, transform 0.2s ease;
}

.language-switch:hover {
  background: rgba(15, 23, 42, 1);
  transform: translateY(-2px);
}

.app-shell.dark-mode .language-switch {
  background: rgba(99, 102, 241, 0.88);
  box-shadow: 0 16px 30px rgba(99, 102, 241, 0.35);
}

.app-shell.dark-mode .language-switch:hover {
  background: rgba(99, 102, 241, 1);
}

@media (max-width: 1280px) {
  .workspace {
    grid-template-columns: 240px 1fr auto;
  }
}

@media (max-width: 1024px) {
  .workspace {
    grid-template-columns: 220px 1fr;
  }
}

@media (max-width: 768px) {
  .workspace {
    grid-template-columns: 1fr;
  }
}
</style>

