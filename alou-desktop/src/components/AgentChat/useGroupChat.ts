import { useCallback, useEffect, useRef } from 'react'
import clusterActionService from '@/services/clusterActionService'
import apiClient from '@/services/api'
import useClusterActionStore from '@/stores/clusterActionStore'
import { i18n } from '@/hooks/useI18n'

interface GroupChatMessage {
  id: string
  type: string
  from: string
  fromName: string
  avatar: string | null
  content: string
  timestamp: number
  metadata: Record<string, unknown>
}

interface PubSubMessage {
  id?: string
  msg_type?: string
  from?: string
  content?: string
  timestamp?: number
  metadata?: {
    avatar?: string
    [key: string]: unknown
  }
}

interface ActionStatus {
  status: string
}

interface UseGroupChatOptions {
  actionId: string | null
  enabled?: boolean
}

interface UseGroupChatReturn {
  messages: GroupChatMessage[]
  status: string | null
  subscribeToGroupChat: (actionId: string, onMessage?: (msg: GroupChatMessage) => void) => Promise<() => void>
  loadGroupMessages: (actionId: string) => Promise<void>
  pollActionStatus: (actionId: string) => Promise<() => void>
}

/**
 * useGroupChat - 群聊状态管理 Hook
 * 管理群聊消息订阅、状态轮询和消息更新
 */
export const useGroupChat = ({ actionId, enabled = true }: UseGroupChatOptions): UseGroupChatReturn => {
  const {
    addGroupChatMessage,
    addGroupChatMessages,
    updateActionStatus,
    setActionDetails,
    getGroupChatMessages,
    getActionStatus,
    getActiveAction,
  } = useClusterActionStore()

  // 检查是否是本地群聊
  const isLocalGroupChat = useCallback((id: string | null): boolean => {
    return !!(id && id.startsWith('local_group_'))
  }, [])

  const subscriptionRef = useRef<NodeJS.Timeout | null>(null)
  const pollIntervalRef = useRef<NodeJS.Timeout | null>(null)
  const lastTimestampRef = useRef<number>(Date.now())
  const unsubscribeMessagesRef = useRef<(() => void) | null>(null)
  const unsubscribeStatusRef = useRef<(() => void) | null>(null)

  // 转换 PubSub 消息格式为群聊消息格式
  const transformPubSubMessage = useCallback((pubSubMsg: PubSubMessage): GroupChatMessage => {
    return {
      id: pubSubMsg.id || `msg_${Date.now()}_${Math.random()}`,
      type: pubSubMsg.msg_type || 'system',
      from: pubSubMsg.from || 'system',
      fromName: pubSubMsg.from === 'system' ? i18n.t('agent.groupChat.system') : (pubSubMsg.from || ''),
      avatar: pubSubMsg.metadata?.avatar || null,
      content: pubSubMsg.content || '',
      timestamp: pubSubMsg.timestamp || Date.now(),
      metadata: pubSubMsg.metadata || {},
    }
  }, [])

  // 订阅群聊消息（轮询方式）
  const subscribeToGroupChat = useCallback(
    async (actionId: string, onMessage?: (msg: GroupChatMessage) => void): Promise<() => void> => {
      if (!actionId) return () => {} // 返回空的清理函数而不是 null

      // 本地群聊不需要轮询后端
      if (isLocalGroupChat(actionId)) {
        return () => {} // 返回空的清理函数
      }

      const topic = `diap/cluster_action/${actionId}`
      let lastTimestamp = lastTimestampRef.current

      const poll = async () => {
        try {
          // 获取新消息
          const response = await apiClient.get(`/pubsub/messages`, {
            params: {
              topic,
              since: lastTimestamp,
            },
          })

          if (response.data?.messages) {
            const newMessages = response.data.messages as PubSubMessage[]
            if (newMessages.length > 0) {
              // 转换消息格式
              const transformedMessages = newMessages.map(transformPubSubMessage)
              
              // 添加到 store
              addGroupChatMessages(actionId, transformedMessages)

              // 更新最后时间戳
              newMessages.forEach((msg) => {
                if ((msg.timestamp || 0) > lastTimestamp) {
                  lastTimestamp = msg.timestamp || Date.now()
                }
              })
              lastTimestampRef.current = lastTimestamp

              // 调用回调
              transformedMessages.forEach((msg) => {
                onMessage?.(msg)
              })
            }
          }
        } catch (error) {
          // 静默处理 404 错误（可能是本地群聊或后端未准备好）
          const axiosError = error as { response?: { status: number } }
          if (axiosError.response?.status !== 404) {
            console.error('[useGroupChat] 获取群聊消息失败:', error)
          }
        }
      }

      // 立即执行一次
      await poll()

      // 设置轮询间隔（1秒）
      const interval = setInterval(poll, 1000)
      subscriptionRef.current = interval

      return () => {
        if (interval) {
          clearInterval(interval)
        }
      }
    },
    [addGroupChatMessages, transformPubSubMessage, isLocalGroupChat],
  )

  // 加载历史消息
  const loadGroupMessages = useCallback(
    async (actionId: string) => {
      if (!actionId) return

      // 本地群聊：添加欢迎消息
      if (isLocalGroupChat(actionId)) {
        // 检查是否已经添加过欢迎消息
        const existingMessages = getGroupChatMessages(actionId)
        const hasWelcome = existingMessages.some((msg: { id: string }) => msg.id === `welcome_${actionId}`)
        
        if (!hasWelcome) {
          const action = getActiveAction()
          const description = action ? (action.description || actionId) : actionId
          const welcomeMessage: GroupChatMessage = {
            id: `welcome_${actionId}`,
            type: 'system',
            from: 'system',
            fromName: i18n.t('agent.groupChat.system'),
            avatar: null,
            content: i18n.t('agent.groupChat.created', { description }),
            timestamp: Date.now(),
            metadata: {},
          }
          addGroupChatMessage(actionId, welcomeMessage)
        }
        return
      }

      try {
        const topic = `diap/cluster_action/${actionId}`
        const response = await apiClient.get(`/pubsub/messages`, {
          params: {
            topic,
          },
        })

        if (response.data?.messages) {
          const messages = (response.data.messages as PubSubMessage[]).map(transformPubSubMessage)
          addGroupChatMessages(actionId, messages)

          // 更新最后时间戳
          if (messages.length > 0) {
            const latestTimestamp = Math.max(...messages.map((m) => m.timestamp))
            lastTimestampRef.current = latestTimestamp
          }
        }
      } catch (error) {
        // 静默处理 404 错误
        const axiosError = error as { response?: { status: number } }
        if (axiosError.response?.status !== 404) {
          console.error('[useGroupChat] 加载历史消息失败:', error)
        }
      }
    },
    [addGroupChatMessages, addGroupChatMessage, transformPubSubMessage, isLocalGroupChat, getActiveAction, getGroupChatMessages],
  )

  // 轮询行动状态
  const pollActionStatus = useCallback(
    async (actionId: string): Promise<() => void> => {
      if (!actionId) return () => {}

      // 本地群聊：使用本地状态，不轮询后端
      if (isLocalGroupChat(actionId)) {
        const action = getActiveAction()
        if (action) {
          updateActionStatus(actionId, action.status || 'Active')
        }
        return () => {} // 返回空的清理函数
      }

      // 检查是否是集群行动（action_ 开头）还是 AI 任务（task_ 开头）
      const isClusterAction = actionId.startsWith('action_')
      const isAiTask = actionId.startsWith('task_')

      if (!isClusterAction && !isAiTask) {
        console.warn('[useGroupChat] 未知的行动类型:', actionId)
        return () => {}
      }

      // 如果是 AI 任务，不在这里轮询（由 useAsyncTaskPolling 处理）
      if (isAiTask) {
        console.log('[useGroupChat] AI 任务状态由 useAsyncTaskPolling 处理，跳过群聊轮询:', actionId)
        return () => {}
      }

      const poll = async () => {
        try {
          const status = await clusterActionService.getActionStatus(actionId) as ActionStatus
          updateActionStatus(actionId, status.status)

          // 如果行动完成，获取最终结果
          if (status.status === 'Completed' || status.status === 'Failed' || status.status === 'Cancelled') {
            try {
              const results = await clusterActionService.getActionResults(actionId)
              setActionDetails(actionId, results)
            } catch (error) {
              // 静默处理错误
              const axiosError = error as { response?: { status: number } }
              if (axiosError.response?.status !== 404) {
                console.error('[useGroupChat] 获取行动结果失败:', error)
              }
            }
          }
        } catch (error) {
          // 静默处理 404 错误
          const axiosError = error as { response?: { status: number } }
          if (axiosError.response?.status !== 404) {
            console.error('[useGroupChat] 轮询行动状态失败:', error)
          }
        }
      }

      // 立即执行一次
      await poll()

      // 设置轮询间隔（2秒）
      const interval = setInterval(poll, 2000)
      pollIntervalRef.current = interval

      return () => {
        if (interval) {
          clearInterval(interval)
        }
      }
    },
    [updateActionStatus, setActionDetails, isLocalGroupChat, getActiveAction]
  )

  // 初始化订阅和轮询
  useEffect(() => {
    // 清理函数必须始终返回，即使条件不满足
    const cleanup = () => {
      if (unsubscribeMessagesRef.current && typeof unsubscribeMessagesRef.current === 'function') {
        unsubscribeMessagesRef.current()
        unsubscribeMessagesRef.current = null
      }
      if (unsubscribeStatusRef.current && typeof unsubscribeStatusRef.current === 'function') {
        unsubscribeStatusRef.current()
        unsubscribeStatusRef.current = null
      }
      if (subscriptionRef.current) {
        clearInterval(subscriptionRef.current)
        subscriptionRef.current = null
      }
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current)
        pollIntervalRef.current = null
      }
    }

    if (!enabled || !actionId) {
      return cleanup
    }

    // 加载历史消息
    loadGroupMessages(actionId)

    // 订阅新消息
    subscribeToGroupChat(actionId, (message) => {
      console.log('[useGroupChat] 收到新消息:', message)
    }).then((unsub) => {
      unsubscribeMessagesRef.current = unsub
    }).catch(() => {
      unsubscribeMessagesRef.current = () => {}
    })

    // 轮询状态
    pollActionStatus(actionId).then((unsub) => {
      unsubscribeStatusRef.current = unsub
    }).catch(() => {
      unsubscribeStatusRef.current = () => {}
    })

    // 返回清理函数
    return cleanup
  }, [enabled, actionId, subscribeToGroupChat, pollActionStatus, loadGroupMessages])

  // 获取当前消息列表
  const messages = actionId ? getGroupChatMessages(actionId) : []

  // 获取当前状态
  const status = actionId ? getActionStatus(actionId) : null

  return {
    messages,
    status,
    subscribeToGroupChat,
    loadGroupMessages,
    pollActionStatus,
  }
}

export default useGroupChat
