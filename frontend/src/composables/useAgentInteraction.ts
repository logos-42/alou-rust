import { computed, ref } from 'vue'
import type { ContextEvent, InteractionLog, Message } from '@/types/agent'

interface InteractionOptions {
  actionLabels?: Record<string, string>
  logLimit?: number
  contextLimit?: number
}

const DEFAULT_LABELS: Record<string, string> = {
  channel_selected: '切换频道',
  toggle_theme: '主题切换',
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

const DEFAULT_LOG_LIMIT = 20
const DEFAULT_CONTEXT_LIMIT = 50

export function useAgentInteraction(options: InteractionOptions = {}) {
  const actionLabels = { ...DEFAULT_LABELS, ...options.actionLabels }
  const logLimit = options.logLimit ?? DEFAULT_LOG_LIMIT
  const contextLimit = options.contextLimit ?? DEFAULT_CONTEXT_LIMIT

  const messages = ref<Message[]>([])
  const interactionLogs = ref<InteractionLog[]>([])
  const contextEvents = ref<ContextEvent[]>([])
  const isConversationVisible = ref(false)

  function recordInteraction(action: string, detail?: Record<string, unknown>, label?: string) {
    const timestamp = Date.now()
    const entry: InteractionLog = {
      id: `log_${timestamp}_${Math.random().toString(36).slice(2, 6)}`,
      action,
      label: label || actionLabels[action] || action,
      timestamp,
      detail
    }

    interactionLogs.value = [entry, ...interactionLogs.value].slice(0, logLimit)

    contextEvents.value.push({ action, detail, timestamp })
    if (contextEvents.value.length > contextLimit) {
      contextEvents.value.splice(0, contextEvents.value.length - contextLimit)
    }

    window.dispatchEvent(new CustomEvent('agent-context-event', { detail: { action, detail, timestamp } }))
  }

  function appendMessage(message: Message) {
    messages.value.push(message)
    if (!isConversationVisible.value) {
      isConversationVisible.value = true
    }
  }

  function flushContextEvents() {
    const snapshot = contextEvents.value.splice(0, contextEvents.value.length)
    return snapshot
  }

  function openConversation() {
    if (!isConversationVisible.value) {
      isConversationVisible.value = true
    }
  }

  function closeConversation() {
    isConversationVisible.value = false
  }

  const hasConversation = computed(() => messages.value.length > 0)

  return {
    messages,
    interactionLogs,
    contextEvents,
    isConversationVisible,
    hasConversation,
    recordInteraction,
    appendMessage,
    flushContextEvents,
    openConversation,
    closeConversation
  }
}

