import { useCallback, useMemo, useState } from 'react'

/**
 * 默认标签映射
 */
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
  wallet_instruction: '钱包指令',
}

const DEFAULT_LOG_LIMIT = 20
const DEFAULT_CONTEXT_LIMIT = 50

/**
 * 交互日志条目接口
 */
export interface InteractionLogEntry {
  id: string
  action: string
  label: string
  timestamp: number
  detail?: any
}

/**
 * 上下文事件接口
 */
export interface ContextEvent {
  action: string
  detail?: any
  timestamp: number
}

/**
 * 消息接口
 */
export interface Message {
  id?: string
  type?: string
  content?: string
  timestamp?: number
  [key: string]: any
}

/**
 * Hook配置选项接口
 */
export interface UseAgentInteractionOptions {
  actionLabels?: Record<string, string>
  logLimit?: number
  contextLimit?: number
}

/**
 * Hook返回结果接口
 */
export interface UseAgentInteractionReturn {
  messages: Message[]
  interactionLogs: InteractionLogEntry[]
  contextEvents: ContextEvent[]
  isConversationVisible: boolean
  hasConversation: boolean
  recordInteraction: (action: string, detail?: any, label?: string) => void
  appendMessage: (message: Message) => void
  flushContextEvents: () => ContextEvent[]
  openConversation: () => void
  closeConversation: () => void
}

/**
 * 分发上下文事件
 */
const dispatchContextEvent = (payload: ContextEvent): void => {
  if (typeof window === 'undefined') return
  window.dispatchEvent(new CustomEvent('agent-context-event', { detail: payload }))
}

/**
 * 限制数组长度
 */
const clampArray = <T>(list: T[], limit: number): T[] => {
  if (list.length <= limit) return list
  return list.slice(0, limit)
}

/**
 * 配置选项转换
 */
const optionsToConfig = (options?: UseAgentInteractionOptions) => ({
  labels: { ...DEFAULT_LABELS, ...(options?.actionLabels || {}) },
  logLimit: options?.logLimit ?? DEFAULT_LOG_LIMIT,
  contextLimit: options?.contextLimit ?? DEFAULT_CONTEXT_LIMIT,
})

/**
 * 智能体交互 Hook
 * 用于记录和管理用户与智能体的交互
 */
export const useAgentInteraction = (options?: UseAgentInteractionOptions): UseAgentInteractionReturn => {
  const { labels, logLimit, contextLimit } = useMemo(() => optionsToConfig(options), [options])
  const [messages, setMessages] = useState<Message[]>([])
  const [interactionLogs, setInteractionLogs] = useState<InteractionLogEntry[]>([])
  const [contextEvents, setContextEvents] = useState<ContextEvent[]>([])
  const [isConversationVisible, setConversationVisible] = useState(false)

  /**
   * 记录交互
   */
  const recordInteraction = useCallback(
    (action: string, detail?: any, label?: string) => {
      const timestamp = Date.now()
      const entry: InteractionLogEntry = {
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

  /**
   * 添加消息
   */
  const appendMessage = useCallback((message: Message) => {
    setMessages((prev) => [...prev, message])
    setConversationVisible(true)
  }, [])

  /**
   * 清空上下文事件并返回快照
   */
  const flushContextEvents = useCallback((): ContextEvent[] => {
    let snapshot: ContextEvent[] = []
    setContextEvents((prev) => {
      snapshot = prev.slice()
      return []
    })
    return snapshot
  }, [])

  /**
   * 打开对话
   */
  const openConversation = useCallback(() => {
    setConversationVisible(true)
  }, [])

  /**
   * 关闭对话
   */
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

export default useAgentInteraction
