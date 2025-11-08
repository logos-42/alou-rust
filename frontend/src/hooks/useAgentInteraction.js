import { useCallback, useMemo, useState } from 'react'

const DEFAULT_LABELS = {
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
  wallet_instruction: '钱包指令',
}

const DEFAULT_LOG_LIMIT = 20
const DEFAULT_CONTEXT_LIMIT = 50

const dispatchContextEvent = (payload) => {
  if (typeof window === 'undefined') return
  window.dispatchEvent(new CustomEvent('agent-context-event', { detail: payload }))
}

const clampArray = (list, limit) => {
  if (list.length <= limit) return list
  return list.slice(0, limit)
}

const optionsToConfig = (options) => ({
  labels: { ...DEFAULT_LABELS, ...(options?.actionLabels || {}) },
  logLimit: options?.logLimit ?? DEFAULT_LOG_LIMIT,
  contextLimit: options?.contextLimit ?? DEFAULT_CONTEXT_LIMIT,
})

export const useAgentInteraction = (options) => {
  const { labels, logLimit, contextLimit } = useMemo(() => optionsToConfig(options), [options])
  const [messages, setMessages] = useState([])
  const [interactionLogs, setInteractionLogs] = useState([])
  const [contextEvents, setContextEvents] = useState([])
  const [isConversationVisible, setConversationVisible] = useState(false)

  const recordInteraction = useCallback(
    (action, detail, label) => {
      const timestamp = Date.now()
      const entry = {
        id: `log_${timestamp}_${Math.random().toString(36).slice(2, 6)}`,
        action,
        label: label || labels[action] || action,
        timestamp,
        detail,
      }

      setInteractionLogs((prev) => {
        const next = [entry, ...prev]
        return clampArray(next, logLimit)
      })

      setContextEvents((prev) => {
        const next = [...prev, { action, detail, timestamp }]
        if (next.length > contextLimit) {
          next.splice(0, next.length - contextLimit)
        }
        return next
      })

      dispatchContextEvent({ action, detail, timestamp })
    },
    [labels, logLimit, contextLimit],
  )

  const appendMessage = useCallback((message) => {
    setMessages((prev) => [...prev, message])
    setConversationVisible(true)
  }, [])

  const flushContextEvents = useCallback(() => {
    let snapshot = []
    setContextEvents((prev) => {
      snapshot = prev.slice()
      return []
    })
    return snapshot
  }, [])

  const openConversation = useCallback(() => {
    setConversationVisible(true)
  }, [])

  const closeConversation = useCallback(() => {
    setConversationVisible(false)
  }, [])

  const hasConversation = messages.length > 0

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
    closeConversation,
  }
}

